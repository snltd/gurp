use anyhow::ensure;
use camino::Utf8PathBuf;
use common::constants::{ONE_RESOURCE_NO_CHANGE, ONE_RESOURCE_ONE_CHANGE, PROTECTED_DIRS};
use common::types::{ApplyOpts, ApplySummary};
use nix::unistd::{Gid, Uid};
use serde::Deserialize;
use std::fs;
use util::file;
use util::file::{FileMetadata, NameOrId};

#[derive(Deserialize, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct GurpDirectoryEnsure {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(rename = "name")]
    pub path: Utf8PathBuf,
    #[serde(flatten)]
    pub desired_state: DesiredDirectoryState,
}

#[derive(Deserialize, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct DesiredDirectoryState {
    pub group: NameOrId,
    pub mode: String,
    pub owner: NameOrId,
}

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct DirectoryState {
    pub gid: Gid,
    pub mode: String,
    pub uid: Uid,
}

#[derive(Deserialize, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct GurpDirectoryRemove {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(rename = "name")]
    pub path: Utf8PathBuf,
}

impl GurpDirectoryEnsure {
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        let mut changed = false;

        if self.path.exists() {
            ensure!(
                self.path.is_dir(),
                "{} exists and is not a directory",
                self.path
            );
        } else {
            tracing::info!("creating directory: {}", self.path);
            changed = true;

            if !opts.noop {
                fs::create_dir_all(&self.path)?;
            }
        }

        if file::ensure_metadata(
            &self.path,
            FileMetadata {
                group: &self.desired_state.group,
                mode: &self.desired_state.mode,
                owner: &self.desired_state.owner,
            },
            opts,
        )? {
            changed = true
        }

        if changed {
            Ok(ONE_RESOURCE_ONE_CHANGE)
        } else {
            Ok(ONE_RESOURCE_NO_CHANGE)
        }
    }
}

