use crate::common::constants::{
    ONE_RESOURCE_NO_CHANGE, ONE_RESOURCE_NOOP, ONE_RESOURCE_ONE_CHANGE, ONE_RESOURCE_ONE_ERROR,
    PROTECTED_DIRS,
};
use crate::common::types::{ApplyContext, ApplySummary, Changes, Opts};
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
    pub name: Utf8PathBuf, // The Path
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
    pub name: Utf8PathBuf, // The Path
}

/*
impl TryFrom<&Janet> for GurpDirectory {
    type Error = anyhow::Error;

    fn try_from(value: &Janet) -> anyhow::Result<Self> {
        let data = value.extract_struct()?;
        let name = data.get_field_pathbuf("name")?;
        let exists = name.exists();
        let action = janet_helpers::action_as_enum(&data)?;

        let state = match action {
            Action::Ensure => Some(DirectoryState {
                mode: data.get_field_string("mode")?,
                gid: users_and_groups::group_from(&data.get_field_string("group")?)?,
                uid: users_and_groups::owner_from(&data.get_field_string("owner")?)?,
            }),
            Action::Remove => None,
        };

        Ok(GurpDirectory {
            action,
            exists,
            id: data.get_field_string("_id")?,
            name: data.get_field_pathbuf("name")?,
            desired_state: state,
        })
    }
}

crate::unpack_fn!(ensure_list, Directory, GurpDirectory);
crate::unpack_fn!(remove_list, Directory, GurpDirectory);
crate::impl_apply!(GurpDirectory);
*/

impl GurpDirectoryEnsure {
    pub fn apply(
        &self,
        _apply_context: &ApplyContext,
        opts: &Opts,
    ) -> anyhow::Result<ApplySummary> {
        if !self.name.exists() {
            tracing::info!("creating directory: {}", self.name);

            if opts.noop {
                return Ok(ONE_RESOURCE_ONE_CHANGE);
            }

            fs::create_dir_all(&self.name)?;
        }

        let path = &self.name;
        let current = self.current_state()?;
        let desired = DirectoryState {
            uid: users_and_groups::owner_from(&self.desired_state.owner)?,
            gid: users_and_groups::group_from(&self.desired_state.group)?,
            mode: self.desired_state.mode.clone(),
        };

        let changes = self.changes(&current, &desired);

        if changes.is_empty() {
            tracing::info!("no change: {}", self.name);
            return Ok(ONE_RESOURCE_NO_CHANGE);
        }

        if changes.contains(&"group") || changes.contains(&"owner") {
            tracing::info!(
                "change owner:group: {} {}:{} -> {}:{}",
                self.name,
                current.uid,
                current.gid,
                desired.uid,
                desired.gid
            );
            users_and_groups::set_user(path, desired.uid, desired.gid)?;
        }

        if changes.contains(&"mode") {
            tracing::info!(
                "change mode: {} {} -> {}",
                self.name,
                current.mode,
                desired.mode,
            );
            users_and_groups::set_mode(path, &current.mode, &desired.mode)?;
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
            tracing::debug!("to change for {}: {}", self.name, to_change.join(", "));
        }
        to_change
    }

    fn current_state(&self) -> anyhow::Result<DirectoryState> {
        tracing::debug!("getting state: {}", &self.name);
        let path = &self.name;
        let metadata = fs::metadata(path)?;

        Ok(DirectoryState {
            uid: metadata.uid().into(),
            gid: metadata.gid().into(),
            mode: format!("{:04o}", metadata.mode() & 0o777).to_owned(),
        })
    }
}

