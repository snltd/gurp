use crate::common::svcs;
use crate::common::types::ChangedIds;
use crate::prelude::*;
use serde::Deserialize;
use std::collections::HashSet;

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
    pub restarters: HashSet<String>,
    #[serde(rename = "reloaded-by")]
    pub reloaders: HashSet<String>,
}

impl GurpSvcEnsure {
    pub fn apply(&self, changed_ids: &ChangedIds, opts: &Opts) -> anyhow::Result<ApplySummary> {
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
                    if opts.noop {
                        Ok(ONE_RESOURCE_NOOP)
                    } else {
                        svcs::run_svcadm(&self.name, "restart")?;
                        Ok(ONE_RESOURCE_ONE_CHANGE)
                    }
                } else if !changed_ids.is_disjoint(&self.reloaders) {
                    tracing::info!("{}: reloading service", self.name);
                    if opts.noop {
                        Ok(ONE_RESOURCE_NOOP)
                    } else {
                        svcs::run_svcadm(&self.name, "reload")?;
                        Ok(ONE_RESOURCE_ONE_CHANGE)
                    }
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

            if opts.noop {
                Ok(ONE_RESOURCE_NOOP)
            } else {
                svcs::set_state(&self.name, &current_state, &self.desired_state)?;
                Ok(ONE_RESOURCE_ONE_CHANGE)
            }
        }
    }
}
