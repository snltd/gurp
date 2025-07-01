use crate::common::constants::{
    ONE_RESOURCE_NO_CHANGE, ONE_RESOURCE_NOOP, ONE_RESOURCE_ONE_CHANGE, ONE_RESOURCE_ONE_ERROR,
    PROTECTED_DIRS,
};
use crate::common::types::{ApplySummary, Changes, Opts};
use crate::common::users_and_groups;
use camino::Utf8PathBuf;
use nix::unistd::{Gid, Uid};
use serde::Deserialize;
use std::fs;
use std::os::unix::fs::MetadataExt;

// THINGS TO KNOW / THINGS TO DO.
// Creating a directory is `mkdir -p` style.
// You can only define users and groups by their names. UIDs/GIDs do not work.

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct GurpDirectoryEnsure {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(rename = "name")]
    pub path: Utf8PathBuf,
    #[serde(flatten)]
    pub desired_state: DesiredDirectoryState,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct DesiredDirectoryState {
    pub group: String,
    pub mode: String,
    pub owner: String,
}

#[derive(Debug, PartialEq, Eq)]
pub struct DirectoryState {
    pub gid: Gid,
    pub mode: String,
    pub uid: Uid,
}

#[derive(Deserialize, Debug)]
pub struct GurpDirectoryRemove {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(rename = "name")]
    pub path: Utf8PathBuf,
}

