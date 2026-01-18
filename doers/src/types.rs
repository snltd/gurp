use crate::apk::{GurpApkEnsure, GurpApkRemove};
use crate::bridge::{GurpBridgeEnsure, GurpBridgeRemove};
use crate::cron::{GurpCronEnsure, GurpCronRemove};
use crate::directory::{GurpDirectoryEnsure, GurpDirectoryRemove};
use crate::etherstub::{GurpEtherstubEnsure, GurpEtherstubRemove};
use crate::file::{GurpFileEnsure, GurpFileRemove};
use crate::file_line::{GurpFileLineEnsure, GurpFileLineRemove};
use crate::gem::{GurpGemEnsure, GurpGemRemove};
use crate::group::{GurpGroupEnsure, GurpGroupRemove};
use crate::ip_address::{GurpIpAddressEnsure, GurpIpAddressRemove};
use crate::ip_interface::{GurpIpInterfaceEnsure, GurpIpInterfaceRemove};
use crate::ip_properties::GurpIpPropertiesEnsure;
use crate::ipnat::{GurpIpnatEnsure, GurpIpnatRemove};
use crate::misc::GurpMiscEnsure;
use crate::network_flow::{GurpNetworkFlowEnsure, GurpNetworkFlowRemove};
use crate::pkg::{GurpPkgEnsure, GurpPkgRemove};
use crate::pkgin::{GurpPkginEnsure, GurpPkginRemove};
use crate::publisher::{GurpPublisherEnsure, GurpPublisherRemove};
use crate::route::{GurpRouteEnsure, GurpRouteRemove};
use crate::smf::{GurpSmfEnsure, GurpSmfRemove};
use crate::svc::GurpSvcEnsure;
use crate::svcprop::{GurpSvcpropEnsure, GurpSvcpropRemove};
use crate::symlink::{GurpSymlinkEnsure, GurpSymlinkRemove};
use crate::user::{GurpUserEnsure, GurpUserRemove};
use crate::vlan::{GurpVlanEnsure, GurpVlanRemove};
use crate::vnic::{GurpVnicEnsure, GurpVnicRemove};
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
    pub apk: Vec<GurpApkEnsure>,
    #[serde(default)]
    pub bridge: Vec<GurpBridgeEnsure>,
    #[serde(default)]
    pub cron: Vec<GurpCronEnsure>,
    #[serde(default)]
    pub directory: Vec<GurpDirectoryEnsure>,
    #[serde(default)]
    pub etherstub: Vec<GurpEtherstubEnsure>,
    #[serde(default)]
    pub file: Vec<GurpFileEnsure>,
    #[serde(default)]
    pub file_line: Vec<GurpFileLineEnsure>,
    #[serde(default)]
    pub gem: Vec<GurpGemEnsure>,
    #[serde(default)]
    pub group: Vec<GurpGroupEnsure>,
    #[serde(default)]
    pub ip_address: Vec<GurpIpAddressEnsure>,
    #[serde(default)]
    pub ip_interface: Vec<GurpIpInterfaceEnsure>,
    #[serde(default)]
    pub ip_properties: Vec<GurpIpPropertiesEnsure>,
    #[serde(default)]
    pub ipnat: Vec<GurpIpnatEnsure>,
    #[serde(default)]
    pub misc: Vec<GurpMiscEnsure>,
    #[serde(default)]
    pub network_flow: Vec<GurpNetworkFlowEnsure>,
    #[serde(default)]
    pub pkg: Vec<GurpPkgEnsure>,
    #[serde(default)]
    pub pkgin: Vec<GurpPkginEnsure>,
    #[serde(default)]
    pub publisher: Vec<GurpPublisherEnsure>,
    #[serde(default)]
    pub route: Vec<GurpRouteEnsure>,
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
    pub vlan: Vec<GurpVlanEnsure>,
    #[serde(default)]
    pub vnic: Vec<GurpVnicEnsure>,
    #[serde(default)]
    pub zfs: Vec<GurpZfsEnsure>,
    #[serde(default)]
    pub zone: Vec<GurpZoneEnsure>,
}

#[derive(Default, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct RemoveResources {
    #[serde(default)]
    pub apk: Vec<GurpApkRemove>,
    #[serde(default)]
    pub bridge: Vec<GurpBridgeRemove>,
    #[serde(default)]
    pub cron: Vec<GurpCronRemove>,
    #[serde(default)]
    pub directory: Vec<GurpDirectoryRemove>,
    #[serde(default)]
    pub etherstub: Vec<GurpEtherstubRemove>,
    #[serde(default)]
    pub file: Vec<GurpFileRemove>,
    #[serde(default)]
    pub file_line: Vec<GurpFileLineRemove>,
    #[serde(default)]
    pub gem: Vec<GurpGemRemove>,
    #[serde(default)]
    pub group: Vec<GurpGroupRemove>,
    #[serde(default)]
    pub ip_address: Vec<GurpIpAddressRemove>,
    #[serde(default)]
    pub ip_interface: Vec<GurpIpInterfaceRemove>,
    #[serde(default)]
    pub ipnat: Vec<GurpIpnatRemove>,
    #[serde(default)]
    pub network_flow: Vec<GurpNetworkFlowRemove>,
    #[serde(default)]
    pub pkg: Vec<GurpPkgRemove>,
    #[serde(default)]
    pub pkgin: Vec<GurpPkginRemove>,
    #[serde(default)]
    pub publisher: Vec<GurpPublisherRemove>,
    #[serde(default)]
    pub route: Vec<GurpRouteRemove>,
    #[serde(default)]
    pub smf: Vec<GurpSmfRemove>,
    #[serde(default)]
    pub svcprop: Vec<GurpSvcpropRemove>,
    #[serde(default)]
    pub symlink: Vec<GurpSymlinkRemove>,
    #[serde(default)]
    pub user: Vec<GurpUserRemove>,
    #[serde(default)]
    pub vlan: Vec<GurpVlanRemove>,
    #[serde(default)]
    pub vnic: Vec<GurpVnicRemove>,
    #[serde(default)]
    pub zfs: Vec<GurpZfsRemove>,
    #[serde(default)]
    pub zone: Vec<GurpZoneRemove>,
}

pub type ChangedIds = BTreeSet<String>;
