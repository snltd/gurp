use crate::constants::PROTECTED_FILES;
use anyhow::ensure;
use blake3::Hash;
use common::prelude::*;
use regex::Regex;
use serde::Deserialize;
use std::fmt::Debug;
use std::fs;
use std::io::Write;
use util::file;

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct GurpFileEnsure {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(rename = "name")]
    pub path: Utf8PathBuf,
    #[serde(flatten)]
    pub desired_state: DesiredFileState,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct DesiredFileState {
    pub group: String,
    pub mode: String,
    pub owner: String,
    pub content: Option<String>,
    pub ignore_pattern: Option<String>,
    pub from: Option<Utf8PathBuf>,
    pub backup_suffix: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct GurpFileRemove {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(rename = "name")]
    pub path: Utf8PathBuf,
}

impl GurpFileEnsure {
    fn content_xor_file(&self) -> bool {
        (self.desired_state.content.is_some() && self.desired_state.from.is_none())
            || (self.desired_state.content.is_none() && self.desired_state.from.is_some())
    }

    fn source_exists_if_needed(&self) -> bool {
        match &self.desired_state.from {
            Some(file) => {
                if file.exists() {
                    true
                } else {
                    tracing::error!("source file {} not found", file);
                    false
                }
            }

            None => true,
        }
    }

    fn file_has_changed(&self) -> anyhow::Result<bool> {
        let (desired_hash, current_hash) = if let Some(pattern) = &self.desired_state.ignore_pattern
        {
            (
                if let Some(from_file) = &self.desired_state.from {
                    self.hash_of_filtered_file(from_file, pattern)?
                } else if let Some(content) = &self.desired_state.content {
                    self.hash_of(&self.filter(content, pattern)?)
                } else {
                    bail!("No :content or :from")
                },
                self.hash_of_filtered_file(&self.path, pattern)?,
            )
        } else {
            (
                if let Some(from_file) = &self.desired_state.from {
                    self.hash_of_file(from_file)?
                } else if let Some(content) = &self.desired_state.content {
                    self.hash_of(content)
                } else {
                    bail!("No :content or :from");
                },
                self.hash_of_file(&self.path)?,
            )
        };

        Ok(desired_hash != current_hash)
    }

    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        ensure!(
            self.content_xor_file(),
            "file '{}' must have exactly one of :content or :from",
            &self.path
        );

        ensure!(self.source_exists_if_needed(), "Missing source file");

        let mut changes = 0;

        if self.path.exists() {
            if self.file_has_changed()? {
                if !opts.noop {
                    self.write_file(opts)?;
                }
                changes = 1;
            } else {
                tracing::debug!("file content is correct");
            }
        } else {
            return_if_noop!(opts);

            changes = 1;
            self.write_file(opts)?;
        }

        file::ensure_metadata(
            FileMetadata {
                group: &self.desired_state.group,
                mode: &self.desired_state.mode,
                owner: &self.desired_state.owner,
                path: &self.path,
                changes,
            },
            opts,
        )
    }

    fn hash_of(&self, content: &str) -> Hash {
        blake3::hash(content.as_bytes())
    }

    fn hash_of_file(&self, path: &Utf8PathBuf) -> anyhow::Result<Hash> {
        let raw = fs::read_to_string(path)?;
        Ok(self.hash_of(&raw))
    }

    fn hash_of_filtered_file(&self, path: &Utf8PathBuf, pattern: &str) -> anyhow::Result<Hash> {
        let raw = fs::read_to_string(path)?;
        let filtered = self.filter(&raw, pattern)?;
        Ok(self.hash_of(&filtered))
    }

    fn filter(&self, content: &str, filter: &str) -> anyhow::Result<String> {
        tracing::debug!("filtering content on '{}'", filter);
        let rx = Regex::new(filter)?;
        let filtered_lines: Vec<_> = content.lines().filter(|l| !rx.is_match(l)).collect();
        Ok(filtered_lines.join("\n"))
    }

    fn write_file(&self, opts: &ApplyOpts) -> anyhow::Result<()> {
        if let Some(from) = &self.desired_state.from {
            tracing::debug!("copying {} -> {}", from, self.path);
            fs::copy(from, &self.path)?;
            Ok(())
        } else if let Some(content) = &self.desired_state.content {
            if let Some(suffix) = &self.desired_state.backup_suffix {
                let suffix = if suffix == "TIMESTAMP" {
                    helpers::epoch_time_as_string()
                } else {
                    suffix.to_owned()
                };

                let backup_target = &self.path.with_extension(suffix);
                tracing::debug!("Backing up to {}", backup_target);

                if !opts.noop {
                    fs::rename(&self.path, backup_target)?;
                    file::ensure_metadata(
                        FileMetadata {
                            group: "root",
                            owner: "root",
                            mode: "0o0400",
                            path: backup_target,
                            changes: 1,
                        },
                        opts,
                    )?;
                }
            }

            tracing::debug!("Writing content to {}", self.path);

            if !opts.noop {
                let mut fh = fs::File::create(&self.path)?;
                fh.write_all(content.as_bytes())?;
            }

            Ok(())
        } else {
            bail!("can write neither :from nor :content");
        }
    }
}

