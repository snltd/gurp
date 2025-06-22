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
use std::process::{Command, Stdio};

// THINGS TO KNOW / THINGS TO DO.
// This might be a bad idea. Hardcoded ways to do a bunch of certain things that I want to do.
// There's no misc/remove, only misc/ensure, at least for now.

const SHARECTL_BIN: &str = "/usr/sbin/sharectl";
const SMBADM_BIN: &str = "/usr/sbin/smbadm";

#[derive(Debug, PartialEq, Eq)]
pub struct GurpMisc {
    pub action: Action,
    pub id: String,
    pub desired_state: MiscState,
    pub doer: String,
}

type NfsDomain = String;
type Username = String;

#[derive(Debug, PartialEq, Eq)]
pub struct MiscState {
    pub nfs_domain: Option<NfsDomain>,
    pub enable_smb: Option<Username>,
}

crate::unpack_fn!(ensure_list, Misc, GurpMisc);

impl TryFrom<&Janet> for GurpMisc {
    type Error = anyhow::Error;

    fn try_from(value: &Janet) -> anyhow::Result<Self> {
        let data = value.extract_struct()?;
        let action = janet_helpers::action_as_enum(&data)?;

        if action != Action::Ensure {
            return Err(anyhow!("misc can only be ensured"));
        }

        Ok(GurpMisc {
            action: Action::Ensure,
            id: data.get_field_string("_id")?,
            desired_state: MiscState {
                nfs_domain: data.get_field_string_opt("nfs-domain"),
                enable_smb: data.get_field_string_opt("enable-smb"),
            },
            doer: "misc".to_owned(),
        })
    }
}

impl Apply for GurpMisc {
    fn apply(&self, apply_context: &ApplyContext, opts: &Opts) -> anyhow::Result<ApplySummary> {
        let output = Output::new(&self.doer, opts);
        self.apply_ensure(apply_context, opts, &output)
    }
}

impl GurpMisc {
    fn apply_ensure(
        &self,
        _c: &ApplyContext,
        opts: &Opts,
        output: &Output,
    ) -> anyhow::Result<ApplySummary> {
        let mut aggr = ApplySummary::default();

        if let Some(domain) = &self.desired_state.nfs_domain {
            aggr = aggr + self.ensure_nfs_domain(domain, opts, output);
        }

        if let Some(user) = &self.desired_state.enable_smb {
            aggr = aggr
                + match self.enable_smb_share(user, opts, output) {
                    Ok(summary) => summary,
                    Err(e) => {
                        error!(opts, "doer/misc", "smbadm check: {}", e);
                        ONE_RESOURCE_ONE_ERROR
                    }
                };
        }

        Ok(aggr)
    }

    fn enable_smb_share(
        &self,
        username: &str,
        opts: &Opts,
        output: &Output,
    ) -> anyhow::Result<ApplySummary> {
        let mut get_status_cmd = Command::new(SMBADM_BIN);
        // smbadm lookup rob | grep NONE_MAPPED
        get_status_cmd.arg("lookup").arg(username);

        debug!(
            opts,
            "doer/misc",
            "{}",
            helpers::command_to_string(&get_status_cmd)
        );

        match get_status_cmd.output() {
            // if it returns 0 and doesn't say "NONE_MAPPED" then I think it's configured
            Ok(txt) => {
                if !String::from_utf8_lossy(&txt.stdout).contains("NONE_MAPPED") {
                    output.no_change(format!("SMB config for {}", username));
                    return Ok(ONE_RESOURCE_NO_CHANGE);
                }
            }
            // I'm not sure whether or not an error here means we shouldn't continue. I don't think
            // it does
            Err(_) => {
                debug!(opts, "doer/misc", "error running smbadm lookup; continuing");
            }
        }

        output.creating(format!("smb user {}", username));

        // If we're still here, we can enable the user
        //
        let mut enable_cmd = Command::new(SMBADM_BIN);
        enable_cmd.arg("enable-user").arg(username);

        debug!(
            opts,
            "doer/misc",
            "{}",
            helpers::command_to_string(&enable_cmd)
        );

        match get_status_cmd.output() {
            Ok(_) => Ok(ONE_RESOURCE_ONE_CHANGE),
            Err(e) => Err(anyhow!(e)),
        }
    }

    fn ensure_nfs_domain(
        &self,
        desired_domain: &str,
        opts: &Opts,
        output: &Output,
    ) -> ApplySummary {
        let mut get_cmd = Command::new(SHARECTL_BIN);
        get_cmd
            .arg("get")
            .arg("-p")
            .arg("nfsmapid_domain")
            .arg("nfs")
            .stderr(Stdio::piped());

        debug!(
            opts,
            "doer/misc",
            "{}",
            helpers::command_to_string(&get_cmd)
        );

        let sharectl_output = match get_cmd.output() {
            Ok(txt) => txt,
            Err(e) => {
                error!(opts, "doer/misc", "cannot get NFS domain: {}", e);
                return ONE_RESOURCE_ONE_ERROR;
            }
        };

        let sharectl_string = String::from_utf8_lossy(&sharectl_output.stdout);
        let chunks: Vec<_> = sharectl_string.split('=').collect();

        if chunks.len() != 2 {
            error!(
                opts,
                "doer/misc", "unexpected sharectl output: {}", sharectl_string
            );
            return ONE_RESOURCE_ONE_ERROR;
        }

        let current_domain = chunks.last().unwrap().trim();

        if current_domain == desired_domain {
            output.no_change("NFS domain");
            return ONE_RESOURCE_NO_CHANGE;
        }

        output.change(
            "NFS domain",
            &current_domain.to_owned(),
            &desired_domain.to_owned(),
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

        debug!(
            opts,
            "doer/misc",
            "{}",
            helpers::command_to_string(&get_cmd)
        );

        match set_cmd.output() {
            Ok(code) => {
                if code.status.success() {
                    ONE_RESOURCE_ONE_CHANGE
                } else {
                    error!(
                        opts,
                        "doer/misc",
                        "error setting NFS domain: {}",
                        String::from_utf8_lossy(&code.stderr),
                    );
                    ONE_RESOURCE_ONE_ERROR
                }
            }
            Err(e) => {
                error!(opts, "doer/misc", "error setting NFS domain: {}", e,);
                ONE_RESOURCE_ONE_ERROR
            }
        }
    }
}
