use anyhow::Context;
use nix::unistd::{Gid, Group, Uid, User};

pub fn owner_from(desired_owner: &str) -> anyhow::Result<Uid> {
    Ok(match desired_owner.parse::<u32>() {
        Ok(val) => val.into(),
        Err(_) => {
            User::from_name(desired_owner)?
                .context(format!("No such user '{desired_owner}'"))?
                .uid
        }
    })
}

pub fn group_from(desired_group: &str) -> anyhow::Result<Gid> {
    Ok(match desired_group.parse::<u32>() {
        Ok(val) => val.into(),
        Err(_) => {
            Group::from_name(desired_group)?
                .context(format!("No such group '{desired_group}'"))?
                .gid
        }
    })
}
