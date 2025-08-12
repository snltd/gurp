use crate::common::types::{ChangedIds, HostConfig};
use crate::prelude::*;
use crate::utils::{janet_helpers, reader};
use janetrs::{TaggedJanet, env::CFunOptions};
use std::collections::BTreeSet;

pub fn apply(
    host_file: &Utf8PathBuf,
    gurp_lib_path: &Option<Utf8PathBuf>,
    opts: &Opts,
) -> anyhow::Result<ApplySummary> {
    let host_config = reader::read_and_enrich_host_config(host_file, gurp_lib_path, opts, false)?;

    if opts.dump_config {
        println!(
            "{}",
            helpers::dump_config(&host_config, "Janet config", opts)
        );
    }

    let mut client = janet_helpers::janet_client();
    client.add_c_fn(CFunOptions::new(c"encode", janet_helpers::encode_c));
    let json_wrapped_host_config = format!("{host_config}\n(encode (machine-config))");
    let json_config = client.run(json_wrapped_host_config)?;

    let json = match json_config.unwrap() {
        TaggedJanet::String(buf) => buf.to_string(),
        other => bail!("expected JSON config as Janet::String; got {}", other),
    };

    tracing::debug!("Janet returned {} char JSON buffer", json.len());
    tracing::debug!("Unpacking JSON into HostConfig");

    let formatted_json = match helpers::pretty_json(&json) {
        Ok(json) => json,
        Err(e) => {
            tracing::error!("JSON processing error: {}", e);
            tracing::error!(json);
            bail!("END");
        }
    };

    if opts.dump_config {
        println!(
            "{}",
            helpers::dump_config(&formatted_json, "JSON Config", opts)
        );
    }

    let host_config: HostConfig = match serde_json::from_str(&formatted_json) {
        Ok(conf) => conf,
        Err(e) => {
            tracing::error!("deserializing error: {}", e);
            let line = e.line();
            let json_lines: Vec<_> = formatted_json.lines().collect();
            let first_line = line.saturating_sub(30);
            let last_line = (line + 15).clamp(0, json_lines.len());

            for l in first_line..=last_line {
                println!(" {:4} | {}", l + 1, json_lines.get(l).unwrap_or(&""));
            }

            bail!("end of deserializing error output")
        }
    };

    ensure_and_remove(&host_config, opts)
}

fn ensure_and_remove(config: &HostConfig, opts: &Opts) -> anyhow::Result<ApplySummary> {
    tracing::info!("Configuring host: {}", config.metadata.name);
    let ensure = &config.resources.ensure;
    let remove = &config.resources.remove;

    let mut summary_total = ApplySummary::default();
    let mut changed_ids: ChangedIds = BTreeSet::new();

    apply_resources!(summary_total, changed_ids, &ensure.publisher, opts);
    apply_resources!(summary_total, changed_ids, &ensure.zfs, opts);
    apply_resources!(summary_total, changed_ids, &ensure.zone, opts);

    if !&ensure.pkg.is_empty() {
        summary_total = summary_total + crate::doers::pkg::collect_and_ensure(&ensure.pkg, opts)?;
    }

    if !&ensure.gem.is_empty() {
        summary_total = summary_total + crate::doers::gem::collect_and_ensure(&ensure.gem, opts)?;
    }

    apply_resources!(summary_total, changed_ids, &ensure.group, opts);
    apply_resources!(summary_total, changed_ids, &ensure.user, opts);
    apply_resources!(summary_total, changed_ids, &ensure.cron, opts);
    apply_resources!(summary_total, changed_ids, &ensure.directory, opts);
    apply_resources!(summary_total, changed_ids, &ensure.symlink, opts);
    apply_resources!(summary_total, changed_ids, &ensure.file, opts);
    apply_resources!(summary_total, changed_ids, &ensure.file_line, opts);
    apply_resources!(summary_total, changed_ids, &ensure.svcprop, opts);
    apply_resources!(summary_total, changed_ids, &ensure.smf, opts);
    apply_resources!(summary_total, changed_ids, &ensure.misc, opts);

    apply_resources!(summary_total, changed_ids, &remove.file_line, opts);
    apply_resources!(summary_total, changed_ids, &remove.file, opts);
    apply_resources!(summary_total, changed_ids, &remove.directory, opts);
    apply_resources!(summary_total, changed_ids, &remove.symlink, opts);
    apply_resources!(summary_total, changed_ids, &remove.svcprop, opts);
    apply_resources!(summary_total, changed_ids, &remove.smf, opts);
    apply_resources!(summary_total, changed_ids, &remove.cron, opts);
    apply_resources!(summary_total, changed_ids, &remove.user, opts);
    apply_resources!(summary_total, changed_ids, &remove.group, opts);
    apply_resources!(summary_total, changed_ids, &remove.publisher, opts);

    if !&remove.pkg.is_empty() {
        summary_total = summary_total + crate::doers::pkg::collect_and_remove(&remove.pkg, opts)?;
    }

    if !&remove.gem.is_empty() {
        summary_total = summary_total + crate::doers::gem::collect_and_remove(&remove.gem, opts)?;
    }

    apply_resources!(summary_total, changed_ids, &remove.zone, opts);
    apply_resources!(summary_total, changed_ids, &remove.zfs, opts);

    for resource in &ensure.svc {
        let summary = resource.apply(&changed_ids, opts)?;
        summary_total = summary_total + summary;
    }

    Ok(summary_total)
}
