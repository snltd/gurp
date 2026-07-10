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
use crate::system_cert::{GurpSystemCertEnsure, GurpSystemCertRemove};
use crate::user::{GurpUserEnsure, GurpUserRemove};
use crate::vlan::{GurpVlanEnsure, GurpVlanRemove};
use crate::vnic::{GurpVnicEnsure, GurpVnicRemove};
use crate::zfs::{GurpZfsEnsure, GurpZfsRemove};
use crate::zone::{GurpZoneEnsure, GurpZoneRemove};
use anyhow::{Context, bail};
use bytesize::ByteSize;
use camino::Utf8PathBuf;
use common::types::{ApplyOpts, ApplySummary, ChangedIds, JsonConfig};
use gurptel::runtime_stats;
use owo_colors::OwoColorize;
use rand::RngExt;
use rand::distr::{Alphanumeric, SampleString};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Error;
use std::collections::BTreeSet;
use std::fs::OpenOptions;
use std::time::Duration;
use std::{env, thread};
use util::{info, json};

pub(crate) type ApplyResult = anyhow::Result<(ApplySummary, ChangedIds)>;

pub trait Apply {
    fn id(&self) -> &str;
    fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary>;
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct HostConfig {
    pub metadata: HostMetadata,
    pub control_data: HostControlData,
    pub resources: HostResources,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct HostMetadata {
    pub name: String,
}

/// This must reflect the struct in janet/src/control-data.janet
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct HostControlData {
    pub splay_seconds: Option<u64>,
    pub gem_path: Option<Utf8PathBuf>,
    pub metrics_to: Option<String>,
    pub logs_to: Option<String>,
    pub self_update: Option<String>,
    #[serde(default)]
    pub strict_hostname: bool,
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
    pub system_cert: Vec<GurpSystemCertEnsure>,
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
    pub system_cert: Vec<GurpSystemCertRemove>,
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
        $( impl Apply for $t {
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
    GurpSystemCertEnsure,
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
    GurpSystemCertRemove,
    GurpUserRemove,
    GurpVlanRemove,
    GurpVnicRemove,
    GurpZfsRemove,
    GurpZoneRemove
);

pub struct Applicator {
    json_config: String,
}

#[derive(PartialEq)]
enum Phase {
    Preflight,
    Apply,
    DoubleCheck,
}

impl Applicator {
    pub fn from(json_config: JsonConfig) -> Self {
        Self { json_config }
    }

    pub fn run(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        if !opts.no_check && !opts.noop {
            self._run(
                &ApplyOpts {
                    noop: true,
                    ..opts.clone()
                },
                Phase::Preflight,
            )?;

            tracing::info!("Completed noop check phase");
        }

        let apply_result = self._run(opts, Phase::Apply)?;

        if apply_result.changes > 0 && opts.double_check {
            let second_apply_result = self._run(
                &ApplyOpts {
                    noop: true,
                    ..opts.clone()
                },
                Phase::DoubleCheck,
            )?;

            if second_apply_result.changes > 0 {
                bail!(
                    "Double-check apply phase caused {} change(s): state is not being asserted",
                    second_apply_result.changes
                )
            } else {
                tracing::debug!("Double-check detected no changes");
                Ok(apply_result)
            }
        } else {
            tracing::debug!("No changes: no need for double-check");
            Ok(apply_result)
        }
    }

    fn _run(&self, opts: &ApplyOpts, phase: Phase) -> anyhow::Result<ApplySummary> {
        let config = self.json_to_hostconfig()?;
        let mut splay = None;

        if let Some(control_splay) = config.control_data.splay_seconds {
            splay = Some(control_splay);
        }

        if let Some(cli_splay) = opts.splay {
            splay = Some(cli_splay);
        }

        if config.control_data.strict_hostname {
            tracing::debug!("Strict hostname checking requested: comparing");
            let my_hostname = info::my_hostname().context("failed to get local hostname")?;

            if config.metadata.name != my_hostname {
                bail!(
                    "Strict hostname check failed. This host is {}, config is for {}",
                    my_hostname,
                    config.metadata.name
                )
            }
        }

        if phase == Phase::Apply
            && let Some(max_splay) = splay
        {
            let mut rng = rand::rng();
            let time_to_wait = rng.random_range(..=max_splay);
            tracing::info!("splay requested: pausing {time_to_wait} seconds...");
            thread::sleep(Duration::from_secs(time_to_wait));
        }

        let message = match phase {
            Phase::Preflight => "Running noop check for",
            Phase::Apply => "Configuring",
            Phase::DoubleCheck => "Double-checking",
        };

        tracing::info!("{message} {}", config.metadata.name);

        let ret = self.apply(&config, opts);

        if phase == Phase::Apply
            && std::env::var("GURP_RSS_STATS").is_ok()
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

        tracing::debug!("Raw JSON is {}", self.json_config);

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

    fn apply_resources<'a, T: Apply>(
        &self,
        resources: &'a [T],
        opts: &'a ApplyOpts,
    ) -> ApplyResult {
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

    #[rustfmt::skip]
    fn apply_ensure(&self, res: &EnsureResources, opts: &ApplyOpts) -> ApplyResult {
        let mut sum = ApplySummary::default();
        let mut ids = ChangedIds::new();

        self.accumulate(&mut sum, &mut ids, self.apply_resources(&res.publisher, opts))?;
        self.accumulate(&mut sum, &mut ids, self.apply_resources(&res.etherstub, opts))?;
        self.accumulate(&mut sum, &mut ids, self.apply_resources(&res.vnic, opts))?;
        self.accumulate(&mut sum, &mut ids, self.apply_resources(&res.vlan, opts))?;
        self.accumulate(&mut sum, &mut ids, self.apply_resources(&res.ip_interface, opts))?;
        self.accumulate(&mut sum, &mut ids, self.apply_resources(&res.ip_address, opts))?;
        self.accumulate(&mut sum, &mut ids, self.apply_resources(&res.bridge, opts))?;
        self.accumulate(&mut sum, &mut ids, self.apply_resources(&res.route, opts))?;
        self.accumulate(&mut sum, &mut ids, self.apply_resources(&res.ip_properties, opts))?;
        self.accumulate(&mut sum, &mut ids, self.apply_resources(&res.network_flow, opts))?;
        self.accumulate(&mut sum, &mut ids, self.apply_resources(&res.zfs, opts))?;
        self.accumulate(&mut sum, &mut ids, self.apply_resources(&res.zone, opts))?;
        self.accumulate(&mut sum, &mut ids, crate::ipfilter::collect_and_ensure(&res.ipfilter, opts))?;
        self.accumulate(&mut sum, &mut ids, crate::ipnat::collect_and_ensure(&res.ipnat, opts))?;
        self.accumulate(&mut sum, &mut ids, crate::pkg::collect_and_ensure(&res.pkg, opts))?;
        self.accumulate(&mut sum, &mut ids, crate::pkgin::collect_and_ensure(&res.pkgin, opts))?;
        self.accumulate(&mut sum, &mut ids, crate::apk::collect_and_ensure(&res.apk, opts))?;
        self.accumulate(&mut sum, &mut ids, crate::gem::collect_and_ensure(&res.gem, opts))?;
        self.accumulate(&mut sum, &mut ids, self.apply_resources(&res.group, opts))?;
        self.accumulate(&mut sum, &mut ids, self.apply_resources(&res.user, opts))?;
        self.accumulate(&mut sum, &mut ids, self.apply_resources(&res.cron, opts))?;
        self.accumulate(&mut sum, &mut ids, self.apply_resources(&res.directory, opts))?;
        self.accumulate(&mut sum, &mut ids, self.apply_resources(&res.file, opts))?;
        self.accumulate(&mut sum, &mut ids, self.apply_resources(&res.file_line, opts))?;
        self.accumulate(&mut sum, &mut ids, self.apply_resources(&res.link, opts))?;
        self.accumulate(&mut sum, &mut ids, self.apply_resources(&res.smf, opts))?;
        self.accumulate(&mut sum, &mut ids, self.apply_resources(&res.svcprop, opts))?;
        self.accumulate(&mut sum, &mut ids, self.apply_resources(&res.system_cert, opts))?;
        self.accumulate(&mut sum, &mut ids, self.apply_resources(&res.misc, opts))?;

        Ok((sum, ids))
    }

    #[rustfmt::skip]
    fn apply_remove(&self, res: &RemoveResources, opts: &ApplyOpts) -> ApplyResult {
        let mut sum = ApplySummary::default();
        let mut ids = ChangedIds::new();

        self.accumulate(&mut sum, &mut ids, self.apply_resources(&res.link, opts))?;
        self.accumulate(&mut sum, &mut ids, self.apply_resources(&res.system_cert, opts))?;
        self.accumulate(&mut sum, &mut ids, self.apply_resources(&res.file_line, opts))?;
        self.accumulate(&mut sum, &mut ids, self.apply_resources(&res.file, opts))?;
        self.accumulate(&mut sum, &mut ids, self.apply_resources(&res.directory, opts))?;
        self.accumulate(&mut sum, &mut ids, self.apply_resources(&res.svcprop, opts))?;
        self.accumulate(&mut sum, &mut ids, self.apply_resources(&res.smf, opts))?;
        self.accumulate(&mut sum, &mut ids, self.apply_resources(&res.cron, opts))?;
        self.accumulate(&mut sum, &mut ids, self.apply_resources(&res.user, opts))?;
        self.accumulate(&mut sum, &mut ids, self.apply_resources(&res.group, opts))?;
        self.accumulate(&mut sum, &mut ids, self.apply_resources(&res.publisher, opts))?;
        self.accumulate(&mut sum, &mut ids, crate::gem::collect_and_remove(&res.gem, opts))?;
        self.accumulate(&mut sum, &mut ids, crate::pkg::collect_and_remove(&res.pkg, opts))?;
        self.accumulate(&mut sum, &mut ids, crate::pkgin::collect_and_remove(&res.pkgin, opts))?;
        self.accumulate(&mut sum, &mut ids, crate::apk::collect_and_remove(&res.apk, opts))?;
        self.accumulate(&mut sum, &mut ids, self.apply_resources(&res.ipnat, opts))?;
        self.accumulate(&mut sum, &mut ids, self.apply_resources(&res.ipfilter, opts))?;
        self.accumulate(&mut sum, &mut ids, self.apply_resources(&res.zone, opts))?;
        self.accumulate(&mut sum, &mut ids, self.apply_resources(&res.zfs, opts))?;
        self.accumulate(&mut sum, &mut ids, self.apply_resources(&res.bridge, opts))?;
        self.accumulate(&mut sum, &mut ids, self.apply_resources(&res.network_flow, opts))?;
        self.accumulate(&mut sum, &mut ids, self.apply_resources(&res.route, opts))?;
        self.accumulate(&mut sum, &mut ids, self.apply_resources(&res.ip_address, opts))?;
        self.accumulate(&mut sum, &mut ids, self.apply_resources(&res.ip_interface, opts))?;
        self.accumulate(&mut sum, &mut ids, self.apply_resources(&res.vlan, opts))?;
        self.accumulate(&mut sum, &mut ids, self.apply_resources(&res.vnic, opts))?;
        self.accumulate(&mut sum, &mut ids, self.apply_resources(&res.etherstub, opts))?;

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

        if let Some(self_update) = &config.control_data.self_update {
            summary_total += self.update_gurp(self_update)?;
        }

        Ok(summary_total)
    }

    // This should only handle deserialing errors.
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

    fn update_gurp(&self, update_from: &str) -> anyhow::Result<ApplySummary> {
        // We update by copying in the new Gurp and doing a mv.
        let gurp_path = env::current_exe().context("cannot get current Gurp path")?;

        let gurp_path = Utf8PathBuf::from_path_buf(gurp_path)
            .ok()
            .context("Gurp path is not valid UTF-8")?;

        let gurp_dir = gurp_path.parent().context("cannot get gurp's parent dir")?;

        let suffix: String = Alphanumeric.sample_string(&mut rand::rng(), 8);
        let tmp_path = gurp_dir.join(format!("gurp.tmp-{suffix}"));

        let mut tmp_file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&tmp_path)?;

        todo!()
    }
}

#[cfg(test)]
mod test {
    // TODO we need tests for various fail modes.
}
