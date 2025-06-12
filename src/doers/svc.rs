use crate::common::constants::{
    ONE_RESOURCE_NO_CHANGE, ONE_RESOURCE_NOOP, ONE_RESOURCE_ONE_CHANGE, ONE_RESOURCE_ONE_ERROR,
};
use crate::common::output::Output;
use crate::common::traits::Apply;
use crate::common::types::{Action, ApplyContext, ApplySummary, Opts, Resource};
use crate::utils::helpers;
use crate::utils::janet_helpers::{self, JanetExt, JanetStructExt};
use crate::{debug, error};
use anyhow::anyhow;
use colored::Colorize;
use janetrs::{Janet, JanetArray};
use paste::paste;
use std::io::Write;
use std::process::{Command, Stdio};

const TAG_LINE: &str = "# gurp managed ID";
const SVCADM_BIN: &str = "/usr/sbin/svcadm";
const SVCS_BIN: &str = "/bin/svc";

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
    pub restarters: Vec<String>,
    pub reloaders: Vec<String>,
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
            restarters: data.get_field_string_tuple("restarted-by")?,
            reloaders: data.get_field_string_tuple("reloaded-by")?,
            doer: "svc".to_owned(),
        })
    }
}

impl Apply for GurpSvc {
    fn apply(&self, apply_context: &ApplyContext, opts: &Opts) -> anyhow::Result<ApplySummary> {
        let output = Output::new(&self.doer, opts);
        self.apply_ensure(opts, apply_context, &output)
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

        if current_state == self.desired_state {
            output.no_change(&self.name);
            Ok(ONE_RESOURCE_NO_CHANGE)
        } else {
            output.change(&self.name, &current_state, &self.desired_state);
            Ok(ONE_RESOURCE_ONE_CHANGE)
        }
    }

    fn set_state(&self, opts: &Opts) -> anyhow::Result<String> {}

    fn current_state(&self, opts: &Opts) -> anyhow::Result<String> {
        let mut cmd = Command::new(SVCS_BIN);
        cmd.arg("-Ho")
            .arg("state")
            .arg(&self.name)
            .stderr(Stdio::piped());

        debug!(opts, "doer/cron", "{}", helpers::command_to_string(&cmd));

        let output = cmd.output()?;

        if output.status.success() {
            Ok(String::from_utf8(output.stdout)?)
        } else {
            Err(anyhow!(String::from_utf8(output.stderr)?))
        }
    }
}
