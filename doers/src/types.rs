use crate::apk::{ApkEnsure, ApkRemove};
use crate::bridge::{BridgeEnsure, BridgeRemove};
use crate::cron::{CronEnsure, CronRemove};
use crate::directory::{DirectoryEnsure, DirectoryRemove};
use crate::etherstub::{EtherstubEnsure, EtherstubRemove};
use crate::file::ensure::FileEnsure;
use crate::file::remove::FileRemove;
use crate::file_line::{FileLineEnsure, FileLineRemove};
use crate::gem::{GemEnsure, GemRemove};
use crate::group::{GroupEnsure, GroupRemove};
use crate::ip_address::{IpAddressEnsure, IpAddressRemove};
use crate::ip_interface::{IpInterfaceEnsure, IpInterfaceRemove};
use crate::ip_properties::IpPropertiesEnsure;
use crate::ipfilter::{IpfilterEnsure, IpfilterRemove};
use crate::ipnat::{IpnatEnsure, IpnatRemove};
use crate::link::{LinkEnsure, LinkRemove};
use crate::misc::MiscEnsure;
use crate::network_flow::{NetworkFlowEnsure, NetworkFlowRemove};
use crate::pkg::{PkgEnsure, PkgRemove};
use crate::pkgin::{PkginEnsure, PkginRemove};
use crate::publisher::{PublisherEnsure, PublisherRemove};
use crate::route::{RouteEnsure, RouteRemove};
use crate::self_update;
use crate::smf::{SmfEnsure, SmfRemove};
use crate::svc::SvcEnsure;
use crate::svcprop::{SvcpropEnsure, SvcpropRemove};
use crate::system_cert::{SystemCertEnsure, SystemCertRemove};
use crate::user::{UserEnsure, UserRemove};
use crate::vlan::{VlanEnsure, VlanRemove};
use crate::vnic::{VnicEnsure, VnicRemove};
use crate::zfs::{ZfsEnsure, ZfsRemove};
use crate::zone::{ZoneEnsure, ZoneRemove};
use anyhow::{Context, bail};
use bytesize::ByteSize;
use camino::Utf8PathBuf;
use common::types::{ApplyOpts, ApplySummary, ChangedIds, JsonConfig};
use gurptel::runtime_stats;
use os_types::GurpId;
use owo_colors::OwoColorize;
use rand::RngExt;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Error;
use std::collections::BTreeSet;
use std::thread;
use std::time::Duration;
use util::{info, json};

pub(crate) type ApplyResult = anyhow::Result<(ApplySummary, ChangedIds)>;

pub trait Apply {
    fn id(&self) -> &GurpId;
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
    pub apk: Vec<ApkEnsure>,
    #[serde(default)]
    pub bridge: Vec<BridgeEnsure>,
    #[serde(default)]
    pub cron: Vec<CronEnsure>,
    #[serde(default)]
    pub directory: Vec<DirectoryEnsure>,
    #[serde(default)]
    pub etherstub: Vec<EtherstubEnsure>,
    #[serde(default)]
    pub file: Vec<FileEnsure>,
    #[serde(default)]
    pub file_line: Vec<FileLineEnsure>,
    #[serde(default)]
    pub gem: Vec<GemEnsure>,
    #[serde(default)]
    pub group: Vec<GroupEnsure>,
    #[serde(default)]
    pub ip_address: Vec<IpAddressEnsure>,
    #[serde(default)]
    pub ip_interface: Vec<IpInterfaceEnsure>,
    #[serde(default)]
    pub ip_properties: Vec<IpPropertiesEnsure>,
    #[serde(default)]
    pub ipfilter: Vec<IpfilterEnsure>,
    #[serde(default)]
    pub ipnat: Vec<IpnatEnsure>,
    #[serde(default)]
    pub link: Vec<LinkEnsure>,
    #[serde(default)]
    pub misc: Vec<MiscEnsure>,
    #[serde(default)]
    pub network_flow: Vec<NetworkFlowEnsure>,
    #[serde(default)]
    pub pkg: Vec<PkgEnsure>,
    #[serde(default)]
    pub pkgin: Vec<PkginEnsure>,
    #[serde(default)]
    pub publisher: Vec<PublisherEnsure>,
    #[serde(default)]
    pub route: Vec<RouteEnsure>,
    #[serde(default)]
    pub svcprop: Vec<SvcpropEnsure>,
    #[serde(default)]
    pub smf: Vec<SmfEnsure>,
    #[serde(default)]
    pub svc: Vec<SvcEnsure>,
    #[serde(default)]
    pub system_cert: Vec<SystemCertEnsure>,
    #[serde(default)]
    pub user: Vec<UserEnsure>,
    #[serde(default)]
    pub vlan: Vec<VlanEnsure>,
    #[serde(default)]
    pub vnic: Vec<VnicEnsure>,
    #[serde(default)]
    pub zfs: Vec<ZfsEnsure>,
    #[serde(default)]
    pub zone: Vec<ZoneEnsure>,
}

