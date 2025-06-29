use crate::common::constants::{
    ONE_RESOURCE_NO_CHANGE, ONE_RESOURCE_NOOP, ONE_RESOURCE_ONE_CHANGE, ONE_RESOURCE_ONE_ERROR,
    PROTECTED_FILES,
};
use crate::common::types::{ApplySummary, Changes, Opts};
use crate::common::users_and_groups;
use anyhow::bail;
use blake3::Hash;
use camino::Utf8PathBuf;
use nix::unistd::{Gid, Uid};
use serde::Deserialize;
use std::fmt::Debug;
use std::fs;
use std::io::Write;

// THINGS TO KNOW / THINGS TO DO.
// Files are not backed up

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct GurpFileEnsure {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(rename = "name")]
    pub path: Utf8PathBuf,
    #[serde(flatten)]
    pub desired_state: DesiredFileState,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct DesiredFileState {
    pub group: String,
    pub mode: String,
    pub owner: String,
    pub content: Option<String>,
    pub from: Option<Utf8PathBuf>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct FileState<'a> {
    pub gid: Gid,
    pub mode: String,
    pub uid: Uid,
    pub content: Option<&'a String>,
    pub from: Option<Utf8PathBuf>,
    pub hash: Option<Hash>,
}

#[derive(Deserialize, Debug)]
pub struct GurpFileRemove {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(rename = "name")]
    pub path: Utf8PathBuf,
}

impl GurpFileEnsure {
    pub fn apply(&self, opts: &Opts) -> anyhow::Result<ApplySummary> {
        let desired = FileState {
            content: self.desired_state.content.as_ref(),
            from: self.desired_state.from.clone(),
            uid: users_and_groups::owner_from(&self.desired_state.owner)?,
            gid: users_and_groups::group_from(&self.desired_state.group)?,
            mode: self.desired_state.mode.clone(),
            hash: None,
        };

        if !self.path.exists() {
            tracing::info!("creating: {}", self.path);

            if opts.noop {
                return Ok(ONE_RESOURCE_ONE_CHANGE);
            }

            self.write_contents_to_file(&desired)?;
        }

        let current = self.current_state()?;
        let changes = self.changes(&current, &desired)?;

        if changes.is_empty() {
            tracing::info!("no change: {}", self.path);
            return Ok(ONE_RESOURCE_NO_CHANGE);
        }

        if changes.contains(&"content") {
            tracing::info!("change content: {}", self.path);
            self.write_contents_to_file(&desired)?;
        }

        if changes.contains(&"group") || changes.contains(&"owner") {
            tracing::info!(
                "change owner:group : {} {}:{} -> {}:{}",
                self.path,
                current.uid,
                current.gid,
                desired.uid,
                desired.gid
            );
            users_and_groups::set_user(&self.path, desired.uid, desired.gid)?;
        }

        if changes.contains(&"mode") {
            tracing::info!(
                "change mode: {} {} -> {}",
                self.path,
                current.mode,
                desired.mode,
            );
            users_and_groups::set_mode(&self.path, &current.mode, &desired.mode)?;
        }

        Ok(ONE_RESOURCE_ONE_CHANGE)
    }

    fn changes<'a>(&self, current: &FileState, desired: &FileState) -> anyhow::Result<Changes<'a>> {
        let mut to_change = Vec::new();

        if let Some(current_hash) = current.hash {
            let desired_hash = if let Some(content) = &desired.content {
                blake3::hash(content.as_bytes())
            } else if let Some(from) = &desired.from {
                blake3::hash(&fs::read(from)?)
            } else {
                bail!("have neither from nor content");
            };

            if desired_hash != current_hash {
                to_change.push("content");
            }
        }

        if current.gid != desired.gid {
            to_change.push("group");
        }

        if current.uid != desired.uid {
            to_change.push("owner");
        }

        if current.mode != desired.mode {
            to_change.push("mode");
        }

        tracing::debug!("to change for {}: {}", self.path, to_change.join(", "));
        Ok(to_change)
    }

