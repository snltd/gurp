use camino::Utf8PathBuf;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::ops::Add;

pub type ExitCode = u8;

#[derive(Debug, Default)]
pub struct ApplyOpts {
    pub noop: bool,
    pub colour: bool,
    pub line_no: bool,
    pub gurp_lib_path: Option<Utf8PathBuf>,
    pub dump_config: bool,
    pub dump_diffs: bool,
    pub compile_only: bool,
    pub metrics_to: Option<String>,
    pub precompiled: bool,
    pub image: bool,
    pub server: Option<String>,      // client mode only
    pub as_json: bool,               // client mode only
    pub hostname: Option<String>,    // currently client mode only
    pub server_name: Option<String>, // server mode only
    pub client_name: Option<String>, // server mode only
    pub destroy: bool,
}

#[derive(Debug, Default)]
pub struct CompileOpts {
    pub format: String,
    pub output_file: Option<Utf8PathBuf>,
}

#[derive(Debug)]
pub struct ServerOpts {
    pub config_dir: Utf8PathBuf,
    pub metrics_to: Option<String>,
}

#[derive(Debug, Default, PartialEq, Copy, Clone)]
pub struct ApplySummary {
    pub resources: u32,
    pub changes: u32,
}

impl Add for ApplySummary {
    type Output = ApplySummary;

    fn add(self, other: ApplySummary) -> ApplySummary {
        ApplySummary {
            resources: self.resources + other.resources,
            changes: self.changes + other.changes,
        }
    }
}

#[derive(Deserialize, Debug, Hash, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct SmfDefinition {
    pub name: String,
    pub duration: Option<String>,
    pub description: Option<String>,
    pub fmri: String,
    pub default_enabled: bool,
    pub single_instance: bool,
    pub start_method: Option<SmfDefinitionExecMethod>,
    pub stop_method: Option<SmfDefinitionExecMethod>,
    pub refresh_method: Option<SmfDefinitionExecMethod>,
    pub property_groups: Option<PropertyGroupMap>,
    pub properties: Option<PropertyMap>,
    pub dependencies: Option<Vec<SmfDefinitionDependencySvc>>,
    pub dependents: Option<Vec<SmfDefinitionDependentSvc>>,
}

#[derive(PartialEq, Debug, Deserialize, Hash)]
#[serde(rename_all = "kebab-case")]
pub struct SmfDefinitionDependencySvc {
    pub name: String,
    pub restart_on: String,
    pub fmri: String,
    pub grouping: String,
    #[serde(rename = "type")]
    pub dep_type: String,
}

#[derive(PartialEq, Debug, Deserialize, Hash)]
#[serde(rename_all = "kebab-case")]
pub struct SmfDefinitionDependentSvc {
    pub name: String,
    pub restart_on: String,
    pub fmri: String,
    pub grouping: String,
    #[serde(rename = "type")]
    pub dep_type: String,
}

#[derive(Deserialize, Debug, Hash, PartialEq)]
pub struct SmfDefinitionExecMethod {
    pub exec: String,
    pub timeout: u32,
    pub context: Option<SmfDefinitionExecMethodContext>,
}

#[derive(Deserialize, Debug, Hash, PartialEq)]
pub struct SmfDefinitionExecMethodContext {
    pub user: String,
    pub group: Option<String>,
    pub privileges: Option<String>,
    pub environment: Option<BTreeMap<String, String>>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Hash)]
#[serde(untagged)]
pub enum PropertyValue {
    Bool(bool),
    Int(i64),
    String(String),
}

impl fmt::Display for PropertyValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PropertyValue::Bool(b) => write!(f, "{b}"),
            PropertyValue::Int(i) => write!(f, "{i}"),
            PropertyValue::String(s) => write!(f, "\"{s}\""),
        }
    }
}

#[derive(Debug, Deserialize, PartialEq, Hash)]
pub struct PropertyStruct {
    pub value: PropertyValue,
    #[serde(rename = "type")]
    pub prop_type: String,
}

pub type PropertyName = String;
pub type PropertyGroupName = String;
pub type PropertyGroupType = String;
pub type PropertyList = Vec<PropertyName>;
pub type PropertyMap = BTreeMap<String, PropertyStruct>;
pub type PropertyGroupMap = BTreeMap<PropertyGroupName, PropertyGroupType>;
pub type PropertyGroupList = BTreeSet<PropertyGroupName>;
pub type SvcProps = BTreeMap<PropertyName, PropertyStruct>;

#[derive(Debug)]
pub struct FileMetadata<'a> {
    pub group: &'a str,
    pub mode: &'a str,
    pub owner: &'a str,
    pub path: &'a Utf8PathBuf,
    pub changes: u32,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(untagged)]
pub enum StrOrNumber {
    Str(String),
    Number(u32),
}

impl fmt::Display for StrOrNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StrOrNumber::Str(s) => write!(f, "{s}"),
            StrOrNumber::Number(n) => write!(f, "{n}"),
        }
    }
}

pub type VlanID = u16;
