use crate::types::{ChangedIds, HostConfig};
use camino::Utf8PathBuf;
use common::prelude::*;
use embed::compiler;
use std::collections::BTreeSet;

pub fn apply(host_file: Option<&Utf8PathBuf>, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
    let json = compiler::extract_json(host_file, opts)?;

    tracing::debug!(
        "Unpacking {} bytes of JSON config into HostConfig",
        json.len()
    );

    let host_config: HostConfig = match serde_json::from_str(&json) {
        Ok(conf) => conf,
        Err(e) => {
            compiler::display_error(e, &json)?;
            bail!("end of deserializing output");
        }
    };

    ensure_and_remove(&host_config, opts)
}

fn ensure_and_remove(config: &HostConfig, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
    tracing::info!("Configuring host: {}", config.metadata.name);
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
    apply_resources!(summary_total, changed_ids, &ensure.route, opts);
    apply_resources!(summary_total, changed_ids, &ensure.ip_properties, opts);
    apply_resources!(summary_total, changed_ids, &ensure.network_flow, opts);
    apply_resources!(summary_total, changed_ids, &ensure.zfs, opts);
    apply_resources!(summary_total, changed_ids, &ensure.zone, opts);

    if !&ensure.ipnat.is_empty() {
        summary_total = summary_total + crate::ipnat::collect_and_ensure(&ensure.ipnat, opts)?;
    }

    if !&ensure.pkg.is_empty() {
        summary_total = summary_total + crate::pkg::collect_and_ensure(&ensure.pkg, opts)?;
    }

    if !&ensure.pkgin.is_empty() {
        summary_total = summary_total + crate::pkgin::collect_and_ensure(&ensure.pkgin, opts)?;
    }

    if !&ensure.apk.is_empty() {
        summary_total = summary_total + crate::apk::collect_and_ensure(&ensure.apk, opts)?;
    }

    if !&ensure.gem.is_empty() {
        summary_total = summary_total + crate::gem::collect_and_ensure(&ensure.gem, opts)?;
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
        summary_total = summary_total + crate::gem::collect_and_remove(&remove.gem, opts)?;
    }

    if !&remove.pkg.is_empty() {
        summary_total = summary_total + crate::pkg::collect_and_remove(&remove.pkg, opts)?;
    }

    if !&remove.pkgin.is_empty() {
        summary_total = summary_total + crate::pkgin::collect_and_remove(&remove.pkgin, opts)?;
    }

    if !&remove.apk.is_empty() {
        summary_total = summary_total + crate::apk::collect_and_remove(&remove.apk, opts)?;
    }

    apply_resources!(summary_total, changed_ids, &remove.ipnat, opts);
    apply_resources!(summary_total, changed_ids, &remove.zone, opts);
    apply_resources!(summary_total, changed_ids, &remove.zfs, opts);
    apply_resources!(summary_total, changed_ids, &remove.network_flow, opts);
    apply_resources!(summary_total, changed_ids, &remove.route, opts);
    apply_resources!(summary_total, changed_ids, &remove.ip_address, opts);
    apply_resources!(summary_total, changed_ids, &remove.ip_interface, opts);
    apply_resources!(summary_total, changed_ids, &remove.vlan, opts);
    apply_resources!(summary_total, changed_ids, &remove.vnic, opts);
    apply_resources!(summary_total, changed_ids, &remove.etherstub, opts);

    for resource in &ensure.svc {
        let summary = resource.apply(&changed_ids, opts)?;
        summary_total = summary_total + summary;
    }

    Ok(summary_total)
}
