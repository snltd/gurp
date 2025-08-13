use crate::cron::{GurpCronEnsure, GurpCronRemove};
use crate::directory::{GurpDirectoryEnsure, GurpDirectoryRemove};
use crate::file::{GurpFileEnsure, GurpFileRemove};
use crate::file_line::{GurpFileLineEnsure, GurpFileLineRemove};
use crate::gem::{GurpGemEnsure, GurpGemRemove};
use crate::group::{GurpGroupEnsure, GurpGroupRemove};
use crate::misc::GurpMiscEnsure;
use crate::pkg::{GurpPkgEnsure, GurpPkgRemove};
use crate::pkgin::{GurpPkginEnsure, GurpPkginRemove};
use crate::publisher::{GurpPublisherEnsure, GurpPublisherRemove};
use crate::smf::{GurpSmfEnsure, GurpSmfRemove};
use crate::svc::GurpSvcEnsure;
use crate::svcprop::{GurpSvcpropEnsure, GurpSvcpropRemove};
use crate::symlink::{GurpSymlinkEnsure, GurpSymlinkRemove};
use crate::user::{GurpUserEnsure, GurpUserRemove};
use crate::zfs::{GurpZfsEnsure, GurpZfsRemove};
use crate::zone::{GurpZoneEnsure, GurpZoneRemove};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet; // Because they are hashable

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
    pub pkgin: Vec<GurpPkginEnsure>,
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
    pub pkgin: Vec<GurpPkginRemove>,
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
