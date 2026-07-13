use crate::file::types::{CompareMethod, DesiredFileState, FileSource};
use crate::file::{from_content, from_file, from_struct, from_url};
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
            self.path
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
            FileSource::Url => from_url::run(&self.path, &self.desired_state, &compare, opts),
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
    use pretty_assertions::assert_eq;
    use tester::{deserialized_example, fixture, my_group, my_user};
    use util::file::NameOrId;

    #[test]
    fn test_deserialize_ensure_file_from_content() {
        assert_eq!(
            GurpFileEnsure {
                id: "/NO-ROLE/file/_example_file_from-content".to_owned(),
                path: Utf8PathBuf::from("/example/file/from-content"),
                desired_state: DesiredFileState {
                    backup_suffix: None,
                    content: Some("words\n and\nstuff\n".to_owned()),
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

        assert!(file_and_url.apply(&ApplyOpts::default()).is_err());

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

        assert!(file_and_content.apply(&ApplyOpts::default()).is_err());

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

        assert!(no_source.apply(&ApplyOpts::default()).is_err());
    }
}
