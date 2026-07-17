use super::{atomic_write, hash};
use anyhow::{Context, bail, ensure};
use camino::Utf8Path;
use common::constants::{CLIENT_API_VERSION, CLIENT_RETRIES, SERVER_PORT};
use common::types::{ApplyOpts, CompileError, FileChecksum, NetworkError};
use std::fs::File;
use std::io::Write;
use std::io::{self, BufWriter, Seek, SeekFrom};
use std::thread;
use std::time::Duration;

pub struct RemoteFileCopy<'a> {
    pub url: &'a str,
    pub path: &'a Utf8Path,
    pub checksum: Option<&'a FileChecksum>,
    pub backup_suffix: Option<&'a str>,
}

// Downloads a file to disk
pub fn remote_file_to_disk(file: &RemoteFileCopy, opts: &ApplyOpts) -> anyhow::Result<()> {
    let response = match ureq::get(file.url).call() {
        Ok(resp) => resp,
        Err(ureq::Error::StatusCode(code)) => {
            return Err(NetworkError::Http(code).into());
        }
        Err(e) => {
            log_ureq_error(file.url, &e);
            return Err(NetworkError::Transport(e.to_string()).into());
        }
    };

    let mut body = response.into_body();
    let mut reader = body.as_reader();

    atomic_write::install(file.path, file.backup_suffix, opts, |f| {
        {
            let mut writer = BufWriter::new(&mut *f);
            io::copy(&mut reader, &mut writer).with_context(|| {
                format!("failed to copy content of {} to {}", file.url, file.path)
            })?;
            writer.flush()?;
        }

        if let Some(checksum) = &file.checksum {
            ensure!(
                has_good_checksum(f, checksum)?,
                "incorrect checksum for {}",
                file.url
            );
        }

        Ok(())
    })?;

    Ok(())
}

// Downloads a file to memory
pub fn remote_file_to_memory(url: &str) -> Result<Vec<u8>, NetworkError> {
    let mut response = match ureq::get(url).call() {
        Ok(resp) => resp,
        Err(ureq::Error::StatusCode(code)) => {
            return Err(NetworkError::Http(code));
        }
        Err(e) => {
            log_ureq_error(url, &e);
            return Err(NetworkError::Transport(e.to_string()));
        }
    };

    response
        .body_mut()
        .with_config()
        .limit(1000 * 1024 * 1024)
        .read_to_vec()
        .map_err(|e| NetworkError::Transport(e.to_string()))
}

pub fn remote_file_to_string(url: &str) -> anyhow::Result<String> {
    String::from_utf8(remote_file_to_memory(url).with_context(|| format!("failed to fetch {url}"))?)
        .context("failed to convert remote file to string")
}

pub fn config_from_server(
    server: &str,
    hostname: &str,
    format: &str,
) -> Result<Vec<u8>, CompileError> {
    let mut tries = 1;
    let mut err: Option<CompileError> = None;

    loop {
        tracing::debug!("try {tries}/{CLIENT_RETRIES}");

        match fetch_precompiled_file(server, hostname, format) {
            Ok(resp) => {
                return Ok(resp);
            }
            Err(e) => {
                tracing::error!("error calling remote server: {e}");
                if !e.is_retryable() {
                    tracing::error!("error is not retryable: bailing");
                    return Err(e);
                }
                if tries == CLIENT_RETRIES {
                    break;
                }
                tracing::info!("sleeping for retry");
                thread::sleep(Duration::from_secs(tries * tries));
                tries += 1;
                err = Some(e);
            }
        }
    }

    Err(err.unwrap())
}

fn fetch_precompiled_file(
    server: &str,
    hostname: &str,
    format: &str,
) -> Result<Vec<u8>, CompileError> {
    // We tell the server what we think it's called so it can build file resources we can find.
    // This lets us use a raw IP address, DNS name, whatever.
    let url = format!(
        "http://{server}:{SERVER_PORT}/{CLIENT_API_VERSION}/config/{hostname}?server_name={server}&format={format}"
    );
    tracing::info!("fetching config from {url}");
    remote_file_to_memory(&url).map_err(CompileError::Network)
}

fn log_ureq_error(url: &str, e: &ureq::Error) {
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
}

fn has_good_checksum(f: &mut File, expected: &FileChecksum) -> anyhow::Result<bool> {
    f.seek(SeekFrom::Start(0))?;

    let actual = match expected.algorithm.as_str() {
        "sha256" => hash::sha256_of_reader(&mut *f)?,
        "blake3" => hash::of_reader(&mut *f)?.to_string(),
        other => bail!("unsupported hash type: {other}"),
    };

    tracing::debug!(
        "comparing checksums: expected={} actual={}",
        expected.value.trim(),
        actual.trim()
    );

    Ok(expected.value.trim() == actual.trim())
}
