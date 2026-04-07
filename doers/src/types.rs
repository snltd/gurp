use crate::apk::{GurpApkEnsure, GurpApkRemove};
use crate::bridge::{GurpBridgeEnsure, GurpBridgeRemove};
use crate::cron::{GurpCronEnsure, GurpCronRemove};
use crate::directory::{GurpDirectoryEnsure, GurpDirectoryRemove};
use crate::etherstub::{GurpEtherstubEnsure, GurpEtherstubRemove};
use crate::file::ensure::GurpFileEnsure;
use crate::file::remove::GurpFileRemove;
use crate::file_line::{GurpFileLineEnsure, GurpFileLineRemove};
use crate::gem::{GurpGemEnsure, GurpGemRemove};
use crate::group::{GurpGroupEnsure, GurpGroupRemove};
use crate::ip_address::{GurpIpAddressEnsure, GurpIpAddressRemove};
use crate::ip_interface::{GurpIpInterfaceEnsure, GurpIpInterfaceRemove};
use crate::ip_properties::GurpIpPropertiesEnsure;
use crate::ipfilter::{GurpIpfilterEnsure, GurpIpfilterRemove};
use crate::ipnat::{GurpIpnatEnsure, GurpIpnatRemove};
use crate::link::{GurpLinkEnsure, GurpLinkRemove};
use crate::misc::GurpMiscEnsure;
use crate::network_flow::{GurpNetworkFlowEnsure, GurpNetworkFlowRemove};
use crate::pkg::{GurpPkgEnsure, GurpPkgRemove};
use crate::pkgin::{GurpPkginEnsure, GurpPkginRemove};
use crate::publisher::{GurpPublisherEnsure, GurpPublisherRemove};
use crate::route::{GurpRouteEnsure, GurpRouteRemove};
use crate::smf::{GurpSmfEnsure, GurpSmfRemove};
use crate::svc::GurpSvcEnsure;
use crate::svcprop::{GurpSvcpropEnsure, GurpSvcpropRemove};
use crate::user::{GurpUserEnsure, GurpUserRemove};
use crate::vlan::{GurpVlanEnsure, GurpVlanRemove};
use crate::vnic::{GurpVnicEnsure, GurpVnicRemove};
use crate::zfs::{GurpZfsEnsure, GurpZfsRemove};
use crate::zone::{GurpZoneEnsure, GurpZoneRemove};
use anyhow::{Context, bail};
use bytesize::ByteSize;
use colored::Colorize;
use common::types::{ApplyOpts, ApplySummary, ChangedIds};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Error;
use std::collections::BTreeSet;
use util::{json, runtime_stats};

pub(crate) type ApplyResult = anyhow::Result<(ApplySummary, ChangedIds)>;

pub trait Apply {
    fn id(&self) -> &str;
    fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary>;
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
    pub ipfilter: Vec<GurpIpfilterEnsure>,
    #[serde(default)]
    pub ipnat: Vec<GurpIpnatEnsure>,
    #[serde(default)]
    pub link: Vec<GurpLinkEnsure>,
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
    pub ipfilter: Vec<GurpIpfilterRemove>,
    #[serde(default)]
    pub ipnat: Vec<GurpIpnatRemove>,
    #[serde(default)]
    pub link: Vec<GurpLinkRemove>,
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

macro_rules! impl_apply {
    ($($t:ty),*) => {
        $(
            impl Apply for $t {
                fn id(&self) -> &str { &self.id }
                fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
                    self.apply(opts)
                }
            }
        )*
    }
}

impl_apply!(
    GurpBridgeEnsure,
    GurpCronEnsure,
    GurpDirectoryEnsure,
    GurpEtherstubEnsure,
    GurpFileEnsure,
    GurpFileLineEnsure,
    GurpGroupEnsure,
    GurpIpAddressEnsure,
    GurpIpInterfaceEnsure,
    GurpIpPropertiesEnsure,
    GurpLinkEnsure,
    GurpMiscEnsure,
    GurpNetworkFlowEnsure,
    GurpPublisherEnsure,
    GurpRouteEnsure,
    GurpSvcpropEnsure,
    GurpSmfEnsure,
    GurpUserEnsure,
    GurpVlanEnsure,
    GurpVnicEnsure,
    GurpZfsEnsure,
    GurpZoneEnsure,
    GurpBridgeRemove,
    GurpCronRemove,
    GurpDirectoryRemove,
    GurpEtherstubRemove,
    GurpFileRemove,
    GurpFileLineRemove,
    GurpGroupRemove,
    GurpIpAddressRemove,
    GurpIpInterfaceRemove,
    GurpIpfilterRemove,
    GurpIpnatRemove,
    GurpLinkRemove,
    GurpNetworkFlowRemove,
    GurpPublisherRemove,
    GurpRouteRemove,
    GurpSmfRemove,
    GurpSvcpropRemove,
    GurpUserRemove,
    GurpVlanRemove,
    GurpVnicRemove,
    GurpZfsRemove,
    GurpZoneRemove
);

