use crate::common::constants::JSON_LIB;
use crate::common::types::{ApplySummary, ChangedIds, HostConfig, Opts};
use crate::utils::helpers;
use crate::utils::{janet_helpers, reader};
use crate::{apply_resources, debug};
use anyhow::bail;
use camino::Utf8PathBuf;
use janetrs::TaggedJanet;
use std::collections::HashSet;

pub fn apply(
    host_file: &Utf8PathBuf,
    gurp_lib_path: &Option<Utf8PathBuf>,
    opts: &Opts,
) -> anyhow::Result<ApplySummary> {
    let host_config = reader::read_and_enrich_host_config(host_file, gurp_lib_path, opts, false)?;

    debug!(
        opts,
        "host-apply",
        "Janet host config follows:\n{}",
        reader::format_janet_listing(&host_config)
    );

    let client = janet_helpers::janet_client();
    let json_wrapped_host_config = format!("{JSON_LIB}\n{host_config}\n(encode (machine-config))");
    let json_buffer = client.run(json_wrapped_host_config)?;

    let json = match json_buffer.unwrap() {
        TaggedJanet::Buffer(buf) => buf.to_string(),
        other => bail!("expected Janet::Buffer, got {}", other),
    };

    tracing::debug!("Janet returned {} char JSON buffer", json.len());
    tracing::debug!("Unpacking JSON into HostConfig");

    debug!(
        opts,
        "host-apply",
        "JSON host config follows:\n{}",
        helpers::pretty_json(&json)?
    );

    let host_config: HostConfig = serde_json::from_str(&json)?;

    debug!(
        opts,
        "host-apply", "Rust host config follows:\n{:#?}", host_config
    );

    ensure_and_remove(&host_config, opts)
}

fn ensure_and_remove(config: &HostConfig, opts: &Opts) -> anyhow::Result<ApplySummary> {
    tracing::info!("Configuring host: {}", config.metadata.name);
    let ensure = &config.resources.ensure;
    let remove = &config.resources.remove;

    let mut summary_total = ApplySummary::default();
    let mut changed_ids: ChangedIds = HashSet::new();

    apply_resources!(summary_total, changed_ids, &ensure.zfs, opts);
    apply_resources!(summary_total, changed_ids, &ensure.user, opts);
    crate::doers::pkg::collect_and_ensure(&ensure.pkg, opts)?;
    crate::doers::gem::collect_and_ensure(&ensure.gem, opts)?;
    apply_resources!(summary_total, changed_ids, &ensure.cron, opts);
    apply_resources!(summary_total, changed_ids, &ensure.directory, opts);
    apply_resources!(summary_total, changed_ids, &ensure.symlink, opts);
    apply_resources!(summary_total, changed_ids, &ensure.file, opts);
    apply_resources!(summary_total, changed_ids, &ensure.file_line, opts);
    apply_resources!(summary_total, changed_ids, &ensure.smf, opts);
    apply_resources!(summary_total, changed_ids, &ensure.misc, opts);

    apply_resources!(summary_total, changed_ids, &remove.file_line, opts);
    apply_resources!(summary_total, changed_ids, &remove.file, opts);
    apply_resources!(summary_total, changed_ids, &remove.directory, opts);
    apply_resources!(summary_total, changed_ids, &remove.symlink, opts);
    apply_resources!(summary_total, changed_ids, &remove.smf, opts);
    apply_resources!(summary_total, changed_ids, &remove.cron, opts);
    crate::doers::pkg::collect_and_remove(&remove.pkg, opts)?;
    crate::doers::gem::collect_and_remove(&remove.gem, opts)?;
    apply_resources!(summary_total, changed_ids, &remove.user, opts);
    apply_resources!(summary_total, changed_ids, &remove.zfs, opts);

    for resource in &ensure.svc {
        let summary = resource.apply(&changed_ids, opts)?;
        summary_total = summary_total + summary;
    }

    Ok(summary_total)
}