impl GurpDirectoryEnsure {
    pub fn apply(&self, opts: &Opts) -> anyhow::Result<ApplySummary> {
        if !self.path.exists() {
            tracing::info!("creating directory: {}", self.path);

            if opts.noop {
                return Ok(ONE_RESOURCE_NOOP);
            }

            fs::create_dir_all(&self.path)?;
        }

        let current = self.current_state()?;
        let desired = DirectoryState {
            uid: users_and_groups::owner_from(&self.desired_state.owner)?,
            gid: users_and_groups::group_from(&self.desired_state.group)?,
            mode: self.desired_state.mode.clone(),
        };

        let changes = self.changes(&current, &desired);

        if changes.is_empty() {
            tracing::info!("no change: {}", self.path);
            return Ok(ONE_RESOURCE_NO_CHANGE);
        }

        if changes.contains(&"group") || changes.contains(&"owner") {
            tracing::info!(
                "change owner:group: {} {}:{} -> {}:{}",
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

    fn changes<'a>(&self, current: &DirectoryState, desired: &DirectoryState) -> Changes<'a> {
        let mut to_change = Vec::new();

        if current.gid != desired.gid {
            to_change.push("group");
        }

        if current.uid != desired.uid {
            to_change.push("owner");
        }

        if current.mode != desired.mode {
            to_change.push("mode");
        }

        if !to_change.is_empty() {
            tracing::debug!("to change for {}: {}", self.path, to_change.join(", "));
        }
        to_change
    }

    fn current_state(&self) -> anyhow::Result<DirectoryState> {
        tracing::debug!("getting state: {}", &self.path);
        let path = &self.path;
        let metadata = fs::metadata(path)?;

        Ok(DirectoryState {
            uid: metadata.uid().into(),
            gid: metadata.gid().into(),
            mode: format!("{:04o}", metadata.mode() & 0o777).to_owned(),
        })
    }
}

impl GurpDirectoryRemove {
    pub fn apply(&self, opts: &Opts) -> anyhow::Result<ApplySummary> {
        if self.path.exists() {
            if PROTECTED_DIRS.contains(&self.path) {
                tracing::warn!("protected resource: {}", self.path);
                return Ok(ONE_RESOURCE_ONE_ERROR);
            }

            tracing::info!("removing directory: {}", self.path);

            if opts.noop {
                Ok(ONE_RESOURCE_NOOP)
            } else {
                fs::remove_dir_all(&self.path)?;
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
    use crate::test_utils::spec_helper::{defopts, defopts_noop, janet2json, my_group, my_user};
    use assert_fs::TempDir;
    use assert_fs::prelude::*;
    use indoc::formatdoc;
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn test_directory_ensure_apply_noop() {
        let temp = TempDir::new().unwrap();
        let dir = Utf8PathBuf::from_path_buf(temp.child("test_directory").to_path_buf()).unwrap();
        let json_def = janet2json(&format!("(directory/ensure \"{}\")", dir));
        assert!(!dir.exists());
        let sut: GurpDirectoryEnsure = serde_json::from_str(&json_def).unwrap();
        assert_eq!(ONE_RESOURCE_NOOP, sut.apply(&defopts_noop()).unwrap());
        assert!(!dir.exists());
    }

    #[test]
    fn test_directory_ensure_already_exists() {
        let temp = TempDir::new().unwrap();
        let dir = temp.child("test_directory");
        dir.create_dir_all().unwrap();
        fs::set_permissions(dir.path(), fs::Permissions::from_mode(0o750)).unwrap();

        let dir_path = Utf8PathBuf::from_path_buf(dir.to_path_buf()).unwrap();
        assert!(dir_path.exists());

        let json_def = janet2json(&formatdoc! {"
            (directory/ensure \"{}\"
                :mode \"0750\"
                :owner \"{}\"
                :group \"{}\")
            ",
            dir_path,
            my_user(),
            my_group(),
        });
        let sut: GurpDirectoryEnsure = serde_json::from_str(&json_def).unwrap();
        assert_eq!(ONE_RESOURCE_NO_CHANGE, sut.apply(&defopts_noop()).unwrap());
        assert!(dir_path.exists());
    }

    #[test]
    fn test_directory_ensure_change_mode() {
        let temp = TempDir::new().unwrap();
        let dir = temp.child("test_directory");
        dir.create_dir_all().unwrap();
        fs::set_permissions(dir.path(), fs::Permissions::from_mode(0o750)).unwrap();

        let dir_path = Utf8PathBuf::from_path_buf(dir.to_path_buf()).unwrap();
        assert!(dir_path.exists());

        let json_def = janet2json(&formatdoc! {"
            (directory/ensure \"{}\"
                :mode \"0775\"
                :owner \"{}\"
                :group \"{}\")
            ",
            dir_path,
            my_user(),
            my_group(),
        });

        let sut: GurpDirectoryEnsure = serde_json::from_str(&json_def).unwrap();
        assert_eq!(ONE_RESOURCE_ONE_CHANGE, sut.apply(&defopts_noop()).unwrap());
        assert!(dir_path.exists());
        let metadata = fs::metadata(dir_path).unwrap();
        assert_eq!(metadata.permissions().mode() & 0o777, 0o775);
    }

    #[test]
    fn test_directory_remove_apply_does_not_exist() {
        let json_def = janet2json(r#"(directory/remove "/no/such/dir")"#);
        let sut: GurpDirectoryRemove = serde_json::from_str(&json_def).unwrap();
        assert_eq!(ONE_RESOURCE_NO_CHANGE, sut.apply(&defopts()).unwrap());
    }

    #[test]
    fn test_directory_remove_apply_not_allowed() {
        let json_def = janet2json(r#"(directory/remove "/usr")"#);
        let sut: GurpDirectoryRemove = serde_json::from_str(&json_def).unwrap();
        assert_eq!(ONE_RESOURCE_ONE_ERROR, sut.apply(&defopts()).unwrap());
    }

    #[test]
    fn test_directory_remove_apply_works() {
        let temp = TempDir::new().unwrap();
        let dir = temp.child("test_directory");
        dir.create_dir_all().unwrap();

        let json_def = janet2json(&format!("(directory/remove \"{}\")", dir.to_string_lossy()));
        let sut: GurpDirectoryRemove = serde_json::from_str(&json_def).unwrap();

        assert!(dir.exists());
        assert_eq!(ONE_RESOURCE_ONE_CHANGE, sut.apply(&defopts()).unwrap());
        assert!(!dir.exists());
    }

    #[test]
    fn test_directory_remove_apply_noop() {
        let temp = TempDir::new().unwrap();
        let dir = temp.child("test_directory");
        dir.create_dir_all().unwrap();

        let json_def = janet2json(&format!("(directory/remove \"{}\")", dir.to_string_lossy()));
        let sut: GurpDirectoryRemove = serde_json::from_str(&json_def).unwrap();

        assert!(dir.exists());
        assert_eq!(ONE_RESOURCE_NOOP, sut.apply(&defopts_noop()).unwrap());
        assert!(dir.exists());
    }
}
