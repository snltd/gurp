use crate::constants::PROTECTED_FILES;
use anyhow::ensure;
use blake3::Hash;
use common::helpers;
use common::prelude::*;
use regex::Regex;
use serde::Deserialize;
use serde_json::Value;
use std::cell::RefCell;
use std::fmt::Debug;
use std::fs;
use std::io::Write;
use util::{file, http};

// THINGS TO KNOW
//
// remote file hashes are SHA256, even though we use Blake3 internally.

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
    pub backup_suffix: Option<String>,
    pub content: Option<String>,
    pub from_struct: Option<Value>,
    pub from_url: Option<String>,
    pub from: Option<Utf8PathBuf>,
    pub group: String,
    pub ignore_pattern: Option<String>,
    pub mode: String,
    pub owner: String,
    pub to_format: Option<String>,
    pub with_checksum: Option<String>,
    pub remote_content: RefCell<Option<Vec<u8>>>,
}

#[derive(Deserialize, Debug)]
pub struct GurpFileRemove {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(rename = "name")]
    pub path: Utf8PathBuf,
}

impl GurpFileEnsure {
    fn remote_content(&self, url: &str) -> anyhow::Result<()> {
        // As usual, complete MVP.
        // I don't think I want to cache anything between Gurp runs, so I don't have anywhere to
        // store ETags or whatever. (And I can't be sure the thing serving will serve them.)
        // Therefore, we're going to have to pull the file every time. Read it into memory and pop
        // it in the RefCell.
        let content = http::remove_file_to_memory(url)?;

        if let Some(checksum) = self.desired_state.with_checksum.as_ref() {
            let remote_checksum = sha256::digest(&content);

            ensure!(
                checksum == &remote_checksum,
                "Remote file has incorrect checksum"
            );
        }

        *self.desired_state.remote_content.borrow_mut() = Some(content);

        Ok(())
    }

    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        ensure!(
            self.content_xor_file_xor_content_struct(),
            "file '{}' must have exactly one of :content, :from, :from-url, or :from-struct",
            &self.path
        );

        ensure!(self.source_exists_if_needed(), "Missing source file");

        if let Some(remote_url) = &self.desired_state.from_url {
            self.remote_content(remote_url)?;
        }

        let mut changes = 0;

        if self.path.exists() {
            if self.file_has_changed()? {
                tracing::info!("updating {}", self.path);

                changes = 1;
                self.write_file(opts)?;
            } else {
                tracing::debug!("{} content is correct", self.path);
            }
        } else {
            tracing::info!("Creating {}", self.path);

            changes = 1;
            self.write_file(opts)?;
        }

        if opts.noop {
            return Ok(ApplySummary {
                resources: 1,
                changes,
            });
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

    fn content_xor_file_xor_content_struct(&self) -> bool {
        [
            self.desired_state.content.as_ref().map(|_| ()),
            self.desired_state.from_url.as_ref().map(|_| ()),
            self.desired_state.from.as_ref().map(|_| ()),
            self.desired_state.from_struct.as_ref().map(|_| ()),
        ]
        .iter()
        .filter(|v| v.is_some())
        .count()
            == 1
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
            // Filters apply
            (
                if let Some(from_file) = &self.desired_state.from {
                    self.hash_of_filtered_file(from_file, pattern)?
                } else if let Some(content) = &self.desired_state.content {
                    self.hash_of(&self.filter(content, pattern)?)
                } else if let Some(remote_content) =
                    self.desired_state.remote_content.borrow().as_ref()
                {
                    self.hash_of_bytes(remote_content.as_slice())
                } else if let Some(from_struct) = &self.desired_state.from_struct {
                    self.hash_of(
                        &self.filter(
                            &self.struct_to_file(
                                from_struct,
                                self.desired_state.to_format.as_deref(),
                            )?,
                            pattern,
                        )?,
                    )
                } else {
                    bail!("unable to determine filtered content hash of {}", self.path);
                },
                self.hash_of_filtered_file(&self.path, pattern)?,
            )
        } else {
            // Filters do not apply
            (
                if let Some(from_file) = &self.desired_state.from {
                    self.hash_of_file(from_file)?
                } else if let Some(content) = &self.desired_state.content {
                    self.hash_of(content)
                } else if let Some(remote_content) =
                    self.desired_state.remote_content.borrow().as_ref()
                {
                    self.hash_of_bytes(remote_content)
                } else if let Some(from_struct) = &self.desired_state.from_struct {
                    self.hash_of(
                        &self
                            .struct_to_file(from_struct, self.desired_state.to_format.as_deref())?,
                    )
                } else {
                    bail!(
                        "unable to determine unfiltered content hash of {}",
                        self.path
                    );
                },
                self.hash_of_file(&self.path)?,
            )
        };

        Ok(desired_hash != current_hash)
    }

