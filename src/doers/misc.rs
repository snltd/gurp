use crate::common::constants::{
    ONE_RESOURCE_NO_CHANGE, ONE_RESOURCE_NOOP, ONE_RESOURCE_ONE_CHANGE, ONE_RESOURCE_ONE_ERROR,
};
use crate::common::traits::Apply;
use crate::common::types::{Action, ApplyContext, ApplySummary, Opts, Resource};
use crate::utils::helpers;
use crate::utils::janet_helpers::{self, JanetExt, JanetStructExt};
use anyhow::{anyhow, bail};
use janetrs::{Janet, JanetArray};
use paste::paste;
use std::process::{Command, Stdio};

// THINGS TO KNOW / THINGS TO DO.
// This might be a bad idea. Hardcoded ways to do a bunch of certain things that I want to do.
// There's no misc/remove, only misc/ensure, at least for now.
// dispadmin only takes the scheduler class.

const SHARECTL_BIN: &str = "/usr/sbin/sharectl";
const SMBADM_BIN: &str = "/usr/sbin/smbadm";
const DISPADMIN_BIN: &str = "/usr/sbin/dispadmin";

#[derive(Debug, PartialEq, Eq)]
pub struct GurpMisc {
    pub action: Action,
    pub id: String,
    pub desired_state: MiscState,
}

type NfsDomain = String;
type Username = String;

#[derive(Debug, PartialEq, Eq)]
pub struct MiscState {
    pub nfs_domain: Option<NfsDomain>,
    pub enable_smb: Option<Username>,
    pub dispadmin_class: Option<String>,
}

crate::unpack_fn!(ensure_list, Misc, GurpMisc);

impl TryFrom<&Janet> for GurpMisc {
    type Error = anyhow::Error;

    fn try_from(value: &Janet) -> anyhow::Result<Self> {
        let data = value.extract_struct()?;
        let action = janet_helpers::action_as_enum(&data)?;

        if action != Action::Ensure {
            bail!("misc can only be ensured");
        }

        Ok(GurpMisc {
            action: Action::Ensure,
            id: data.get_field_string("_id")?,
            desired_state: MiscState {
                nfs_domain: data.get_field_string_opt("nfs-domain"),
                enable_smb: data.get_field_string_opt("enable-smb"),
                dispadmin_class: data.get_field_string_opt("scheduler-class"),
            },
        })
    }
}

impl Apply for GurpMisc {
    fn apply(&self, apply_context: &ApplyContext, opts: &Opts) -> anyhow::Result<ApplySummary> {
        self.apply_ensure(apply_context, opts)
    }
}

impl GurpMisc {
    fn apply_ensure(&self, _c: &ApplyContext, opts: &Opts) -> anyhow::Result<ApplySummary> {
        let mut aggr = ApplySummary::default();

        if let Some(domain) = &self.desired_state.nfs_domain {
            aggr = aggr + self.ensure_nfs_domain(domain, opts)?;
        }

        if let Some(user) = &self.desired_state.enable_smb {
            aggr = aggr
                + match self.enable_smb_share(user) {
                    Ok(summary) => summary,
                    Err(e) => {
                        tracing::error!("smbadm check: {}", e);
                        ONE_RESOURCE_ONE_ERROR
                    }
                };
        }

        if let Some(class) = &self.desired_state.dispadmin_class {
            aggr = aggr
                + match self.set_scheduler_class(class, opts) {
                    Ok(summary) => summary,
                    Err(e) => {
                        tracing::error!("dispadmin error: {}", e);
                        ONE_RESOURCE_ONE_ERROR
                    }
                };
        }

        Ok(aggr)
    }

