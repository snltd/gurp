use crate::prelude::*;
use anyhow::anyhow;
use serde::Deserialize;
use std::process::{Command, Stdio};

// THINGS TO KNOW / THINGS TO DO.
// This might be a bad idea. Hardcoded ways to do a bunch of certain things that I want to do.
// There's no misc/remove, only misc/ensure, at least for now.
// dispadmin only takes the scheduler class.

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct GurpMiscEnsure {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(flatten)]
    pub desired_state: MiscState,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct MiscState {
    pub nfs_domain: Option<NfsDomain>,
    pub enable_smb: Option<Username>,
    pub scheduler: Option<SchedulerClass>,
}

type NfsDomain = String;
type Username = String;
type SchedulerClass = String;

impl GurpMiscEnsure {
    pub fn apply(&self, opts: &Opts) -> anyhow::Result<ApplySummary> {
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

        if let Some(class) = &self.desired_state.scheduler {
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
        tracing::debug!("calling misc/enable_smb_share");
        let mut get_status_cmd = Command::new(SMBADM_BIN);
        get_status_cmd.arg("lookup").arg(username);

        tracing::debug!(command = helpers::command_to_string(&get_status_cmd));

        match get_status_cmd.output() {
            // if it returns 0 and doesn't say "NONE_MAPPED" then I think it's configured
            Ok(txt) => {
                if !String::from_utf8_lossy(&txt.stdout).contains("NONE_MAPPED") {
                    tracing::debug!("no change: smb config {}", username);
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
        tracing::debug!("calling misc/ensure_nfs_domain");
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
            tracing::debug!("no change to NFS domain: {}", current_domain);
            return Ok(ONE_RESOURCE_NO_CHANGE);
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
            .arg(format!("nfsmapid_domain={desired_domain}"))
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
        tracing::debug!("calling misc/set_scheduler_class");
        let mut get_cmd = Command::new(DISPADMIN_BIN);
        get_cmd.arg("-d").stderr(Stdio::piped());

        tracing::debug!(command = helpers::command_to_string(&get_cmd));

        let dispadmin_output = get_cmd.output()?;
        let dispadmin_stdout = String::from_utf8_lossy(&dispadmin_output.stdout);
        let dispadmin_stderr = String::from_utf8_lossy(&dispadmin_output.stderr);
        let chunks: Vec<_> = dispadmin_stdout.split_whitespace().collect();

        if chunks.len() < 2 {
            tracing::debug!(
                stdout = dispadmin_stdout.as_ref(),
                stderr = dispadmin_stderr.as_ref()
            );
            bail!("unexpected dispadmin output: run with debug to see output");
        }

        let current_class = chunks.first().unwrap().trim();

        if current_class == desired_class {
            tracing::debug!("no change to scheduler class: {}", current_class);
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
