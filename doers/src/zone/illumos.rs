use crate::zone::constants::OMNIOS_RELEASES_URL;
use anyhow::bail;
use camino::Utf8PathBuf;
use common::constants::IMG_CACHE_DIR;
use util::http;

// Images follow the form:
// https://downloads.omnios.org/media/stable/omnios-r151056.ngz.zfs.xz
// For now we will simply download the stable NGZ image

pub fn image_path(image: Option<&str>) -> anyhow::Result<Utf8PathBuf> {
    // if we're given an image and it looks like a URL, fetch it. If it looks like a file, stat
    // it; if we don't have one, fetch the current stable
    // if let Some() = image {
    match image {
        Some(path) => {
            if path.starts_with("/") {
                Ok(Utf8PathBuf::from(path))
            } else if path.starts_with("http") {
                get_image(path)
            } else {
                bail!("illumos image must be a fully qualified path or URL")
            }
        }
        None => get_image(&default_image()?),
    }
}

fn default_image() -> anyhow::Result<String> {
    tracing::debug!(
        "fetching latest release images from {}",
        OMNIOS_RELEASES_URL
    );

    let html = ureq::get(OMNIOS_RELEASES_URL)
        .call()?
        .into_body()
        .read_to_string()?;

    let link = html
        .split("href=\"")
        .skip(1)
        .filter_map(|s| s.split('"').next())
        .find(|s| s.ends_with(".ngz.zfs.xz"));

    if let Some(link) = link {
        let image_url = format!("{OMNIOS_RELEASES_URL}{link}");
        tracing::debug!("using image {}", image_url);
        Ok(image_url)
    } else {
        bail!("could not find ngz.zfs.xz image");
    }
}

fn get_image(img_url: &str) -> anyhow::Result<Utf8PathBuf> {
    let chunks = img_url.split("/");

    if let Some(img_basename) = chunks.last() {
        let img_path = Utf8PathBuf::from(IMG_CACHE_DIR).join(img_basename);

        if img_path.exists() {
            tracing::debug!("found image at {img_path}");
        } else {
            tracing::debug!("no image at {img_path}: downloading");
            http::remote_file_to_disk(img_url, &img_path)?;
        }

        Ok(img_path)
    } else {
        bail!("could not get image basename");
    }
}