impl GurpDirectoryRemove {
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        if self.path.exists() {
            ensure!(
                self.path.is_dir(),
                "asked to remove {} but it is not a directory",
                self.path
            );

            ensure!(
                !PROTECTED_DIRS.contains(&self.path),
                format!("protected resource: {}", self.path)
            );

            tracing::info!("removing directory: {}", self.path);

            if !opts.noop {
                fs::remove_dir_all(&self.path)?;
            }

            Ok(ONE_RESOURCE_ONE_CHANGE)
        } else {
            tracing::debug!("not present: {}", self.path);
            Ok(ONE_RESOURCE_NO_CHANGE)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use camino_tempfile_ext::prelude::*;
    use indoc::formatdoc;
    use pretty_assertions::assert_eq;
    use std::os::unix::fs::PermissionsExt;
    use tester::{defopts, defopts_noop, deserialized_example, janet2json, my_group, my_user};

    #[test]
    fn test_deserialize_ensure_directory_defaults() {
        assert_eq!(
            GurpDirectoryEnsure {
                path: Utf8PathBuf::from("/example/dir_1"),
                id: "/NO-ROLE/directory/_example_dir_1".to_owned(),
                desired_state: DesiredDirectoryState {
                    owner: NameOrId::Name("root".to_owned()),
                    group: NameOrId::Name("root".to_owned()),
                    mode: "0755".to_owned(),
                }
            },
            deserialized_example("directory/ensure-default-dir.janet")
        );
    }

    #[test]
    fn test_deserialize_ensure_directory_with_ids() {
        assert_eq!(
            GurpDirectoryEnsure {
                path: Utf8PathBuf::from("/example/dir_3"),
                id: "/NO-ROLE/directory/my-dir".to_owned(),
                desired_state: DesiredDirectoryState {
                    owner: NameOrId::Id(4),
                    group: NameOrId::Id(12),
                    mode: "2750".to_owned(),
                }
            },
            deserialized_example("directory/ensure-with-ids.janet")
        );
    }

    #[test]
    fn test_deserialize_ensure_directory_with_names() {
        assert_eq!(
            GurpDirectoryEnsure {
                path: Utf8PathBuf::from("/example/dir_2"),
                id: "/NO-ROLE/directory/all-the-specs".to_owned(),
                desired_state: DesiredDirectoryState {
                    owner: NameOrId::Name("adm".to_owned()),
                    group: NameOrId::Name("sys".to_owned()),
                    mode: "0700".to_owned(),
                }
            },
            deserialized_example("directory/ensure-with-names.janet")
        );
    }

    #[test]
    fn test_deserialize_remove_directory() {
        assert_eq!(
            GurpDirectoryRemove {
                id: "/NO-ROLE/directory/_example".to_owned(),
                path: Utf8PathBuf::from("/example"),
            },
            deserialized_example("directory/remove-dir.janet")
        );
    }

    #[test]
    fn test_directory_ensure_apply_noop() {
        let temp_dir = Utf8TempDir::new().unwrap();
        let dir = temp_dir.child("test_directory");
        let json_def = janet2json(&format!("(directory/ensure \"{}\")", dir.as_path()));
        let sut: GurpDirectoryEnsure = serde_json::from_str(&json_def).unwrap();

        assert!(!dir.exists());
        assert_eq!(ONE_RESOURCE_ONE_CHANGE, sut.apply(&defopts_noop()).unwrap());
        assert!(!dir.exists());
    }

    #[test]
    fn test_directory_ensure_already_exists() {
        let temp_dir = Utf8TempDir::new().unwrap();
        let dir = temp_dir.child("test_directory");
        dir.create_dir_all().unwrap();
        fs::set_permissions(&dir, fs::Permissions::from_mode(0o0750)).unwrap();

        assert!(dir.exists());

        let json_def = janet2json(&formatdoc! { r#"
            (directory/ensure "{}"
                :mode "0750"
                :owner "{}"
                :group "{}")
            "#,
            dir.as_path(),
            my_user(),
            my_group(),
        });

        let sut: GurpDirectoryEnsure = serde_json::from_str(&json_def).unwrap();

        assert_eq!(ONE_RESOURCE_NO_CHANGE, sut.apply(&defopts_noop()).unwrap());
        assert!(dir.exists());
    }

    #[test]
    fn test_directory_ensure_change_mode() {
        let temp_dir = Utf8TempDir::new().unwrap();
        let dir = temp_dir.child("test_directory");
        dir.create_dir_all().unwrap();
        fs::set_permissions(&dir, fs::Permissions::from_mode(0o0750)).unwrap();

        assert!(dir.exists());

        let json_def = janet2json(&formatdoc! { r#"
            (directory/ensure "{}"
                :mode "0775"
                :owner "{}"
                :group "{}")
            "#,
            &dir.as_path(),
            my_user(),
            my_group(),
        });

        let sut: GurpDirectoryEnsure = serde_json::from_str(&json_def).unwrap();

        assert_eq!(ONE_RESOURCE_ONE_CHANGE, sut.apply(&defopts()).unwrap());
        assert!(dir.exists());

        let metadata = fs::metadata(&dir).unwrap();

        assert_eq!(metadata.permissions().mode() & 0o7777, 0o0775);
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
        assert!(sut.apply(&defopts()).is_err());
    }

    #[test]
    fn test_directory_remove_apply_works() {
        let temp = Utf8TempDir::new().unwrap();
        let dir = temp.child("test_directory");
        dir.create_dir_all().unwrap();

        let json_def = janet2json(&format!("(directory/remove \"{}\")", dir.as_path()));
        let sut: GurpDirectoryRemove = serde_json::from_str(&json_def).unwrap();

        assert!(dir.exists());
        assert_eq!(ONE_RESOURCE_ONE_CHANGE, sut.apply(&defopts()).unwrap());
        assert!(!dir.exists());
    }

    #[test]
    fn test_directory_remove_apply_noop() {
        let temp_dir = Utf8TempDir::new().unwrap();
        let dir = temp_dir.child("test_directory");
        dir.create_dir_all().unwrap();

        let json_def = janet2json(&format!("(directory/remove \"{}\")", dir.as_path()));
        let sut: GurpDirectoryRemove = serde_json::from_str(&json_def).unwrap();

        assert!(dir.exists());
        assert_eq!(ONE_RESOURCE_ONE_CHANGE, sut.apply(&defopts_noop()).unwrap());
        assert!(dir.exists());
    }
}
