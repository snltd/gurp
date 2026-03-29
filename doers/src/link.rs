use anyhow::{bail, ensure};
use camino::Utf8PathBuf;
use common::constants::{ONE_RESOURCE_NO_CHANGE, ONE_RESOURCE_ONE_CHANGE};
use common::types::{ApplyOpts, ApplySummary};
use serde::Deserialize;
use std::fmt::Debug;
use std::fs;
use std::os::unix;
use std::os::unix::fs::MetadataExt;

// Just so we're all clear the TARGET is the end of the link that is created, and the SOURCE
// is the thing the link points to. (i.e. which probably already exists)

#[derive(Deserialize, Debug)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(rename_all = "kebab-case")]
pub struct GurpLinkEnsure {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(rename = "name")]
    pub target: Utf8PathBuf,
    pub source: Utf8PathBuf,
    #[serde(rename = "type")]
    pub link_type: LinkType,
    pub force_link: bool,
}

#[derive(Deserialize, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct GurpLinkRemove {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(rename = "name")]
    pub path: Utf8PathBuf,
}

#[derive(Deserialize, Debug)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(rename_all = "lowercase")]
pub enum LinkType {
    Symbolic,
    Hard,
}

impl GurpLinkEnsure {
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        let target = &self.target;
        let source = &self.source;

        ensure!(source.exists(), "source not found: {source}");

