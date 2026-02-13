use crate::file::NameOrId;
use anyhow::Context;
use nix::unistd::{Gid, Group, Uid, User};

pub fn owner_from(owner: &NameOrId) -> anyhow::Result<Uid> {
    Ok(match owner {
        NameOrId::Id(val) => Uid::from_raw(*val),
        NameOrId::Name(val) => {
            User::from_name(val)?
                .context(format!("No such user '{val}'"))?
                .uid
        }
    })
}

pub fn group_from(group: &NameOrId) -> anyhow::Result<Gid> {
    Ok(match group {
        NameOrId::Id(val) => Gid::from_raw(*val),
        NameOrId::Name(val) => {
            Group::from_name(val)?
                .context(format!("No such group '{val}'"))?
                .gid
        }
    })
}
