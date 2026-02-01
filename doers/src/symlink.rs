use anyhow::{bail, ensure};
use common::constants::{ONE_RESOURCE_NO_CHANGE, ONE_RESOURCE_ONE_CHANGE};
use common::types::{ApplyOpts, ApplySummary};
use serde::Deserialize;
use std::fmt::Debug;
use std::fs;
use std::os::unix;

// THINGS TO KNOW / THINGS TO DO.
// Only does symbolic links.

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct GurpSymlinkEnsure {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(rename = "name")]
    pub path: Utf8PathBuf,
    pub source: Utf8PathBuf,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct GurpSymlinkRemove {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(rename = "name")]
    pub path: Utf8PathBuf,
}

impl GurpSymlinkEnsure {
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        let target = &self.path;
        let source = &self.source;

        ensure!(source.exists(), "source not found: {source}");

        if !target.exists() {
            tracing::info!("creating symlink: {} -> {}", target, source);
            return_if_noop!(opts);

            unix::fs::symlink(source, target)?;
            Ok(ONE_RESOURCE_ONE_CHANGE)
        } else if target.is_symlink() {
            let current_source = target.read_link_utf8()?;
            if current_source == *source {
                tracing::debug!("no change: {}", self.path);
                Ok(ONE_RESOURCE_NO_CHANGE)
            } else {
                tracing::info!(
                    "change symlink source: [{}] {} -> {}",
                    target,
                    &current_source,
                    source
                );
                return_if_noop!(opts);

                fs::remove_file(target)?;
                unix::fs::symlink(source, target)?;
                Ok(ONE_RESOURCE_ONE_CHANGE)
            }
        } else {
            bail!("{} exists and is not a symlink", &target);
        }
    }
}

impl GurpSymlinkRemove {
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        if self.path.exists() {
            tracing::info!("removing symlink: {}", self.path);
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
    use std::os::unix;
    use tester::{defopts, defopts_noop, janet2json};

    #[test]
    fn test_symlink_create() {
        let temp_dir = Utf8TempDir::new().unwrap();
        let source_file = temp_dir.child("source-file");
        source_file.write_str("some-content").unwrap();
        let source_path = temp_dir.child("source-file");
        let target_path = temp_dir.child("target");

        let json_def = janet2json(&format!(
            " (symlink/ensure \"{}\" :source \"{}\")",
            target_path.as_path(),
            source_path.as_path(),
        ));

        assert!(!target_path.exists());
        let sut: GurpSymlinkEnsure = serde_json::from_str(&json_def).unwrap();
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

        let json_def = janet2json(&format!(
            " (symlink/ensure \"{}\" :source \"{}\")",
            target_path.as_path(),
            source_path.as_path(),
        ));

        assert!(!target_path.exists());
        let sut: GurpSymlinkEnsure = serde_json::from_str(&json_def).unwrap();
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

        let json_def = janet2json(&format!("(symlink/remove \"{}\")", target.as_path()));

        assert!(target.exists());
        let sut: GurpSymlinkRemove = serde_json::from_str(&json_def).unwrap();
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

        let json_def = janet2json(&format!("(symlink/remove \"{}\")", target.as_path()));

        assert!(target.exists());
        let sut: GurpSymlinkRemove = serde_json::from_str(&json_def).unwrap();
        assert_eq!(ONE_RESOURCE_NOOP, sut.apply(&defopts_noop()).unwrap());
        assert!(target.exists());
    }

    #[test]
    fn test_symlink_remove_missing() {
        let json_def = janet2json("(symlink/remove \"/no/such/file\")");
        let sut: GurpSymlinkRemove = serde_json::from_str(&json_def).unwrap();
        assert_eq!(ONE_RESOURCE_NO_CHANGE, sut.apply(&defopts_noop()).unwrap());
    }
}
