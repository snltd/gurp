use crate::file::{self, FileMetadata, NameOrId};
use anyhow::Context;
use camino::Utf8Path;
use camino_tempfile::Builder;
use common::types::ApplyOpts;
use std::fs::{self, File};
use std::time::{SystemTime, UNIX_EPOCH};

/// Everything that writes a file ends up here. It handles noops, backups, and creates the
/// desired content as a temp file before persisting it to the required path. Used by the
/// file doers, and super::http::remote_file_to_disk
pub fn install(
    path: &Utf8Path,
    backup_suffix: Option<&str>,
    opts: &ApplyOpts,
    write_fn: impl FnOnce(&mut File) -> anyhow::Result<()>,
) -> anyhow::Result<bool> {
    if opts.noop {
        return Ok(true);
    }

    let parent = path
        .parent()
        .with_context(|| format!("cannot determine parent directory of {path}"))?;

    let mut tmp = Builder::new()
        .prefix(".gurp-tmp-")
        .tempfile_in(parent)
        .with_context(|| format!("cannot create temp file in {parent}"))?;

    write_fn(tmp.as_file_mut())?;

    tmp.as_file()
        .sync_all()
        .with_context(|| format!("cannot sync {path}"))?;

    if let Some(suffix) = backup_suffix {
        back_up(path, suffix, opts).with_context(|| format!("cannot back up {path}"))?;
    }

    tmp.persist(path)
        .with_context(|| format!("cannot install {path}"))?;

    Ok(true)
}

pub fn back_up(path: &Utf8Path, suffix: &str, opts: &ApplyOpts) -> anyhow::Result<()> {
    let suffix = if suffix == "TIMESTAMP" {
        epoch_time_as_string()
    } else {
        suffix.to_owned()
    };

    let backup_target = path.with_extension(suffix);
    tracing::debug!("Backing up to {}", backup_target);

    if !opts.noop {
        fs::rename(path, &backup_target)
            .with_context(|| format!("failed to rename {path} to {backup_target}"))?;
        file::ensure_metadata(
            &backup_target,
            FileMetadata {
                group: &NameOrId::Name("root".to_owned()),
                owner: &NameOrId::Name("root".to_owned()),
                mode: "0400",
            },
            opts,
        )?;
    }

    Ok(())
}

fn epoch_time_as_string() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("cannot get epoch time")
        .as_secs()
        .to_string()
}
