use crate::users_and_groups;
use anyhow::{Context, bail};
use camino::{Utf8Path, Utf8PathBuf};
use common::types::ApplyOpts;
use nix::sys::stat::{self, FileStat};
use nix::unistd::{Gid, Uid};
use os_types::FileMode;
use serde::Deserialize;
use std::os::unix::fs::PermissionsExt;
use std::{env, fs};

#[derive(Debug)]
pub struct FileMetadata<'a> {
    pub group: &'a NameOrId,
    pub mode: &'a FileMode,
    pub owner: &'a NameOrId,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum NameOrId {
    Name(String),
    Id(u32),
}

impl Default for NameOrId {
    fn default() -> Self {
        Self::Id(0)
    }
}

impl std::fmt::Display for NameOrId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NameOrId::Name(s) => write!(f, "{}", s),
            NameOrId::Id(id) => write!(f, "{}", id),
        }
    }
}

pub fn ensure_metadata(
    path: &Utf8Path,
    md: FileMetadata,
    opts: &ApplyOpts,
) -> anyhow::Result<bool> {
    // If it's a create noop, the object will not exist
    if !path.exists() {
        return Ok(false);
    }

    let metadata = metadata(path)?;
    let new_uid = new_uid(md.owner, &metadata)?;
    let new_gid = new_gid(md.group, &metadata)?;
    let mut changed = false;

    if new_uid.is_some() {
        changed = true;
    }

    if new_gid.is_some() {
        changed = true;
    }

    if changed && !opts.noop {
        set_user(path, new_uid, new_gid, opts)?;
    }

    // We don't care about file type because we're trying to be generic, so mask off the file
    // type bit.
    let current_mode = FileMode::from_u32(metadata.st_mode as u32 & 0o7777);

    if current_mode != *md.mode {
        println!("CURRENT MODE {} : DESIRED MODE {}", current_mode, md.mode);
        changed = true;

        if !opts.noop {
            set_mode(path, &current_mode, md.mode, opts)
                .with_context(|| format!("failed to set mode {} on {path}", md.mode))?;
        }
    }

    Ok(changed)
}

pub fn metadata(path: &Utf8Path) -> anyhow::Result<FileStat> {
    let metadata = stat::stat(path.as_std_path())
        .with_context(|| format!("failed to get metadata for {path}"))?;

    Ok(metadata)
}

fn new_uid(desired_owner: &NameOrId, metadata: &FileStat) -> anyhow::Result<Option<Uid>> {
    let current_uid: Uid = metadata.st_uid.into();
    let desired_uid = users_and_groups::owner_from(desired_owner)?;

    if current_uid == desired_uid {
        tracing::debug!("owner is correct");
        Ok(None)
    } else {
        Ok(Some(desired_uid))
    }
}

fn new_gid(desired_group: &NameOrId, metadata: &FileStat) -> anyhow::Result<Option<Gid>> {
    let current_gid: Gid = metadata.st_gid.into();
    let desired_gid = users_and_groups::group_from(desired_group)
        .with_context(|| format!("failed to get group of {desired_group}"))?;

    if current_gid == desired_gid {
        tracing::debug!("group is correct");
        Ok(None)
    } else {
        Ok(Some(desired_gid))
    }
}

fn set_user(
    path: &Utf8Path,
    uid: Option<Uid>,
    gid: Option<Gid>,
    opts: &ApplyOpts,
) -> anyhow::Result<()> {
    if let Some(uid) = uid {
        if let Some(gid) = gid {
            tracing::info!("{path}: setting user: group to {}:{}", uid, gid);
        } else {
            tracing::info!("{path}: setting user to {}", uid);
        }
    } else if let Some(gid) = gid {
        tracing::info!("{path}: setting group to {}", gid);
    } else {
        bail!("UID and GID both empty");
    }

    if !opts.noop {
        nix::unistd::chown(path.as_std_path(), uid, gid)
            .with_context(|| format!("failed to change owner of {}", path))?;
    }

    Ok(())
}

fn set_mode(
    path: &Utf8Path,
    current_mode: &FileMode,
    desired_mode: &FileMode,
    opts: &ApplyOpts,
) -> anyhow::Result<()> {
    println!("{path}: changing mode {current_mode} -> {desired_mode}",);

    tracing::info!("{path}: changing mode {current_mode} -> {desired_mode}");

    if !opts.noop {
        fs::set_permissions(path, fs::Permissions::from_mode(desired_mode.as_u32()))
            .with_context(|| format!("failed to set permissions on {path}"))?;
    }

    Ok(())
}

pub fn current_dir() -> anyhow::Result<Utf8PathBuf> {
    let cwd = env::current_dir().context("cannot get current dir from env")?;
    Utf8PathBuf::try_from(cwd).map_err(|e| e.into())
}
