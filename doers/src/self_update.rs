use anyhow::Context;
use camino::Utf8PathBuf;
use common::constants::{ONE_RESOURCE_NO_CHANGE, ONE_RESOURCE_ONE_CHANGE};
use common::types::{ApplyOpts, ApplySummary};
use serde_json::Value;
use std::env;
use std::fs::File;
use url::Url;
use util::http::RemoteFileCopy;
use util::{atomic_write, file, http, info};

pub(crate) fn update_gurp(update_from: &str, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
    tracing::info!("updating Gurp from {update_from}");

    let gurp_path = env::current_exe().context("cannot get current Gurp path")?;
    let gurp_path = Utf8PathBuf::from_path_buf(gurp_path)
        .ok()
        .context("Gurp path is not valid UTF-8")?;

    let metadata = file::metadata(&gurp_path)?;

    if update_from == "SERVER" {
        let server = opts
            .client
            .server
            .as_deref()
            .context("requested server update, but server is not set")?;

        let my_hash = info::BUILD_HASH.to_string();

        let base_url =
            Url::parse(&format!("http://{server}:1867/v1")).context("cannot build server URL")?;

        if my_hash == server_hash(&base_url)? {
            tracing::debug!("gurp hash aligns with server hash: {my_hash}");
            return Ok(ONE_RESOURCE_NO_CHANGE);
        }

        http::remote_file_to_disk(
            &RemoteFileCopy {
                url: &base_url
                    .join("gurp-binary")
                    .context("cannot attach binary path to server URL")?,
                path: &gurp_path,
                backup_suffix: None,
                checksum: None,
            },
            opts,
        )?;
    } else {
        let src = Utf8PathBuf::from(update_from);
        atomic_write::install(&src, None, opts, |f| {
            let mut source =
                File::open(&src).with_context(|| format!("failed to read source file {src}"))?;

            std::io::copy(&mut source, f)
                .with_context(|| format!("failed copy {src} -> {gurp_path}"))?;
            Ok(())
        })?;
    };

    file::ensure_metadata(
        &gurp_path,
        file::FileMetadata {
            group: &file::NameOrId::Id(metadata.st_uid),
            mode: "0755",
            owner: &file::NameOrId::Id(metadata.st_uid),
        },
        opts,
    )?;

    Ok(ONE_RESOURCE_ONE_CHANGE)
}

fn server_hash(base_url: &Url) -> anyhow::Result<String> {
    let version_url = base_url
        .join("version")
        .context("cannot attach version path to server URL")?;

    let response: Value = ureq::get(version_url.as_str())
        .call()
        .with_context(|| format!("failed to fetch get Gurp version from {version_url}",))?
        .into_body()
        .read_json()
        .context("failed to parse Gurp version data")?;

    Ok(response["sha"].to_string())
}
