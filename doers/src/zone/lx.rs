use crate::zone::config::{GurpZoneDns, ImageSource};
use crate::zone::constants::{
    LX_RELEASES_URL, READINESS_WAIT_INTERVAL, READINESS_WAIT_TIMEOUT_NATIVE,
};
use crate::zone::helpers;
use crate::zone::types::ZoneImage;
use anyhow::{Context, bail};
use camino::Utf8PathBuf;
use common::constants::PS_BIN;
use serde_json::Value;
use std::fs;
use std::thread::sleep;
use std::time::Duration;
use url::Url;

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
    match image.image_source {
        Some(image_source) => match image_source {
            ImageSource::Path(path) => Ok(path.to_owned()),
            ImageSource::Url(url) => helpers::get_image(url, image.checksum),
            ImageSource::Name(name) => {
                let url = find_image_url(name)?
                    .with_context(|| format!("failed to find LX image '{name}'"))?;
                helpers::get_image(&url, image.checksum)
            }
        },
        None => bail!("LX zones require an :image"),
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

fn find_image_url(pattern: &str) -> anyhow::Result<Option<Url>> {
    let maybe_image = fetch_latest_release_images()
        .context("failed to get release images")?
        .and_then(|mut latest_images| {
            latest_images.sort();
            latest_images.reverse();
            latest_images.into_iter().find(|i| i.contains(pattern))
        });

    maybe_image
        .map(|i| Url::parse(&i).with_context(|| format!("failed to parse image name {i}")))
        .transpose()
}

// Because there are a bunch of possible images, it's hard to know what to look for here. For
// starters I'm going to try, "are you running half-a-dozen processes"?
//
fn is_ready(zone: &str) -> anyhow::Result<bool> {
    let ps_output = cmd_output!(PS_BIN, "-e", "-z", zone)
        .with_context(|| format!("failed to get process table for zone {zone}"))?;
    Ok(ps_output.lines().count() > 5)
}
