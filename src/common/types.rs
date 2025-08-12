use crate::doers::cron::{GurpCronEnsure, GurpCronRemove};
use crate::doers::directory::{GurpDirectoryEnsure, GurpDirectoryRemove};
use crate::doers::file::{GurpFileEnsure, GurpFileRemove};
use crate::doers::file_line::{GurpFileLineEnsure, GurpFileLineRemove};
use crate::doers::gem::{GurpGemEnsure, GurpGemRemove};
use crate::doers::group::{GurpGroupEnsure, GurpGroupRemove};
use crate::doers::misc::GurpMiscEnsure;
use crate::doers::pkg::{GurpPkgEnsure, GurpPkgRemove};
use crate::doers::publisher::{GurpPublisherEnsure, GurpPublisherRemove};
use crate::doers::smf::{GurpSmfEnsure, GurpSmfRemove};
use crate::doers::svc::GurpSvcEnsure;
use crate::doers::svcprop::{GurpSvcpropEnsure, GurpSvcpropRemove};
use crate::doers::symlink::{GurpSymlinkEnsure, GurpSymlinkRemove};
use crate::doers::user::{GurpUserEnsure, GurpUserRemove};
use crate::doers::zfs::{GurpZfsEnsure, GurpZfsRemove};
use crate::doers::zone::{GurpZoneEnsure, GurpZoneRemove};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet}; // Because they are hashable
use std::ops::Add;

#[derive(Clone)]
pub struct Opts {
    pub noop: bool,
    pub colour: bool,
    pub line_no: bool,
    pub dump_config: bool,
}

#[derive(Deserialize, Debug)]
pub struct HostConfig {
    pub metadata: HostMetadata,
    pub resources: HostResources,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct HostMetadata {
    pub name: String,
}

#[derive(Deserialize, Debug)]
pub struct HostResources {
    #[serde(default)]
    pub ensure: EnsureResources,
    #[serde(default)]
    pub remove: RemoveResources,
}

#[derive(Default, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct EnsureResources {
    #[serde(default)]
    pub cron: Vec<GurpCronEnsure>,
    #[serde(default)]
    pub directory: Vec<GurpDirectoryEnsure>,
    #[serde(default)]
    pub file: Vec<GurpFileEnsure>,
    #[serde(default)]
    pub file_line: Vec<GurpFileLineEnsure>,
    #[serde(default)]
    pub group: Vec<GurpGroupEnsure>,
    #[serde(default)]
    pub misc: Vec<GurpMiscEnsure>,
    #[serde(default)]
    pub gem: Vec<GurpGemEnsure>,
    #[serde(default)]
    pub pkg: Vec<GurpPkgEnsure>,
    #[serde(default)]
    pub publisher: Vec<GurpPublisherEnsure>,
    #[serde(default)]
    pub svcprop: Vec<GurpSvcpropEnsure>,
    #[serde(default)]
    pub smf: Vec<GurpSmfEnsure>,
    #[serde(default)]
    pub svc: Vec<GurpSvcEnsure>,
    #[serde(default)]
    pub symlink: Vec<GurpSymlinkEnsure>,
    #[serde(default)]
    pub user: Vec<GurpUserEnsure>,
    #[serde(default)]
    pub zfs: Vec<GurpZfsEnsure>,
    #[serde(default)]
    pub zone: Vec<GurpZoneEnsure>,
}

#[derive(Default, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct RemoveResources {
    #[serde(default)]
    pub cron: Vec<GurpCronRemove>,
    #[serde(default)]
    pub directory: Vec<GurpDirectoryRemove>,
    #[serde(default)]
    pub file: Vec<GurpFileRemove>,
    #[serde(default)]
    pub file_line: Vec<GurpFileLineRemove>,
    #[serde(default)]
    pub group: Vec<GurpGroupRemove>,
    #[serde(default)]
    pub svcprop: Vec<GurpSvcpropRemove>,
    #[serde(default)]
    pub gem: Vec<GurpGemRemove>,
    #[serde(default)]
    pub pkg: Vec<GurpPkgRemove>,
    #[serde(default)]
    pub publisher: Vec<GurpPublisherRemove>,
    #[serde(default)]
    pub smf: Vec<GurpSmfRemove>,
    #[serde(default)]
    pub symlink: Vec<GurpSymlinkRemove>,
    #[serde(default)]
    pub user: Vec<GurpUserRemove>,
    #[serde(default)]
    pub zfs: Vec<GurpZfsRemove>,
    #[serde(default)]
    pub zone: Vec<GurpZoneRemove>,
}

pub type Changes<'a> = Vec<&'a str>;
pub type ChangedIds = BTreeSet<String>;
pub type ExitCode = u8;

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

#[cfg_attr(test, derive(PartialEq))]
#[derive(Deserialize, Debug, Hash)]
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

#[cfg_attr(test, derive(PartialEq))]
#[derive(Deserialize, Debug, Hash)]
pub struct SmfDefinitionExecMethod {
    pub exec: String,
    pub timeout: u32,
    pub context: Option<SmfDefinitionExecMethodContext>,
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Deserialize, Debug, Hash)]
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
