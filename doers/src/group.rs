use anyhow::ensure;
use common::constants::{
    GROUPADD_BIN, GROUPDEL_BIN, GROUPMOD_BIN, ONE_RESOURCE_NO_CHANGE, PROTECTED_GROUPS,
};
use common::types::{ApplyOpts, ApplySummary};
use nix::unistd::Group;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct GurpGroupEnsure {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
    pub gid: u32,
}

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct GurpGroupRemove {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
}

impl GurpGroupEnsure {
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        if let Some(group) = Group::from_name(&self.name)? {
            if group.gid.as_raw() == self.gid {
                tracing::debug!("group {} is gid {}", self.name, self.gid);
                Ok(ONE_RESOURCE_NO_CHANGE)
            } else {
                tracing::info!("changing {} GID {} -> {}", self.name, group.gid, self.gid);
                self.group_cmd(GROUPMOD_BIN, opts)
            }
        } else {
            tracing::info!("creating group: {}", self.name);
            self.group_cmd(GROUPADD_BIN, opts)
        }
    }

    fn group_cmd(&self, command: &str, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        let mut cmd = cmd!(command, "-g", &self.gid.to_string(), &self.name);
        return_if_noop!(opts);

        one_change_or_stderr!(cmd, format!("group error {}", self.name))
    }
}

impl GurpGroupRemove {
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        if Group::from_name(&self.name)?.is_some() {
            ensure!(
                !PROTECTED_GROUPS.contains(&self.name.as_str()),
                format!("protected resource: {}", self.name)
            );

            tracing::info!("removing group: {}", self.name);

            let mut cmd = cmd!(GROUPDEL_BIN, &self.name);

            return_if_noop!(opts);

            one_change_or_stderr!(cmd, format!("error deleting group {}", self.name))
        } else {
            tracing::debug!("not present: {}", self.name);
            Ok(ONE_RESOURCE_NO_CHANGE)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use tester::deserialized_example;

    #[test]
    fn test_group_deserialize_ensure_01() {
        assert_eq!(
            GurpGroupEnsure {
                name: "new-group".to_owned(),
                id: "/NO-ROLE/group/new-group".to_owned(),
                gid: 264,
            },
            deserialized_example::<GurpGroupEnsure>("group/ensure-01.janet")
        );
    }

    #[test]
    fn test_group_deserialize_remove_01() {
        assert_eq!(
            GurpGroupRemove {
                name: "old-group".to_owned(),
                id: "/NO-ROLE/group/old-group".to_owned(),
            },
            deserialized_example::<GurpGroupRemove>("group/remove-01.janet")
        );
    }
}
