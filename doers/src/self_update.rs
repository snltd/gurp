use anyhow::Context;
use camino::Utf8PathBuf;
use common::constants::{ONE_RESOURCE_NO_CHANGE, ONE_RESOURCE_ONE_CHANGE};
use common::types::{ApplyOpts, ApplySummary};
use os_types::FileMode;
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

        let hash_url = &base_url
            .join("gurp-binary-hash")
            .context("cannot attach binary hash to server URL")?;

        let binary_url = &base_url
            .join("gurp-binary-hash")
            .context("cannot attach binary path to server URL")?;

        let server_hash = http::url_to_string(hash_url)?;

        if server_hash == gurp_hash.to_string() {
            tracing::debug!("no need to update gurp from server {gurp_hash} == {server_hash}");
            return Ok(ONE_RESOURCE_NO_CHANGE);
        }

        tracing::info!("updating Gurp from server sh {server_hash} gh {gurp_hash}");

        http::url_to_disk(
            &RemoteFileCopy {
                url: binary_url,
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
