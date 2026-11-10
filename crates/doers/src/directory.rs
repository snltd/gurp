use anyhow::{Context, ensure};
use camino::Utf8PathBuf;
use common::constants::{ONE_RESOURCE_NO_CHANGE, ONE_RESOURCE_ONE_CHANGE, PROTECTED_DIRS};
use common::types::{ApplyOpts, ApplySummary};
use nix::unistd::{Gid, Uid};
use os_types::{FileMode, GurpId};
use serde::Deserialize;
use std::fs;
use util::archive::{self, ArchiveSource};
use util::checksum::Checksum;
use util::file::{self, FileMetadata, NameOrId};

#[derive(Deserialize, Debug)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(rename_all = "kebab-case")]
pub struct DirectoryEnsure {
    #[serde(rename = "_id")]
    pub id: GurpId,
    #[serde(rename = "name")]
    pub path: Utf8PathBuf,
    #[serde(flatten)]
    pub desired_state: DesiredDirectoryState,
    pub checksum: Option<Checksum>,
    pub create_from: Option<String>,
    pub create_with: Option<String>,
}

#[derive(Deserialize, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct DesiredDirectoryState {
    pub group: NameOrId,
    pub mode: FileMode,
    pub owner: NameOrId,
}

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct DirectoryState {
    pub gid: Gid,
    pub mode: FileMode,
    pub uid: Uid,
}

#[derive(Deserialize, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct DirectoryRemove {
    #[serde(rename = "_id")]
    pub id: GurpId,
    #[serde(rename = "name")]
    pub path: Utf8PathBuf,
}

impl DirectoryEnsure {
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        let mut changed = false;

        if self.path.exists() {
            ensure!(
                self.path.is_dir(),
                "{} exists and is not a directory",
                self.path
            );
        } else {
            changed = true;
            self.create_directory(opts)?
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

    fn create_directory(&self, opts: &ApplyOpts) -> anyhow::Result<()> {
        if let Some(ref archive_str) = self.create_from {
            tracing::info!(
                "creating directory: {} from archive {}",
                &self.path,
                archive_str
            );

            let parent = self
                .path
                .parent()
                .with_context(|| format!("cannot get parent of {}", self.path))?;

            if !opts.noop {
                archive::unpack(
                    &ArchiveSource::try_from(archive_str.as_str())?,
                    parent,
                    self.checksum.as_ref(),
                    opts,
                )?;
            }
        } else {
            tracing::info!("creating directory: {}", self.path);

            if !opts.noop {
                fs::create_dir_all(&self.path)?;
            }

            if let Some(ref archive_str) = self.create_with {
                tracing::info!(
                    "populating {} with contents of archive {}",
                    self.path,
                    archive_str,
                );

                if !opts.noop {
                    archive::unpack(
                        &ArchiveSource::try_from(archive_str.as_str())?,
                        &self.path,
                        self.checksum.as_ref(),
                        opts,
                    )?;
                }
            }
        }

        Ok(())
    }
}

impl DirectoryRemove {
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
    use snltest::fixture;
    use std::os::unix::fs::PermissionsExt;
    use tester::{deserialized_example, janet2json, my_group, my_user};

    #[test]
    fn test_deserialize_ensure_directory_defaults() {
        assert_eq!(
            DirectoryEnsure {
                path: Utf8PathBuf::from("/example/dir_1"),
                id: GurpId::new("/NO-ROLE/directory/_example_dir_1").unwrap(),
                create_from: None,
                create_with: None,
                checksum: None,
                desired_state: DesiredDirectoryState {
                    owner: NameOrId::Name("root".to_owned()),
                    group: NameOrId::Name("root".to_owned()),
                    mode: FileMode::new("755").unwrap(),
                }
            },
            deserialized_example("directory/ensure-default-dir.janet")
        );
    }

    #[test]
    fn test_deserialize_ensure_directory_with_ids() {
        assert_eq!(
            DirectoryEnsure {
                path: Utf8PathBuf::from("/example/dir_3"),
                id: GurpId::new("/NO-ROLE/directory/my-dir").unwrap(),
                create_from: None,
                create_with: None,
                checksum: None,
                desired_state: DesiredDirectoryState {
                    owner: NameOrId::Id(4),
                    group: NameOrId::Id(12),
                    mode: FileMode::new("2750").unwrap(),
                }
            },
            deserialized_example("directory/ensure-with-ids.janet")
        );
    }

    #[test]
    fn test_deserialize_ensure_directory_with_names() {
        assert_eq!(
            DirectoryEnsure {
                path: Utf8PathBuf::from("/example/dir_2"),
                id: GurpId::new("/NO-ROLE/directory/all-the-specs").unwrap(),
                create_from: None,
                create_with: None,
                checksum: None,
                desired_state: DesiredDirectoryState {
                    owner: NameOrId::Name("adm".to_owned()),
                    group: NameOrId::Name("sys".to_owned()),
                    mode: FileMode::new("0700").unwrap(),
                }
            },
            deserialized_example("directory/ensure-with-names.janet")
        );
    }

