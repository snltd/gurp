use anyhow::{bail, ensure};
use camino::{Utf8Path, Utf8PathBuf};
use common::constants::{ONE_RESOURCE_NO_CHANGE, ONE_RESOURCE_ONE_CHANGE};
use common::types::{ApplyOpts, ApplySummary};
use serde::Deserialize;
use std::fmt::Debug;
use std::fs;
use std::os::unix;

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct GurpLinkEnsure {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(rename = "name")]
    pub target: Utf8PathBuf,
    pub source: Utf8PathBuf,
    #[serde(rename = "type")]
    pub link_type: String,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct GurpLinkRemove {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(rename = "name")]
    pub path: Utf8PathBuf,
}

impl GurpLinkEnsure {
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        let target = &self.target;
        let source = &self.source;

        ensure!(source.exists(), "source not found: {source}");

        if !target.exists() {
            tracing::info!("creating link: {} -> {}", target, source);
            return_if_noop!(opts);

            self.create_link(source, target)
        } else {
            let current_matches = match self.link_type.as_str() {
                "symbolic" => {
                    if target.is_symlink() {
                        let current_source = target.read_link_utf8()?;
                        current_source == *source
                    } else {
                        false
                    }
                }
                "hard" => self.are_hard_linked(source, target)?,
                other => bail!("unknown link type: {other}"),
            };

            if current_matches {
                tracing::debug!("no change: {}", self.target);
                Ok(ONE_RESOURCE_NO_CHANGE)
            } else {
                // Need to recreate the link
                if target.is_symlink() {
                    let current_source = target.read_link_utf8()?;
                    tracing::info!(
                        "change link source: [{}] {} -> {}",
                        target,
                        &current_source,
                        source
                    );
                } else {
                    tracing::info!(
                        "change link source: [{}] (existing file) -> {}",
                        target,
                        source
                    );
                }
                return_if_noop!(opts);

                fs::remove_file(target)?;
                self.create_link(source, target)
            }
        }
    }

    fn create_link(&self, source: &Utf8Path, target: &Utf8Path) -> anyhow::Result<ApplySummary> {
        match self.link_type.as_str() {
            "symbolic" => unix::fs::symlink(source, target)?,
            "hard" => fs::hard_link(source, target)?,
            other => bail!("unknown link type: {other}"),
        }

        Ok(ONE_RESOURCE_ONE_CHANGE)
    }

    fn are_hard_linked(&self, source: &Utf8Path, target: &Utf8Path) -> anyhow::Result<bool> {
        use std::os::unix::fs::MetadataExt;

        let source_metadata = fs::metadata(source)?;
        let target_metadata = fs::metadata(target)?;

        Ok(source_metadata.ino() == target_metadata.ino()
            && source_metadata.dev() == target_metadata.dev())
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
    fn test_deserialize_link_ensure_01() {
        assert_eq!(
            GurpLinkEnsure {
                id: "/NO-ROLE/link/example-symlink".to_owned(),
                target: Utf8PathBuf::from("/symlink/is/here"),
                source: Utf8PathBuf::from("/link/points/here"),
                link_type: "symbolic".to_owned(),
            },
            deserialized_example("link/ensure-01.janet")
        );
    }

    #[test]
    fn test_deserialize_link_ensure_02() {
        assert_eq!(
            GurpLinkEnsure {
                id: "/NO-ROLE/link/_link_is_here".to_owned(),
                target: Utf8PathBuf::from("/link/is/here"),
                source: Utf8PathBuf::from("/link/points/here"),
                link_type: "hard".to_owned(),
            },
            deserialized_example("link/ensure-02.janet")
        );
    }

    #[test]
    fn test_deserialize_link_remove_01() {
        assert_eq!(
            GurpLinkRemove {
                id: "/NO-ROLE/link/_dont_want_this_link".to_owned(),
                path: Utf8PathBuf::from("/dont/want/this/link"),
            },
            deserialized_example("link/remove-01.janet")
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
}
