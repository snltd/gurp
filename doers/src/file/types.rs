use camino::Utf8PathBuf;
use os_types::FileMode;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
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
    pub url_replacements: Option<UrlReplacements>,
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
    Modified(ContentModifiers<'a>), // The user's :ignore-pattern and/or :url-replacements
}

pub struct ContentModifiers<'a> {
    pub Filter: Option<&'a str>,
    pub UrlReplacements: Option<&'a UrlReplacements>,
}

pub type UrlReplacements = HashMap<String, String>;
