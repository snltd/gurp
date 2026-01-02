use crate::types::{ChangedIds, HostConfig};
use anyhow::{Context, ensure};
use camino::Utf8PathBuf;
use colored::Colorize;
use common::constants::SERVER_PORT;
use common::helpers;
use common::prelude::*;
use janet_int::helpers as janet_helpers;
use janet_int::reader;
use janetrs::env::CFunOptions;
use std::collections::BTreeSet;
use std::fs;
use std::thread;
use std::time::Duration;
use util::http;

fn formatted_json(raw_json: &str) -> anyhow::Result<String> {
    match helpers::pretty_json(raw_json) {
        Ok(json) => Ok(json),
        Err(e) => {
            tracing::error!("JSON processing error: {}", e);
            tracing::error!(raw_json);
            bail!("END");
        }
    }
}

fn fetch_from_server(server: &str, hostname: &str) -> anyhow::Result<String> {
    let mut tries = 1;

    while tries < 5 {
        tracing::debug!("try {tries} of 5");
        match fetch_precompiled_file(server, hostname) {
            Ok(resp) => {
                return Ok(resp);
            }
            Err(e) => {
                tracing::error!("error calling remote server: {e}");
                tracing::info!("sleeping for retry");
                thread::sleep(Duration::from_secs(tries * tries));
                tries += 1;
            }
        }
    }

    bail!("failed to get config from server");
}

fn get_config_json(host_file: Option<&Utf8PathBuf>, opts: &ApplyOpts) -> anyhow::Result<String> {
    if opts.image {
        let image_file = host_file.context("No host file specified")?;
        Ok(load_image(image_file)?)
    } else if opts.precompiled {
        let host_file = host_file.context("No host file specified")?;
        Ok(load_precompiled_file(host_file)?)
    } else if let Some(server) = opts.server.as_ref() {
        let hostname = opts
            .hostname
            .clone()
            .map_or_else(helpers::my_hostname, Ok)?;

        let host_config = fetch_from_server(server, &hostname)?;

        if opts.dump_config {
            let formatted_json = helpers::pretty_json(&host_config)?;

            println!(
                "{}",
                helpers::dump_config(&formatted_json, "Janet config", opts)
            );
        }

        Ok(host_config)
    } else {
        let host_file = host_file.context("No host file specified")?;
        let host_config = reader::assembled_config(host_file, opts)?;

        if opts.dump_config {
            println!(
                "{}",
                helpers::dump_config(&host_config, "Janet config", opts)
            );
        }

        let json = janet_helpers::run_config(&host_config)?;

        tracing::debug!("Janet returned {} char JSON buffer", json.len());

        if opts.dump_config {
            let formatted_json = formatted_json(&json)?;

            println!(
                "{}",
                helpers::dump_config(&formatted_json, "JSON Config", opts)
            );
        }

        Ok(json)
    }
}

pub fn apply(host_file: Option<&Utf8PathBuf>, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
    let json = get_config_json(host_file, opts)?;

    tracing::debug!("Unpacking JSON into HostConfig");

    let host_config: HostConfig = match serde_json::from_str(&json) {
        Ok(conf) => conf,
        Err(e) => {
            tracing::error!("deserializing error: {}", e);
            let formatted_json = formatted_json(&json)?;
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

            bail!("end of deserializing error output")
        }
    };

    ensure_and_remove(&host_config, opts)
}

fn load_precompiled_file(path: &Utf8PathBuf) -> anyhow::Result<String> {
    ensure!(path.exists(), "Cannot find JSON file at {}", path);

    Ok(fs::read_to_string(path)?)
}

fn load_image(path: &Utf8PathBuf) -> anyhow::Result<String> {
    ensure!(path.exists(), "Cannot find image file at {}", path);
    let mut client = janet_helpers::janet_client();
    client.add_c_fn(CFunOptions::new(c"encode-to-json", janet_helpers::encode_c));

    let janet_instructions = format!(
        "(merge-module (curenv) (load-image (slurp \"{path}\")) \"\" true)
                        (encode-to-json (machine-config))"
    );

    let janet_result = client.run(janet_instructions)?;
    Ok(janet_result.unwrap().to_string())
}

// We tell the server what we think it's called so it can build file resources we can find. This
// lets use use a raw IP address, DNS name, whatever.
fn fetch_precompiled_file(server: &str, hostname: &str) -> anyhow::Result<String> {
    let url = format!("http://{server}:{SERVER_PORT}/config/{hostname}?server_name={server}");
    tracing::info!("fetching config from {url}");
    let response = http::remove_file_to_memory(&url)?;
    let string_response = String::from_utf8(response)?;
    Ok(string_response)
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