pub struct Applicator {
    json_config: String,
}

fn apply_resources<'a, T: Apply>(resources: &'a [T], opts: &'a ApplyOpts) -> ApplyResult {
    let total_count = resources.len();
    let mut changed_ids: ChangedIds = BTreeSet::new();
    let mut apply_summary = ApplySummary::default();

    for (i, resource) in resources.iter().enumerate() {
        let id = resource.id();

        if let Some(only) = opts.only.as_ref() {
            let rx = Regex::new(only).context("failed to generate ID filter regex")?;

            if !rx.is_match(id) {
                continue;
            }
        }

        if std::env::var("GURP_RSS_STATS").is_ok()
            && let Some(rss) = runtime_stats::rss_bytes()
        {
            tracing::info!("RSS before {id}: {}", ByteSize(rss as u64));
        }

        let chunks: Vec<_> = id.split("/").collect();

        if chunks.len() >= 3 {
            tracing::debug!("applying {} {}/{total_count}: {id}", chunks[1], i + 1,);
        } else {
            tracing::debug!("applying [{}/{total_count}]: {id}", i + 1);
        }

        let summary = match resource.apply(opts) {
            Ok(summary) => summary,
            Err(e) => {
                tracing::error!("from {} doer: {}", chunks[2], e);
                let err: anyhow::Error = e;
                return Err(err.context(format!("failed to apply resource {id}")));
            }
        };

        if summary.changes > 0 {
            changed_ids.insert(id.to_owned());
        }

        apply_summary += summary;
    }

    Ok((apply_summary, changed_ids))
}

impl Applicator {
    pub fn from(json_config: String) -> Self {
        Self { json_config }
    }

