use crate::common::constants::{
    ONE_RESOURCE_NO_CHANGE, ONE_RESOURCE_NOOP, ONE_RESOURCE_ONE_CHANGE, ONE_RESOURCE_ONE_ERROR,
    PROTECTED_DIRS,
};
use crate::common::output::Output;
use crate::common::traits::Apply;
use crate::common::types::{Action, ApplySummary, Changes, Opts, Resource};
use crate::common::users_and_groups;
use crate::utils::janet_helpers::{self, JanetExt, JanetStructExt};
use camino::Utf8PathBuf;
use janetrs::{Janet, JanetArray};
use nix::unistd::{Gid, Uid};
use paste::paste;
use std::fs;
use std::os::unix::fs::MetadataExt;

// THINGS TO KNOW / THINGS TO DO.
// Creating a directory is `mkdir -p` style.
// You can only define users and groups by their names. UIDs/GIDs do not work.

#[derive(Debug, PartialEq, Eq)]
pub struct GurpDirectory {
    pub action: Action,
    pub exists: bool,
    pub id: String,
    pub name: Utf8PathBuf, // The Path
    pub desired_state: Option<DirectoryState>,
    pub doer: String,
}

#[derive(Debug, PartialEq, Eq)]
pub struct DirectoryState {
    pub gid: Gid,
    pub mode: String,
    pub uid: Uid,
}

impl TryFrom<&Janet> for GurpDirectory {
    type Error = anyhow::Error;

    fn try_from(value: &Janet) -> anyhow::Result<Self> {
        let data = value.extract_struct()?;
        let name = data.get_field_pathbuf("name")?;
        let exists = name.exists();
        let action = janet_helpers::action_as_enum(&data)?;

        let state = match action {
            Action::Ensure => Some(DirectoryState {
                gid: users_and_groups::group_from(&data.get_field_string("group")?)?,
                mode: data.get_field_string("mode")?,
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
            doer: "directory".to_owned(),
        })
    }
}

crate::unpack_fn!(ensure_list, Directory, GurpDirectory);
crate::unpack_fn!(remove_list, Directory, GurpDirectory);
crate::impl_apply!(GurpDirectory);

impl GurpDirectory {
    fn apply_ensure(&self, opts: &Opts, output: &Output) -> anyhow::Result<ApplySummary> {
        if !self.exists {
            output.creating(&self.name);

            if opts.noop {
                return Ok(ONE_RESOURCE_ONE_CHANGE);
            }

            fs::create_dir_all(&self.name)?;
        }

        let path = &self.name;
        let current = self.current_state()?;
        let desired = self.desired_state.as_ref().unwrap();
        let changes = self.changes(&current, desired);

        if changes.is_empty() {
            output.no_change(&self.name);
            return Ok(ONE_RESOURCE_NO_CHANGE);
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
            if PROTECTED_DIRS.contains(&self.name) {
                output.protected(&self.name);
                return Ok(ONE_RESOURCE_ONE_ERROR);
            }

            output.removing(&self.name);

            if opts.noop {
                Ok(ONE_RESOURCE_NOOP)
            } else {
                fs::remove_dir_all(&self.name)?;
                Ok(ONE_RESOURCE_ONE_CHANGE)
            }
        } else {
            output.not_present(&self.name);
            Ok(ONE_RESOURCE_NO_CHANGE)
        }
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

        to_change
    }

    fn current_state(&self) -> anyhow::Result<DirectoryState> {
        let path = &self.name;
        let metadata = fs::metadata(path)?;

        Ok(DirectoryState {
            uid: metadata.uid().into(),
            gid: metadata.gid().into(),
            mode: format!("{:04o}", metadata.mode() & 0o777).to_owned(),
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils::spec_helper::{defopts, defopts_noop, init_janet};
    use assert_fs::TempDir;
    use assert_fs::prelude::*;
    use camino::Utf8PathBuf;

    #[test]
    fn test_directory_remove_apply_does_not_exist() {
        let dir_does_not_exist = GurpDirectory {
            name: Utf8PathBuf::from("/does/not/exist/dir-to-test"),
            exists: false,
            id: "/test-role/directory/dir-to-test".to_owned(),
            action: Action::Remove,
            desired_state: None,
            doer: "directory".to_owned(),
        };

        assert_eq!(
            ONE_RESOURCE_NO_CHANGE,
            dir_does_not_exist.apply(&defopts()).unwrap()
        );
    }

    #[test]
    fn test_directory_remove_apply_not_allowed() {
        let disallowed_dir = GurpDirectory {
            name: Utf8PathBuf::from("/"),
            id: "/test-role/directory/root".to_owned(),
            exists: true,
            action: Action::Remove,
            desired_state: None,
            doer: "directory".to_owned(),
        };

        assert_eq!(
            ONE_RESOURCE_ONE_ERROR,
            disallowed_dir.apply(&defopts()).unwrap()
        );
    }

    #[test]
    fn test_directory_remove_apply_works() {
        let temp = TempDir::new().unwrap();
        let dir = temp.child("test_directory");
        dir.create_dir_all().unwrap();

        let test_dir = GurpDirectory {
            name: Utf8PathBuf::from_path_buf(dir.to_path_buf()).unwrap(),
            id: "/test-role/directory/test_directory".to_owned(),
            exists: true,
            action: Action::Remove,
            desired_state: None,
            doer: "directory".to_owned(),
        };

        assert!(dir.exists());
        assert_eq!(ONE_RESOURCE_ONE_CHANGE, test_dir.apply(&defopts()).unwrap());
        assert!(!dir.exists());
    }

    #[test]
    fn test_directory_remove_apply_noop() {
        let temp = TempDir::new().unwrap();
        let dir = temp.child("test_directory");
        dir.create_dir_all().unwrap();

        let test_dir = GurpDirectory {
            name: Utf8PathBuf::from_path_buf(dir.to_path_buf()).unwrap(),
            id: "/test-role/directory/test_directory".to_owned(),
            exists: true,
            action: Action::Remove,
            desired_state: None,
            doer: "directory".to_owned(),
        };

        assert!(dir.exists());
        assert_eq!(
            ONE_RESOURCE_NO_CHANGE,
            test_dir.apply(&defopts_noop()).unwrap()
        );
        assert!(dir.exists());
    }

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
                    gid: 2.into(),
                    mode: "0755".to_owned(),
                    uid: 0.into(),
                }),
                doer: "directory".to_owned(),
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
                doer: "directory".to_owned(),
            },
            GurpDirectory::try_from(&example_dir_remove).unwrap()
        );
    }
}
