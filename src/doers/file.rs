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
use regex::Regex;
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
#[serde(rename_all = "kebab-case")]
pub struct DesiredFileState {
    pub group: String,
    pub mode: String,
    pub owner: String,
    pub content: Option<String>,
    pub ignore_pattern: Option<String>,
    pub from: Option<Utf8PathBuf>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct FileState<'a> {
    pub gid: Gid,
    pub mode: String,
    pub uid: Uid,
    pub content: Option<&'a str>,
    pub ignore_pattern: Option<String>,
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
        let filtered;
        let content = if let Some(filter) = &self.desired_state.ignore_pattern {
            if let Some(content) = &self.desired_state.content {
                filtered = self.filter_content(content, filter)?;
                Some(filtered.as_str())
            } else {
                self.desired_state.content.as_deref()
            }
        } else {
            self.desired_state.content.as_deref()
        };

        let desired = FileState {
            content,
            from: self.desired_state.from.clone(),
            uid: users_and_groups::owner_from(&self.desired_state.owner)?,
            gid: users_and_groups::group_from(&self.desired_state.group)?,
            ignore_pattern: self.desired_state.ignore_pattern.clone(),
            mode: self.desired_state.mode.clone(),
            hash: None,
        };

        if (desired.content.is_none() && desired.from.is_none())
            || (desired.content.is_some() && desired.from.is_some())
        {
            bail!(
                "file '{}' must have exactly one of :content or :from",
                &self.path
            );
        }

        let mut need_to_read_hash = true;

        if !self.path.exists() {
            tracing::info!("creating: {}", self.path);

            if opts.noop {
                return Ok(ONE_RESOURCE_NOOP);
            }

            self.write_contents_to_file(&desired)?;
            need_to_read_hash = false;
        }

        let current = self.current_state(need_to_read_hash)?;
        let changes = self.changes(&current, &desired)?;

        if changes.is_empty() {
            tracing::debug!("no change: {}", self.path);
            return Ok(ONE_RESOURCE_NO_CHANGE);
        }

        if opts.noop {
            tracing::info!("{} change: {}", self.path, changes.join(", "));
            return Ok(ONE_RESOURCE_NOOP);
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
            // File existed before this run. Are its contents correct? We already have its hash,
            // and if the user gave us a filter, that hash is of the filtered file.
            let desired_hash = if let Some(content) = &desired.content {
                // If we were given content and an ignore filter, the content is already filtered
                // so we only have to hash it
                self.content_hash(content)
            } else if let Some(from_file) = &desired.from {
                // If we've been given a from file, we may need to filter it
                if let Some(pattern) = &desired.ignore_pattern {
                    self.hash_of_filtered_file(from_file, pattern)?
                } else {
                    blake3::hash(&fs::read(from_file)?)
                }
            } else {
                bail!("have neither from nor content");
            };

            if desired_hash != current_hash {
                to_change.push("content");
                to_change.push("owner");
                to_change.push("mode");
            }
        } // else the file has just been created and we know its contents are correct

        if current.gid != desired.gid {
            to_change.push("group");
        }

        if current.uid != desired.uid {
            to_change.push("owner");
        }

        if current.mode != desired.mode {
            to_change.push("mode");
        }

