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

const SVCPROP_BIN: &str = "/usr/sbin/svcprop";

#[derive(Debug, PartialEq, Eq)]
pub struct GurpSvcprop {
    pub action: Action,
    pub id: String,
    pub desired_state: SvcpropState,
}

#[derive(Debug, PartialEq, Eq)]
pub struct SvcpropState {
    pub nfs_domain: Option<NfsDomain>,
    pub enable_smb: Option<Username>,
}

crate::unpack_fn!(ensure_list, Svcprop, GurpSvcprop);

impl TryFrom<&Janet> for GurpSvcprop {
    type Error = anyhow::Error;

    fn try_from(value: &Janet) -> anyhow::Result<Self> {
        let data = value.extract_struct()?;
        let action = janet_helpers::action_as_enum(&data)?;

        if action != Action::Ensure {
            bail!("misc can only be ensured");
        }

        Ok(GurpSvcprop {
            action: Action::Ensure,
            id: data.get_field_string("_id")?,
            desired_state: SvcpropState {
                nfs_domain: data.get_field_string_opt("nfs-domain"),
                enable_smb: data.get_field_string_opt("enable-smb"),
            },
        })
    }
}

impl Apply for GurpSvcprop {
    fn apply(&self, apply_context: &ApplyContext, opts: &Opts) -> anyhow::Result<ApplySummary> {
        self.apply_ensure(apply_context, opts)
    }
}

impl GurpSvcprop {
    fn apply_ensure(&self, _c: &ApplyContext, opts: &Opts) -> anyhow::Result<ApplySummary> {
        let mut aggr = ApplySummary::default();

        if let Some(domain) = &self.desired_state.nfs_domain {
            aggr = aggr + self.ensure_nfs_domain(domain, opts);
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

    fn ensure_nfs_domain(&self, desired_domain: &str, opts: &Opts) -> ApplySummary {
        let mut get_cmd = Command::new(SHARECTL_BIN);
        get_cmd
            .arg("get")
            .arg("-p")
            .arg("nfsmapid_domain")
            .arg("nfs")
            .stderr(Stdio::piped());

        tracing::debug!(command = helpers::command_to_string(&get_cmd));

        let sharectl_output = match get_cmd.output() {
            Ok(txt) => txt,
            Err(e) => {
                tracing::error!("cannot get NFS domain: {}", e);
                return ONE_RESOURCE_ONE_ERROR;
            }
        };

        let sharectl_string = String::from_utf8_lossy(&sharectl_output.stdout);
        let chunks: Vec<_> = sharectl_string.split('=').collect();

        if chunks.len() != 2 {
            tracing::error!("unexpected sharectl output: {}", sharectl_string);
            return ONE_RESOURCE_ONE_ERROR;
        }

        let current_domain = chunks.last().unwrap().trim();

        if current_domain == desired_domain {
            tracing::info!("no change to NFS domain: {}", current_domain);
            return ONE_RESOURCE_NO_CHANGE;
        }

        tracing::info!(
            "change NFS domain: {} -> {}",
            current_domain,
            desired_domain
        );

        if opts.noop {
            return ONE_RESOURCE_NOOP;
        }

        let mut set_cmd = Command::new(SHARECTL_BIN);
        set_cmd
            .arg("set")
            .arg("-p")
            .arg(format!("nfsmapid_domain={}", desired_domain))
            .arg("nfs")
            .stderr(Stdio::piped());

        tracing::debug!(command = helpers::command_to_string(&get_cmd));

        match set_cmd.output() {
            Ok(code) => {
                if code.status.success() {
                    ONE_RESOURCE_ONE_CHANGE
                } else {
                    tracing::error!(
                        "error setting NFS domain: {}",
                        String::from_utf8_lossy(&code.stderr),
                    );
                    ONE_RESOURCE_ONE_ERROR
                }
            }
            Err(e) => {
                tracing::error!("error setting NFS domain: {}", e,);
                ONE_RESOURCE_ONE_ERROR
            }
        }
    }
}
