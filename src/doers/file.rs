use crate::common::constants::{
    ONE_RESOURCE_NO_CHANGE, ONE_RESOURCE_NOOP, ONE_RESOURCE_ONE_CHANGE, ONE_RESOURCE_ONE_ERROR,
    PROTECTED_FILES,
};
use crate::common::output::Output;
use crate::common::traits::Apply;
use crate::common::types::{Action, ApplySummary, Changes, Opts, Resource};
use crate::common::users_and_groups;
use crate::utils::janet_helpers::{self, JanetExt, JanetStructExt};
use anyhow::anyhow;
use blake3::Hash;
use camino::Utf8PathBuf;
use janetrs::{Janet, JanetArray};
use nix::unistd::{Gid, Uid};
use paste::paste;
use std::fmt::Debug;
use std::fs;
use std::io::Write;
use std::os::unix::fs::MetadataExt;

// THINGS TO KNOW / THINGS TO DO.
// You can only define users and groups by their names. UIDs/GIDs do not work.

#[derive(Debug, PartialEq, Eq)]
pub struct GurpFile {
    pub action: Action,
    pub exists: bool,
    pub id: String,
    pub name: Utf8PathBuf, // The Path
    pub desired_state: Option<FileState>,
    pub doer: String,
}

#[derive(Debug, PartialEq, Eq)]
pub struct FileState {
    pub gid: Gid,
    pub mode: String,
    pub uid: Uid,
    pub content: Option<String>,
    pub from: Option<Utf8PathBuf>,
    pub hash: Option<Hash>,
}

impl TryFrom<&Janet> for GurpFile {
    type Error = anyhow::Error;

    fn try_from(value: &Janet) -> anyhow::Result<Self> {
        let data = value.extract_struct()?;
        let name = data.get_field_pathbuf("name")?;
        let exists = name.exists();
        let action = janet_helpers::action_as_enum(&data)?;

        let content = data
            .get(Janet::keyword("content".into()))
            .map(|c| c.to_string());

        let from = data
            .get(Janet::keyword("from".into()))
            .map(|c| Utf8PathBuf::from(c.to_string()));

        let state = match action {
            Action::Ensure => {
                if content.is_none() && from.is_none() {
                    return Err(anyhow!("file must have :content or :from"));
                }

                if content.is_some() && from.is_some() {
                    return Err(anyhow!("file cannot have both :content and :from"));
                }

                Some(FileState {
                    gid: users_and_groups::group_from(&data.get_field_string("group")?)?,
                    mode: data.get_field_string("mode")?,
                    uid: users_and_groups::owner_from(&data.get_field_string("owner")?)?,
                    from,
                    content,
                    hash: None,
                })
            }
            Action::Remove => None,
        };

        Ok(GurpFile {
            action,
            exists,
            id: data.get_field_string("_id")?,
            name: data.get_field_pathbuf("name")?,
            desired_state: state,
            doer: "file".to_owned(),
        })
    }
}

crate::unpack_fn!(ensure_list, File, GurpFile);
crate::unpack_fn!(remove_list, File, GurpFile);
crate::impl_apply!(GurpFile);

impl GurpFile {
    fn apply_ensure(&self, opts: &Opts, output: &Output) -> anyhow::Result<ApplySummary> {
        let desired = self.desired_state.as_ref().unwrap();

        if !self.exists {
            output.creating(&self.name);

            if opts.noop {
                return Ok(ONE_RESOURCE_ONE_CHANGE);
            }

            self.write_contents_to_file(desired)?;
        }

        let path = &self.name;
        let current = self.current_state()?;
        let changes = self.changes(&current, desired);

        if changes.is_empty() {
            output.no_change(&self.name);
            return Ok(ONE_RESOURCE_NO_CHANGE);
        }

        if changes.contains(&"content") {
            self.write_contents_to_file(desired)?;
        }

        if changes.contains(&"group") || changes.contains(&"owner") {
            users_and_groups::set_user(path, desired.uid, desired.gid)?;
        }

        if changes.contains(&"mode") {
            users_and_groups::set_mode(path, &current.mode, &desired.mode)?;
        }

        Ok(ONE_RESOURCE_ONE_CHANGE)
    }

