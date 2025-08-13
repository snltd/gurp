use super::config::GurpZoneDns;
use anyhow::bail;
use camino::Utf8PathBuf;
use serde_json::Value;
use std::fs::{self, File};
use std::io::copy;

const RELEASES_URL: &str = "https://api.github.com/repos/omniosorg/lx-images/releases";
const IMG_CACHE: &str = "/var/tmp";

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

fn get_image(url: &str, path: &Utf8PathBuf) -> anyhow::Result<()> {
    tracing::info!("downloading {url} -> {path}");
    let response = ureq::get(url).call()?;
    let mut reader = response.into_reader();

    let mut file = File::create(path)?;
    copy(&mut reader, &mut file)?;

    Ok(())
}

fn fetch_latest_release_images() -> anyhow::Result<Option<Vec<String>>> {
    tracing::debug!("fetching latest release images");
    let response: Value = ureq::get(RELEASES_URL).call()?.into_json()?;

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
            let img_path = Utf8PathBuf::from(IMG_CACHE).join(img_basename);
            if img_path.exists() {
                tracing::debug!("found image at {img_path}");
            } else {
                tracing::debug!("no image at {img_path}: downloading");
                get_image(&img_url, &img_path)?;
            }

            Ok(Some(img_path))
        } else {
            bail!("could not get image basename");
        }
    } else {
        Ok(None)
    }
}
