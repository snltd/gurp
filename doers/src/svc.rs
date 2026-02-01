use common::constants::{ONE_RESOURCE_NO_CHANGE, ONE_RESOURCE_ONE_CHANGE, SVCADM_BIN};
use common::types::{ApplyOpts, ApplySummary, ChangedIds};
use serde::Deserialize;
use std::collections::BTreeSet;
use util::svcs;

// THINGS TO KNOW / THINGS TO DO.
// There's no svc/remove, only svc/ensure

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct GurpSvcEnsure {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
    #[serde(rename = "state")]
    pub desired_state: String,
    #[serde(rename = "restarted-by")]
    pub restarters: BTreeSet<String>,
    #[serde(rename = "reloaded-by")]
    pub reloaders: BTreeSet<String>,
}

impl GurpSvcEnsure {
    pub fn apply(
        &self,
        changed_ids: &ChangedIds,
        opts: &ApplyOpts,
    ) -> anyhow::Result<ApplySummary> {
        let current_state = svcs::current_state(&self.name)?;

        if current_state == self.desired_state {
            if changed_ids.is_empty() {
                tracing::debug!("no changed resources, so no {} svc trigger", &self.name);
                Ok(ONE_RESOURCE_NO_CHANGE)
            } else {
                tracing::debug!(
                    "changed resources: {}",
                    changed_ids
                        .iter()
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                );

                if !changed_ids.is_disjoint(&self.restarters) {
                    tracing::info!("{}: restarting service", self.name);
                    let mut cmd = cmd!(SVCADM_BIN, "restart", &self.name);
                    return_if_noop!(opts);
                    one_change_or_stderr!(cmd, format!("error restarting svc '{}'", self.name))
                } else if !changed_ids.is_disjoint(&self.reloaders) {
                    tracing::info!("{}: reloading service", self.name);
                    let mut cmd = cmd!(SVCADM_BIN, "reload", &self.name);
                    return_if_noop!(opts);
                    one_change_or_stderr!(cmd, format!("error reloading svc '{}'", self.name))
                } else {
                    tracing::debug!("{}: no service trigger", self.name);
                    Ok(ONE_RESOURCE_NO_CHANGE)
                }
            }
        } else {
            tracing::info!(
                "change {} state: {} -> {}",
                self.name,
                current_state,
                self.desired_state
            );

            return_if_noop!(opts);

            svcs::set_state(&self.name, &current_state, &self.desired_state)?;
            Ok(ONE_RESOURCE_ONE_CHANGE)
        }
    }
}
