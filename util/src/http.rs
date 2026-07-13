use super::atomic_write;
use anyhow::Context;
use camino::Utf8Path;
use common::constants::{CLIENT_API_VERSION, CLIENT_RETRIES, SERVER_PORT};
use common::types::{ApplyOpts, CompileError, NetworkError};
use std::io::{self, BufWriter};
use std::thread;
use std::time::Duration;

// Downloads a file to disk
pub fn remote_file_to_disk(
    url: &str,
    path: &Utf8Path,
    backup_suffix: Option<&str>,
    opts: &ApplyOpts,
) -> anyhow::Result<()> {
    let response = match ureq::get(url).call() {
        Ok(resp) => resp,
        Err(ureq::Error::StatusCode(code)) => {
            return Err(NetworkError::Http(code).into());
        }
        Err(e) => {
            log_ureq_error(url, &e);
            return Err(NetworkError::Transport(e.to_string()).into());
        }
    };

    let mut body = response.into_body();
    let mut reader = body.as_reader();

    atomic_write::install(path, backup_suffix, opts, |f| {
        let mut writer = BufWriter::new(f);
        io::copy(&mut reader, &mut writer)
            .with_context(|| format!("failed to copy content of {url} to {path}"))?;
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