        if target.exists() {
            tracing::debug!("link target exists: {target}");
            let target_metadata = target.symlink_metadata()?;

            if target_metadata.is_dir() {
                if self.force_link {
                    tracing::info!("removing existing directory {target}");

                    if !opts.noop {
                        fs::remove_dir_all(target)?;
                    }

                    self.create_link(opts)
                } else {
                    bail!("target exists, is a directory and force-link is not set: {target}")
                }
            } else if target_metadata.is_symlink() {
                //
                // The target exists and is a symbolic link. If the user wants a symbolic link,
                // check it and if it's wrong, remove and re-create it.
                //
                // If the user wants a hard link, we'll remove it and create a hard link.
                //
                if self.link_is_correct()? {
                    Ok(ONE_RESOURCE_NO_CHANGE)
                } else {
                    tracing::info!(
                        "change link source: [{}] (existing symlink) -> {}",
                        target,
                        source
                    );
                    self.remove_target(opts)?;
                    self.create_link(opts)
                }
            } else {
                //
                // The target is probably a file. Maybe a hard link. Could even be something crazy
                // like a FIFO. Who knows?
                //
                // If the user wants a symlink and has set force-link, remove the target and make
                // a new link. If they want a symlink and haven't set that: error.
                //
                // If the user wants a hard link, compare the inodes of the source and target. If
                // they're the same we can report no change. If they're not, refer to force-link.
                // If it's true, blow away the source and create the link; if it's not: error.
                //
                match self.link_type {
                    LinkType::Symbolic => {
                        if self.force_link {
                            if self.link_is_correct()? {
                                Ok(ONE_RESOURCE_NO_CHANGE)
                            } else {
                                tracing::info!(
                                    "change link source: [{}] (existing symlink) -> {}",
                                    target,
                                    source
                                );
                                self.remove_target(opts)?;
                                self.create_link(opts)
                            }
                        } else {
                            bail!(
                                "link target [{}] is a file, and force-link is not set",
                                self.target
                            );
                        }
                    }
                    LinkType::Hard => {
                        if self.link_is_correct()? {
                            Ok(ONE_RESOURCE_NO_CHANGE)
                        } else {
                            tracing::info!(
                                "change link source: [{}] (existing symlink) -> {}",
                                target,
                                source
                            );
                            self.remove_target(opts)?;
                            self.create_link(opts)
                        }
                    }
                }
            }
        } else {
            self.create_link(opts)
        }
    }

    fn link_is_correct(&self) -> Result<bool, anyhow::Error> {
        let target_metadata = self.target.symlink_metadata()?;

        match self.link_type {
            LinkType::Symbolic => {
                if target_metadata.is_symlink() {
                    let current_source = &self.target.read_link_utf8()?;
                    if current_source == &self.source {
                        tracing::debug!("no change: {}", self.target);
                        return Ok(true);
                    }
                }
            }
            LinkType::Hard => {
                let source_metadata = fs::metadata(&self.source)?;
                let target_metadata = fs::metadata(&self.target)?;

                if source_metadata.ino() == target_metadata.ino()
                    && source_metadata.dev() == target_metadata.dev()
                {
                    tracing::debug!("no change: {}", self.target);
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    fn create_link(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        tracing::info!("creating link: {} -> {}", self.target, self.source);

        if !opts.noop {
            match self.link_type {
                LinkType::Symbolic => unix::fs::symlink(&self.source, &self.target)?,
                LinkType::Hard => fs::hard_link(&self.source, &self.target)?,
            }
        }

        Ok(ONE_RESOURCE_ONE_CHANGE)
    }

    fn remove_target(&self, opts: &ApplyOpts) -> anyhow::Result<()> {
        tracing::info!("removing existing link target: {}", self.target);
        if !opts.noop {
            fs::remove_file(&self.target)?;
        }

        Ok(())
    }
}

impl GurpLinkRemove {
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        if self.path.exists() {
            tracing::info!("removing link: {}", self.path);
            return_if_noop!(opts);

            fs::remove_file(&self.path)?;
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
    use common::constants::ONE_RESOURCE_NOOP;
    use pretty_assertions::assert_eq;
    use std::os::unix;
    use tester::{defopts, defopts_noop, deserialized_example, janet2json};

    #[test]
    fn test_deserialize_link_ensure_symlink_forced() {
        assert_eq!(
            GurpLinkEnsure {
                id: "/NO-ROLE/link/example-symlink".to_owned(),
                target: Utf8PathBuf::from("/symlink/is/here"),
                source: Utf8PathBuf::from("/link/points/here"),
                force_link: true,
                link_type: LinkType::Symbolic,
            },
            deserialized_example("link/ensure-symlink-forced.janet")
        );
    }

    #[test]
    fn test_deserialize_link_ensure_hard_link() {
        assert_eq!(
            GurpLinkEnsure {
                id: "/NO-ROLE/link/_link_is_here".to_owned(),
                target: Utf8PathBuf::from("/link/is/here"),
                source: Utf8PathBuf::from("/link/points/here"),
                force_link: false,
                link_type: LinkType::Hard,
            },
            deserialized_example("link/ensure-hard-link.janet")
        );
    }

    #[test]
    fn test_deserialize_link_remove_link() {
        assert_eq!(
            GurpLinkRemove {
                id: "/NO-ROLE/link/_dont_want_this_link".to_owned(),
                path: Utf8PathBuf::from("/dont/want/this/link"),
            },
            deserialized_example("link/remove-link.janet")
        );
    }

    #[test]
    fn test_symlink_create() {
        let temp_dir = Utf8TempDir::new().unwrap();
        let source_file = temp_dir.child("source-file");
        source_file.write_str("some-content").unwrap();
        let source_path = temp_dir.child("source-file");
        let target_path = temp_dir.child("target");

        let json_def = janet2json(&indoc::formatdoc! { r#"
            (link/ensure "{}"
                :source "{}")
            "#,
            target_path.as_path(),
            source_path.as_path(),
        });

        assert!(!target_path.exists());
        let sut: GurpLinkEnsure = serde_json::from_str(&json_def).unwrap();
        assert_eq!(ONE_RESOURCE_ONE_CHANGE, sut.apply(&defopts()).unwrap());
        assert!(target_path.exists());
        assert!(target_path.is_symlink());
    }

    #[test]
    fn test_symlink_create_noop() {
        let temp_dir = Utf8TempDir::new().unwrap();
        let source_file = temp_dir.child("source-file");
        source_file.write_str("some-content").unwrap();
        let source_path = temp_dir.child("source-file");
        let target_path = temp_dir.child("target");

        let json_def = janet2json(&indoc::formatdoc! { r#"
            (link/ensure "{}"
                :source "{}")
            "#,
            target_path.as_path(),
            source_path.as_path(),
        });

        assert!(!target_path.exists());
        let sut: GurpLinkEnsure = serde_json::from_str(&json_def).unwrap();
        assert_eq!(ONE_RESOURCE_NOOP, sut.apply(&defopts_noop()).unwrap());
        assert!(!target_path.exists());
    }

    #[test]
    fn test_symlink_remove() {
        let temp = Utf8TempDir::new().unwrap();
        let source = temp.child("source");
        let target = temp.child("target");
        source.write_str("some-content").unwrap();
        unix::fs::symlink(source, &target).unwrap();
        let json_def = janet2json(&format!(r#"(link/remove "{}")"#, target.as_path()));
        assert!(target.exists());
        let sut: GurpLinkRemove = serde_json::from_str(&json_def).unwrap();
        assert_eq!(ONE_RESOURCE_ONE_CHANGE, sut.apply(&defopts()).unwrap());
        assert!(!target.exists());
    }

    #[test]
    fn test_symlink_remove_noop() {
        let temp = Utf8TempDir::new().unwrap();
        let source = temp.child("source");
        let target = temp.child("target");
        source.write_str("some-content").unwrap();
        unix::fs::symlink(source, &target).unwrap();
        let json_def = janet2json(&format!(r#"(link/remove "{}")"#, target.as_path()));
        assert!(target.exists());
        let sut: GurpLinkRemove = serde_json::from_str(&json_def).unwrap();
        assert_eq!(ONE_RESOURCE_NOOP, sut.apply(&defopts_noop()).unwrap());
        assert!(target.exists());
    }

    #[test]
    fn test_symlink_remove_missing() {
        let json_def = janet2json(r#"(link/remove "/no/such/file")"#);
        let sut: GurpLinkRemove = serde_json::from_str(&json_def).unwrap();
        assert_eq!(ONE_RESOURCE_NO_CHANGE, sut.apply(&defopts_noop()).unwrap());
    }

    #[test]
    fn test_hardlink_create() {
        let temp_dir = Utf8TempDir::new().unwrap();
        let source_file = temp_dir.child("source-file");
        source_file.write_str("some-content").unwrap();
        let source_path = temp_dir.child("source-file");
        let target_path = temp_dir.child("target");

        let json_def = janet2json(&indoc::formatdoc! { r#"
            (link/ensure "{}"
                         :source "{}"
                         :type "hard")"#,
            target_path.as_path(),
            source_path.as_path(),
        });

        assert!(!target_path.exists());
        let sut: GurpLinkEnsure = serde_json::from_str(&json_def).unwrap();
        assert_eq!(ONE_RESOURCE_ONE_CHANGE, sut.apply(&defopts()).unwrap());
        assert!(target_path.exists());
        use std::os::unix::fs::MetadataExt;
        let source_meta = std::fs::metadata(source_path.as_path()).unwrap();
        let target_meta = std::fs::metadata(target_path.as_path()).unwrap();
        assert_eq!(source_meta.ino(), target_meta.ino());
        assert_eq!(source_meta.dev(), target_meta.dev());
    }

    #[test]
    fn test_hardlink_no_change() {
        let temp_dir = Utf8TempDir::new().unwrap();
        let source_file = temp_dir.child("source-file");
        source_file.write_str("some-content").unwrap();
        let source_path = temp_dir.child("source-file");
        let target_path = temp_dir.child("target");
        std::fs::hard_link(source_path.as_path(), target_path.as_path()).unwrap();

        let json_def = janet2json(&indoc::formatdoc! { r#"
            (link/ensure "{}"
                         :source "{}"
                         :type "hard")"#,
            target_path.as_path(),
            source_path.as_path(),
        });

        let sut: GurpLinkEnsure = serde_json::from_str(&json_def).unwrap();
        assert_eq!(ONE_RESOURCE_NO_CHANGE, sut.apply(&defopts()).unwrap());

        use std::os::unix::fs::MetadataExt;
        let source_meta = std::fs::metadata(source_path.as_path()).unwrap();
        let target_meta = std::fs::metadata(target_path.as_path()).unwrap();
        assert_eq!(source_meta.ino(), target_meta.ino());
    }

    #[test]
    fn test_hardlink_correction() {
        let temp_dir = Utf8TempDir::new().unwrap();
        let old_source_file = temp_dir.child("old-source");
        old_source_file.write_str("old-content").unwrap();
        let old_source_path = temp_dir.child("old-source");
        let new_source_file = temp_dir.child("new-source");
        new_source_file.write_str("new-content").unwrap();
        let new_source_path = temp_dir.child("new-source");
        let target_path = temp_dir.child("target");

        std::fs::hard_link(old_source_path.as_path(), target_path.as_path()).unwrap();

        use std::os::unix::fs::MetadataExt;
        let old_meta = std::fs::metadata(old_source_path.as_path()).unwrap();
        let target_meta_before = std::fs::metadata(target_path.as_path()).unwrap();
        assert_eq!(old_meta.ino(), target_meta_before.ino());

        let json_def = janet2json(&indoc::formatdoc! { r#"
            (link/ensure "{}"
                         :source "{}"
                         :type "hard")"#,
            target_path.as_path(),
            new_source_path.as_path(),
        });

        let sut: GurpLinkEnsure = serde_json::from_str(&json_def).unwrap();
        assert_eq!(ONE_RESOURCE_ONE_CHANGE, sut.apply(&defopts()).unwrap());

        let new_meta = std::fs::metadata(new_source_path.as_path()).unwrap();
        let target_meta_after = std::fs::metadata(target_path.as_path()).unwrap();
        assert_eq!(new_meta.ino(), target_meta_after.ino());
        assert_ne!(old_meta.ino(), target_meta_after.ino());
        assert_eq!(
            fs::read_to_string(target_path.as_path()).unwrap(),
            "new-content".to_owned()
        );
    }

    #[test]
    fn test_symlink_over_file_force_link_true() {
        let temp_dir = Utf8TempDir::new().unwrap();
        let source_file = temp_dir.child("source-file");
        source_file.write_str("source-content").unwrap();
        let source_path = temp_dir.child("source-file");
        let target_file = temp_dir.child("target");
        target_file.write_str("target-content").unwrap();
        assert!(target_file.exists());

        let json_def = janet2json(&indoc::formatdoc! { r#"
            (link/ensure "{}"
                :force-link true
                :source "{}")
            "#,
            target_file.as_path(),
            source_path.as_path(),
        });

        let sut: GurpLinkEnsure = serde_json::from_str(&json_def).unwrap();
        assert_eq!(ONE_RESOURCE_ONE_CHANGE, sut.apply(&defopts()).unwrap());
        assert!(target_file.exists());
        assert!(target_file.is_symlink());
        assert_eq!(
            "source-content".to_owned(),
            fs::read_to_string(&target_file).unwrap()
        );
    }

    #[test]
    fn test_symlink_over_directory() {
        let temp_dir = Utf8TempDir::new().unwrap();
        let source_file = temp_dir.child("source-file");
        source_file.write_str("source-content").unwrap();
        let source_path = temp_dir.child("source-file");
        let target_dir = temp_dir.child("target");
        fs::create_dir(&target_dir).unwrap();
        assert!(target_dir.exists());

        let json_def = janet2json(&indoc::formatdoc! { r#"
            (link/ensure "{}"
                :force-link false
                :source "{}")
            "#,
            target_dir.as_path(),
            source_path.as_path(),
        });

        let sut: GurpLinkEnsure = serde_json::from_str(&json_def).unwrap();
        assert!(
            sut.apply(&defopts())
                .unwrap_err()
                .to_string()
                .contains("target exists, is a directory and force-link is not set")
        );
        assert!(target_dir.exists());
        assert!(target_dir.is_dir());

        let json_def = janet2json(&indoc::formatdoc! { r#"
            (link/ensure "{}"
                :force-link true
                :source "{}")
            "#,
            target_dir.as_path(),
            source_path.as_path(),
        });

        let sut: GurpLinkEnsure = serde_json::from_str(&json_def).unwrap();
        assert_eq!(ONE_RESOURCE_ONE_CHANGE, sut.apply(&defopts()).unwrap());
        assert!(target_dir.exists());
        assert!(target_dir.is_symlink());
    }

    #[test]
    fn test_symlink_over_file_force_link_false() {
        let temp_dir = Utf8TempDir::new().unwrap();
        let source_file = temp_dir.child("source-file");
        source_file.write_str("source-content").unwrap();
        let source_path = temp_dir.child("source-file");
        let target_file = temp_dir.child("target");
        target_file.write_str("target-content").unwrap();
        assert!(target_file.exists());

        let json_def = janet2json(&indoc::formatdoc! { r#"
            (link/ensure "{}"
                :force-link false
                :source "{}")
            "#,
            target_file.as_path(),
            source_path.as_path(),
        });

        let sut: GurpLinkEnsure = serde_json::from_str(&json_def).unwrap();
        assert!(
            sut.apply(&defopts())
                .unwrap_err()
                .to_string()
                .contains("is a file, and force-link is not set")
        );
        assert!(target_file.exists());
        assert_eq!(
            "target-content".to_owned(),
            fs::read_to_string(&target_file).unwrap()
        );
        assert!(!target_file.is_symlink());
    }
}
