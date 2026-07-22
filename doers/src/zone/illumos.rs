use crate::zone::helpers;
use crate::zone::types::ZoneImage;
use crate::zone::{config::ImageChecksum, constants::OMNIOS_RELEASES_URL};
use anyhow::{Context, bail};
use camino::Utf8PathBuf;
use url::Url;

// Images follow the form:
// https://downloads.omnios.org/media/stable/omnios-r151056.ngz.zfs.xz
// For now we will simply download the stable NGZ image

pub fn image_path(image: ZoneImage) -> anyhow::Result<Utf8PathBuf> {
    // if we're given an image and it looks like a URL, fetch it. If it looks like a file, stat
    // it; if we don't have one, fetch the current stable
    // if let Some() = image {
    match image.user_string {
        Some(user_string) => {
            if user_string.starts_with("/") {
                Ok(Utf8PathBuf::from(user_string))
            } else if user_string.starts_with("http") {
                helpers::get_image(user_string, image.checksum)
            } else {
                bail!("illumos image must be a fully qualified path or URL")
            }
        }
        None => helpers::get_image(
            &default_image()?,
            Some(&ImageChecksum {
                sumtype: "sha256".into(),
                value: ".sha256".into(),
            }),
        ),
    }
}

fn default_image() -> anyhow::Result<Url> {
    let release_str = OMNIOS_RELEASES_URL.as_str();

    tracing::debug!("fetching latest release images from {release_str}",);

    let html = ureq::get(OMNIOS_RELEASES_URL.as_str())
        .call()
        .with_context(|| format!("failed to fetch OmniOS image page from {release_str}"))?
        .into_body()
        .read_to_string()?;

    let link = html
        .split("href=\"")
        .skip(1)
        .filter_map(|s| s.split('"').next())
        .find(|s| s.ends_with(".ngz.zfs.xz"));

    if let Some(link) = link {
        let image_url = OMNIOS_RELEASES_URL
            .join(link)
            .with_context(|| format!("cannot create image link from {release_str} and {link}"))?;

        tracing::debug!("using image {}", image_url.as_str());
        Ok(image_url)
    } else {
        bail!("could not find ngz.zfs.xz image");
    }
}
