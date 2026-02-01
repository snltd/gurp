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
use anyhow::bail;
use colored::Colorize;
use common::types::{ApplyOpts, ApplySummary, ChangedIds};
use serde::{Deserialize, Serialize};
use serde_json::Error;
use std::collections::BTreeSet;
use util::json; // Because they are hashable

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

macro_rules! apply_resources {
    ($summary_total:ident, $changed_ids:ident, $resources:expr, $opts:expr) => {
        let total_count = $resources.len();
        for (i, resource) in $resources.iter().enumerate() {
            let chunks: Vec<_> = resource.id.split("/").collect();
            if chunks.len() >= 3 {
                tracing::debug!(
                    "applying {} {}/{}: {}",
                    chunks[1],
                    i + 1,
                    total_count,
                    resource.id
                );
            } else {
                tracing::debug!("applying [{}/{}]: {}", i + 1, total_count, resource.id);
            }
            let summary = match resource.apply($opts) {
                Ok(summary) => summary,
                Err(e) => {
                    tracing::error!("from {} doer: {}", chunks[2], e);
                    let err: anyhow::Error = e.into();
                    return Err(err.context(format!("failed to apply resource {}", resource.id)));
                }
            };
            $summary_total += summary;
            if summary.changes > 0 {
                $changed_ids.insert(resource.id.clone());
            }
        }
    };
}
pub struct Applicator {
    json_config: String,
}

impl Applicator {
    pub fn from(json_config: String) -> Self {
        Self { json_config }
    }

    pub fn run(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        let config = self.json_to_hostconfig()?;

        tracing::info!("Configuring host: {}", config.metadata.name);
        self.apply(&config, opts)
    }

    fn json_to_hostconfig(&self) -> anyhow::Result<HostConfig> {
        tracing::debug!(
            "Unpacking {} bytes of JSON config into HostConfig",
            self.json_config.len()
        );

        let host_config: HostConfig = match serde_json::from_str(&self.json_config) {
            Ok(conf) => conf,
            Err(e) => {
                self.display_error(e)?;
                bail!("end of deserializing output");
            }
        };

        Ok(host_config)
    }

