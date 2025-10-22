use camino::Utf8PathBuf;
use std::fs::File;
use std::io::{Read, copy};

// Downloads a file to disk
pub fn download_file(url: &str, path: &Utf8PathBuf) -> anyhow::Result<()> {
    tracing::info!("downloading {url} -> {path}");

    let response = ureq::get(url).call()?;
    let mut reader = response.into_body().into_reader();

    let mut file = File::create(path)?;
    copy(&mut reader, &mut file)?;

    Ok(())
}

// Downloads a file to memory
pub fn pull_file(url: &str) -> anyhow::Result<String> {
    tracing::info!("pulling {url}");

    let response = ureq::get(url).call()?;
    let mut reader = response.into_body().into_reader();

    let mut buf = String::new();
    reader.read_to_string(&mut buf)?;
    Ok(buf)
}
