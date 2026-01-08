use anyhow::Context;
use camino::Utf8PathBuf;
use std::fs::{self};

// Downloads a file to disk
pub fn remote_file_to_disk(url: &str, path: &Utf8PathBuf) -> anyhow::Result<()> {
    tracing::info!("download {url} -> {path}");
    let response = remote_file_to_memory(url)?;
    fs::write(path, response)?;
    Ok(())
}

// Downloads a file to memory
pub fn remote_file_to_memory(url: &str) -> anyhow::Result<Vec<u8>> {
    tracing::debug!("requesting {url}");

    let mut response = match ureq::get(url).call() {
        Ok(resp) => resp,
        Err(e) => {
            match &e {
                ureq::Error::StatusCode(code) => {
                    tracing::error!("got {} code from server for {}", code, url)
                }
                ureq::Error::Io(err) => {
                    tracing::error!("I/O error: {} on {}", err, url)
                }
                ureq::Error::HostNotFound => {
                    tracing::error!("Host not found: {}", url)
                }
                ureq::Error::Http(err) => {
                    tracing::error!("HTTP error: {} on {}", err, url)
                }
                ureq::Error::BadUri(err) => {
                    tracing::error!("Bad URI: {} on {}", err, url)
                }
                ureq::Error::BodyExceedsLimit(size) => {
                    tracing::error!("file send back is too big: limit {}b for {}", size, url)
                }
                _ => tracing::error!("unhandled error: {} on {}", e, url),
            }
            return Err(e).context(format!("failed to fetch file '{}'", &url));
        }
    };

    let ret = response
        .body_mut()
        .with_config()
        .limit(1000 * 1024 * 1024)
        .read_to_vec()?;

    Ok(ret)
}
