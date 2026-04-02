use crate::file::types::{CompareMethod, DesiredFileState, FileSource};
use crate::file::{from_content, from_file, from_struct};
use anyhow::{Context, bail, ensure};
use camino::Utf8PathBuf;
use common::types::{ApplyOpts, ApplySummary};
use serde::Deserialize;
use std::fmt::Debug;

#[derive(Deserialize, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct GurpFileEnsure {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(rename = "name")]
    pub path: Utf8PathBuf,
    #[serde(flatten)]
    pub desired_state: DesiredFileState,
}

impl GurpFileEnsure {
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        ensure!(
            self.single_source(),
            "file '{}' must have exactly one of :content, :from, :from-url, or :from-struct",
            &self.path
        );

        ensure!(
            self.path.is_absolute(),
            "path must be absolute [{}]",
            self.id
        );

        if self.path.exists() {
            ensure!(
                self.path.is_file(),
                "{} exists and is not a file",
                self.path
            );
        }

        ensure!(
            &self
                .path
                .parent()
                .context(format!("cannot get parent of {}", self.path))?
                .exists(),
            "cannot create {}: parent dir does not exist",
            self.path
        );

        let source = self.source_type()?;

        let compare = if let Some(pattern) = &self.desired_state.ignore_pattern {
            CompareMethod::Filter(pattern)
        } else {
            CompareMethod::Hash
        };

        match source {
            FileSource::File => from_file::run(&self.path, &self.desired_state, &compare, opts),
            FileSource::Literal => {
                from_content::run(&self.path, &self.desired_state, &compare, opts)
            }
            FileSource::Url => {
                // if compare == CompareMethod::Hash {
                //     remote::check_hash(url, compare_method, &self.path);
                // }
                todo!()
            }
            FileSource::Struct => from_struct::run(&self.path, &self.desired_state, &compare, opts),
        }
    }

    fn source_type(&self) -> anyhow::Result<FileSource> {
        if self.desired_state.from.is_some() {
            Ok(FileSource::File)
        } else if self.desired_state.content.is_some() {
            Ok(FileSource::Literal)
        } else if self.desired_state.from_url.is_some() {
            Ok(FileSource::Url)
        } else if self.desired_state.from_struct.is_some() {
            Ok(FileSource::Struct)
        } else {
            bail!("impossible source type");
        }
    }

    fn single_source(&self) -> bool {
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
}

#[cfg(test)]
mod test {
    use super::*;
    use camino::Utf8PathBuf;
    use camino_tempfile_ext::prelude::*;
    use common::constants::{ONE_RESOURCE_NO_CHANGE, ONE_RESOURCE_NOOP, ONE_RESOURCE_ONE_CHANGE};
    use httpmock::prelude::*;
    use indoc::formatdoc;
    use pretty_assertions::assert_eq;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use tester::{
        defopts, defopts_noop, deserialized_example, fixture, janet2json, load_fixture, my_group,
        my_user,
    };
    use util::file::NameOrId;

    #[test]
    fn test_deserialize_ensure_file_from_content() {
        assert_eq!(
            GurpFileEnsure {
                id: "/NO-ROLE/file/_example_file_from-content".to_owned(),
                path: Utf8PathBuf::from("/example/file/from-content"),
                desired_state: DesiredFileState {
                    backup_suffix: None,
                    content: Some("words and stuff".to_owned()),
                    from_struct: None,
                    from_url: None,
                    from: None,
                    ignore_pattern: None,
                    mode: "0600".to_owned(),
                    group: NameOrId::Name("root".to_owned()),
                    owner: NameOrId::Name("sys".to_owned()),
                    to_format: None,
                    with_checksum: None,
                    only_fetch_from_url_once: false,
                    url_is_server: false,
                }
            },
            deserialized_example("file/ensure-from-content.janet")
        );
    }

    #[test]
    fn test_deserialize_ensure_file_from_url_with_checksum() {
        assert_eq!(
            GurpFileEnsure {
                id: "/NO-ROLE/file/remote-file".to_owned(),
                path: Utf8PathBuf::from("/example/file/from-url"),
                desired_state: DesiredFileState {
                    backup_suffix: None,
                    content: None,
                    from: None,
                    from_struct: None,
                    from_url: Some(
                        "https://raw.githubusercontent.com/snltd/gurp/refs/heads/main/LICENSE.txt"
                            .to_owned()
                    ),
                    with_checksum: Some(
                        "561a47aa1d1bfc3a95ce45345639f9ce2d9ad332b05cfe5da74ad77f2842ee16"
                            .to_owned()
                    ),
                    ignore_pattern: None,
                    mode: "0644".to_owned(),
                    owner: NameOrId::Name("root".to_owned()),
                    group: NameOrId::Name("root".to_owned()),
                    to_format: None,
                    only_fetch_from_url_once: false,
                    url_is_server: false,
                }
            },
            deserialized_example("file/ensure-from-url-with-checksum.janet")
        );
    }