    fn enable_smb_share(&self, username: &str) -> anyhow::Result<ApplySummary> {
        let mut get_status_cmd = Command::new(SMBADM_BIN);
        get_status_cmd.arg("lookup").arg(username);

        tracing::debug!(command = helpers::command_to_string(&get_status_cmd));

        match get_status_cmd.output() {
            // if it returns 0 and doesn't say "NONE_MAPPED" then I think it's configured
            Ok(txt) => {
                if !String::from_utf8_lossy(&txt.stdout).contains("NONE_MAPPED") {
                    tracing::info!("no change: smb config {}", username);
                    return Ok(ONE_RESOURCE_NO_CHANGE);
                }
            }
            // I'm not sure whether or not an error here means we shouldn't continue. I don't think
            // it does
            Err(_) => {
                tracing::debug!("error running smbadm lookup; continuing");
            }
        }

        tracing::info!("enabling smb user: {}", username);

        // If we're still here, we can enable the user
        //
        let mut enable_cmd = Command::new(SMBADM_BIN);
        enable_cmd.arg("enable-user").arg(username);

        tracing::debug!(command = helpers::command_to_string(&enable_cmd));

        match get_status_cmd.output() {
            Ok(_) => Ok(ONE_RESOURCE_ONE_CHANGE),
            Err(e) => Err(anyhow!(e)),
        }
    }

    fn ensure_nfs_domain(&self, desired_domain: &str, opts: &Opts) -> anyhow::Result<ApplySummary> {
        let mut get_cmd = Command::new(SHARECTL_BIN);
        get_cmd
            .arg("get")
            .arg("-p")
            .arg("nfsmapid_domain")
            .arg("nfs")
            .stderr(Stdio::piped());

        tracing::debug!(command = helpers::command_to_string(&get_cmd));

        let sharectl_output = get_cmd.output()?;
        let sharectl_string = String::from_utf8_lossy(&sharectl_output.stdout);
        let chunks: Vec<_> = sharectl_string.split('=').collect();

        if chunks.len() != 2 {
            bail!("unexpected sharectl output: {}", sharectl_string);
        }

        let current_domain = chunks.last().unwrap().trim();

        if current_domain == desired_domain {
            bail!("no change to NFS domain: {}", current_domain);
        }

        tracing::info!(
            "change NFS domain: {} -> {}",
            current_domain,
            desired_domain
        );

        let mut set_cmd = Command::new(SHARECTL_BIN);
        set_cmd
            .arg("set")
            .arg("-p")
            .arg(format!("nfsmapid_domain={}", desired_domain))
            .arg("nfs")
            .stderr(Stdio::piped());

        tracing::debug!(command = helpers::command_to_string(&get_cmd));

        if opts.noop {
            return Ok(ONE_RESOURCE_NOOP);
        }

        let output = set_cmd.output()?;

        if output.status.success() {
            Ok(ONE_RESOURCE_ONE_CHANGE)
        } else {
            bail!(
                "error setting NFS domain class: {}",
                String::from_utf8_lossy(&output.stderr),
            )
        }
    }

    fn set_scheduler_class(
        &self,
        desired_class: &str,
        opts: &Opts,
    ) -> anyhow::Result<ApplySummary> {
        let mut get_cmd = Command::new(DISPADMIN_BIN);
        get_cmd.arg("-d").stderr(Stdio::piped());

        tracing::debug!(command = helpers::command_to_string(&get_cmd));

        let dispadmin_output = get_cmd.output()?;
        let dispadmin_string = String::from_utf8_lossy(&dispadmin_output.stdout);
        let chunks: Vec<_> = dispadmin_string.split('=').collect();

        if chunks.len() != 2 {
            bail!("unexpected dispadmin output: {}", dispadmin_string);
        }

        let current_class = chunks.last().unwrap().trim();

        if current_class == desired_class {
            tracing::info!("no change to scheduler class: {}", current_class);
            return Ok(ONE_RESOURCE_NO_CHANGE);
        }

        tracing::info!(
            "change scheduler class: {} -> {}",
            current_class,
            desired_class
        );

        let mut set_cmd = Command::new(DISPADMIN_BIN);

        set_cmd.arg("-d").arg(desired_class).stderr(Stdio::piped());

        tracing::debug!(command = helpers::command_to_string(&get_cmd));

        if opts.noop {
            return Ok(ONE_RESOURCE_NOOP);
        }

        let output = set_cmd.output()?;

        if output.status.success() {
            Ok(ONE_RESOURCE_ONE_CHANGE)
        } else {
            bail!(
                "error setting scheduler class: {}",
                String::from_utf8_lossy(&output.stderr),
            )
        }
    }
}