    fn apply(&self, config: &HostConfig, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        let ensure = &config.resources.ensure;
        let remove = &config.resources.remove;

        let mut summary_total = ApplySummary::default();
        let mut changed_ids: ChangedIds = BTreeSet::new();

        apply_resources!(summary_total, changed_ids, &ensure.publisher, opts);
        apply_resources!(summary_total, changed_ids, &ensure.etherstub, opts);
        apply_resources!(summary_total, changed_ids, &ensure.vnic, opts);
        apply_resources!(summary_total, changed_ids, &ensure.vlan, opts);
        apply_resources!(summary_total, changed_ids, &ensure.ip_interface, opts);
        apply_resources!(summary_total, changed_ids, &ensure.ip_address, opts);
        apply_resources!(summary_total, changed_ids, &ensure.bridge, opts);
        apply_resources!(summary_total, changed_ids, &ensure.route, opts);
        apply_resources!(summary_total, changed_ids, &ensure.ip_properties, opts);
        apply_resources!(summary_total, changed_ids, &ensure.network_flow, opts);
        apply_resources!(summary_total, changed_ids, &ensure.zfs, opts);
        apply_resources!(summary_total, changed_ids, &ensure.zone, opts);

        if !&ensure.ipnat.is_empty() {
            summary_total += crate::ipnat::collect_and_ensure(&ensure.ipnat, opts)?;
        }

        if !&ensure.pkg.is_empty() {
            summary_total += crate::pkg::collect_and_ensure(&ensure.pkg, opts)?;
        }

        if !&ensure.pkgin.is_empty() {
            summary_total += crate::pkgin::collect_and_ensure(&ensure.pkgin, opts)?;
        }

        if !&ensure.apk.is_empty() {
            summary_total += crate::apk::collect_and_ensure(&ensure.apk, opts)?;
        }

        if !&ensure.gem.is_empty() {
            summary_total += crate::gem::collect_and_ensure(&ensure.gem, opts)?;
        }

        apply_resources!(summary_total, changed_ids, &ensure.group, opts);
        apply_resources!(summary_total, changed_ids, &ensure.user, opts);
        apply_resources!(summary_total, changed_ids, &ensure.cron, opts);
        apply_resources!(summary_total, changed_ids, &ensure.directory, opts);
        apply_resources!(summary_total, changed_ids, &ensure.file, opts);
        apply_resources!(summary_total, changed_ids, &ensure.file_line, opts);
        apply_resources!(summary_total, changed_ids, &ensure.symlink, opts);
        apply_resources!(summary_total, changed_ids, &ensure.svcprop, opts);
        apply_resources!(summary_total, changed_ids, &ensure.smf, opts);
        apply_resources!(summary_total, changed_ids, &ensure.misc, opts);

        apply_resources!(summary_total, changed_ids, &remove.symlink, opts);
        apply_resources!(summary_total, changed_ids, &remove.file_line, opts);
        apply_resources!(summary_total, changed_ids, &remove.file, opts);
        apply_resources!(summary_total, changed_ids, &remove.directory, opts);
        apply_resources!(summary_total, changed_ids, &remove.svcprop, opts);
        apply_resources!(summary_total, changed_ids, &remove.smf, opts);
        apply_resources!(summary_total, changed_ids, &remove.cron, opts);
        apply_resources!(summary_total, changed_ids, &remove.user, opts);
        apply_resources!(summary_total, changed_ids, &remove.group, opts);
        apply_resources!(summary_total, changed_ids, &remove.publisher, opts);

        if !&remove.gem.is_empty() {
            summary_total += crate::gem::collect_and_remove(&remove.gem, opts)?;
        }

        if !&remove.pkg.is_empty() {
            summary_total += crate::pkg::collect_and_remove(&remove.pkg, opts)?;
        }

        if !&remove.pkgin.is_empty() {
            summary_total += crate::pkgin::collect_and_remove(&remove.pkgin, opts)?;
        }

        if !&remove.apk.is_empty() {
            summary_total += crate::apk::collect_and_remove(&remove.apk, opts)?;
        }

        apply_resources!(summary_total, changed_ids, &remove.ipnat, opts);
        apply_resources!(summary_total, changed_ids, &remove.zone, opts);
        apply_resources!(summary_total, changed_ids, &remove.zfs, opts);
        apply_resources!(summary_total, changed_ids, &remove.bridge, opts);
        apply_resources!(summary_total, changed_ids, &remove.network_flow, opts);
        apply_resources!(summary_total, changed_ids, &remove.route, opts);
        apply_resources!(summary_total, changed_ids, &remove.ip_address, opts);
        apply_resources!(summary_total, changed_ids, &remove.ip_interface, opts);
        apply_resources!(summary_total, changed_ids, &remove.vlan, opts);
        apply_resources!(summary_total, changed_ids, &remove.vnic, opts);
        apply_resources!(summary_total, changed_ids, &remove.etherstub, opts);

        for resource in &ensure.svc {
            summary_total += resource.apply(&changed_ids, opts)?;
        }

        Ok(summary_total)
    }

    fn display_error(&self, e: Error) -> anyhow::Result<()> {
        tracing::error!("deserializing error: {}", e);

        let formatted_json = json::formatted(&self.json_config)?;
        let error_line = e.line();
        let json_lines: Vec<_> = formatted_json.lines().collect();
        let first_line = error_line.saturating_sub(30);
        let last_line = (error_line + 15).clamp(0, json_lines.len());

        for l in first_line..=last_line {
            let output_line = format!(" {:4} | {}", l + 1, json_lines.get(l).unwrap_or(&""));

            if l == error_line {
                println!("{}", output_line.bold());
            } else {
                println!("{output_line}");
            }
        }

        Ok(())
    }
}