    fn hash_of_bytes(&self, content: &[u8]) -> Hash {
        blake3::hash(content)
    }

    fn hash_of(&self, content: &str) -> Hash {
        self.hash_of_bytes(content.as_bytes())
    }

    fn hash_of_file(&self, path: &Utf8PathBuf) -> anyhow::Result<Hash> {
        let mut hasher = blake3::Hasher::new();
        let mut fh = fs::File::open(path)?;
        std::io::copy(&mut fh, &mut hasher)?;
        Ok(hasher.finalize())
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
        self.back_up_file(opts)?;

        let new_content = if let Some(content) = &self.desired_state.content {
            Some(content)
        } else if let Some(from_struct) = &self.desired_state.from_struct {
            Some(&self.struct_to_file(from_struct, self.desired_state.to_format.as_deref())?)
        } else {
            None
        };

        if opts.dump_diffs
            && let Some(new_content) = new_content
            && let Some(existing_content) = fs::read_to_string(&self.path).ok()
        {
            println!(
                "{}",
                &helpers::dump_diff(
                    &existing_content,
                    new_content,
                    self.path.as_str(),
                    opts.colour
                )
            );
        }

        if let Some(from) = &self.desired_state.from {
            tracing::debug!("copying {} -> {}", from, self.path);

            if !opts.noop {
                fs::copy(from, &self.path)?;
            }

            Ok(())
        } else if let Some(content) = new_content {
            tracing::debug!("Writing content to {}", self.path);

            if !opts.noop {
                let mut fh = fs::File::create(&self.path)?;
                fh.write_all(content.as_bytes())?;
            }

            Ok(())
        } else if let Some(content) = &self.desired_state.remote_content.borrow().as_ref() {
            if !opts.noop {
                let mut fh = fs::File::create(&self.path)?;
                fh.write_all(content)?;
            }

            Ok(())
        } else {
            bail!("nothing to write. Require :from, :content, :from-url, or :from-struct");
        }
    }

    fn back_up_file(&self, opts: &ApplyOpts) -> anyhow::Result<()> {
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
        } else {
            tracing::debug!("No backup of {} requested", &self.path);
        }

