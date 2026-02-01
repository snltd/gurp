use common::constants::{DLADM_BIN, ONE_RESOURCE_NO_CHANGE, ONE_RESOURCE_ONE_CHANGE};
use common::types::{ApplyOpts, ApplySummary};
use serde::Deserialize;

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct GurpEtherstubEnsure {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct GurpEtherstubRemove {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
}

fn etherstub_exists(etherstub_name: &str) -> anyhow::Result<bool> {
    let dladm_output = cmd_output!(DLADM_BIN, "show-etherstub", "-p", "-o", "link")?;
    Ok(dladm_output.lines().any(|l| l == etherstub_name))
}

impl GurpEtherstubEnsure {
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        if etherstub_exists(&self.name)? {
            tracing::debug!("{} already exists", self.name);
            Ok(ONE_RESOURCE_NO_CHANGE)
        } else {
            tracing::info!("creating {}", self.name);
            return_if_noop!(opts);

            cmd_output!(DLADM_BIN, "create-etherstub", &self.name)?;
            Ok(ONE_RESOURCE_ONE_CHANGE)
        }
    }
}

impl GurpEtherstubRemove {
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        if etherstub_exists(&self.name)? {
            tracing::info!("Removing {}", self.name);
            return_if_noop!(opts);

            cmd_output!(DLADM_BIN, "delete-etherstub", &self.name)?;
            Ok(ONE_RESOURCE_ONE_CHANGE)
        } else {
            tracing::debug!("{} does not exist", self.name);
            Ok(ONE_RESOURCE_NO_CHANGE)
        }
    }
}
