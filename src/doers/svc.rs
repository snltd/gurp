use crate::common::constants::{ONE_RESOURCE_NO_CHANGE, ONE_RESOURCE_ONE_CHANGE};
use crate::common::output::Output;
use crate::common::traits::Apply;
use crate::common::types::{Action, ApplyContext, ApplySummary, Opts, Resource};
use crate::debug;
use crate::utils::helpers;
use crate::utils::janet_helpers::{self, JanetExt, JanetStructExt};
use anyhow::anyhow;
use janetrs::{Janet, JanetArray};
use paste::paste;
use std::collections::HashSet;
use std::process::{Command, Stdio};

const SVCADM_BIN: &str = "/usr/sbin/svcadm";
const SVCS_BIN: &str = "/bin/svcs";

// THINGS TO KNOW / THINGS TO DO.
// There's no svc/remove, only svc/ensure.
//
#[derive(Debug, PartialEq, Eq)]
pub struct GurpSvc {
    pub action: Action,
    pub id: String,
    pub name: String,
    pub desired_state: String,
    pub doer: String,
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
            return Err(anyhow!("svcs can only be ensured"));
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
            doer: "svc".to_owned(),
        })
    }
}

impl Apply for GurpSvc {
    fn apply(&self, apply_context: &ApplyContext, opts: &Opts) -> anyhow::Result<ApplySummary> {
        let output = Output::new(&self.doer, opts);
        self.apply_ensure(apply_context, opts, &output)
    }
}

impl GurpSvc {
    fn apply_ensure(
        &self,
        apply_context: &ApplyContext,
        opts: &Opts,
        output: &Output,
    ) -> anyhow::Result<ApplySummary> {
        let current_state = self.current_state(opts)?;

        debug!(
            opts,
            "doer/svc", "changed resources: {:?}", apply_context.changed_ids
        );

        if current_state == self.desired_state {
            if !apply_context.changed_ids.is_disjoint(&self.restarters) {
                output.action(&self.name, "RESTARTING");
                self.run_svcadm("restart", opts)?;
                Ok(ONE_RESOURCE_ONE_CHANGE)
            } else if !apply_context.changed_ids.is_disjoint(&self.reloaders) {
                output.action(&self.name, "RELOADING");
                self.run_svcadm("reload", opts)?;
                Ok(ONE_RESOURCE_ONE_CHANGE)
            } else {
                output.no_change(&self.name);
                Ok(ONE_RESOURCE_NO_CHANGE)
            }
        } else {
            output.change(&self.name, &current_state, &self.desired_state);
            self.set_state(&current_state, opts)?;
            Ok(ONE_RESOURCE_ONE_CHANGE)
        }
    }

    fn set_state(&self, current_state: &str, opts: &Opts) -> anyhow::Result<String> {
        let action = if current_state == "maintenance" {
            debug!(opts, "doer/svc", "Trying to clear {}", self.name);
            "clear"
        } else if self.desired_state == "online" {
            "enable"
        } else if self.desired_state == "disabled" {
            "disable"
        } else {
            return Err(anyhow!(
                "unknown or unsupported state: {}",
                self.desired_state
            ));
        };

        self.run_svcadm(action, opts)
    }

    fn run_svcadm(&self, action: &str, opts: &Opts) -> anyhow::Result<String> {
        let mut cmd = Command::new(SVCADM_BIN);
        cmd.arg(action).arg(&self.name).stderr(Stdio::piped());

        debug!(opts, "doer/svc", "{}", helpers::command_to_string(&cmd));
        let output = cmd.output()?;

        if output.status.success() {
            Ok(String::from_utf8(output.stdout)?)
        } else {
            Err(anyhow!(String::from_utf8(output.stderr)?))
        }
    }

    fn current_state(&self, opts: &Opts) -> anyhow::Result<String> {
        let mut cmd = Command::new(SVCS_BIN);
        cmd.arg("-Ho")
            .arg("state")
            .arg(&self.name)
            .stderr(Stdio::piped());

        debug!(opts, "doer/svc", "{}", helpers::command_to_string(&cmd));

        let output = cmd.output()?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
        } else {
            Err(anyhow!(
                String::from_utf8_lossy(&output.stderr).into_owned()
            ))
        }
    }
}
