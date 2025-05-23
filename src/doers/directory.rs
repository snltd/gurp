use crate::doers::types::{DirectoryEnsure, DirectoryRemove, DirectoryStateEnsure, Ensure, Remove};
use crate::utils::janet_helpers::{JanetExt, JanetStructExt};
use crate::utils::types::Opts;
use crate::{debug, info, verbose};
use anyhow::Context;
use camino::Utf8PathBuf;
use colored::Colorize;
use janetrs::{Janet, JanetArray};
use nix::unistd::{Gid, Group, Uid, User};
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::sync::LazyLock;

// THINGS TO KNOW / THINGS TO DO.
// References to other resources do not work yet, for this or any other resource type.
// Removing a directory ALWAYS removes all its contents.
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

impl TryFrom<&Janet> for DirectoryEnsure {
    type Error = anyhow::Error;

    fn try_from(value: &Janet) -> anyhow::Result<DirectoryEnsure> {
        let data = value.extract_struct()?;

        Ok(DirectoryEnsure {
            id: data.get_field_string("_id")?,
            name: data.get_field_string("name")?,
            group: data.get_field_string("group")?,
            owner: data.get_field_string("owner")?,
            mode: data.get_field_string("mode")?,
            path: data.get_field_pathbuf("path")?,
        })
    }
}

impl TryFrom<&Janet> for DirectoryRemove {
    type Error = anyhow::Error;

    fn try_from(value: &Janet) -> anyhow::Result<DirectoryRemove> {
        let data = value.extract_struct()?;

        Ok(DirectoryRemove {
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
            let dir = DirectoryEnsure::try_from(r)?;
            Ok(Ensure::Directory(dir))
        })
        .collect()
}

pub fn unpack_remove_list(resource_list: &JanetArray) -> anyhow::Result<Vec<Remove>> {
    resource_list
        .iter()
        .map(|r| {
            let dir = DirectoryRemove::try_from(r)?;
            Ok(Remove::Directory(dir))
        })
        .collect()
}

type Changes<'a> = Vec<&'a str>;

fn diff_states<'a>(current: DirectoryStateEnsure, desired: DirectoryStateEnsure) -> Changes<'a> {
    let mut to_change = Vec::new();

    if current.group != desired.group {
        to_change.push("group");
    }

    if current.owner != desired.owner {
        to_change.push("owner");
    }

    if current.mode != desired.owner {
        to_change.push("mode ");
    }

    to_change
}

impl DirectoryEnsure {
    fn state(&self) -> anyhow::Result<DirectoryStateEnsure> {
        directory_state(&self.path, &self.name)
    }

    fn desired_state(&self) -> anyhow::Result<DirectoryStateEnsure> {
        Ok(DirectoryStateEnsure {
            name: self.name.to_owned(),
            group: self.group.to_owned(),
            owner: self.owner.to_owned(),
            mode: self.mode.to_owned(),
        })
    }

    fn apply(&self, opts: &Opts) -> anyhow::Result<bool> {
        let current_state = if self.path.exists() {
            self.state()
        } else {
            fs::create_dir_all(&self.path)?;
            directory_state(&self.path, &self.name)
        }?;

        let desired_state = self.desired_state()?;

        let changes = diff_states(current_state, desired_state);

        if changes.is_empty() {
            verbose!(
                opts,
                "directory: {} [{}] : no change required",
                self.path,
                self.name
            );
            return Ok(false);
        }

        if changes.contains(&"owner") {
            info!(
                opts,
                "directory: {} [{}] : owner {} -> {}",
                self.path,
                self.name,
                current_state.owner,
                desired_state.owner
            );
        }

        if changes.contains(&"group") {
            info!(
                opts,
                "directory: {} [{}] : group {} -> {}",
                self.path,
                self.name,
                current_state.group,
                desired_state.group
            );
        }

        if changes.contains(&"mode") {
            info!(
                opts,
                "directory: {} [{}] : mode{} -> {}",
                self.path,
                self.name,
                current_state.mode,
                desired_state.mode
            );
        }

        Ok(true)
    }
}

fn directory_state(path: &Utf8PathBuf, name: &str) -> anyhow::Result<Option<DirectoryStateEnsure>> {
    if path.exists() {
        let metadata = fs::metadata(&path)?;

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

        Ok(Some(DirectoryStateEnsure {
            name: name.to_owned(),
            group: group.to_owned(),
            owner: owner.to_owned(),
            mode: mode.to_owned(),
        }))
    } else {
        Ok(None)
    }
}

impl DirectoryRemove {
    // We only care if it exists
    fn state(&self) -> bool {
        self.path.exists()
    }

