use anyhow::Context;
use common::constants::{ONE_RESOURCE_NO_CHANGE, ONE_RESOURCE_ONE_CHANGE, SVCADM_BIN};
use common::types::{ApplyOpts, ApplySummary, ChangedIds};
use os_types::GurpId;
use serde::Deserialize;
use std::collections::BTreeSet;
use util::svcs;

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct SvcEnsure {
    #[serde(rename = "_id")]
    pub id: GurpId,
    pub name: String,
    #[serde(rename = "state")]
    pub desired_state: String,
    #[serde(rename = "restarted-by")]
    pub restarters: BTreeSet<GurpId>,
    #[serde(rename = "reloaded-by")]
    pub reloaders: BTreeSet<GurpId>,
}

impl SvcEnsure {
    pub fn apply(
        &self,
        changed_ids: &ChangedIds,
        opts: &ApplyOpts,
    ) -> anyhow::Result<ApplySummary> {
        let svc = &self.name;
        let current_state = svcs::current_state(svc)?;

        if current_state == self.desired_state {
            if changed_ids.is_empty() {
                tracing::debug!("no changed resources, so no {svc} svc trigger");
                Ok(ONE_RESOURCE_NO_CHANGE)
            } else {
                tracing::debug!(
                    "changed resources: {}",
                    changed_ids
                        .iter()
                        .map(|s| s.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                );

                if !changed_ids.is_disjoint(&self.restarters) {
                    tracing::info!("{svc}: restarting service");
                    cmd_change_or_noop!(opts, SVCADM_BIN, "restart", svc)
                        .with_context(|| format!("failed to restart {svc}"))
                } else if !changed_ids.is_disjoint(&self.reloaders) {
                    tracing::info!("{}: reloading service", self.name);
                    cmd_change_or_noop!(opts, SVCADM_BIN, "reload", svc)
                        .with_context(|| format!("failed to reload {svc}"))
                } else {
                    tracing::debug!("{svc}: no service trigger");
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
            SvcEnsure {
                id: GurpId::new("/NO-ROLE/svc/important_service").unwrap(),
                name: "important/service".to_owned(),
                desired_state: "enabled".to_owned(),
                restarters: BTreeSet::from([GurpId::new("/test-role/file/stub").unwrap()]),
                reloaders: BTreeSet::new(),
            },
            deserialized_example("svc/ensure-svc-with-restarter.janet")
        );
    }
}
