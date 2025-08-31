use crate::users_and_groups;
use anyhow::bail;
use camino::Utf8PathBuf;
use common::types::{ApplyOpts, ApplySummary, FileMetadata};
use nix::sys::stat::{self, FileStat};
use nix::unistd::{Gid, Uid};
use std::fs;
use std::os::unix::fs::PermissionsExt;

pub fn ensure_metadata(spec: FileMetadata, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
    let metadata = metadata(spec.path)?;
    let new_uid = new_uid(spec.owner, &metadata)?;
    let new_gid = new_gid(spec.group, &metadata)?;
    let mut changes = spec.changes;

    if new_uid.is_some() || new_gid.is_some() {
        changes = 1;
        set_user(spec.path, new_uid, new_gid, opts)?;
    }

    let current_mode = format!("{:04o}", metadata.st_mode & 0o7777);

    if current_mode != spec.mode {
        changes = 1;
        set_mode(spec.path, &current_mode, spec.mode, opts)?;
    }

    Ok(ApplySummary {
        resources: 1,
        changes,
    })
}

fn metadata(path: &Utf8PathBuf) -> anyhow::Result<FileStat> {
    let metadata = stat::stat(path.as_std_path())?;
    Ok(metadata)
}

fn new_uid(desired_owner: &str, metadata: &FileStat) -> anyhow::Result<Option<Uid>> {
    let current_uid: Uid = metadata.st_uid.into();
    let desired_uid = users_and_groups::owner_from(desired_owner)?;

    if current_uid == desired_uid {
        tracing::debug!("owner is correct");
        Ok(None)
    } else {
        Ok(Some(desired_uid))
    }
}

fn new_gid(desired_group: &str, metadata: &FileStat) -> anyhow::Result<Option<Gid>> {
    let current_gid: Gid = metadata.st_gid.into();
    let desired_gid = users_and_groups::group_from(desired_group)?;

    if current_gid == desired_gid {
        tracing::debug!("owner is correct");
        Ok(None)
    } else {
        Ok(Some(desired_gid))
    }
}

fn set_user(
    path: &Utf8PathBuf,
    uid: Option<Uid>,
    gid: Option<Gid>,
    opts: &ApplyOpts,
) -> anyhow::Result<()> {
    if let Some(uid) = uid {
        if let Some(gid) = gid {
            tracing::info!("Setting user: group to {}:{}", uid, gid);
        } else {
            tracing::info!("Setting user to {}", uid);
        }
    } else if let Some(gid) = gid {
        tracing::info!("Setting group to {}", gid);
    } else {
        bail!("UID and GID both empty");
    }

    if opts.noop {
        Ok(())
    } else {
        Ok(nix::unistd::chown(path.as_std_path(), uid, gid)?)
    }
}

fn set_mode(
    path: &Utf8PathBuf,
    current_mode: &str,
    desired_mode: &str,
    opts: &ApplyOpts,
) -> anyhow::Result<()> {
    tracing::info!("Changing mode {} -> {}", current_mode, desired_mode);
    let mode = u32::from_str_radix(desired_mode, 8)?;

    if opts.noop {
        Ok(())
    } else {
        Ok(fs::set_permissions(path, fs::Permissions::from_mode(mode))?)
    }
}
