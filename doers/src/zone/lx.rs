use super::config::GurpZoneDns;
use crate::zone::constants::*;
use anyhow::bail;
use camino::Utf8PathBuf;
use common::prelude::*;
use serde_json::Value;
use std::fs;
use std::thread::sleep;
use std::time::Duration;
use util::http;

pub fn set_up_dns(zonepath: &Utf8PathBuf, dns_conf: &GurpZoneDns) -> anyhow::Result<()> {
    let resolv_path = zonepath.join("root").join("etc").join("resolv.conf");
    tracing::debug!("creating {}", resolv_path);

    let mut content = format!("domain {}\n", dns_conf.domain);

    for ns in &dns_conf.nameservers {
        content.push_str(&format!("nameserver {ns}\n"));
    }

    fs::write(resolv_path, content)?;

    Ok(())
}

fn fetch_latest_release_images() -> anyhow::Result<Option<Vec<String>>> {
    tracing::debug!("fetching latest release images");

    let response: Value = ureq::get(LX_RELEASES_URL).call()?.into_body().read_json()?;

    Ok(response
        .get(0)
        .and_then(|o| o.get("assets"))
        .and_then(|arr| arr.as_array())
        .map(|assets| {
            assets
                .iter()
                .filter_map(|e| e.get("browser_download_url")?.as_str())
                .map(str::to_owned)
                .collect()
        }))
}

fn find_image(pattern: &str) -> anyhow::Result<Option<String>> {
    if let Some(mut latest_images) = fetch_latest_release_images()? {
        latest_images.sort();
        latest_images.reverse();
        Ok(latest_images.iter().find(|i| i.contains(pattern)).cloned())
    } else {
        Ok(None)
    }
}

pub fn image_path(pattern: &str) -> anyhow::Result<Option<Utf8PathBuf>> {
    if let Some(img_url) = find_image(pattern)? {
        let chunks = img_url.split("/");
        if let Some(img_basename) = chunks.last() {
            let img_path = Utf8PathBuf::from(IMG_CACHE_DIR).join(img_basename);
            if img_path.exists() {
                tracing::debug!("found image at {img_path}");
            } else {
                tracing::debug!("no image at {img_path}: downloading");
                http::download_file(&img_url, &img_path)?;
            }

            Ok(Some(img_path))
        } else {
            bail!("could not get image basename");
        }
    } else {
        Ok(None)
    }
}

fn is_ready_lx(zone: &str) -> anyhow::Result<bool> {
    let ps_output = cmd_output!(PS_BIN, "-e", "-z", zone)?;
    Ok(ps_output.lines().count() > 5)
}

pub fn wait_for_readiness(zone: &str) -> anyhow::Result<bool> {
    // Because there are a bunch of possible images, it's hard to know what to look for here. For
    // starters I'm going to try, "are you running half-a-dozen processes"?
    //
    let elapsed = Duration::from_secs(0);
    loop {
        if is_ready_lx(zone)? {
            return Ok(true);
        }

        sleep(READINESS_WAIT_INTERVAL);
        let elapsed = elapsed + READINESS_WAIT_INTERVAL;

        if elapsed >= READINESS_WAIT_TIMEOUT {
            bail!("Timed out waiting for {} be ready", zone)
        }
    }
}