    #[test]
    fn test_file_create_noop() {
        let temp_dir = Utf8TempDir::new().unwrap();
        let temp_file = temp_dir.child("test-file");

        let json_def = janet2json(&formatdoc! {r#"
            (file/ensure "{}"
                :content "some-junk"
                :mode "0750"
                :owner "{}"
                :group "{}")
            "#,
            temp_file.as_path(),
            my_user(),
            my_group(),
        });

        assert!(!temp_file.exists());
        let sut: GurpFileEnsure = serde_json::from_str(&json_def).unwrap();
        assert_eq!(ONE_RESOURCE_NOOP, sut.apply(&defopts_noop()).unwrap());
        assert!(!temp_file.exists());
    }

    #[test]
    fn test_file_create_from_content() {
        let temp_dir = Utf8TempDir::new().unwrap();
        let temp_file = temp_dir.path().join("test-file");

        assert!(!temp_file.exists());

        let json_def = janet2json(&formatdoc! {r#"
            (file/ensure "{}"
                :content "stuff"
                :mode "0640"
                :owner "{}"
                :group "{}")
            "#,
            temp_file.as_path(),
            my_user(),
            my_group(),
        });

        let sut: GurpFileEnsure = serde_json::from_str(&json_def).unwrap();

        assert_eq!(ONE_RESOURCE_ONE_CHANGE, sut.apply(&defopts()).unwrap());
        assert!(temp_file.exists());

        let metadata = fs::metadata(&temp_file).unwrap();

        assert_eq!(metadata.permissions().mode() & 0o7777, 0o640);
        assert_eq!("stuff", fs::read_to_string(temp_file).unwrap());
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

        let temp_dir = Utf8TempDir::new().unwrap();
        let temp_file = temp_dir.path().join("test-file");

        assert!(!temp_file.exists());

        let json_def = janet2json(&formatdoc! {r#"
            (file/ensure "{}"
                :from-url "{}"
                :with-checksum "9c1b427039a6c786db0277fb96e3b0851a972dcdad832441e802d8b0de936ec3"
                :mode "0640"
                :owner "{}"
                :group "{}")
            "#,
            temp_file.as_path(),
            server.url("/sample/file"),
            my_user(),
            my_group(),
        });

        let sut: GurpFileEnsure = serde_json::from_str(&json_def).unwrap();

        assert_eq!(ONE_RESOURCE_ONE_CHANGE, sut.apply(&defopts()).unwrap());
        assert!(temp_file.exists());
        conf_mock.assert();

        let metadata = fs::metadata(&temp_file).unwrap();

        assert_eq!(metadata.permissions().mode() & 0o7777, 0o640);
        assert_eq!(
            load_fixture("file/url-sample-file"),
            fs::read_to_string(temp_file).unwrap()
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
        let temp_dir = Utf8TempDir::new().unwrap();
        let temp_file = temp_dir.path().join("binary-file");

        assert!(!temp_file.exists());

        let sut = GurpFileEnsure {
            id: "IRRELEVANT".to_owned(),
            path: temp_file.clone(),
            desired_state: DesiredFileState {
                mode: "0755".to_owned(),
                group: NameOrId::Name(my_group()),
                owner: NameOrId::Name(my_user()),
                content: None,
                ignore_pattern: None,
                from: Some(fixture("file/binary-file")),
                backup_suffix: None,
                from_struct: None,
                to_format: None,
                from_url: None,
                with_checksum: None,
                only_fetch_from_url_once: false,
                url_is_server: false,
            },
        };

        assert_eq!(ONE_RESOURCE_ONE_CHANGE, sut.apply(&defopts()).unwrap());
        assert!(temp_file.exists());
    }

    #[test]
    fn test_not_exactly_one_source_fails() {
        let file_and_url = GurpFileEnsure {
            id: "IRRELEVANT".to_owned(),
            path: Utf8PathBuf::from("/does/not/matter"),
            desired_state: DesiredFileState {
                group: NameOrId::Name(my_group()),
                owner: NameOrId::Name(my_user()),
                mode: "2755".to_owned(),
                content: None,
                ignore_pattern: None,
                from: Some(fixture("file/binary-file")),
                backup_suffix: None,
                from_struct: None,
                to_format: None,
                from_url: Some("http://example.com/file".to_owned()),
                with_checksum: Some("abc123".to_owned()),
                only_fetch_from_url_once: false,
                url_is_server: false,
            },
        };

        assert!(file_and_url.apply(&defopts()).is_err());

        let file_and_content = GurpFileEnsure {
            id: "IRRELEVANT".to_owned(),
            path: Utf8PathBuf::from("/does/not/matter"),
            desired_state: DesiredFileState {
                group: NameOrId::Name(my_group()),
                owner: NameOrId::Name(my_user()),
                mode: "2755".to_owned(),
                content: Some("content".to_owned()),
                ignore_pattern: None,
                from: Some(fixture("file/binary-file")),
                backup_suffix: None,
                from_struct: None,
                to_format: None,
                from_url: None,
                with_checksum: None,
                only_fetch_from_url_once: false,
                url_is_server: false,
            },
        };

        assert!(file_and_content.apply(&defopts()).is_err());

        let no_source = GurpFileEnsure {
            id: "IRRELEVANT".to_owned(),
            path: Utf8PathBuf::from("/does/not/matter"),
            desired_state: DesiredFileState {
                group: NameOrId::Name(my_group()),
                owner: NameOrId::Name(my_user()),
                mode: "2755".to_owned(),
                content: None,
                ignore_pattern: None,
                from: None,
                backup_suffix: None,
                from_struct: None,
                to_format: None,
                from_url: None,
                with_checksum: None,
                only_fetch_from_url_once: false,
                url_is_server: false,
            },
        };

        assert!(no_source.apply(&defopts()).is_err());
    }

    #[test]
    fn test_create_binary_file_setuid() {
        let temp_dir = Utf8TempDir::new().unwrap();
        let temp_file = temp_dir.path().join("binary-file");

        assert!(!temp_file.exists());

        let sut = GurpFileEnsure {
            id: "IRRELEVANT".to_owned(),
            path: temp_file.clone(),
            desired_state: DesiredFileState {
                group: NameOrId::Name(my_group()),
                owner: NameOrId::Name(my_user()),
                mode: "2755".to_owned(),
                content: None,
                ignore_pattern: None,
                from: Some(fixture("file/binary-file")),
                backup_suffix: None,
                from_struct: None,
                to_format: None,
                from_url: None,
                with_checksum: None,
                only_fetch_from_url_once: false,
                url_is_server: false,
            },
        };

        assert_eq!(ONE_RESOURCE_ONE_CHANGE, sut.apply(&defopts()).unwrap());

        let metadata = fs::metadata(&temp_file).unwrap();

        assert_eq!(metadata.permissions().mode() & 0o7777, 0o2755);
        assert!(temp_file.exists());
    }

    #[test]
    fn test_file_create_from_template() {
        let temp_dir = Utf8TempDir::new().unwrap();
        let temp_file = temp_dir.path().join("test-file");

        assert!(!temp_file.exists());

        // escape { with another {
        let json_def = janet2json(&formatdoc! {r#"
            (file/ensure "{}"
                :content (template-out "{{{{ name }}}} is running a {{{{ thing }}}}"
                                        {{ :name "gurp" :thing "test" }})
                :mode "0600"
                :owner "{}"
                :group "{}")
            "#,
            temp_file.as_path(),
            my_user(),
            my_group(),
        });

        let sut: GurpFileEnsure = serde_json::from_str(&json_def).unwrap();
        assert_eq!(ONE_RESOURCE_ONE_CHANGE, sut.apply(&defopts()).unwrap());
        assert!(temp_file.exists());
        let metadata = fs::metadata(&temp_file).unwrap();
        assert_eq!(metadata.permissions().mode() & 0o7777, 0o600);
        assert_eq!(
            "gurp is running a test",
            fs::read_to_string(temp_file).unwrap()
        );
    }

    #[test]
    fn test_file_ensure_already_correct() {
        let temp_dir = Utf8TempDir::new().unwrap();
        temp_dir.child("test-file").write_str("stuff").unwrap();
        let temp_file = temp_dir.path().join("test-file");

        fs::set_permissions(&temp_file, fs::Permissions::from_mode(0o0750)).unwrap();
        assert!(temp_file.exists());

        let json_def = janet2json(&formatdoc! {r#"
            (file/ensure "{}"
                :content "stuff"
                :mode "0750"
                :owner "{}"
                :group "{}")
            "#,
            temp_file,
            my_user(),
            my_group(),
        });

        let sut: GurpFileEnsure = serde_json::from_str(&json_def).unwrap();
        assert_eq!(ONE_RESOURCE_NO_CHANGE, sut.apply(&defopts()).unwrap());
        assert!(temp_file.exists());
    }

    #[test]
    fn test_update_file_from_content_and_set_mode() {
        let temp_dir = Utf8TempDir::new().unwrap();
        temp_dir
            .child("test-file")
            .write_str("the-wrong-stuff")
            .unwrap();

        let temp_file = temp_dir.path().join("test-file");

        assert!(temp_file.exists());

        let json_def = janet2json(&formatdoc! {r#"
            (file/ensure "{}"
                :content "the-right-stuff"
                :mode "0400"
                :owner "{}"
                :group "{}")
            "#,
            temp_file,
            my_user(),
            my_group(),
        });

        let sut: GurpFileEnsure = serde_json::from_str(&json_def).unwrap();
        assert_eq!(ONE_RESOURCE_ONE_CHANGE, sut.apply(&defopts()).unwrap());

        assert!(temp_file.exists());
        assert_eq!(
            "the-right-stuff".to_owned(),
            fs::read_to_string(&temp_file).unwrap()
        );

        let metadata = fs::metadata(&temp_file).unwrap();
        assert_eq!(metadata.permissions().mode() & 0o7777, 0o0400);
    }

    #[test]
    fn test_update_file_from_file_and_set_mode() {
        let temp_dir = Utf8TempDir::new().unwrap();
        temp_dir
            .child("test-file")
            .write_str("the-wrong-stuff")
            .unwrap();

        let temp_file = temp_dir.path().join("test-file");

        assert!(temp_file.exists());

        let json_def = janet2json(&formatdoc! {r#"
            (do
            (setdyn :gurp-config-root "{}")
            (file/ensure "{}"
                :from "{}"
                :mode "0444"
                :owner "{}"
                :group "{}"))
            "#,
            temp_file.parent().unwrap(),
            temp_file,
            &fixture("file/copy-file"),
            my_user(),
            my_group(),
        });

        let sut: GurpFileEnsure = serde_json::from_str(&json_def).unwrap();
        assert_eq!(ONE_RESOURCE_ONE_CHANGE, sut.apply(&defopts()).unwrap());

        assert!(temp_file.exists());
        assert_eq!(
            "some-content\n".to_owned(),
            fs::read_to_string(&temp_file).unwrap()
        );

        let metadata = fs::metadata(&temp_file).unwrap();
        assert_eq!(metadata.permissions().mode() & 0o7777, 0o444);
    }

    #[test]
    fn test_ignored_line_means_no_change_with_content() {
        let content = "today is 2015-01-30\nBut this never changes.\nAnd nor does this.\n";
        let temp_dir = Utf8TempDir::new().unwrap();
        temp_dir.child("test-file").write_str(content).unwrap();

        let temp_file = temp_dir.path().join("test-file");

        fs::set_permissions(&temp_file, fs::Permissions::from_mode(0o600)).unwrap();
        assert!(temp_file.exists());

        let json_def = janet2json(&formatdoc! {"
            (file/ensure \"{}\"
                :content \"today is 2025-06-26\\nBut this never changes.\\nAnd nor does this.\"
                :mode \"0600\"
                :ignore-pattern \"^today is\"
                :owner \"{}\"
                :group \"{}\")
            ",
            temp_file,
            my_user(),
            my_group(),
        });

        let sut: GurpFileEnsure = serde_json::from_str(&json_def).unwrap();

        assert_eq!(ONE_RESOURCE_NO_CHANGE, sut.apply(&defopts()).unwrap());
        assert_eq!(content, fs::read_to_string(&temp_file).unwrap());
    }

    #[test]
    fn test_ignored_line_means_no_change_with_from() {
        let content = "today is 2015-01-30\nBut this never changes.\nAnd nor does this.\n";
        let temp_dir = Utf8TempDir::new().unwrap();
        temp_dir.child("test-file").write_str(content).unwrap();

        let temp_file = temp_dir.path().join("test-file");

        fs::set_permissions(&temp_file, fs::Permissions::from_mode(0o600)).unwrap();
        assert!(temp_file.exists());

        let json_def = janet2json(&formatdoc! {r#"
            (file/ensure "{}"
                :from "{}"
                :mode "0600"
                :ignore-pattern "^today is"
                :owner "{}"
                :group "{}")
            "#,
            temp_file,
            fixture("file/ignore-line-file"),
            my_user(),
            my_group(),
        });

        let sut: GurpFileEnsure = serde_json::from_str(&json_def).unwrap();

        assert_eq!(ONE_RESOURCE_NO_CHANGE, sut.apply(&defopts()).unwrap());
        assert_eq!(content, fs::read_to_string(&temp_file).unwrap());
    }
}
