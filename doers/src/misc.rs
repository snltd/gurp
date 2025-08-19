use common::prelude::*;
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
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        let mut aggr = ApplySummary::default();

        if let Some(domain) = &self.desired_state.nfs_domain {
            aggr = aggr + self.ensure_nfs_domain(domain, opts)?;
        }

        if let Some(user) = &self.desired_state.enable_smb {
            aggr = aggr
                + match self.enable_smb_share(user, opts) {
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

    fn enable_smb_share(&self, username: &str, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        tracing::debug!("calling misc/enable_smb_share");

        match cmd_output!(SMBADM_BIN, "lookup", username) {
            // if it returns 0 and doesn't say "NONE_MAPPED" then I think it's configured
            Ok(txt) => {
                if !txt.contains("NONE_MAPPED") {
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

        let mut cmd = cmd!(SMBADM_BIN, "enable-user", username);
        return_if_noop!(opts);
        one_change_or_stderr!(cmd, "error enabling SMB share")
    }

    fn ensure_nfs_domain(&self, desired_domain: &str, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        tracing::debug!("calling misc/ensure_nfs_domain");

        let status = cmd_output!(SHARECTL_BIN, "get", "-p", "nfsmapid_domain", "nfs")?;
        let chunks: Vec<_> = status.split('=').collect();

        if chunks.len() != 2 {
            bail!("unexpected sharectl output: {}", status);
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

        let mut cmd = cmd!(
            SHARECTL_BIN,
            "set",
            "-p",
            format!("nfsmapid_domain={desired_domain}"),
            "nfs"
        );

        return_if_noop!(opts);
        one_change_or_stderr!(cmd, "error setting NFS domain")
    }

    fn set_scheduler_class(
        &self,
        desired_class: &str,
        opts: &ApplyOpts,
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

        let mut cmd = cmd!(DISPADMIN_BIN, "-d", desired_class);
        return_if_noop!(opts);
        one_change_or_stderr!(cmd, "error setting scheduler class")
    }
}
