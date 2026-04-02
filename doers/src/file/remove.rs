use anyhow::ensure;
use camino::Utf8PathBuf;
use common::constants::{ONE_RESOURCE_NO_CHANGE, ONE_RESOURCE_ONE_CHANGE, PROTECTED_FILES};
use common::types::{ApplyOpts, ApplySummary};
use serde::Deserialize;
use std::fmt::Debug;
use std::fs;

#[derive(Deserialize, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct GurpFileRemove {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(rename = "name")]
    pub path: Utf8PathBuf,
}

impl GurpFileRemove {
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        if self.path.exists() {
            ensure!(
                !PROTECTED_FILES.contains(&self.path),
                format!("protected resource: {}", self.path)
            );

            tracing::info!("removing: {}", self.path);
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
    use camino::Utf8PathBuf;
    use camino_tempfile_ext::prelude::*;
    use common::constants::ONE_RESOURCE_NOOP;
    use pretty_assertions::assert_eq;
    use tester::{defopts, defopts_noop, deserialized_example, janet2json};
    #[test]
    fn test_deserialize_remove_file() {
        assert_eq!(
            GurpFileRemove {
                id: "/NO-ROLE/file/_path_to_file".to_owned(),
                path: Utf8PathBuf::from("/path/to/file"),
            },
            deserialized_example("file/remove-file.janet")
        );
    }

    #[test]
    fn test_file_remove_does_not_exist() {
        let json_def = janet2json(r#"(file/remove "/path/does/not/exist")"#);
        let sut: GurpFileRemove = serde_json::from_str(&json_def).unwrap();

        assert_eq!(ONE_RESOURCE_NO_CHANGE, sut.apply(&defopts()).unwrap());
    }

    #[test]
    fn test_file_remove_forbidden() {
        let json_def = janet2json(r#"(file/remove "/bin/ps")"#);
        let sut: GurpFileRemove = serde_json::from_str(&json_def).unwrap();

        assert!(sut.apply(&defopts()).is_err());
    }

    #[test]
    fn test_file_remove() {
        let temp_dir = Utf8TempDir::new().unwrap();
        temp_dir
            .child("test-file")
            .write_str("transient-stuff")
            .unwrap();

        let temp_file = temp_dir.path().join("test-file");

        assert!(temp_file.exists());

        let json_def = janet2json(&format!("(file/remove \"{temp_file}\")"));
        let sut: GurpFileRemove = serde_json::from_str(&json_def).unwrap();

        assert_eq!(ONE_RESOURCE_ONE_CHANGE, sut.apply(&defopts()).unwrap());
        assert!(!temp_file.exists());
    }

    #[test]
    fn test_file_remove_noop() {
        let temp_dir = Utf8TempDir::new().unwrap();
        temp_dir
            .child("test-file")
            .write_str("transient-stuff")
            .unwrap();

        let temp_file = temp_dir.path().join("test-file");

        assert!(temp_file.exists());

        let json_def = janet2json(&format!("(file/remove \"{temp_file}\")"));
        let sut: GurpFileRemove = serde_json::from_str(&json_def).unwrap();

        assert_eq!(ONE_RESOURCE_NOOP, sut.apply(&defopts_noop()).unwrap());
        assert!(temp_file.exists());
    }
}