    fn apply(&self, opts: &Opts) -> anyhow::Result<bool> {
        if !self.path.exists() {
            debug!(
                opts,
                "directory {} [{}]: {} does not exist", self.name, self.id, self.path
            );
            return Ok(false);
        }

        if NOT_ALLOWED_TO_REMOVE.contains(&self.path) {
            eprintln!("Not allowed to remove {}", self.path);
            return Ok(false);
        }

        verbose!(opts, "directory {}: REMOVE", self.name);

        if opts.noop {
            Ok(false)
        } else {
            fs::remove_dir_all(&self.path)?;
            Ok(true)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils::spec_helper::{defopts, defopts_noop, init_janet};
    use assert_fs::TempDir;
    use assert_fs::prelude::*;
    use camino::Utf8PathBuf;
    // use predicates::prelude;

    #[test]
    fn test_directory_ensure_apply_does_not_exist() {
        let temp = TempDir::new().unwrap();
        let expected_dir = temp.to_path_buf().join("test_directory");

        assert!(!expected_dir.exists());

        let required_dir = DirectoryEnsure {
            id: "/test-role/directory/test_directory".to_owned(),
            group: "sysadmin".to_owned(),
            mode: "0755".to_owned(),
            name: "test_directory".to_owned(),
            owner: "rob".to_owned(),
            path: Utf8PathBuf::from_path_buf(expected_dir).unwrap(),
        };

        required_dir.apply(&defopts());
    }

    #[test]
    fn test_directory_remove_apply_does_not_exist() {
        let dir_does_not_exist = DirectoryRemove {
            name: "tester".to_owned(),
            id: "/test-role/directory/dir-to-test".to_owned(),
            path: Utf8PathBuf::from("/does/not/exist/dir-to-test"),
        };

        assert!(!dir_does_not_exist.apply(&defopts()).unwrap());
    }

    #[test]
    fn test_directory_remove_apply_not_allowed() {
        let disallowed_dir = DirectoryRemove {
            name: "root".to_owned(),
            id: "/test-role/directory/root".to_owned(),
            path: Utf8PathBuf::from("/"),
        };

        assert!(!disallowed_dir.apply(&defopts()).unwrap());
    }

    #[test]
    fn test_directory_remove_apply_works() {
        let temp = TempDir::new().unwrap();
        let dir = temp.child("test_directory");
        dir.create_dir_all().unwrap();

        let test_dir = DirectoryRemove {
            name: "tester".to_owned(),
            id: "/test-role/directory/test_directory".to_owned(),
            path: Utf8PathBuf::from_path_buf(dir.to_path_buf()).unwrap(),
        };

        assert!(dir.exists());
        assert!(test_dir.apply(&defopts()).unwrap());
        assert!(!dir.exists());
    }

    #[test]
    fn test_directory_remove_apply_noop() {
        let temp = TempDir::new().unwrap();
        let dir = temp.child("test_directory");
        dir.create_dir_all().unwrap();

        let test_dir = DirectoryRemove {
            name: "tester".to_owned(),
            id: "/test-role/directory/test_directory".to_owned(),
            path: Utf8PathBuf::from_path_buf(dir.to_path_buf()).unwrap(),
        };

        assert!(dir.exists());
        assert!(!test_dir.apply(&defopts_noop()).unwrap());
        assert!(dir.exists());
    }

    #[test]
    fn test_directory_ensure_state() {
        let temp = TempDir::new().unwrap();
        let dir = temp.child("test_directory");
        dir.create_dir_all().unwrap();

        let dir_exists = DirectoryEnsure {
            id: "/test-role/directory/test_directory".to_owned(),
            group: "sysadmin".to_owned(),
            mode: "0755".to_owned(),
            name: "test_directory".to_owned(),
            owner: "rob".to_owned(),
            path: Utf8PathBuf::from_path_buf(dir.to_path_buf()).unwrap(),
        };

        let result = dir_exists.state().unwrap();

        assert!(result.exists);
        assert_eq!("0755".to_owned(), result.mode.unwrap());
        assert!(result.owner.is_some());
        assert!(result.group.is_some());

        let dir_does_not_exist = DirectoryEnsure {
            id: "/test-role/directory/test_directory".to_owned(),
            group: "sysadmin".to_owned(),
            mode: "0755".to_owned(),
            name: "test_directory".to_owned(),
            owner: "rob".to_owned(),
            path: Utf8PathBuf::from("/no/such/test_directory"),
        };

        assert_eq!(
            DirectoryStateEnsure {
                name: "test_directory".to_owned(),
                exists: false,
                mode: None,
                owner: None,
                group: None,
            },
            dir_does_not_exist.state().unwrap()
        );
    }

    #[test]
    fn test_directory_remove_state() {
        let temp = TempDir::new().unwrap();
        let dir = temp.child("dir-to-test");
        dir.create_dir_all().unwrap();

        let dir_exists = DirectoryRemove {
            name: "tester".to_owned(),
            id: "/test-role/directory/dir-to-test".to_owned(),
            path: Utf8PathBuf::from_path_buf(dir.to_path_buf()).unwrap(),
        };

        assert!(dir_exists.state());

        let dir_does_not_exist = DirectoryRemove {
            name: "tester".to_owned(),
            id: "/test-role/directory/dir-to-test".to_owned(),
            path: Utf8PathBuf::from("/does/not/exist/dir-to-test"),
        };

        assert!(!dir_does_not_exist.state());
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

        let expected_ensure = DirectoryEnsure {
            id: "/test-role/directory/test_directory".to_owned(),
            group: "sysadmin".to_owned(),
            mode: "0755".to_owned(),
            name: "test_directory".to_owned(),
            owner: "rob".to_owned(),
            path: Utf8PathBuf::from("/tmp/merp"),
        };

        assert_eq!(
            expected_ensure,
            DirectoryEnsure::try_from(&example_dir_ensure).unwrap()
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

        let expected_remove = DirectoryRemove {
            name: "merp".to_owned(),
            id: "/test-role/directory/merp".to_owned(),
            path: Utf8PathBuf::from("/tmp/merp"),
        };

        assert_eq!(
            expected_remove,
            DirectoryRemove::try_from(&example_dir_remove).unwrap()
        );
    }
}
