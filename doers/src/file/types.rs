use camino::Utf8PathBuf;
use os_types::FileMode;
use serde::Deserialize;
use serde_json::Value;
use std::fmt::Debug;
use url::Url;
use util::file::NameOrId;

#[derive(Deserialize, Debug, Default)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(rename_all = "kebab-case")]
pub struct DesiredFileState {
    pub backup_suffix: Option<String>,
    pub content: Option<String>,
    pub from_struct: Option<Value>,
    pub from_url: Option<Url>,
    pub from: Option<Utf8PathBuf>,
    pub group: NameOrId,
    pub ignore_pattern: Option<String>,
    pub mode: FileMode,
    pub owner: NameOrId,
    pub to_format: Option<OutputFileFormat>,
    pub with_checksum: Option<String>,
    #[serde(default)]
    pub only_fetch_from_url_once: bool,
    #[serde(default)]
    pub url_is_server: bool,
}

#[derive(Deserialize, Debug)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(rename_all = "lowercase")]
pub enum OutputFileFormat {
    Yaml,
    Toml,
    Json,
    Ini,
    #[serde(rename = "k=v")]
    KeyValue,
}

pub enum FileSource {
    Literal,
    File,
    Url,
    Struct,
}

pub enum CompareMethod<'a> {
    Hash,
    Filter(&'a str), // The user specified :ignore-pattern
}
