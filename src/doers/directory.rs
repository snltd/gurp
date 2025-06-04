use crate::doers::constants::{
    ONE_RESOURCE_NO_CHANGE, ONE_RESOURCE_NOOP, ONE_RESOURCE_ONE_CHANGE, ONE_RESOURCE_ONE_ERROR,
    PROTECTED_DIRS,
};
use crate::doers::types::{Action, Apply, ApplySummary, Changes, Ensure, Remove};
use crate::utils::janet_helpers::{self, JanetExt, JanetStructExt};
use crate::utils::types::Opts;
use crate::{change, creating, debug, info, no_change, not_there, verbose};
use anyhow::{Context, anyhow};
use camino::Utf8PathBuf;
use colored::Colorize;
use janetrs::{Janet, JanetArray};
use nix::unistd::{Gid, Group, Uid, User};
use std::fs;
use std::os::unix;
use std::os::unix::fs::MetadataExt;
use std::os::unix::fs::PermissionsExt;

// THINGS TO KNOW / THINGS TO DO.
// Creating a directory is `mkdir -p` style.
// You can only define users and groups by their names. UIDs/GIDs do not work.

pub struct GurpDirectory {
    pub action: Action,
    pub exists: bool,
    pub id: String,
    pub name: Utf8PathBuf, // The Path
    pub desired_state: Option<DirectoryState>,
    pub doer: String,
}

#[derive(Debug, PartialEq)]
pub struct DirectoryState {
    pub group: String,
    pub mode: String,
    pub owner: String,
}

impl TryFrom<&Janet> for GurpDirectory {
    type Error = anyhow::Error;

    fn try_from(value: &Janet) -> anyhow::Result<Self> {
        let data = value.extract_struct()?;
        let path = data.get_field_pathbuf("path")?;
        let exists = path.exists();
        let action = janet_helpers::action_as_enum(&data)?;

        let state = match action {
            Action::Ensure => Some(DirectoryState {
                group: data.get_field_string("group")?,
                mode: data.get_field_string("mode")?,
                owner: data.get_field_string("owner")?,
            }),
            Action::Remove => None,
        };

        Ok(GurpDirectory {
            action,
            doer: "directory".to_owned(),
            exists,
            id: data.get_field_string("_id")?,
            name: data.get_field_pathbuf("name")?,
            desired_state: state,
        })
    }
}

pub fn unpack_ensure_list(resource_list: &JanetArray) -> anyhow::Result<Vec<Ensure>> {
    resource_list
        .iter()
        .map(|r| {
            let dir = GurpDirectory::try_from(r)?;
            Ok(Ensure::Directory(dir))
        })
        .collect()
}

pub fn unpack_remove_list(resource_list: &JanetArray) -> anyhow::Result<Vec<Remove>> {
    resource_list
        .iter()
        .map(|r| {
            let dir = GurpDirectory::try_from(r)?;
            Ok(Remove::Directory(dir))
        })
        .collect()
}

impl GurpDirectory {
    fn apply(&self, opts: &Opts) -> anyhow::Result<ApplySummary> {
        match self.action {
            Action::Ensure => self.apply_ensure(opts),
            Action::Remove => self.apply_remove(opts),
        }
    }

    fn apply_remove(&self, opts: &Opts) -> anyhow::Result<ApplySummary> {
        if self.exists {
            if PROTECTED_DIRS.contains(&self.name) {
                eprintln!("Not allowed to remove {}", self.name);
                return Ok(ONE_RESOURCE_ONE_ERROR);
            }

            // info!(opts, "directory {}: REMOVE", self.name);

            if opts.noop {
                Ok(ONE_RESOURCE_NOOP)
            } else {
                fs::remove_dir_all(&self.name)?;
                Ok(ONE_RESOURCE_ONE_CHANGE)
            }
        } else {
            // not_there!(opts);
            Ok(ONE_RESOURCE_NO_CHANGE)
        }
    }

    fn apply_ensure(&self, opts: &Opts) -> anyhow::Result<ApplySummary> {
        if !self.exists {
            // creating!();

            if opts.noop {
                return Ok(ONE_RESOURCE_ONE_CHANGE);
            }

            fs::create_dir_all(&self.name)?;
        }

        let path = self.name;
        let current = self.current_state()?;
        let desired = self.desired_state.unwrap();
        let changes = self.changes(&current, &desired);

        if changes.is_empty() {
            // no_change!(opts);
            return Ok(ONE_RESOURCE_NO_CHANGE);
        }

        let final_owner = if changes.contains(&"owner") {
            // change!(self, current_state, owner);
            desired.owner
        } else {
            current.owner
        };

        let final_group = if changes.contains(&"group") {
            // change!(self, current_state, group);
            desired.group
        } else {
            current.group
        };

        if changes.contains(&"group") || changes.contains(&"owner") {
            let user = User::from_name(&final_owner)?
                .ok_or_else(|| anyhow::anyhow!("No such user '{}'", final_owner))?;
            let group = Group::from_name(&final_group)?
                .ok_or_else(|| anyhow::anyhow!("No such group '{}'", final_group))?;

            unix::fs::chown(&path, Some(user.uid.as_raw()), Some(group.gid.as_raw()))?;
        }

        if changes.contains(&"mode") {
            // change!(self, current_state, mode);
            let mode = u32::from_str_radix(&desired.mode, 8)?;
            fs::set_permissions(path, fs::Permissions::from_mode(mode))?;
        }

        Ok(ONE_RESOURCE_ONE_CHANGE)
    }

