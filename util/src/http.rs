use camino::Utf8PathBuf;
use std::fs::File;
use std::io::copy;

pub fn download_file(url: &str, path: &Utf8PathBuf) -> anyhow::Result<()> {
    tracing::info!("downloading {url} -> {path}");

    let response = ureq::get(url).call()?;
    let mut reader = response.into_body().into_reader();

    let mut file = File::create(path)?;
    copy(&mut reader, &mut file)?;

    Ok(())
}
