use common::constants::{ONE_RESOURCE_NO_CHANGE, ONE_RESOURCE_ONE_CHANGE, SVCADM_BIN};
use common::types::{ApplyOpts, ApplySummary, ChangedIds};
use serde::Deserialize;
use std::collections::BTreeSet;
use util::svcs;

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
                    cmd_change_or_noop!(opts, SVCADM_BIN, "restart", &self.name)
                } else if !changed_ids.is_disjoint(&self.reloaders) {
                    tracing::info!("{}: reloading service", self.name);
                    cmd_change_or_noop!(opts, SVCADM_BIN, "reload", &self.name)
                } else {
                    tracing::debug!("{}: no service trigger", self.name);
                    Ok(ONE_RESOURCE_NO_CHANGE)
                }
            }
        } else {
            svcs::set_state(&self.name, &current_state, &self.desired_state, opts)?;

            Ok(ONE_RESOURCE_ONE_CHANGE)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::collections::BTreeSet;
    use tester::deserialized_example;

    #[test]
    fn test_deserialize_ensure_svc_with_restarter() {
        assert_eq!(
            GurpSvcEnsure {
                id: "/NO-ROLE/svc/important_service".to_owned(),
                name: "important/service".to_owned(),
                desired_state: "enabled".to_owned(),
                restarters: BTreeSet::from(["/test-role/file/stub".to_owned()]),
                reloaders: BTreeSet::new(),
            },
            deserialized_example("svc/ensure-svc-with-restarter.janet")
        );
    }
}
