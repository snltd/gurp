use crate::doers::constants::{
    ONE_RESOURCE_NO_CHANGE, ONE_RESOURCE_NOOP, ONE_RESOURCE_ONE_CHANGE, ONE_RESOURCE_ONE_ERROR,
};
use crate::doers::types::{Apply, ApplySummary, Changes, Ensure, HasId, Remove};
use crate::utils::janet_helpers::{JanetExt, JanetStructExt};
use crate::utils::types::Opts;
use crate::{debug, info, verbose};
use anyhow::Context;
use camino::Utf8PathBuf;
use colored::Colorize;
use janetrs::{Janet, JanetArray};
use nix::unistd::{Gid, Group, Uid, User};
use std::fs;
use std::os::unix;
use std::os::unix::fs::MetadataExt;
use std::os::unix::fs::PermissionsExt;
use std::sync::LazyLock;

// THINGS TO KNOW / THINGS TO DO.
// Creating a directory is `mkdir -p` style.
// You can only define users and groups by their names. UIDs/GIDs do not work.

static NOT_ALLOWED_TO_REMOVE: LazyLock<Vec<Utf8PathBuf>> = LazyLock::new(|| {
    vec![
        Utf8PathBuf::from("/"),
        Utf8PathBuf::from("/bin"),
        Utf8PathBuf::from("/etc"),
        Utf8PathBuf::from("/lib"),
        Utf8PathBuf::from("/sbin"),
        Utf8PathBuf::from("/usr"),
        Utf8PathBuf::from("/usr/lib"),
    ]
});

#[derive(Debug, PartialEq)]
pub struct DirectoryToEnsure {
    pub id: String,
    pub group: String,
    pub mode: String,
    pub name: String,
    pub owner: String,
    pub path: Utf8PathBuf,
}

#[derive(Debug, PartialEq)]
pub struct DirectoryToRemove {
    pub id: String,
    pub path: Utf8PathBuf,
    pub name: String,
}

#[derive(Debug, PartialEq)]
pub struct DirectoryEnsureState {
    pub group: String,
    pub mode: String,
    pub owner: String,
    pub name: String,
}

#[derive(Debug, PartialEq)]
pub struct DirectoryRemoveState {
    pub exists: bool,
}

impl TryFrom<&Janet> for DirectoryToEnsure {
    type Error = anyhow::Error;

    fn try_from(value: &Janet) -> anyhow::Result<DirectoryToEnsure> {
        let data = value.extract_struct()?;

        Ok(DirectoryToEnsure {
            id: data.get_field_string("_id")?,
            name: data.get_field_string("name")?,
            group: data.get_field_string("group")?,
            owner: data.get_field_string("owner")?,
            mode: data.get_field_string("mode")?,
            path: data.get_field_pathbuf("path")?,
        })
    }
}

impl HasId for DirectoryToEnsure {
    fn id(&self) -> &str {
        &self.id
    }
}

impl HasId for DirectoryToRemove {
    fn id(&self) -> &str {
        &self.id
    }
}

impl TryFrom<&Janet> for DirectoryToRemove {
    type Error = anyhow::Error;

    fn try_from(value: &Janet) -> anyhow::Result<DirectoryToRemove> {
        let data = value.extract_struct()?;

        Ok(DirectoryToRemove {
            name: data.get_field_string("name")?,
            id: data.get_field_string("_id")?,
            path: data.get_field_pathbuf("path")?,
        })
    }
}

pub fn unpack_ensure_list(resource_list: &JanetArray) -> anyhow::Result<Vec<Ensure>> {
    resource_list
        .iter()
        .map(|r| {
            let dir = DirectoryToEnsure::try_from(r)?;
            Ok(Ensure::Directory(dir))
        })
        .collect()
}

pub fn unpack_remove_list(resource_list: &JanetArray) -> anyhow::Result<Vec<Remove>> {
    resource_list
        .iter()
        .map(|r| {
            let dir = DirectoryToRemove::try_from(r)?;
            Ok(Remove::Directory(dir))
        })
        .collect()
}

