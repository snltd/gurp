use crate::common::constants::{
    ONE_RESOURCE_NO_CHANGE, ONE_RESOURCE_NOOP, ONE_RESOURCE_ONE_CHANGE,
};
use crate::common::types::{ApplyContext, ApplySummary, Opts};
use anyhow::bail;
use camino::Utf8PathBuf;
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
    pub name: Utf8PathBuf, // The Path
    pub source: Utf8PathBuf,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct GurpSymlinkRemove {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: Utf8PathBuf, // The Path
}

impl GurpSymlinkEnsure {
    pub fn apply(
        &self,
        _apply_context: &ApplyContext,
        opts: &Opts,
    ) -> anyhow::Result<ApplySummary> {
        let target = &self.name;
        let source = &self.source;

        if !source.exists() {
            bail!("source not found: {}", source);
        }

        if !target.exists() {
            tracing::info!("creating symlink: {} -> {}", target, source);

            if opts.noop {
                Ok(ONE_RESOURCE_NOOP)
            } else {
                unix::fs::symlink(source, target)?;
                Ok(ONE_RESOURCE_ONE_CHANGE)
            }
        } else if target.is_symlink() {
            let current_source = target.read_link_utf8()?;
            if current_source == *source {
                tracing::info!("no change: {}", self.name);
                Ok(ONE_RESOURCE_NO_CHANGE)
            } else {
                tracing::info!(
                    "change symlink source: [{}] {} -> {}",
                    target,
                    &current_source,
                    source
                );
                if opts.noop {
                    Ok(ONE_RESOURCE_NOOP)
                } else {
                    fs::remove_file(target)?;
                    unix::fs::symlink(source, target)?;
                    Ok(ONE_RESOURCE_ONE_CHANGE)
                }
            }
        } else {
            bail!("{} exists and is not a symlink", &target);
        }
    }
}

impl GurpSymlinkRemove {
    pub fn apply(
        &self,
        _apply_context: &ApplyContext,
        opts: &Opts,
    ) -> anyhow::Result<ApplySummary> {
        if self.name.exists() {
            tracing::info!("removing symlink: {}", self.name);
            if opts.noop {
                Ok(ONE_RESOURCE_NOOP)
            } else {
                fs::remove_file(&self.name)?;
                Ok(ONE_RESOURCE_ONE_CHANGE)
            }
        } else {
            tracing::debug!("not present: {}", self.name);
            Ok(ONE_RESOURCE_NO_CHANGE)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils::spec_helper::{defopts, defopts_noop};
    use assert_fs::TempDir;
    use assert_fs::prelude::*;
    use camino::Utf8PathBuf;
    use std::os::unix;

    fn make_ensure_symlink(name: &Utf8PathBuf, source: Utf8PathBuf) -> GurpSymlinkEnsure {
        GurpSymlinkEnsure {
            id: "test-id".to_string(),
            name: name.clone(),
            source,
        }
    }

    fn make_remove_symlink(name: &Utf8PathBuf) -> GurpSymlinkRemove {
        GurpSymlinkRemove {
            id: "test-id".to_string(),
            name: name.clone(),
        }
    }

    #[test]
    fn test_symlink_creation() {
        let temp = TempDir::new().unwrap();
        let src = temp.child("src");
        let dst = temp.child("dst");
        src.write_str("data").unwrap();

        let symlink = make_ensure_symlink(
            &Utf8PathBuf::from_path_buf(dst.path().to_path_buf()).unwrap(),
            Utf8PathBuf::from_path_buf(src.path().to_path_buf()).unwrap(),
        );

        let result = symlink.apply(&ApplyContext::default(), &defopts()).unwrap();
        assert_eq!(result, ONE_RESOURCE_ONE_CHANGE);
        assert!(dst.path().is_symlink());
    }

    #[test]
    fn test_symlink_noop_creation() {
        let temp = TempDir::new().unwrap();
        let src = temp.child("src");
        let dst = temp.child("dst");
        src.write_str("noop").unwrap();

        let symlink = make_ensure_symlink(
            &Utf8PathBuf::from_path_buf(dst.path().to_path_buf()).unwrap(),
            Utf8PathBuf::from_path_buf(src.path().to_path_buf()).unwrap(),
        );

        let result = symlink
            .apply(&ApplyContext::default(), &defopts_noop())
            .unwrap();
        assert_eq!(result, ONE_RESOURCE_NOOP);
        assert!(!dst.path().exists());
    }

    #[test]
    fn test_symlink_removal() {
        let temp = TempDir::new().unwrap();
        let src = temp.child("src");
        let dst = temp.child("dst");
        src.write_str("x").unwrap();
        unix::fs::symlink(src.path(), dst.path()).unwrap();

        let symlink =
            make_remove_symlink(&Utf8PathBuf::from_path_buf(dst.path().to_path_buf()).unwrap());

        let result = symlink.apply(&ApplyContext::default(), &defopts()).unwrap();
        assert_eq!(result, ONE_RESOURCE_ONE_CHANGE);
        assert!(!dst.path().exists());
    }

    #[test]
    fn test_symlink_remove_missing() {
        let temp = TempDir::new().unwrap();
        let ghost = temp.child("ghost");

        let symlink =
            make_remove_symlink(&Utf8PathBuf::from_path_buf(ghost.path().to_path_buf()).unwrap());

        let result = symlink.apply(&ApplyContext::default(), &defopts()).unwrap();
        assert_eq!(result, ONE_RESOURCE_NO_CHANGE);
    }
}
