use anyhow::Context;
use common::constants::{DLADM_BIN, ONE_RESOURCE_NO_CHANGE};
use common::types::{ApplyOpts, ApplySummary};
use os_types::{GurpId, LinkName};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(test, derive(PartialEq))]
pub struct GurpEtherstubEnsure {
    #[serde(rename = "_id")]
    pub id: GurpId,
    pub name: LinkName,
}

#[derive(Deserialize, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct GurpEtherstubRemove {
    #[serde(rename = "_id")]
    pub id: GurpId,
    pub name: LinkName,
}

impl GurpEtherstubEnsure {
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        if etherstub_exists(&self.name)? {
            tracing::debug!("{} already exists", self.name);
            Ok(ONE_RESOURCE_NO_CHANGE)
        } else {
            tracing::info!("creating {}", self.name);
            cmd_change_or_noop!(opts, DLADM_BIN, "create-etherstub", &self.name)
        }
    }
}

impl GurpEtherstubRemove {
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        if etherstub_exists(&self.name)? {
            tracing::info!("Removing {}", self.name);
            cmd_change_or_noop!(opts, DLADM_BIN, "delete-etherstub", &self.name)
        } else {
            tracing::debug!("{} does not exist", self.name);
            Ok(ONE_RESOURCE_NO_CHANGE)
        }
    }
}

fn etherstub_exists(name: &LinkName) -> anyhow::Result<bool> {
    let dladm_output = cmd_output!(DLADM_BIN, "show-etherstub", "-p", "-o", "link")
        .with_context(|| format!("failed to test state of etherstub {name}"))?;

    Ok(dladm_output.lines().any(|l| l == name.to_string()))
}

#[cfg(test)]
mod test {
    use super::*;
    use tester::deserialized_example;

    #[test]
    fn test_ensure_etherstub_deserialize() {
        assert_eq!(
            GurpEtherstubEnsure {
                id: GurpId::new("/NO-ROLE/etherstub/newstub0").unwrap(),
                name: LinkName::new("newstub0").unwrap(),
            },
            deserialized_example("etherstub/ensure-stub.janet")
        );
    }

    #[test]
    fn test_remove_etherstub_deserialize() {
        assert_eq!(
            GurpEtherstubRemove {
                id: GurpId::new("/NO-ROLE/etherstub/oldstub0").unwrap(),
                name: LinkName::new("oldstub0").unwrap(),
            },
            deserialized_example("etherstub/remove-stub.janet")
        );
    }
}