impl GurpFileRemove {
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        if self.path.exists() {
            if PROTECTED_FILES.contains(&self.path) {
                tracing::warn!("protected resource: {}", self.path);
                return Ok(ONE_RESOURCE_ONE_ERROR);
            }

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
    use assert_fs::TempDir;
    use assert_fs::prelude::*;
    use camino::Utf8PathBuf;
    use indoc::formatdoc;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use tester::{defopts, defopts_noop, fixture, janet2json, my_group, my_user};

    #[test]
    fn test_file_create_noop() {
        let temp = TempDir::new().unwrap();
        let path = Utf8PathBuf::from_path_buf(temp.child("test-file").to_path_buf()).unwrap();

        let json_def = janet2json(&formatdoc! {"
            (file/ensure \"{}\"
                :content \"some-junk\"
                :mode \"0750\"
                :owner \"{}\"
                :group \"{}\")
            ",
            path,
            my_user(),
            my_group(),
        });

        assert!(!path.exists());
        let sut: GurpFileEnsure = serde_json::from_str(&json_def).unwrap();
        assert_eq!(ONE_RESOURCE_NOOP, sut.apply(&defopts_noop()).unwrap());
        assert!(!path.exists());
    }

    #[test]
    fn test_file_create_from_content() {
        let temp = TempDir::new().unwrap();
        let file = temp.join("test-file");
        let path = Utf8PathBuf::from_path_buf(file.to_path_buf()).unwrap();
        assert!(!path.exists());

        let json_def = janet2json(&formatdoc! {"
            (file/ensure \"{}\"
                :content \"stuff\"
                :mode \"0640\"
                :owner \"{}\"
                :group \"{}\")
            ",
            path,
            my_user(),
            my_group(),
        });

        let sut: GurpFileEnsure = serde_json::from_str(&json_def).unwrap();
        assert_eq!(ONE_RESOURCE_ONE_CHANGE, sut.apply(&defopts()).unwrap());
        assert!(path.exists());
        let metadata = fs::metadata(&path).unwrap();
        assert_eq!(metadata.permissions().mode() & 0o7777, 0o640);
        assert_eq!("stuff", fs::read_to_string(path).unwrap());
    }

    // We can't test backup because it does things a normal user can't

    #[test]
    fn test_create_binary_file() {
        let temp = TempDir::new().unwrap();
        let file = temp.join("binary-file");
        let path = Utf8PathBuf::from_path_buf(file.to_path_buf()).unwrap();
        assert!(!path.exists());

        let sut = GurpFileEnsure {
            id: "IRRELEVANT".to_owned(),
            path: path.clone(),
            desired_state: DesiredFileState {
                group: my_group(),
                mode: "0755".to_owned(),
                owner: my_user(),
                content: None,
                ignore_pattern: None,
                from: Some(fixture("file/binary-file")),
                backup_suffix: None,
            },
        };

        assert_eq!(ONE_RESOURCE_ONE_CHANGE, sut.apply(&defopts()).unwrap());
        assert!(path.exists());
    }

    #[test]
    fn test_create_binary_file_setuid() {
        let temp = TempDir::new().unwrap();
        let file = temp.join("binary-file");
        let path = Utf8PathBuf::from_path_buf(file.to_path_buf()).unwrap();
        assert!(!path.exists());

        let sut = GurpFileEnsure {
            id: "IRRELEVANT".to_owned(),
            path: path.clone(),
            desired_state: DesiredFileState {
                group: my_group(),
                mode: "2755".to_owned(),
                owner: my_user(),
                content: None,
                ignore_pattern: None,
                from: Some(fixture("file/binary-file")),
                backup_suffix: None,
            },
        };

        assert_eq!(ONE_RESOURCE_ONE_CHANGE, sut.apply(&defopts()).unwrap());
        let metadata = fs::metadata(&path).unwrap();
        assert_eq!(metadata.permissions().mode() & 0o7777, 0o2755);
        assert!(path.exists());
    }

    #[test]
    fn test_file_create_from_template() {
        let temp = TempDir::new().unwrap();
        let file = temp.join("test-file");
        let path = Utf8PathBuf::from_path_buf(file.to_path_buf()).unwrap();
        assert!(!path.exists());

        // escape { with another {
        let json_def = janet2json(&formatdoc! {"
            (file/ensure \"{}\"
                :content (template-out \"{{{{ name }}}} is running a {{{{ thing }}}}\"
                                        {{ :name \"gurp\" :thing \"test\" }})
                :mode \"0600\"
                :owner \"{}\"
                :group \"{}\")
            ",
            path,
            my_user(),
            my_group(),
        });

        let sut: GurpFileEnsure = serde_json::from_str(&json_def).unwrap();
        assert_eq!(ONE_RESOURCE_ONE_CHANGE, sut.apply(&defopts()).unwrap());
        assert!(path.exists());
        let metadata = fs::metadata(&path).unwrap();
        assert_eq!(metadata.permissions().mode() & 0o7777, 0o600);
        assert_eq!("gurp is running a test", fs::read_to_string(path).unwrap());
    }

    #[test]
    fn test_file_ensure_already_correct() {
        let temp = TempDir::new().unwrap();
        temp.child("test-file").write_str("stuff").unwrap();
        let file = temp.join("test-file");

        let path = Utf8PathBuf::from_path_buf(file.to_path_buf()).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o0750)).unwrap();
        assert!(path.exists());

        let json_def = janet2json(&formatdoc! {"
            (file/ensure \"{}\"
                :content \"stuff\"
                :mode \"0750\"
                :owner \"{}\"
                :group \"{}\")
            ",
            path,
            my_user(),
            my_group(),
        });

        let sut: GurpFileEnsure = serde_json::from_str(&json_def).unwrap();
        assert_eq!(ONE_RESOURCE_NO_CHANGE, sut.apply(&defopts()).unwrap());
        assert!(path.exists());
    }

    #[test]
    fn test_update_file_from_content_and_set_mode() {
        let temp = TempDir::new().unwrap();
        temp.child("test-file")
            .write_str("the-wrong-stuff")
            .unwrap();

        let path = Utf8PathBuf::from_path_buf(temp.to_path_buf())
            .unwrap()
            .join("test-file");
        assert!(path.exists());

        let json_def = janet2json(&formatdoc! {"
            (file/ensure \"{}\"
                :content \"the-right-stuff\"
                :mode \"0400\"
                :owner \"{}\"
                :group \"{}\")
            ",
            path,
            my_user(),
            my_group(),
        });

        let sut: GurpFileEnsure = serde_json::from_str(&json_def).unwrap();
        assert_eq!(ONE_RESOURCE_ONE_CHANGE, sut.apply(&defopts()).unwrap());

        assert!(path.exists());
        assert_eq!(
            "the-right-stuff".to_owned(),
            fs::read_to_string(&path).unwrap()
        );

        let metadata = fs::metadata(&path).unwrap();
        assert_eq!(metadata.permissions().mode() & 0o7777, 0o0400);
    }

    #[test]
    fn test_update_file_from_file_and_set_mode() {
        let temp = TempDir::new().unwrap();
        temp.child("test-file")
            .write_str("the-wrong-stuff")
            .unwrap();

        let path = Utf8PathBuf::from_path_buf(temp.to_path_buf())
            .unwrap()
            .join("test-file");

        assert!(path.exists());

        let json_def = janet2json(&formatdoc! {"
            (file/ensure \"{}\"
                :from \"{}\"
                :mode \"0444\"
                :owner \"{}\"
                :group \"{}\")
            ",
            path,
            &fixture("file/copy-file"),
            my_user(),
            my_group(),
        });

        let sut: GurpFileEnsure = serde_json::from_str(&json_def).unwrap();
        assert_eq!(ONE_RESOURCE_ONE_CHANGE, sut.apply(&defopts()).unwrap());

        assert!(path.exists());
        assert_eq!(
            "some-content\n".to_owned(),
            fs::read_to_string(&path).unwrap()
        );

        let metadata = fs::metadata(&path).unwrap();
        assert_eq!(metadata.permissions().mode() & 0o7777, 0o444);
    }

    #[test]
    fn test_ignored_line_means_no_change_with_content() {
        let content = "today is 2015-01-30\nBut this never changes.\nAnd nor does this.\n";
        let temp = TempDir::new().unwrap();
        temp.child("test-file").write_str(content).unwrap();

        let path = Utf8PathBuf::from_path_buf(temp.to_path_buf())
            .unwrap()
            .join("test-file");

        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        assert!(path.exists());

        let json_def = janet2json(&formatdoc! {"
            (file/ensure \"{}\"
                :content \"today is 2025-06-26\\nBut this never changes.\\nAnd nor does this.\"
                :mode \"0600\"
                :ignore-pattern \"^today is\"
                :owner \"{}\"
                :group \"{}\")
            ",
            path,
            my_user(),
            my_group(),
        });

        let sut: GurpFileEnsure = serde_json::from_str(&json_def).unwrap();
        assert_eq!(ONE_RESOURCE_NO_CHANGE, sut.apply(&defopts()).unwrap());
        assert_eq!(content, fs::read_to_string(&path).unwrap());
    }

    #[test]
    fn test_ignored_line_means_no_change_with_from() {
        let content = "today is 2015-01-30\nBut this never changes.\nAnd nor does this.\n";
        let temp = TempDir::new().unwrap();
        temp.child("test-file").write_str(content).unwrap();

        let path = Utf8PathBuf::from_path_buf(temp.to_path_buf())
            .unwrap()
            .join("test-file");

        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        assert!(path.exists());

        let json_def = janet2json(&formatdoc! {"
            (file/ensure \"{}\"
                :from \"{}\"
                :mode \"0600\"
                :ignore-pattern \"^today is\"
                :owner \"{}\"
                :group \"{}\")
            ",
            path,
            fixture("file/ignore-line-file"),
            my_user(),
            my_group(),
        });

        let sut: GurpFileEnsure = serde_json::from_str(&json_def).unwrap();
        assert_eq!(ONE_RESOURCE_NO_CHANGE, sut.apply(&defopts()).unwrap());
        assert_eq!(content, fs::read_to_string(&path).unwrap());
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
        assert_eq!(ONE_RESOURCE_ONE_ERROR, sut.apply(&defopts()).unwrap());
    }

    #[test]
    fn test_file_remove() {
        let temp = TempDir::new().unwrap();
        temp.child("test-file")
            .write_str("transient-stuff")
            .unwrap();

        let path = Utf8PathBuf::from_path_buf(temp.to_path_buf())
            .unwrap()
            .join("test-file");

        assert!(path.exists());
        let json_def = janet2json(&format!("(file/remove \"{path}\")"));
        let sut: GurpFileRemove = serde_json::from_str(&json_def).unwrap();
        assert_eq!(ONE_RESOURCE_ONE_CHANGE, sut.apply(&defopts()).unwrap());
        assert!(!path.exists());
    }

    #[test]
    fn test_file_remove_noop() {
        let temp = TempDir::new().unwrap();
        temp.child("test-file")
            .write_str("transient-stuff")
            .unwrap();

        let path = Utf8PathBuf::from_path_buf(temp.to_path_buf())
            .unwrap()
            .join("test-file");

        assert!(path.exists());
        let json_def = janet2json(&format!("(file/remove \"{path}\")"));
        let sut: GurpFileRemove = serde_json::from_str(&json_def).unwrap();
        assert_eq!(ONE_RESOURCE_NOOP, sut.apply(&defopts_noop()).unwrap());
        assert!(path.exists());
    }
}
