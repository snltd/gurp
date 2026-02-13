use crate::file::UserOrGroup;
use anyhow::Context;
use nix::unistd::{Gid, Group, Uid, User};

pub fn owner_from(owner: &UserOrGroup) -> anyhow::Result<Uid> {
    Ok(match owner {
        UserOrGroup::Id(val) => Uid::from_raw(*val),
        UserOrGroup::Name(val) => {
            User::from_name(val)?
                .context(format!("No such user '{val}'"))?
                .uid
        }
    })
}

pub fn group_from(group: &UserOrGroup) -> anyhow::Result<Gid> {
    Ok(match group {
        UserOrGroup::Id(val) => Gid::from_raw(*val),
        UserOrGroup::Name(val) => {
            Group::from_name(val)?
                .context(format!("No such group '{val}'"))?
                .gid
        }
    })
}