    pub fn run(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        let config = self.json_to_hostconfig()?;

        tracing::info!("Configuring host: {}", config.metadata.name);
        let ret = self.apply(&config, opts);

        if std::env::var("GURP_RSS_STATS").is_ok()
            && let Some(rss) = runtime_stats::rss_bytes()
        {
            tracing::info!("final RSS: {}", ByteSize(rss as u64));
        }

        ret
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

    fn accumulate(
        &self,
        sum: &mut ApplySummary,
        ids: &mut ChangedIds,
        result: ApplyResult,
    ) -> anyhow::Result<()> {
        let (s, i) = result?;
        *sum += s;
        ids.extend(i);
        Ok(())
    }

    #[rustfmt::skip]
    fn apply_ensure(&self, res: &EnsureResources, opts: &ApplyOpts) -> ApplyResult {
        let mut sum = ApplySummary::default();
        let mut ids = ChangedIds::new();

        self.accumulate(&mut sum, &mut ids, apply_resources(&res.publisher, opts))?;
        self.accumulate(&mut sum, &mut ids, apply_resources(&res.etherstub, opts))?;
        self.accumulate(&mut sum, &mut ids, apply_resources(&res.vnic, opts))?;
        self.accumulate(&mut sum, &mut ids, apply_resources(&res.vlan, opts))?;
        self.accumulate(&mut sum, &mut ids, apply_resources(&res.ip_interface, opts))?;
        self.accumulate(&mut sum, &mut ids, apply_resources(&res.ip_address, opts))?;
        self.accumulate(&mut sum, &mut ids, apply_resources(&res.bridge, opts))?;
        self.accumulate(&mut sum, &mut ids, apply_resources(&res.route, opts))?;
        self.accumulate(&mut sum, &mut ids, apply_resources(&res.ip_properties, opts))?;
        self.accumulate(&mut sum, &mut ids, apply_resources(&res.network_flow, opts))?;
        self.accumulate(&mut sum, &mut ids, apply_resources(&res.zfs, opts))?;
        self.accumulate(&mut sum, &mut ids, apply_resources(&res.zone, opts))?;
        self.accumulate(&mut sum, &mut ids, crate::ipfilter::collect_and_ensure(&res.ipfilter, opts))?;
        self.accumulate(&mut sum, &mut ids, crate::ipnat::collect_and_ensure(&res.ipnat, opts))?;
        self.accumulate(&mut sum, &mut ids, crate::pkg::collect_and_ensure(&res.pkg, opts))?;
        self.accumulate(&mut sum, &mut ids, crate::pkgin::collect_and_ensure(&res.pkgin, opts))?;
        self.accumulate(&mut sum, &mut ids, crate::apk::collect_and_ensure(&res.apk, opts))?;
        self.accumulate(&mut sum, &mut ids, crate::gem::collect_and_ensure(&res.gem, opts))?;
        self.accumulate(&mut sum, &mut ids, apply_resources(&res.group, opts))?;
        self.accumulate(&mut sum, &mut ids, apply_resources(&res.user, opts))?;
        self.accumulate(&mut sum, &mut ids, apply_resources(&res.cron, opts))?;
        self.accumulate(&mut sum, &mut ids, apply_resources(&res.directory, opts))?;
        self.accumulate(&mut sum, &mut ids, apply_resources(&res.file, opts))?;
        self.accumulate(&mut sum, &mut ids, apply_resources(&res.file_line, opts))?;
        self.accumulate(&mut sum, &mut ids, apply_resources(&res.link, opts))?;
        self.accumulate(&mut sum, &mut ids, apply_resources(&res.svcprop, opts))?;
        self.accumulate(&mut sum, &mut ids, apply_resources(&res.smf, opts))?;
        self.accumulate(&mut sum, &mut ids, apply_resources(&res.misc, opts))?;

        Ok((sum, ids))
    }

    #[rustfmt::skip]
    fn apply_remove(&self, res: &RemoveResources, opts: &ApplyOpts) -> ApplyResult {
        let mut sum = ApplySummary::default();
        let mut ids = ChangedIds::new();

        self.accumulate(&mut sum, &mut ids, apply_resources(&res.link, opts))?;
        self.accumulate(&mut sum, &mut ids, apply_resources(&res.file_line, opts))?;
        self.accumulate(&mut sum, &mut ids, apply_resources(&res.file, opts))?;
        self.accumulate(&mut sum, &mut ids, apply_resources(&res.directory, opts))?;
        self.accumulate(&mut sum, &mut ids, apply_resources(&res.svcprop, opts))?;
        self.accumulate(&mut sum, &mut ids, apply_resources(&res.smf, opts))?;
        self.accumulate(&mut sum, &mut ids, apply_resources(&res.cron, opts))?;
        self.accumulate(&mut sum, &mut ids, apply_resources(&res.user, opts))?;
        self.accumulate(&mut sum, &mut ids, apply_resources(&res.group, opts))?;
        self.accumulate(&mut sum, &mut ids, apply_resources(&res.publisher, opts))?;
        self.accumulate(&mut sum, &mut ids, crate::gem::collect_and_remove(&res.gem, opts))?;
        self.accumulate(&mut sum, &mut ids, crate::pkg::collect_and_remove(&res.pkg, opts))?;
        self.accumulate(&mut sum, &mut ids, crate::pkgin::collect_and_remove(&res.pkgin, opts))?;
        self.accumulate(&mut sum, &mut ids, crate::apk::collect_and_remove(&res.apk, opts))?;
        self.accumulate(&mut sum, &mut ids, apply_resources(&res.ipnat, opts))?;
        self.accumulate(&mut sum, &mut ids, apply_resources(&res.ipfilter, opts))?;
        self.accumulate(&mut sum, &mut ids, apply_resources(&res.zone, opts))?;
        self.accumulate(&mut sum, &mut ids, apply_resources(&res.zfs, opts))?;
        self.accumulate(&mut sum, &mut ids, apply_resources(&res.bridge, opts))?;
        self.accumulate(&mut sum, &mut ids, apply_resources(&res.network_flow, opts))?;
        self.accumulate(&mut sum, &mut ids, apply_resources(&res.route, opts))?;
        self.accumulate(&mut sum, &mut ids, apply_resources(&res.ip_address, opts))?;
        self.accumulate(&mut sum, &mut ids, apply_resources(&res.ip_interface, opts))?;
        self.accumulate(&mut sum, &mut ids, apply_resources(&res.vlan, opts))?;
        self.accumulate(&mut sum, &mut ids, apply_resources(&res.vnic, opts))?;
        self.accumulate(&mut sum, &mut ids, apply_resources(&res.etherstub, opts))?;

        Ok((sum, ids))
    }

    fn apply(&self, config: &HostConfig, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        let ensure_ids;
        let remove_ids;
        let ensure_summary;
        let remove_summary;

        if opts.remove_first {
            (remove_summary, remove_ids) = self.apply_remove(&config.resources.remove, opts)?;
            (ensure_summary, ensure_ids) = self.apply_ensure(&config.resources.ensure, opts)?;
        } else {
            (ensure_summary, ensure_ids) = self.apply_ensure(&config.resources.ensure, opts)?;
            (remove_summary, remove_ids) = self.apply_remove(&config.resources.remove, opts)?;
        }

        let changed_ids: ChangedIds = ensure_ids.union(&remove_ids).cloned().collect();
        let mut summary_total = ensure_summary + remove_summary;

        for service in &config.resources.ensure.svc {
            summary_total += service.apply(&changed_ids, opts)?;
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