    fn current_state(&self) -> anyhow::Result<FileState> {
        tracing::debug!("getting state: {}", &self.path);
        let path = &self.path.as_path();
        let metadata = nix::sys::stat::stat(path.as_std_path())?;

        let mode = format!("{:04o}", metadata.st_mode & 0o777);
        let uid = metadata.st_uid.into();
        let gid = metadata.st_gid.into();

        // If we have just created the file ourselves, this will still be false, so we know there's
        // no need to check the contents: they have to be right, and it's a potentially expensive
        // operation.
        //
        let hash = if path.exists() {
            Some(self.file_hash()?)
        } else {
            None
        };

        Ok(FileState {
            gid,
            uid,
            mode: mode.to_owned(),
            content: None,
            from: None,
            hash,
        })
    }

    fn file_hash(&self) -> anyhow::Result<Hash> {
        let mut hasher = blake3::Hasher::new();
        let mut fh = fs::File::open(&self.path)?;
        std::io::copy(&mut fh, &mut hasher)?;
        Ok(hasher.finalize())
    }

    fn write_contents_to_file(&self, desired_state: &FileState) -> anyhow::Result<()> {
        if let Some(content) = &desired_state.content {
            let mut fh = fs::File::create(&self.path)?;
            Ok(fh.write_all(content.as_bytes())?)
        } else if let Some(from) = &desired_state.from {
            tracing::debug!("coping {} -> {}", from, self.path);
            fs::copy(from, &self.path)?;
            Ok(())
        } else {
            bail!("can write neither from nor content");
        }
    }
}

