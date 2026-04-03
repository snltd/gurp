use common::constants::{DLADM_BIN, ONE_RESOURCE_NO_CHANGE};
use common::types::{ApplyOpts, ApplySummary};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(test, derive(PartialEq))]
pub struct GurpEtherstubEnsure {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
}

#[derive(Deserialize, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct GurpEtherstubRemove {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
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

fn etherstub_exists(etherstub_name: &str) -> anyhow::Result<bool> {
    let dladm_output = cmd_output!(DLADM_BIN, "show-etherstub", "-p", "-o", "link")?;
    Ok(dladm_output.lines().any(|l| l == etherstub_name))
}

#[cfg(test)]
mod test {
    use super::*;
    use tester::deserialized_example;

    #[test]
    fn test_ensure_etherstub_deserialize() {
        assert_eq!(
            GurpEtherstubEnsure {
                id: "/NO-ROLE/etherstub/newstub0".to_owned(),
                name: "newstub0".to_owned(),
            },
            deserialized_example("etherstub/ensure-stub.janet")
        );
    }

    #[test]
    fn test_remove_etherstub_deserialize() {
        assert_eq!(
            GurpEtherstubRemove {
                id: "/NO-ROLE/etherstub/oldstub0".to_owned(),
                name: "oldstub0".to_owned(),
            },
            deserialized_example("etherstub/remove-stub.janet")
        );
    }
}