#[derive(Default, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct RemoveResources {
    #[serde(default)]
    pub apk: Vec<ApkRemove>,
    #[serde(default)]
    pub bridge: Vec<BridgeRemove>,
    #[serde(default)]
    pub cron: Vec<CronRemove>,
    #[serde(default)]
    pub directory: Vec<DirectoryRemove>,
    #[serde(default)]
    pub etherstub: Vec<EtherstubRemove>,
    #[serde(default)]
    pub file: Vec<FileRemove>,
    #[serde(default)]
    pub file_line: Vec<FileLineRemove>,
    #[serde(default)]
    pub gem: Vec<GemRemove>,
    #[serde(default)]
    pub group: Vec<GroupRemove>,
    #[serde(default)]
    pub ip_address: Vec<IpAddressRemove>,
    #[serde(default)]
    pub ip_interface: Vec<IpInterfaceRemove>,
    #[serde(default)]
    pub ipfilter: Vec<IpfilterRemove>,
    #[serde(default)]
    pub ipnat: Vec<IpnatRemove>,
    #[serde(default)]
    pub link: Vec<LinkRemove>,
    #[serde(default)]
    pub network_flow: Vec<NetworkFlowRemove>,
    #[serde(default)]
    pub pkg: Vec<PkgRemove>,
    #[serde(default)]
    pub pkgin: Vec<PkginRemove>,
    #[serde(default)]
    pub publisher: Vec<PublisherRemove>,
    #[serde(default)]
    pub route: Vec<RouteRemove>,
    #[serde(default)]
    pub smf: Vec<SmfRemove>,
    #[serde(default)]
    pub svcprop: Vec<SvcpropRemove>,
    #[serde(default)]
    pub system_cert: Vec<SystemCertRemove>,
    #[serde(default)]
    pub user: Vec<UserRemove>,
    #[serde(default)]
    pub vlan: Vec<VlanRemove>,
    #[serde(default)]
    pub vnic: Vec<VnicRemove>,
    #[serde(default)]
    pub zfs: Vec<ZfsRemove>,
    #[serde(default)]
    pub zone: Vec<ZoneRemove>,
}

macro_rules! impl_apply {
    ($($t:ty),*) => {
        $( impl Apply for $t {
                fn id(&self) -> &GurpId { &self.id }
                fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
                    self.apply(opts)
                }
            }
        )*
    }
}

impl_apply!(
    BridgeEnsure,
    CronEnsure,
    DirectoryEnsure,
    EtherstubEnsure,
    FileEnsure,
    FileLineEnsure,
    GroupEnsure,
    IpAddressEnsure,
    IpInterfaceEnsure,
    IpPropertiesEnsure,
    LinkEnsure,
    MiscEnsure,
    NetworkFlowEnsure,
    PublisherEnsure,
    RouteEnsure,
    SvcpropEnsure,
    SmfEnsure,
    SystemCertEnsure,
    UserEnsure,
    VlanEnsure,
    VnicEnsure,
    ZfsEnsure,
    ZoneEnsure,
    BridgeRemove,
    CronRemove,
    DirectoryRemove,
    EtherstubRemove,
    FileRemove,
    FileLineRemove,
    GroupRemove,
    IpAddressRemove,
    IpInterfaceRemove,
    IpfilterRemove,
    IpnatRemove,
    LinkRemove,
    NetworkFlowRemove,
    PublisherRemove,
    RouteRemove,
    SmfRemove,
    SvcpropRemove,
    SystemCertRemove,
    UserRemove,
    VlanRemove,
    VnicRemove,
    ZfsRemove,
    ZoneRemove
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
        if opts.pre_run_noop && !opts.noop {
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

        if apply_result.changes > 0 && opts.post_run_noop {
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

        let ret = self.apply(&config, &phase, opts);

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

                if !rx.is_match(&id.to_string()) {
                    continue;
                }
            }

            if std::env::var("GURP_RSS_STATS").is_ok()
                && let Some(rss) = runtime_stats::rss_bytes()
            {
                tracing::info!("RSS before {id}: {}", ByteSize(rss as u64));
            }

            // let chunks: Vec<_> = id.split("/").collect();
            //
            tracing::debug!(
                "applying {} {}/{total_count}: {id}",
                id.resource_type(),
                i + 1,
            );

            // if chunks.len() >= 3 {
            //     tracing::debug!("applying {} {}/{total_count}: {id}", chunks[1], i + 1,);
            // } else {
            //     tracing::debug!("applying [{}/{total_count}]: {id}", i + 1);
            // }

            let summary = match resource.apply(opts) {
                Ok(summary) => summary,
                Err(e) => {
                    tracing::error!("from {} doer: {}", id.resource_name(), e);
                    let err: anyhow::Error = e;
                    return Err(err.context(format!("failed to apply resource {id}")));
                }
            };

            if summary.changes > 0 {
                changed_ids.insert(id.clone());
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

    fn apply(
        &self,
        config: &HostConfig,
        phase: &Phase,
        opts: &ApplyOpts,
    ) -> anyhow::Result<ApplySummary> {
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

        if *phase == Phase::Apply
            && let Some(self_update) = &config.control_data.self_update
        {
            summary_total += self_update::update_gurp(self_update, opts)?;
        }

        Ok(summary_total)
    }

    // This should only handle deserialing errors.
    fn display_error(&self, e: Error) -> anyhow::Result<()> {
        tracing::error!("deserializing error: {}", e);

        let formatted_json = json::formatted(&self.json_config)?;
        let mut error_line = e.line();
        let error_col = e.column();

        // Looks like Rust sees Janet code as one long line. This isn't unreasonable.
        // So if the line is 1, let's use the column to find where the error in the code.

        let json_lines: Vec<_> = formatted_json.lines().collect();

        if let Some(first_line) = json_lines.first()
            && error_line == 1
            && error_col > first_line.len()
        {
            let mut col_count = 0;
            for (i, l) in json_lines.iter().enumerate() {
                col_count += l.len() - 1;
                if col_count >= error_col {
                    error_line = i;
                    break;
                }
            }
        }

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

#[cfg(test)]
mod test {
    // TODO we need tests for various fail modes.
}
