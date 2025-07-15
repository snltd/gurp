use crate::prelude::*;
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
    pub fn apply(&self, opts: &Opts) -> anyhow::Result<ApplySummary> {
        let target = &self.path;
        let source = &self.source;

        if !source.exists() {
            bail!("source not found: {}", source);
        }

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
    pub fn apply(&self, opts: &Opts) -> anyhow::Result<ApplySummary> {
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
    use crate::test_utils::spec_helper::{defopts, defopts_noop, janet2json};
    use assert_fs::TempDir;
    use assert_fs::prelude::*;
    use camino::Utf8PathBuf;
    use std::os::unix;

    #[test]
    fn test_symlink_create() {
        let temp = TempDir::new().unwrap();
        let source_file = temp.child("source-file");
        source_file.write_str("some-content").unwrap();
        let source_path =
            Utf8PathBuf::from_path_buf(temp.child("source-file").to_path_buf()).unwrap();
        let target_path = Utf8PathBuf::from_path_buf(temp.child("target").to_path_buf()).unwrap();

        let json_def = janet2json(&format!(
            " (symlink/ensure \"{target_path}\" :source \"{source_path}\")"
        ));

        assert!(!target_path.exists());
        let sut: GurpSymlinkEnsure = serde_json::from_str(&json_def).unwrap();
        assert_eq!(ONE_RESOURCE_ONE_CHANGE, sut.apply(&defopts()).unwrap());
        assert!(target_path.exists());
        assert!(target_path.is_symlink());
    }

    #[test]
    fn test_symlink_create_noop() {
        let temp = TempDir::new().unwrap();
        let source_file = temp.child("source-file");
        source_file.write_str("some-content").unwrap();
        let source_path =
            Utf8PathBuf::from_path_buf(temp.child("source-file").to_path_buf()).unwrap();
        let target_path = Utf8PathBuf::from_path_buf(temp.child("target").to_path_buf()).unwrap();

        let json_def = janet2json(&format!(
            " (symlink/ensure \"{target_path}\" :source \"{source_path}\")"
        ));

        assert!(!target_path.exists());
        let sut: GurpSymlinkEnsure = serde_json::from_str(&json_def).unwrap();
        assert_eq!(ONE_RESOURCE_NOOP, sut.apply(&defopts_noop()).unwrap());
        assert!(!target_path.exists());
    }

    #[test]
    fn test_symlink_remove() {
        let temp = TempDir::new().unwrap();
        let source = temp.child("source");
        let target = temp.child("target");
        source.write_str("some-content").unwrap();
        unix::fs::symlink(source.path(), target.path()).unwrap();
        let target_path = Utf8PathBuf::from_path_buf(target.to_path_buf()).unwrap();

        let json_def = janet2json(&format!("(symlink/remove \"{target_path}\")"));

        assert!(target_path.exists());
        let sut: GurpSymlinkRemove = serde_json::from_str(&json_def).unwrap();
        assert_eq!(ONE_RESOURCE_ONE_CHANGE, sut.apply(&defopts()).unwrap());
        assert!(!target_path.exists());
    }

    #[test]
    fn test_symlink_remove_noop() {
        let temp = TempDir::new().unwrap();
        let source = temp.child("source");
        let target = temp.child("target");
        source.write_str("some-content").unwrap();
        unix::fs::symlink(source.path(), target.path()).unwrap();
        let target_path = Utf8PathBuf::from_path_buf(target.to_path_buf()).unwrap();

        let json_def = janet2json(&format!("(symlink/remove \"{target_path}\")"));

        assert!(target_path.exists());
        let sut: GurpSymlinkRemove = serde_json::from_str(&json_def).unwrap();
        assert_eq!(ONE_RESOURCE_NOOP, sut.apply(&defopts_noop()).unwrap());
        assert!(target_path.exists());
    }

    #[test]
    fn test_symlink_remove_missing() {
        let json_def = janet2json("(symlink/remove \"/no/such/file\")");
        let sut: GurpSymlinkRemove = serde_json::from_str(&json_def).unwrap();
        assert_eq!(ONE_RESOURCE_NO_CHANGE, sut.apply(&defopts_noop()).unwrap());
    }
}