impl GurpFileRemove {
    pub fn apply(&self, opts: &Opts) -> anyhow::Result<ApplySummary> {
        if self.path.exists() {
            if PROTECTED_FILES.contains(&self.path) {
                tracing::warn!("protected resource: {}", self.path);
                return Ok(ONE_RESOURCE_ONE_ERROR);
            }

            tracing::info!("removing: {}", self.path);

            if opts.noop {
                Ok(ONE_RESOURCE_NOOP)
            } else {
                fs::remove_file(&self.path)?;
                Ok(ONE_RESOURCE_ONE_CHANGE)
            }
        } else {
            tracing::debug!("not present: {}", self.path);
            Ok(ONE_RESOURCE_NO_CHANGE)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils::spec_helper::{defopts, defopts_noop}; //, init_janet, my_group, my_user};
    use assert_fs::TempDir;
    use assert_fs::prelude::*;
    use camino::Utf8PathBuf;
    // use std::os::unix::fs::MetadataExt;

    #[test]
    fn test_file_remove_apply_does_not_exist() {
        let file_does_not_exist = GurpFileRemove {
            path: Utf8PathBuf::from("/does/not/exist/file-to-test"),
            id: "/test-role/file/file-to-test".to_owned(),
        };

        assert_eq!(
            ONE_RESOURCE_NO_CHANGE,
            file_does_not_exist.apply(&defopts()).unwrap()
        );
    }

    #[test]
    fn test_file_remove_apply_not_allowed() {
        let disallowed_file = GurpFileRemove {
            path: Utf8PathBuf::from("/bin/ps"),
            id: "/test-role/file/_bin_ps".to_owned(),
        };

        assert_eq!(
            ONE_RESOURCE_ONE_ERROR,
            disallowed_file.apply(&defopts()).unwrap()
        );
    }

    #[test]
    fn test_file_remove_apply_works() {
        let temp = TempDir::new().unwrap();
        temp.child("test-file").write_str("stuff").unwrap();
        let file = temp.join("test-file");

        let test_file = GurpFileRemove {
            path: Utf8PathBuf::from_path_buf(file.to_path_buf()).unwrap(),
            id: "/test-role/file/test-file".to_owned(),
        };

        assert!(file.exists());
        assert_eq!(
            ONE_RESOURCE_ONE_CHANGE,
            test_file.apply(&defopts()).unwrap()
        );
        assert!(!file.exists());
    }

    #[test]
    fn test_file_remove_apply_noop() {
        let temp = TempDir::new().unwrap();
        temp.child("test-file").write_str("stuff").unwrap();
        let file = temp.join("test-file");

        let test_file = GurpFileRemove {
            path: Utf8PathBuf::from_path_buf(file.to_path_buf()).unwrap(),
            id: "/test-role/file/test-file".to_owned(),
        };

        assert!(file.exists());
        assert_eq!(
            ONE_RESOURCE_NO_CHANGE,
            test_file.apply(&defopts_noop()).unwrap()
        );
        assert!(file.exists());
    }

    /*
    #[test]
    fn test_unpack_ensure_file() {
        init_janet();

        let example_file_ensure = Janet::wrap(janetrs::structs! {
            ":_id" => "/test-role/file/test-file",
            ":action" => ":ensure",
            ":content" => "some-content",
            ":group" => "14",
            ":mode" => "0755",
            ":name" => "/tmp/merp",
            ":owner" => "264",
        });

        assert_eq!(
            GurpFile {
                name: Utf8PathBuf::from("/tmp/merp"),
                id: "/test-role/file/test-file".to_owned(),
                exists: false,
                action: Action::Ensure,
                desired_state: Some(FileState {
                    content: Some("some-content".to_owned()),
                    from: None,
                    gid: 14.into(),
                    hash: None,
                    mode: "0755".to_owned(),
                    uid: 264.into(),
                }),
            },
            GurpFile::try_from(&example_file_ensure).unwrap()
        );
    }

    #[test]
    fn test_unpack_remove_file() {
        init_janet();
        let example_file_remove = Janet::wrap(janetrs::structs! {
            ":name" => "/tmp/merp",
            ":_id" => "/test-role/file/merp",
            ":label" => "merp",
            ":action" => ":remove",
        });

        assert_eq!(
            GurpFile {
                name: Utf8PathBuf::from("/tmp/merp"),
                id: "/test-role/file/merp".to_owned(),
                exists: false,
                action: Action::Remove,
                desired_state: None,
            },
            GurpFile::try_from(&example_file_remove).unwrap()
        );
    }

    #[test]
    fn test_create_fresh_file() {
        init_janet();

        let temp = TempDir::new().unwrap();
        let file_to_create = temp.join("test-file");

        let example_file_ensure = Janet::wrap(janetrs::structs! {
            ":_id" => "/test-role/file/test-file",
            ":action" => ":ensure",
            ":content" => "some-content",
            ":group" => my_group().as_str(),
            ":mode" => "0600",
            ":name" => file_to_create.to_string_lossy().to_string().as_str(),
            ":owner" => my_user().as_str(),
        });

        let gurp_file = GurpFile::try_from(&example_file_ensure).unwrap();
        gurp_file.apply(&defcontext(), &defopts()).unwrap();

        assert!(file_to_create.exists());
        assert_eq!(
            "some-content".to_owned(),
            fs::read_to_string(&file_to_create).unwrap()
        );
        let metadata = fs::metadata(file_to_create).unwrap();
        let mode = format!("{:04o}", metadata.mode() & 0o777);
        assert_eq!("0600", mode);
    }

    #[test]
    fn test_update_file_and_set_mode() {
        init_janet();

        let temp = TempDir::new().unwrap();
        temp.child("test-file")
            .write_str("the-wrong-stuff")
            .unwrap();
        let file_to_create = temp.join("test-file");

        let example_file_ensure = Janet::wrap(janetrs::structs! {
            ":_id" => "/test-role/file/test-file",
            ":action" => ":ensure",
            ":content" => "the-right-stuff",
            ":group" => my_group().as_str(),
            ":mode" => "0400",
            ":name" => file_to_create.to_string_lossy().to_string().as_str(),
            ":owner" => my_user().as_str(),
        });

        let gurp_file = GurpFile::try_from(&example_file_ensure).unwrap();
        gurp_file.apply(&defcontext(), &defopts()).unwrap();

        assert!(file_to_create.exists());
        assert_eq!(
            "the-right-stuff".to_owned(),
            fs::read_to_string(&file_to_create).unwrap()
        );
        let metadata = fs::metadata(file_to_create).unwrap();
        let mode = format!("{:04o}", metadata.mode() & 0o777);
        assert_eq!("0400", mode);
    }
    */
}