    #[test]
    fn test_deserialize_remove_directory() {
        assert_eq!(
            DirectoryRemove {
                id: GurpId::new("/NO-ROLE/directory/_example").unwrap(),
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
        let sut: DirectoryEnsure = serde_json::from_str(&json_def).unwrap();

        assert!(!dir.exists());
        assert_eq!(
            ONE_RESOURCE_ONE_CHANGE,
            sut.apply(&ApplyOpts {
                noop: true,
                ..Default::default()
            })
            .unwrap()
        );
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

        let sut: DirectoryEnsure = serde_json::from_str(&json_def).unwrap();

        assert_eq!(
            ONE_RESOURCE_NO_CHANGE,
            sut.apply(&ApplyOpts {
                noop: true,
                ..Default::default()
            })
            .unwrap()
        );
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

        let sut: DirectoryEnsure = serde_json::from_str(&json_def).unwrap();

        assert_eq!(
            ONE_RESOURCE_ONE_CHANGE,
            sut.apply(&ApplyOpts::default()).unwrap()
        );
        assert!(dir.exists());

        let metadata = fs::metadata(&dir).unwrap();

        assert_eq!(metadata.permissions().mode() & 0o7777, 0o0775);
    }

    #[test]
    fn test_directory_remove_apply_does_not_exist() {
        let json_def = janet2json(r#"(directory/remove "/no/such/dir")"#);
        let sut: DirectoryRemove = serde_json::from_str(&json_def).unwrap();
        assert_eq!(
            ONE_RESOURCE_NO_CHANGE,
            sut.apply(&ApplyOpts::default()).unwrap()
        );
    }

    #[test]
    fn test_directory_remove_apply_not_allowed() {
        let json_def = janet2json(r#"(directory/remove "/usr")"#);
        let sut: DirectoryRemove = serde_json::from_str(&json_def).unwrap();
        assert!(sut.apply(&ApplyOpts::default()).is_err());
    }

    #[test]
    fn test_directory_remove_apply_works() {
        let temp = Utf8TempDir::new().unwrap();
        let dir = temp.child("test_directory");
        dir.create_dir_all().unwrap();

        let json_def = janet2json(&format!("(directory/remove \"{}\")", dir.as_path()));
        let sut: DirectoryRemove = serde_json::from_str(&json_def).unwrap();

        assert!(dir.exists());
        assert_eq!(
            ONE_RESOURCE_ONE_CHANGE,
            sut.apply(&ApplyOpts::default()).unwrap()
        );
        assert!(!dir.exists());
    }

    #[test]
    fn test_directory_remove_apply_noop() {
        let temp_dir = Utf8TempDir::new().unwrap();
        let dir = temp_dir.child("test_directory");
        dir.create_dir_all().unwrap();

        let json_def = janet2json(&format!("(directory/remove \"{}\")", dir.as_path()));
        let sut: DirectoryRemove = serde_json::from_str(&json_def).unwrap();

        assert!(dir.exists());
        assert_eq!(
            ONE_RESOURCE_ONE_CHANGE,
            sut.apply(&ApplyOpts {
                noop: true,
                ..Default::default()
            })
            .unwrap()
        );
        assert!(dir.exists());
    }

    #[test]
    fn test_directory_create_with_remote_archive_with_checksum() {
        let temp_dir = Utf8TempDir::new().unwrap();
        let dir = temp_dir.child("test_directory");

        assert!(!dir.exists());

        let json_def = janet2json(&formatdoc! { r#"
            (directory/ensure "{}"
                :mode "0700"
                :owner "{}"
                :group "{}"
                :create-with "https://github.com/snltd/gurp/releases/download/v2.5.0/gurp-v2.5.0-omnios.tar.gz"
                (file/checksum
                    :algorithm "sha256"
                    :literal "402a8a3d0054ddb12eff3a2ff35269722f9555b4666d2ce1d69a08bab774a2df"))"#,
            &dir.as_path(),
            my_user(),
            my_group(),
        });

        let sut: DirectoryEnsure = serde_json::from_str(&json_def).unwrap();

        assert_eq!(
            ONE_RESOURCE_ONE_CHANGE,
            sut.apply(&ApplyOpts::default()).unwrap()
        );

        assert!(dir.exists());
        assert!(dir.join("gurp").exists());
        let metadata = fs::metadata(&dir).unwrap();
        assert_eq!(metadata.permissions().mode() & 0o7777, 0o0700);
    }

    #[test]
    fn test_directory_create_from_local_archive_without_checksum() {
        let temp_dir = Utf8TempDir::new().unwrap();
        let dir = temp_dir.child("sample");

        assert!(!dir.exists());

        let json_def = janet2json(&formatdoc! { r#"
            (directory/ensure "{}"
                :mode "0750"
                :owner "{}"
                :group "{}"
                :create-from "{}")"#,
            &dir.as_path(),
            my_user(),
            my_group(),
            fixture!("directory/sample.tar")
        });

        let sut: DirectoryEnsure = serde_json::from_str(&json_def).unwrap();

        assert_eq!(
            ONE_RESOURCE_ONE_CHANGE,
            sut.apply(&ApplyOpts::default()).unwrap()
        );

        assert!(dir.exists());
        assert!(dir.join("sample_file").exists());
        let metadata = fs::metadata(&dir).unwrap();
        assert_eq!(metadata.permissions().mode() & 0o7777, 0o0750);
    }
}