fn diff_states<'a>(current: &DirectoryEnsureState, desired: &DirectoryEnsureState) -> Changes<'a> {
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

impl DirectoryToEnsure {
    fn state(&self) -> anyhow::Result<Option<DirectoryEnsureState>> {
        directory_state(&self.path, &self.name)
    }

    fn desired_state(&self) -> DirectoryEnsureState {
        DirectoryEnsureState {
            name: self.name.clone(),
            group: self.group.clone(),
            owner: self.owner.clone(),
            mode: self.mode.clone(),
        }
    }
}

impl Apply for DirectoryToRemove {
    fn apply(&self, opts: &Opts) -> anyhow::Result<ApplySummary> {
        if !self.path.exists() {
            debug!(
                opts,
                "directory {} [{}]: {} does not exist", self.name, self.id, self.path
            );
            return Ok(ONE_RESOURCE_NO_CHANGE);
        }

        if NOT_ALLOWED_TO_REMOVE.contains(&self.path) {
            eprintln!("Not allowed to remove {}", self.path);
            return Ok(ONE_RESOURCE_ONE_ERROR);
        }

        info!(opts, "directory {}: REMOVE", self.name);

        if opts.noop {
            Ok(ONE_RESOURCE_NOOP)
        } else {
            fs::remove_dir_all(&self.path)?;
            Ok(ONE_RESOURCE_ONE_CHANGE)
        }
    }
}

impl Apply for DirectoryToEnsure {
    fn apply(&self, opts: &Opts) -> anyhow::Result<ApplySummary> {
        let current_state = if self.path.exists() {
            self.state()
        } else {
            info!(opts, "Creating directory {} [{}]", self.path, self.name);

            if opts.noop {
                return Ok(ONE_RESOURCE_ONE_CHANGE);
            }

            fs::create_dir_all(&self.path)?;
            directory_state(&self.path, &self.name)
        }?
        .context(format!("Cannot get state of {}", self.path))?;

        let desired_state = self.desired_state();

        let changes = diff_states(&current_state, &desired_state);

        if changes.is_empty() {
            verbose!(
                opts,
                "directory: {} [{}] : no change required",
                self.path,
                self.name
            );
            return Ok(ONE_RESOURCE_NO_CHANGE);
        }

        let final_owner = if changes.contains(&"owner") {
            info!(
                opts,
                "directory: {} [{}] : owner {} -> {}",
                self.path,
                self.name,
                current_state.owner,
                desired_state.owner
            );
            desired_state.owner
        } else {
            current_state.owner
        };

        let final_group = if changes.contains(&"group") {
            info!(
                opts,
                "directory: {} [{}] : group {} -> {}",
                self.path,
                self.name,
                current_state.group,
                desired_state.group
            );
            desired_state.group
        } else {
            current_state.group
        };

        if changes.contains(&"group") || changes.contains(&"owner") {
            let user = User::from_name(&final_owner)?
                .ok_or_else(|| anyhow::anyhow!("No such user '{}'", final_owner))?;
            let group = Group::from_name(&final_group)?
                .ok_or_else(|| anyhow::anyhow!("No such group '{}'", final_group))?;

            unix::fs::chown(
                &self.path,
                Some(user.uid.as_raw()),
                Some(group.gid.as_raw()),
            )?;
        }

        if changes.contains(&"mode") {
            info!(
                opts,
                "directory: {} [{}] : mode {} -> {}",
                self.path,
                self.name,
                current_state.mode,
                desired_state.mode
            );

            let mode = u32::from_str_radix(&self.mode, 8)?;
            fs::set_permissions(&self.path, fs::Permissions::from_mode(mode))?;
        }

        Ok(ONE_RESOURCE_ONE_CHANGE)
    }
}

fn directory_state(path: &Utf8PathBuf, name: &str) -> anyhow::Result<Option<DirectoryEnsureState>> {
    if path.exists() {
        let metadata = fs::metadata(path)?;

        // TODO deal with numeric and string users and groups
        //
        let mode = format!("{:04o}", metadata.mode() & 0o777);
        let uid = metadata.uid();
        let gid = metadata.gid();

        let owner = User::from_uid(Uid::from_raw(uid))?
            .context("cannot get directory user")?
            .name;
        let group = Group::from_gid(Gid::from_raw(gid))?
            .context("cannot get directory group")?
            .name;

        Ok(Some(DirectoryEnsureState {
            name: name.to_owned(),
            group: group.to_owned(),
            owner: owner.to_owned(),
            mode: mode.to_owned(),
        }))
    } else {
        Ok(None)
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
            name: "tester".to_owned(),
            id: "/test-role/directory/dir-to-test".to_owned(),
            path: Utf8PathBuf::from("/does/not/exist/dir-to-test"),
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
