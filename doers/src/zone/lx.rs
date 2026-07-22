use crate::zone::config::GurpZoneDns;
use crate::zone::constants::{
    LX_RELEASES_URL, READINESS_WAIT_INTERVAL, READINESS_WAIT_TIMEOUT_NATIVE,
};
use crate::zone::helpers;
use crate::zone::types::ZoneImage;
use anyhow::{Context, bail, ensure};
use camino::Utf8PathBuf;
use common::constants::PS_BIN;
use serde_json::Value;
use std::fs;
use std::thread::sleep;
use std::time::Duration;

pub fn set_up_dns(zonepath: &Utf8PathBuf, dns_conf: &GurpZoneDns) -> anyhow::Result<()> {
    let resolv_path = zonepath.join("root").join("etc").join("resolv.conf");
    tracing::debug!("creating {}", resolv_path);

    let mut content = String::new();

    if let Some(domain) = &dns_conf.domain {
        content.push_str(&format!("domain {}\n", domain));
    }

    if let Some(nameservers) = &dns_conf.nameservers {
        for ns in nameservers {
            content.push_str(&format!("nameserver {ns}\n"));
        }
    }

    if !content.is_empty() {
        fs::write(&resolv_path, content)
            .with_context(|| format!("failed to write DNS config to {resolv_path}"))?;
    }

    Ok(())
}

pub fn image_path(image: ZoneImage) -> anyhow::Result<Utf8PathBuf> {
    ensure!(image.user_string.is_some(), "LX zones require an :image");

    match get_image(&image)? {
        Some(path) => Ok(path),
        None => bail!("did not find a suitable LX image"),
    }
}

pub fn wait_for_readiness(zone: &str) -> anyhow::Result<bool> {
    let elapsed = Duration::from_secs(0);
    loop {
        if is_ready(zone)? {
            return Ok(true);
        }

        sleep(READINESS_WAIT_INTERVAL);
        let elapsed = elapsed + READINESS_WAIT_INTERVAL;

        if elapsed >= READINESS_WAIT_TIMEOUT_NATIVE {
            bail!("Timed out waiting for {} be ready", zone)
        }
    }
}

fn fetch_latest_release_images() -> anyhow::Result<Option<Vec<String>>> {
    tracing::debug!("fetching latest release images");
    let response: Value = ureq::get(LX_RELEASES_URL.as_str())
        .call()
        .with_context(|| {
            format!(
                "failed to fetch LX image list from {}",
                LX_RELEASES_URL.as_str()
            )
        })?
        .into_body()
        .read_json()
        .context("failed to parse LX release data")?;

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

fn get_image(img: &ZoneImage) -> anyhow::Result<Option<Utf8PathBuf>> {
    let pattern = img
        .user_string
        .context("searching for lx image, but no user string")?;

    if let Some(img_url) = find_image(pattern)? {
        Ok(Some(helpers::get_image(&img_url, img.checksum)?))
    } else {
        Ok(None)
    }
}

// Because there are a bunch of possible images, it's hard to know what to look for here. For
// starters I'm going to try, "are you running half-a-dozen processes"?
//
fn is_ready(zone: &str) -> anyhow::Result<bool> {
    let ps_output = cmd_output!(PS_BIN, "-e", "-z", zone)
        .with_context(|| format!("failed to get process table for zone {zone}"))?;
    Ok(ps_output.lines().count() > 5)
}
