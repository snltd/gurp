use crate::prelude::*;
use nix::unistd::Group;
use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
pub struct GurpGroupEnsure {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
    pub gid: u32,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct GurpGroupRemove {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
}

impl GurpGroupEnsure {
    pub fn apply(&self, opts: &Opts) -> anyhow::Result<ApplySummary> {
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

    fn group_cmd(&self, command: &str, opts: &Opts) -> anyhow::Result<ApplySummary> {
        let mut cmd = cmd!(command, "-g", &self.gid.to_string(), &self.name);

        return_if_noop!(opts);

        one_change_or_stderr!(cmd, format!("group error {}", self.name))
    }
}

impl GurpGroupRemove {
    pub fn apply(&self, opts: &Opts) -> anyhow::Result<ApplySummary> {
        if Group::from_name(&self.name)?.is_some() {
            if PROTECTED_GROUPS.contains(&self.name.as_str()) {
                tracing::warn!("protected resource: {}", self.name);
                return Ok(ONE_RESOURCE_ONE_ERROR);
            }

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
    use crate::test_utils::spec_helper::janet2json;

    #[test]
    fn test_ensure() {
        let json_def = janet2json(r#"(group/ensure "test-group" :gid 264)"#);

        let expected = GurpGroupEnsure {
            name: "test-group".to_owned(),
            id: "/NO-ROLE/group/test-group".to_owned(),
            gid: 264,
        };

        assert_eq!(expected, serde_json::from_str(&json_def).unwrap());
    }
}