        Ok(())
    }

    // Ini files can't nest structs. If we get anything we don't expect, error. This is very basic.
    //
    fn struct_to_ini(&self, value: &Value) -> anyhow::Result<String> {
        let map = value
            .as_object()
            .ok_or_else(|| anyhow::anyhow!("Requested INI, but data is not a struct"))?;

        let mut ret = String::new();

        for (section_name, values) in map {
            let section_map = values
                .as_object()
                .ok_or_else(|| anyhow::anyhow!("Section '{}' must be a struct", section_name))?;

            if !ret.is_empty() {
                ret.push('\n');
            }

            ret.push_str(&format!("[{section_name}]\n"));

            for (k, v) in section_map {
                let string_k = self.prepped_kvp(k);
                let string_v = self.prepped_kvp(&v.to_string());

                let value = if string_v.chars().all(|c| c.is_alphanumeric()) {
                    string_v
                } else {
                    format!("\"{string_v}\"")
                };

                ret.push_str(&format!("{string_k} = {value}\n"));
            }
        }

        Ok(ret)
    }

    fn prepped_kvp(&self, raw: &str) -> String {
        raw.to_string().trim_matches(['"', ':']).to_owned()
    }

    // Very crude key-value pair. Accepts a map, or an array where alternate entries are key then
    // value. The latter lets you have duplicate keys, which I need.
    fn struct_to_k_equals_v(&self, value: &Value) -> anyhow::Result<String> {
        let mut ret = String::new();

        if let Some(map) = value.as_object() {
            for (k, v) in map {
                let clean_val = v.to_string().trim_matches('"').to_owned();
                ret.push_str(&format!("{k}={clean_val}\n"));
            }
        } else if let Some(map) = value.as_array() {
            if map.len() % 2 != 0 {
                bail!(
                    "KVP array must have an even number of elements. (Got {})",
                    map.len()
                );
            }

            for chunk in map.chunks(2) {
                let string_k = self.prepped_kvp(&chunk[0].to_string());
                let string_v = self.prepped_kvp(&chunk[1].to_string());
                ret.push_str(&format!("{string_k}={string_v}\n"));
            }
        } else {
            bail!("Requested k=v, but data is not a struct or array")
        }

        Ok(ret)
    }

    fn struct_to_file(&self, value: &Value, format: Option<&str>) -> anyhow::Result<String> {
        if let Some(format) = format {
            match format.to_lowercase().as_str() {
                "yaml" => Ok(serde_yaml_bw::to_string(&value)?),
                "toml" => Ok(toml::to_string(&value)?),
                "json" => Ok(serde_json::to_string_pretty(&value)?),
                "ini" => Ok(self.struct_to_ini(value)?),
                "k=v" => Ok(self.struct_to_k_equals_v(value)?),
                other => bail!("Unknown format: {}", other),
            }
        } else {
            bail!("from_struct requires to_format")
        }
    }
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
    use assert_fs::TempDir;
    use assert_fs::prelude::*;
    use camino::Utf8PathBuf;
    use httpmock::prelude::*;
    use indoc::{formatdoc, indoc};
    use pretty_assertions::assert_eq;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use tester::{defopts, defopts_noop, fixture, janet2json, load_fixture, my_group, my_user};

    #[test]
    fn test_file_create_noop() {
        let temp = TempDir::new().unwrap();
        let path = Utf8PathBuf::from_path_buf(temp.child("test-file").to_path_buf()).unwrap();

        let json_def = janet2json(&formatdoc! {r#"
            (file/ensure "{}"
                :content "some-junk"
                :mode "0750"
                :owner "{}"
                :group "{}")
            "#,
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

        let json_def = janet2json(&formatdoc! {r#"
            (file/ensure "{}"
                :content "stuff"
                :mode "0640"
                :owner "{}"
                :group "{}")
            "#,
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

    #[test]
    fn test_file_create_from_url() {
        let server = MockServer::start();

        let conf_mock = server.mock(|when, then| {
            when.method(GET).path("/sample/file");
            then.status(200)
                .header("content-type", "text/plain")
                .body(load_fixture("file/url-sample-file"));
        });

        let temp = TempDir::new().unwrap();
        let file = temp.join("test-file");
        let path = Utf8PathBuf::from_path_buf(file.to_path_buf()).unwrap();
        assert!(!path.exists());

        let json_def = janet2json(&formatdoc! {r#"
            (file/ensure "{}"
                :from-url "{}"
                :with-checksum "9c1b427039a6c786db0277fb96e3b0851a972dcdad832441e802d8b0de936ec3"
                :mode "0640"
                :owner "{}"
                :group "{}")
            "#,
            path,
            server.url("/sample/file"),
            my_user(),
            my_group(),
        });

        let sut: GurpFileEnsure = serde_json::from_str(&json_def).unwrap();
        assert_eq!(ONE_RESOURCE_ONE_CHANGE, sut.apply(&defopts()).unwrap());
        assert!(path.exists());
        conf_mock.assert();
        let metadata = fs::metadata(&path).unwrap();
        assert_eq!(metadata.permissions().mode() & 0o7777, 0o640);
        assert_eq!(
            load_fixture("file/url-sample-file"),
            fs::read_to_string(path).unwrap()
        );
    }

    #[test]
    fn test_file_create_from_url_404() {
        let server = MockServer::start();

        let conf_mock = server.mock(|when, then| {
            when.method(GET).path("/sample/file");
            then.status(404);
        });

        let json_def = janet2json(&formatdoc! {r#"
            (file/ensure "/does/not/matter"
                :from-url "{}"
                :mode "0640"
                :owner "{}"
                :group "{}")
            "#,
            server.url("/sample/file"),
            my_user(),
            my_group(),
        });

        let sut: GurpFileEnsure = serde_json::from_str(&json_def).unwrap();
        assert!(sut.apply(&defopts()).is_err());
        conf_mock.assert();
    }

    #[test]
    fn test_file_create_from_url_bad_checksum() {
        let server = MockServer::start();

        let conf_mock = server.mock(|when, then| {
            when.method(GET).path("/sample/file");
            then.status(200)
                .header("content-type", "text/plain")
                .body(load_fixture("file/url-sample-file"));
        });

        let json_def = janet2json(&formatdoc! {r#"
            (file/ensure "/does/not/matter"
                :from-url "{}"
                :with-checksum "0000000000000000000000000000000000000000000000000000000000000000"
                :mode "0640"
                :owner "{}"
                :group "{}")
            "#,
            server.url("/sample/file"),
            my_user(),
            my_group(),
        });

        let sut: GurpFileEnsure = serde_json::from_str(&json_def).unwrap();
        let err = sut.apply(&defopts()).unwrap_err();
        assert!(
            err.to_string()
                .contains("Remote file has incorrect checksum")
        );
        conf_mock.assert();
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
                from_struct: None,
                to_format: None,
                from_url: None,
                with_checksum: None,
                remote_content: RefCell::new(None),
            },
        };

        assert_eq!(ONE_RESOURCE_ONE_CHANGE, sut.apply(&defopts()).unwrap());
        assert!(path.exists());
    }

    #[test]
    fn test_not_exactly_one_source_fails() {
        let file_and_url = GurpFileEnsure {
            id: "IRRELEVANT".to_owned(),
            path: Utf8PathBuf::from("/does/not/matter"),
            desired_state: DesiredFileState {
                group: my_group(),
                mode: "2755".to_owned(),
                owner: my_user(),
                content: None,
                ignore_pattern: None,
                from: Some(fixture("file/binary-file")),
                backup_suffix: None,
                from_struct: None,
                to_format: None,
                from_url: Some("http://example.com/file".to_owned()),
                with_checksum: Some("abc123".to_owned()),
                remote_content: RefCell::new(None),
            },
        };

        assert!(file_and_url.apply(&defopts()).is_err());

        let file_and_content = GurpFileEnsure {
            id: "IRRELEVANT".to_owned(),
            path: Utf8PathBuf::from("/does/not/matter"),
            desired_state: DesiredFileState {
                group: my_group(),
                mode: "2755".to_owned(),
                owner: my_user(),
                content: Some("content".to_owned()),
                ignore_pattern: None,
                from: Some(fixture("file/binary-file")),
                backup_suffix: None,
                from_struct: None,
                to_format: None,
                from_url: None,
                with_checksum: None,
                remote_content: RefCell::new(None),
            },
        };

        assert!(file_and_content.apply(&defopts()).is_err());

        let no_source = GurpFileEnsure {
            id: "IRRELEVANT".to_owned(),
            path: Utf8PathBuf::from("/does/not/matter"),
            desired_state: DesiredFileState {
                group: my_group(),
                mode: "2755".to_owned(),
                owner: my_user(),
                content: None,
                ignore_pattern: None,
                from: None,
                backup_suffix: None,
                from_struct: None,
                to_format: None,
                from_url: None,
                with_checksum: None,
                remote_content: RefCell::new(None),
            },
        };

        assert!(no_source.apply(&defopts()).is_err());
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
                from_struct: None,
                to_format: None,
                from_url: None,
                with_checksum: None,
                remote_content: RefCell::new(None),
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
        let json_def = janet2json(&formatdoc! {r#"
            (file/ensure "{}"
                :content (template-out "{{{{ name }}}} is running a {{{{ thing }}}}"
                                        {{ :name "gurp" :thing "test" }})
                :mode "0600"
                :owner "{}"
                :group "{}")
            "#,
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
    fn test_file_create_json_from_struct() {
        let temp = TempDir::new().unwrap();
        let file = temp.join("test-file");
        let path = Utf8PathBuf::from_path_buf(file.to_path_buf()).unwrap();
        assert!(!path.exists());

        let expected = indoc! { r#"
                {
                  "my-arr": [
                    "abc",
                    "def",
                    "ghi"
                  ],
                  "my-str": "I am a String",
                  "my-struct": {
                    "key_1": "val 1",
                    "key_2": 123,
                    "key_3": [
                      456,
                      789
                    ]
                  }
                }"#};

        let sut: GurpFileEnsure = serde_json::from_str(&sample_struct(&path, "json")).unwrap();
        assert_eq!(ONE_RESOURCE_ONE_CHANGE, sut.apply(&defopts()).unwrap());
        assert!(path.exists());
        let metadata = fs::metadata(&path).unwrap();
        assert_eq!(metadata.permissions().mode() & 0o7777, 0o600);
        assert_eq!(expected, fs::read_to_string(path).unwrap());
    }

    #[test]
    fn test_file_create_yaml_from_struct() {
        let temp = TempDir::new().unwrap();
        let file = temp.join("test-file");
        let path = Utf8PathBuf::from_path_buf(file.to_path_buf()).unwrap();
        assert!(!path.exists());

        let expected = indoc! { r#"
            my-arr:
            - abc
            - def
            - ghi
            my-str: I am a String
            my-struct:
              key_1: val 1
              key_2: 123
              key_3:
              - 456
              - 789
          "#};

        let sut: GurpFileEnsure = serde_json::from_str(&sample_struct(&path, "yaml")).unwrap();
        assert_eq!(ONE_RESOURCE_ONE_CHANGE, sut.apply(&defopts()).unwrap());
        assert!(path.exists());
        let metadata = fs::metadata(&path).unwrap();
        assert_eq!(metadata.permissions().mode() & 0o7777, 0o600);
        assert_eq!(expected, fs::read_to_string(path).unwrap());
    }

    #[test]
    fn test_file_create_ini_from_struct() {
        let temp = TempDir::new().unwrap();
        let file = temp.join("test-file");
        let path = Utf8PathBuf::from_path_buf(file.to_path_buf()).unwrap();
        assert!(!path.exists());

        let json_def = janet2json(&formatdoc! {r#"
            (file/ensure "{path}"
                :from-struct {{
                    :section_1 {{
                        :key_1 "A spacey string"
                        :key_2 123
                        :key_3 false
                        :key_4 "word"
                    }}
                    :section_2 {{
                        :key_1 "merp"
                        :key_2 "gurp"
                    }}
                }}
                :to-format "ini"
                :mode "0600"
                :owner "{user}"
                :group "{group}")
            "#,
            path = path,
            user = my_user(),
            group = my_group(),
        });

        let expected = indoc! { r#"
                [section_1]
                key_1 = "A spacey string"
                key_2 = 123
                key_3 = false
                key_4 = word

                [section_2]
                key_1 = merp
                key_2 = gurp
        "#};

        let sut: GurpFileEnsure = serde_json::from_str(&json_def).unwrap();
        assert_eq!(ONE_RESOURCE_ONE_CHANGE, sut.apply(&defopts()).unwrap());
        assert!(path.exists());
        let metadata = fs::metadata(&path).unwrap();
        assert_eq!(metadata.permissions().mode() & 0o7777, 0o600);
        assert_eq!(expected, fs::read_to_string(path).unwrap());
    }

    #[test]
    fn test_file_create_ini_from_struct_errors() {
        let sut: GurpFileEnsure =
            serde_json::from_str(&sample_struct(&Utf8PathBuf::from("/tmp/file"), "ini")).unwrap();
        assert!(sut.apply(&defopts()).is_err());
    }

    #[test]
    fn test_file_ensure_already_correct() {
        let temp = TempDir::new().unwrap();
        temp.child("test-file").write_str("stuff").unwrap();
        let file = temp.join("test-file");

        let path = Utf8PathBuf::from_path_buf(file.to_path_buf()).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o0750)).unwrap();
        assert!(path.exists());

        let json_def = janet2json(&formatdoc! {r#"
            (file/ensure "{}"
                :content "stuff"
                :mode "0750"
                :owner "{}"
                :group "{}")
            "#,
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

        let json_def = janet2json(&formatdoc! {r#"
            (file/ensure "{}"
                :content "the-right-stuff"
                :mode "0400"
                :owner "{}"
                :group "{}")
            "#,
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

        let json_def = janet2json(&formatdoc! {r#"
            (file/ensure "{}"
                :from "{}"
                :mode "0444"
                :owner "{}"
                :group "{}")
            "#,
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

        let json_def = janet2json(&formatdoc! {r#"
            (file/ensure "{}"
                :from "{}"
                :mode "0600"
                :ignore-pattern "^today is"
                :owner "{}"
                :group "{}")
            "#,
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
        assert!(sut.apply(&defopts()).is_err());
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

    fn sample_struct(path: &Utf8PathBuf, format: &str) -> String {
        janet2json(&formatdoc! {r#"
            (file/ensure "{path}"
                :from-struct {{
                    :my-struct {{
                        :key_1 "val 1"
                        :key_2 123
                        :key_3 [456 789]
                    }}
                    :my-arr ["abc" "def" "ghi"]
                    :my-str "I am a String"
                }}
                :to-format "{format}"
                :mode "0600"
                :owner "{user}"
                :group "{group}")
            "#,
            path = path,
            format = format,
            user = my_user(),
            group = my_group(),
        })
    }
}