impl GurpDirectoryRemove {
    pub fn apply(
        &self,
        _apply_context: &ApplyContext,
        opts: &Opts,
    ) -> anyhow::Result<ApplySummary> {
        if self.name.exists() {
            if PROTECTED_DIRS.contains(&self.name) {
                tracing::warn!("protected resource: {}", self.name);
                return Ok(ONE_RESOURCE_ONE_ERROR);
            }

            tracing::info!("removing directory: {}", self.name);

            if opts.noop {
                Ok(ONE_RESOURCE_NOOP)
            } else {
                fs::remove_dir_all(&self.name)?;
                Ok(ONE_RESOURCE_ONE_CHANGE)
            }
        } else {
            tracing::debug!("not present: {}", self.name);
            Ok(ONE_RESOURCE_NO_CHANGE)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils::spec_helper::{defcontext, defopts, defopts_noop, init_janet};
    use assert_fs::TempDir;
    use assert_fs::prelude::*;
    use camino::Utf8PathBuf;

    #[test]
    fn test_directory_remove_apply_does_not_exist() {
        let dir_does_not_exist = GurpDirectoryRemove {
            name: Utf8PathBuf::from("/does/not/exist/dir-to-test"),
            id: "/test-role/directory/dir-to-test".to_owned(),
        };

        assert_eq!(
            ONE_RESOURCE_NO_CHANGE,
            dir_does_not_exist.apply(&defcontext(), &defopts()).unwrap()
        );
    }

    #[test]
    fn test_directory_remove_apply_not_allowed() {
        let disallowed_dir = GurpDirectoryRemove {
            name: Utf8PathBuf::from("/"),
            id: "/test-role/directory/root".to_owned(),
        };

        assert_eq!(
            ONE_RESOURCE_ONE_ERROR,
            disallowed_dir.apply(&defcontext(), &defopts()).unwrap()
        );
    }

    #[test]
    fn test_directory_remove_apply_works() {
        let temp = TempDir::new().unwrap();
        let dir = temp.child("test_directory");
        dir.create_dir_all().unwrap();

        let test_dir = GurpDirectoryRemove {
            name: Utf8PathBuf::from_path_buf(dir.to_path_buf()).unwrap(),
            id: "/test-role/directory/test_directory".to_owned(),
        };

        assert!(dir.exists());
        assert_eq!(
            ONE_RESOURCE_ONE_CHANGE,
            test_dir.apply(&defcontext(), &defopts()).unwrap()
        );
        assert!(!dir.exists());
    }

    #[test]
    fn test_directory_remove_apply_noop() {
        let temp = TempDir::new().unwrap();
        let dir = temp.child("test_directory");
        dir.create_dir_all().unwrap();

        let test_dir = GurpDirectoryRemove {
            name: Utf8PathBuf::from_path_buf(dir.to_path_buf()).unwrap(),
            id: "/test-role/directory/test_directory".to_owned(),
        };

        assert!(dir.exists());
        assert_eq!(
            ONE_RESOURCE_NO_CHANGE,
            test_dir.apply(&defcontext(), &defopts_noop()).unwrap()
        );
        assert!(dir.exists());
    }

    #[cfg(not(target_os = "macos"))]
    #[test]
    fn test_unpack_ensure_directory() {
        init_janet();

        let example_dir_ensure = Janet::wrap(janetrs::structs! {
            ":_id" => "/test-role/directory/test_directory",
            ":action" => ":ensure",
            ":group" => "bin",
            ":mode" => "0755",
            ":owner" => "root",
            ":name" => "/tmp/merp",
        });

        assert_eq!(
            GurpDirectory {
                name: Utf8PathBuf::from("/tmp/merp"),
                id: "/test-role/directory/test_directory".to_owned(),
                exists: false,
                action: Action::Ensure,
                desired_state: Some(DirectoryState {
                    group: 2.into(),
                    mode: "0755".to_owned(),
                    owner: 0.into(),
                }),
            },
            GurpDirectory::try_from(&example_dir_ensure).unwrap()
        );
    }

    #[test]
    fn test_unpack_remove_directory() {
        init_janet();
        let example_dir_remove = Janet::wrap(janetrs::structs! {
            ":name" => "/tmp/merp",
            ":_id" => "/test-role/directory/merp",
            ":action" => ":remove",
        });

        assert_eq!(
            GurpDirectory {
                name: Utf8PathBuf::from("/tmp/merp"),
                id: "/test-role/directory/merp".to_owned(),
                exists: false,
                action: Action::Remove,
                desired_state: None,
            },
            GurpDirectory::try_from(&example_dir_remove).unwrap()
        );
    }
}
