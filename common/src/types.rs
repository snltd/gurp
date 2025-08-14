use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::ops::Add;

pub type ExitCode = u8;

#[derive(Clone)]
pub struct Opts {
    pub noop: bool,
    pub colour: bool,
    pub line_no: bool,
    pub dump_config: bool,
}

#[derive(Debug, Default, PartialEq, Copy, Clone)]
pub struct ApplySummary {
    pub resources: u32,
    pub changes: u32,
    pub errors: u32,
}

impl Add for ApplySummary {
    type Output = ApplySummary;

    fn add(self, other: ApplySummary) -> ApplySummary {
        ApplySummary {
            resources: self.resources + other.resources,
            changes: self.changes + other.changes,
            errors: self.errors + other.errors,
        }
    }
}

#[derive(Deserialize, Debug, Hash, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct SmfDefinition {
    pub name: String,
    pub duration: Option<String>,
    pub description: String,
    pub fmri: String,
    pub default_enabled: bool,
    pub single_instance: bool,
    pub start_method: Option<SmfDefinitionExecMethod>,
    pub stop_method: Option<SmfDefinitionExecMethod>,
    pub refresh_method: Option<SmfDefinitionExecMethod>,
    pub property_groups: Option<PropertyGroupMap>,
    pub properties: Option<PropertyMap>,
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Deserialize, Hash)]
#[serde(rename_all = "kebab-case")]
pub struct SmfDefinitionDependencySvc {
    pub name: String,
    pub restart_on: String,
    pub fmri: String,
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