        Ok(to_change)
    }

    fn hash_of_filtered_file(&self, path: &Utf8PathBuf, pattern: &str) -> anyhow::Result<Hash> {
        let raw = fs::read_to_string(path)?;
        let filtered = self.filter_content(&raw, pattern)?;
        Ok(self.content_hash(&filtered))
    }

    fn filter_content(&self, content: &str, filter: &str) -> anyhow::Result<String> {
        tracing::debug!("filtering content on '{}'", filter);
        let rx = Regex::new(filter)?;
        let ret: String = content.lines().filter(|l| !rx.is_match(l)).collect();
        Ok(ret)
    }

    fn current_state(&self, need_to_read_hash: bool) -> anyhow::Result<FileState> {
        tracing::debug!("getting state: {}", &self.path);
        let path = &self.path.as_path();
        let metadata = nix::sys::stat::stat(path.as_std_path())?;

        let mode = format!("{:04o}", metadata.st_mode & 0o777);
        let uid = metadata.st_uid.into();
        let gid = metadata.st_gid.into();

        let hash = if need_to_read_hash {
            if let Some(pattern) = &self.desired_state.ignore_pattern {
                Some(self.hash_of_filtered_file(&self.path, pattern)?)
            } else {
                Some(self.file_hash()?)
            }
        } else {
            None
        };

        Ok(FileState {
            gid,
            ignore_pattern: None,
            uid,
            mode: mode.to_owned(),
            content: None,
            from: None,
            hash,
        })
    }

    fn content_hash(&self, content: &str) -> Hash {
        blake3::hash(content.as_bytes())
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
            tracing::debug!("copying {} -> {}", from, self.path);
            fs::copy(from, &self.path)?;
            Ok(())
        } else {
            bail!("can write neither :from nor :content");
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
    use crate::test_utils::spec_helper::{
        defopts, defopts_noop, fixture, janet2json, my_group, my_user,
    };
    use assert_fs::TempDir;
    use assert_fs::prelude::*;
    use camino::Utf8PathBuf;
    use indoc::formatdoc;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn test_file_create_noop() {
        let temp = TempDir::new().unwrap();
        let path = Utf8PathBuf::from_path_buf(temp.child("test-file").to_path_buf()).unwrap();

        let json_def = janet2json(&formatdoc! {"
            (file/ensure \"{}\"
                :content \"some-junk\"
                :mode \"0750\"
                :owner \"{}\"
                :group \"{}\")
            ",
            path,
            my_user(),
            my_group(),
        });

        assert!(!path.exists());
        let sut: GurpFileEnsure = serde_json::from_str(&json_def).unwrap();
        assert_eq!(ONE_RESOURCE_NOOP, sut.apply(&defopts_noop()).unwrap());
        assert!(!path.exists());
    }

    #[test]
    fn test_file_create_from_content() {
        let temp = TempDir::new().unwrap();
        let file = temp.join("test-file");
        let path = Utf8PathBuf::from_path_buf(file.to_path_buf()).unwrap();
        assert!(!path.exists());

        let json_def = janet2json(&formatdoc! {"
            (file/ensure \"{}\"
                :content \"stuff\"
                :mode \"0640\"
                :owner \"{}\"
                :group \"{}\")
            ",
            path,
            my_user(),
            my_group(),
        });

        let sut: GurpFileEnsure = serde_json::from_str(&json_def).unwrap();
        assert_eq!(ONE_RESOURCE_ONE_CHANGE, sut.apply(&defopts()).unwrap());
        assert!(path.exists());
        let metadata = fs::metadata(&path).unwrap();
        assert_eq!(metadata.permissions().mode() & 0o777, 0o640);
        assert_eq!("stuff", fs::read_to_string(path).unwrap());
    }

    #[test]
    fn test_file_create_from_template() {
        let temp = TempDir::new().unwrap();
        let file = temp.join("test-file");
        let path = Utf8PathBuf::from_path_buf(file.to_path_buf()).unwrap();
        assert!(!path.exists());

        // escape { with another {
        let json_def = janet2json(&formatdoc! {"
            (file/ensure \"{}\"
                :content (template-out \"{{{{ name }}}} is running a {{{{ thing }}}}\"
                                        {{ :name \"gurp\" :thing \"test\" }})
                :mode \"0600\"
                :owner \"{}\"
                :group \"{}\")
            ",
            path,
            my_user(),
            my_group(),
        });

        let sut: GurpFileEnsure = serde_json::from_str(&json_def).unwrap();
        assert_eq!(ONE_RESOURCE_ONE_CHANGE, sut.apply(&defopts()).unwrap());
        assert!(path.exists());
        let metadata = fs::metadata(&path).unwrap();
        assert_eq!(metadata.permissions().mode() & 0o777, 0o600);
        assert_eq!("gurp is running a test", fs::read_to_string(path).unwrap());
    }

    #[test]
    fn test_file_ensure_already_correct() {
        let temp = TempDir::new().unwrap();
        temp.child("test-file").write_str("stuff").unwrap();
        let file = temp.join("test-file");

        let path = Utf8PathBuf::from_path_buf(file.to_path_buf()).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o750)).unwrap();
        assert!(path.exists());

        let json_def = janet2json(&formatdoc! {"
            (file/ensure \"{}\"
                :content \"stuff\"
                :mode \"0750\"
                :owner \"{}\"
                :group \"{}\")
            ",
            path,
            my_user(),
            my_group(),
        });

        let sut: GurpFileEnsure = serde_json::from_str(&json_def).unwrap();
        assert_eq!(ONE_RESOURCE_NO_CHANGE, sut.apply(&defopts_noop()).unwrap());
        assert!(path.exists());
    }

    #[test]
    fn test_update_file_from_content_and_set_mode() {
        let temp = TempDir::new().unwrap();
        temp.child("test-file")
            .write_str("the-wrong-stuff")
            .unwrap();

        let path = Utf8PathBuf::from_path_buf(temp.to_path_buf())
            .unwrap()
            .join("test-file");
        assert!(path.exists());

        let json_def = janet2json(&formatdoc! {"
            (file/ensure \"{}\"
                :content \"the-right-stuff\"
                :mode \"0400\"
                :owner \"{}\"
                :group \"{}\")
            ",
            path,
            my_user(),
            my_group(),
        });

        let sut: GurpFileEnsure = serde_json::from_str(&json_def).unwrap();
        assert_eq!(ONE_RESOURCE_ONE_CHANGE, sut.apply(&defopts()).unwrap());

        assert!(path.exists());
        assert_eq!(
            "the-right-stuff".to_owned(),
            fs::read_to_string(&path).unwrap()
        );

        let metadata = fs::metadata(&path).unwrap();
        assert_eq!(metadata.permissions().mode() & 0o777, 0o400);
    }

    #[test]
    fn test_update_file_from_file_and_set_mode() {
        let temp = TempDir::new().unwrap();
        temp.child("test-file")
            .write_str("the-wrong-stuff")
            .unwrap();

        let path = Utf8PathBuf::from_path_buf(temp.to_path_buf())
            .unwrap()
            .join("test-file");

        assert!(path.exists());

        let json_def = janet2json(&formatdoc! {"
            (file/ensure \"{}\"
                :from \"{}\"
                :mode \"0444\"
                :owner \"{}\"
                :group \"{}\")
            ",
            path,
            &fixture("doers/file/copy-file"),
            my_user(),
            my_group(),
        });

        let sut: GurpFileEnsure = serde_json::from_str(&json_def).unwrap();
        assert_eq!(ONE_RESOURCE_ONE_CHANGE, sut.apply(&defopts()).unwrap());

        assert!(path.exists());
        assert_eq!(
            "some-content\n".to_owned(),
            fs::read_to_string(&path).unwrap()
        );

        let metadata = fs::metadata(&path).unwrap();
        assert_eq!(metadata.permissions().mode() & 0o777, 0o444);
    }

    #[test]
    fn test_ignored_line_means_no_change_with_content() {
        let content = "today is 2015-01-30\nBut this never changes.\nAnd nor does this.\n";
        let temp = TempDir::new().unwrap();
        temp.child("test-file").write_str(content).unwrap();

        let path = Utf8PathBuf::from_path_buf(temp.to_path_buf())
            .unwrap()
            .join("test-file");

        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        assert!(path.exists());

        let json_def = janet2json(&formatdoc! {"
            (file/ensure \"{}\"
                :content \"today is 2025-06-26\\nBut this never changes.\\nAnd nor does this.\"
                :mode \"0600\"
                :ignore-pattern \"^today is\"
                :owner \"{}\"
                :group \"{}\")
            ",
            path,
            my_user(),
            my_group(),
        });

        let sut: GurpFileEnsure = serde_json::from_str(&json_def).unwrap();
        assert_eq!(ONE_RESOURCE_NO_CHANGE, sut.apply(&defopts()).unwrap());
        assert_eq!(content, fs::read_to_string(&path).unwrap());
    }

    #[test]
    fn test_ignored_line_means_no_change_with_from() {
        let content = "today is 2015-01-30\nBut this never changes.\nAnd nor does this.\n";
        let temp = TempDir::new().unwrap();
        temp.child("test-file").write_str(content).unwrap();

        let path = Utf8PathBuf::from_path_buf(temp.to_path_buf())
            .unwrap()
            .join("test-file");

        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        assert!(path.exists());

        let json_def = janet2json(&formatdoc! {"
            (file/ensure \"{}\"
                :from \"{}\"
                :mode \"0600\"
                :ignore-pattern \"^today is\"
                :owner \"{}\"
                :group \"{}\")
            ",
            path,
            fixture("doers/file/ignore-line-file"),
            my_user(),
            my_group(),
        });

        let sut: GurpFileEnsure = serde_json::from_str(&json_def).unwrap();
        assert_eq!(ONE_RESOURCE_NO_CHANGE, sut.apply(&defopts()).unwrap());
        assert_eq!(content, fs::read_to_string(&path).unwrap());
    }

    #[test]
    fn test_file_remove_does_not_exist() {
        let json_def = janet2json(r#"(file/remove "/path/does/not/exist")"#);
        let sut: GurpFileRemove = serde_json::from_str(&json_def).unwrap();
        assert_eq!(ONE_RESOURCE_NO_CHANGE, sut.apply(&defopts()).unwrap());
    }

    #[test]
    fn test_file_remove_forbidden() {
        let json_def = janet2json(r#"(file/remove "/bin/ps")"#);
        let sut: GurpFileRemove = serde_json::from_str(&json_def).unwrap();
        assert_eq!(ONE_RESOURCE_ONE_ERROR, sut.apply(&defopts()).unwrap());
    }

    #[test]
    fn test_file_remove() {
        let temp = TempDir::new().unwrap();
        temp.child("test-file")
            .write_str("transient-stuff")
            .unwrap();

        let path = Utf8PathBuf::from_path_buf(temp.to_path_buf())
            .unwrap()
            .join("test-file");

        assert!(path.exists());
        let json_def = janet2json(&format!("(file/remove \"{path}\")"));
        let sut: GurpFileRemove = serde_json::from_str(&json_def).unwrap();
        assert_eq!(ONE_RESOURCE_ONE_CHANGE, sut.apply(&defopts()).unwrap());
        assert!(!path.exists());
    }

    #[test]
    fn test_file_remove_noop() {
        let temp = TempDir::new().unwrap();
        temp.child("test-file")
            .write_str("transient-stuff")
            .unwrap();

        let path = Utf8PathBuf::from_path_buf(temp.to_path_buf())
            .unwrap()
            .join("test-file");

        assert!(path.exists());
        let json_def = janet2json(&format!("(file/remove \"{path}\")"));
        let sut: GurpFileRemove = serde_json::from_str(&json_def).unwrap();
        assert_eq!(ONE_RESOURCE_NOOP, sut.apply(&defopts_noop()).unwrap());
        assert!(path.exists());
    }
}
