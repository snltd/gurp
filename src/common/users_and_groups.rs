use anyhow::Context;
use camino::Utf8PathBuf;
use nix::unistd::{Gid, Group, Uid, User};
use std::fs;
use std::os::unix::fs::PermissionsExt;

pub fn owner_from(desired_owner: &str) -> anyhow::Result<Uid> {
    Ok(match desired_owner.parse::<u32>() {
        Ok(val) => val.into(),
        Err(_) => {
            User::from_name(desired_owner)?
                .context(format!("No such user'{}'", desired_owner))?
                .uid
        }
    })
}

pub fn group_from(desired_group: &str) -> anyhow::Result<Gid> {
    Ok(match desired_group.parse::<u32>() {
        Ok(val) => val.into(),
        Err(_) => {
            Group::from_name(desired_group)?
                .context(format!("No such group'{}'", desired_group))?
                .gid
        }
    })
}

pub fn set_user(path: &Utf8PathBuf, uid: Uid, gid: Gid) -> anyhow::Result<()> {
    Ok(nix::unistd::chown(
        path.as_std_path(),
        Some(uid),
        Some(gid),
    )?)
}

pub fn set_mode(path: &Utf8PathBuf, _current_mode: &str, desired_mode: &str) -> anyhow::Result<()> {
    let mode = u32::from_str_radix(desired_mode, 8)?;
    Ok(fs::set_permissions(path, fs::Permissions::from_mode(mode))?)
}