    fn apply_remove(&self, opts: &Opts, output: &Output) -> anyhow::Result<ApplySummary> {
        if self.exists {
            if PROTECTED_FILES.contains(&self.name) {
                output.protected(&self.name);
                return Ok(ONE_RESOURCE_ONE_ERROR);
            }

            output.removing(&self.name);

            if opts.noop {
                Ok(ONE_RESOURCE_NOOP)
            } else {
                fs::remove_file(&self.name)?;
                Ok(ONE_RESOURCE_ONE_CHANGE)
            }
        } else {
            output.not_present(&self.name);
            Ok(ONE_RESOURCE_NO_CHANGE)
        }
    }

    fn changes<'a>(&self, current: &FileState, desired: &FileState) -> Changes<'a> {
        let mut to_change = Vec::new();

        if let Some(current_hash) = current.hash {
            if let Some(content) = &desired.content {
                let content_hash = blake3::hash(content.as_bytes());
                if content_hash != current_hash {
                    to_change.push("content");
                }
            } else {
                eprintln!("TODO: implement non-content writing");
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

        to_change
    }

    fn current_state(&self) -> anyhow::Result<FileState> {
        let path = &self.name.as_path();
        let metadata = nix::sys::stat::stat(path.as_std_path())?;

        let mode = format!("{:04o}", metadata.st_mode & 0o777);
        let uid = metadata.st_uid.into();
        let gid = metadata.st_gid.into();

        // If we have just created the file ourselves, this will still be false, so we know there's
        // no need to check the contents: they have to be right, and it's a potentially expensive
        // operation.
        //
        let hash = if self.exists {
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
        let mut fh = fs::File::open(&self.name)?;
        std::io::copy(&mut fh, &mut hasher)?;
        Ok(hasher.finalize())
    }

    fn write_contents_to_file(&self, desired_state: &FileState) -> anyhow::Result<()> {
        if let Some(content) = &desired_state.content {
            let mut fh = fs::File::create(&self.name)?;
            Ok(fh.write_all(content.as_bytes())?)
        } else {
            Err(anyhow!("Only content writing is currently supported"))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils::spec_helper::{defopts, defopts_noop, init_janet, my_group, my_user};
    use assert_fs::TempDir;
    use assert_fs::prelude::*;
    use camino::Utf8PathBuf;

    #[test]
    fn test_file_remove_apply_does_not_exist() {
        let file_does_not_exist = GurpFile {
            name: Utf8PathBuf::from("/does/not/exist/file-to-test"),
            exists: false,
            id: "/test-role/file/file-to-test".to_owned(),
            action: Action::Remove,
            desired_state: None,
            doer: "file".to_owned(),
        };

        assert_eq!(
            ONE_RESOURCE_NO_CHANGE,
            file_does_not_exist.apply(&defopts()).unwrap()
        );
    }

    #[test]
    fn test_file_remove_apply_not_allowed() {
        let disallowed_file = GurpFile {
            name: Utf8PathBuf::from("/bin/ps"),
            id: "/test-role/file/_bin_ps".to_owned(),
            exists: true,
            action: Action::Remove,
            desired_state: None,
            doer: "file".to_owned(),
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

        let test_file = GurpFile {
            name: Utf8PathBuf::from_path_buf(file.to_path_buf()).unwrap(),
            id: "/test-role/file/test-file".to_owned(),
            exists: true,
            action: Action::Remove,
            desired_state: None,
            doer: "file".to_owned(),
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

        let test_file = GurpFile {
            name: Utf8PathBuf::from_path_buf(file.to_path_buf()).unwrap(),
            id: "/test-role/file/test-file".to_owned(),
            exists: true,
            action: Action::Remove,
            desired_state: None,
            doer: "file".to_owned(),
        };

        assert!(file.exists());
        assert_eq!(
            ONE_RESOURCE_NO_CHANGE,
            test_file.apply(&defopts_noop()).unwrap()
        );
        assert!(file.exists());
    }

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
                doer: "file".to_owned(),
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
                doer: "file".to_owned(),
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
        gurp_file.apply(&defopts()).unwrap();

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
        gurp_file.apply(&defopts()).unwrap();

        assert!(file_to_create.exists());
        assert_eq!(
            "the-right-stuff".to_owned(),
            fs::read_to_string(&file_to_create).unwrap()
        );
        let metadata = fs::metadata(file_to_create).unwrap();
        let mode = format!("{:04o}", metadata.mode() & 0o777);
        assert_eq!("0400", mode);
    }
}
