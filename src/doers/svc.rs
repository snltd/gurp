use crate::common::constants::{
    ONE_RESOURCE_NO_CHANGE, ONE_RESOURCE_NOOP, ONE_RESOURCE_ONE_CHANGE,
};
use crate::common::svcs;
use crate::common::traits::Apply;
use crate::common::types::{Action, ApplyContext, ApplySummary, Opts, Resource};
use crate::utils::janet_helpers::{self, JanetExt, JanetStructExt};
use anyhow::bail;
use janetrs::{Janet, JanetArray};
use paste::paste;
use std::collections::HashSet;

// THINGS TO KNOW / THINGS TO DO.
// There's no svc/remove, only svc/ensure.
//
#[derive(Debug, PartialEq, Eq)]
pub struct GurpSvc {
    pub action: Action,
    pub id: String,
    pub name: String,
    pub desired_state: String,
    pub restarters: HashSet<String>,
    pub reloaders: HashSet<String>,
}

crate::unpack_fn!(ensure_list, Svc, GurpSvc);

impl TryFrom<&Janet> for GurpSvc {
    type Error = anyhow::Error;

    fn try_from(value: &Janet) -> anyhow::Result<Self> {
        let data = value.extract_struct()?;
        let action = janet_helpers::action_as_enum(&data)?;

        if action != Action::Ensure {
            bail!("svcs can only be ensured");
        }

        Ok(GurpSvc {
            action: Action::Ensure,
            id: data.get_field_string("_id")?,
            name: data.get_field_string("name")?,
            desired_state: data.get_field_string("state")?,
            restarters: data
                .get_field_string_tuple("restarted-by")?
                .into_iter()
                .collect(),
            reloaders: data
                .get_field_string_tuple("reloaded-by")?
                .into_iter()
                .collect(),
        })
    }
}

impl Apply for GurpSvc {
    fn apply(&self, apply_context: &ApplyContext, opts: &Opts) -> anyhow::Result<ApplySummary> {
        self.apply_ensure(apply_context, opts)
    }
}

impl GurpSvc {
    fn apply_ensure(
        &self,
        apply_context: &ApplyContext,
        opts: &Opts,
    ) -> anyhow::Result<ApplySummary> {
        let current_state = svcs::current_state(&self.name)?;

        tracing::debug!(
            "changed resources: {}",
            apply_context
                .changed_ids
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );

        if current_state == self.desired_state {
            if !apply_context.changed_ids.is_disjoint(&self.restarters) {
                tracing::info!("{}: restarting service", self.name);
                if opts.noop {
                    Ok(ONE_RESOURCE_NOOP)
                } else {
                    svcs::run_svcadm(&self.name, "restart")?;
                    Ok(ONE_RESOURCE_ONE_CHANGE)
                }
            } else if !apply_context.changed_ids.is_disjoint(&self.reloaders) {
                tracing::info!("{}: reloading service", self.name);
                if opts.noop {
                    Ok(ONE_RESOURCE_NOOP)
                } else {
                    svcs::run_svcadm(&self.name, "reload")?;
                    Ok(ONE_RESOURCE_ONE_CHANGE)
                }
            } else if opts.noop {
                Ok(ONE_RESOURCE_NOOP)
            } else {
                tracing::info!("{}: no change", self.name);
                Ok(ONE_RESOURCE_NO_CHANGE)
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
