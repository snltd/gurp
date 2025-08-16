use anyhow::Context;
use camino::Utf8PathBuf;
use common::constants::IMG_CACHE;
use std::fs::File;
use std::io::copy;

fn download_to_file(url: &str, path: &Utf8PathBuf) -> anyhow::Result<()> {
    tracing::info!("downloading {url} -> {path}");
    let response = ureq::get(url).call()?;
    let mut reader = response.into_reader();

    let mut file = File::create(path)?;
    copy(&mut reader, &mut file)?;

    Ok(())
}

pub fn image_in_cache(img_url: &str) -> anyhow::Result<Utf8PathBuf> {
    let img_basename = img_url
        .split('/')
        .last()
        .context("cannot parse image URI")?;

    let img_path = Utf8PathBuf::from(IMG_CACHE).join(img_basename);

    if img_path.exists() {
        tracing::debug!("found cached image {img_path}");
    } else {
        tracing::debug!("no image at {img_path}: downloading");
        download_to_file(img_url, &img_path)?;
    }

    Ok(img_path)
}
