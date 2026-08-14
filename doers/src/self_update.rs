use anyhow::Context;
use camino::Utf8PathBuf;
use common::constants::{ONE_RESOURCE_NO_CHANGE, ONE_RESOURCE_ONE_CHANGE};
use common::types::{ApplyOpts, ApplySummary};
use os_types::FileMode;
use serde_json::Value;
use std::fs::File;
use url::Url;
use util::http::RemoteFileCopy;
use util::{atomic_write, file, hash, http, info};

pub(crate) fn update_gurp(update_from: &str, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
    let gurp_path = info::gurp_path()?;
    let gurp_hash = hash::of_file(&gurp_path)?;
    let metadata = file::metadata(&gurp_path)?;

    if update_from == "SERVER" {
        let server = opts
            .client
            .server
            .as_deref()
            .context("requested server update, but server is not set")?;

        let base_url =
            Url::parse(&format!("http://{server}:1867/v1/")).context("cannot build server URL")?;

        let my_hash = info::BUILD_HASH.to_string();
        let server_hash = server_hash(&base_url)?;

        if server_hash == my_hash {
            tracing::debug!("no need to update gurp from server");
            return Ok(ONE_RESOURCE_NO_CHANGE);
        }

        tracing::info!("updating Gurp from server sh {server_hash} mh {my_hash}");

        http::url_to_disk(
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

        if hash::of_file(&src)? == gurp_hash {
            tracing::debug!("no need to update gurp from {src}");
        } else {
            tracing::info!("updating Gurp from {update_from}");
            atomic_write::install(&gurp_path, None, opts, |f| {
                let mut source = File::open(&src)
                    .with_context(|| format!("failed to read source file {src}"))?;

                std::io::copy(&mut source, f)
                    .with_context(|| format!("failed copy {src} -> {gurp_path}"))?;
                Ok(())
            })?;
        }
    };

    file::ensure_metadata(
        &gurp_path,
        file::FileMetadata {
            group: &file::NameOrId::Id(metadata.st_uid),
            mode: &FileMode::new("0755").unwrap(),
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

    Ok(response["sha"]
        .as_str()
        .context("sha field missing or not a string")?
        .to_string())
}