    fn changes<'a>(&self, current: &DirectoryState, desired: &DirectoryState) -> Changes<'a> {
        let mut to_change = Vec::new();

        if current.group != desired.group {
            to_change.push("group");
        }

        if current.owner != desired.owner {
            to_change.push("owner");
        }

        if current.mode != desired.mode {
            to_change.push("mode");
        }

        to_change
    }

    fn current_state(&self) -> anyhow::Result<DirectoryState> {
        let path = &self.name;
        let metadata = fs::metadata(&path)?;

        // TODO deal with numeric and string users and groups
        //
        let mode = format!("{:04o}", metadata.mode() & 0o777);
        let uid = metadata.uid();
        let gid = metadata.gid();

        let owner = User::from_uid(Uid::from_raw(uid))?
            .context(format!("cannot get owner for directory {}", path))?
            .name;

        let group = Group::from_gid(Gid::from_raw(gid))?
            .context(format!("cannot get group for directory {}", path))?
            .name;

        Ok(DirectoryState {
            group: group.to_owned(),
            owner: owner.to_owned(),
            mode: mode.to_owned(),
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
        let dir_does_not_exist = DirectoryToRemove {
            name: Utf8PathBuf::from("/does/not/exist/dir-to-test"),
            id: "/test-role/directory/dir-to-test".to_owned(),
        };

        assert_eq!(
            ONE_RESOURCE_NO_CHANGE,
            dir_does_not_exist.apply(&defopts()).unwrap()
        );
    }

    #[test]
    fn test_directory_remove_apply_not_allowed() {
        let disallowed_dir = DirectoryToRemove {
            name: "root".to_owned(),
            id: "/test-role/directory/root".to_owned(),
            path: Utf8PathBuf::from("/"),
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

        let test_dir = DirectoryToRemove {
            name: "tester".to_owned(),
            id: "/test-role/directory/test_directory".to_owned(),
            path: Utf8PathBuf::from_path_buf(dir.to_path_buf()).unwrap(),
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

        let test_dir = DirectoryToRemove {
            name: "tester".to_owned(),
            id: "/test-role/directory/test_directory".to_owned(),
            path: Utf8PathBuf::from_path_buf(dir.to_path_buf()).unwrap(),
        };

        assert!(dir.exists());
        assert_eq!(
            ONE_RESOURCE_NO_CHANGE,
            test_dir.apply(&defopts_noop()).unwrap()
        );
        assert!(dir.exists());
    }

    #[test]
    fn test_directory_ensure_state() {
        let temp = TempDir::new().unwrap();
        let dir = temp.child("test_directory");
        dir.create_dir_all().unwrap();

        let dir_exists = DirectoryToEnsure {
            id: "/test-role/directory/test_directory".to_owned(),
            group: "sysadmin".to_owned(),
            mode: "0755".to_owned(),
            name: "test_directory".to_owned(),
            owner: "rob".to_owned(),
            path: Utf8PathBuf::from_path_buf(dir.to_path_buf()).unwrap(),
        };

        let result_option = dir_exists.state().unwrap();
        let result = result_option.unwrap();
        assert_eq!("0755".to_owned(), result.mode);

        let dir_does_not_exist = DirectoryToEnsure {
            id: "/test-role/directory/test_directory".to_owned(),
            group: "sysadmin".to_owned(),
            mode: "0755".to_owned(),
            name: "test_directory".to_owned(),
            owner: "rob".to_owned(),
            path: Utf8PathBuf::from("/no/such/test_directory"),
        };

        assert!(dir_does_not_exist.state().unwrap().is_none());
    }

    #[test]
    fn test_unpack_ensure_directory() {
        init_janet();

        let example_dir_ensure = Janet::wrap(janetrs::structs! {
            ":_id" => "/test-role/directory/test_directory",
            ":action" => "ensure",
            ":group" => "sysadmin",
            ":mode" => "0755",
            ":name" => "test_directory",
            ":owner" => "rob",
            ":path" => "/tmp/merp",
        });

        let expected_ensure = DirectoryToEnsure {
            id: "/test-role/directory/test_directory".to_owned(),
            group: "sysadmin".to_owned(),
            mode: "0755".to_owned(),
            name: "test_directory".to_owned(),
            owner: "rob".to_owned(),
            path: Utf8PathBuf::from("/tmp/merp"),
        };

        assert_eq!(
            expected_ensure,
            DirectoryToEnsure::try_from(&example_dir_ensure).unwrap()
        );
    }

    #[test]
    fn test_unpack_remove_directory() {
        init_janet();
        let example_dir_remove = Janet::wrap(janetrs::structs! {
            ":_id" => "/test-role/directory/merp",
            ":name" => "merp",
            ":action" => "remove",
            ":path" => "/tmp/merp",
        });

        let expected_remove = DirectoryToRemove {
            name: "merp".to_owned(),
            id: "/test-role/directory/merp".to_owned(),
            path: Utf8PathBuf::from("/tmp/merp"),
        };

        assert_eq!(
            expected_remove,
            DirectoryToRemove::try_from(&example_dir_remove).unwrap()
        );
    }
}
