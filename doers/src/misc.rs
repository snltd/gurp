use anyhow::{Context, bail, ensure};
use common::cmd;
use common::constants::{
    DISPADMIN_BIN, ONE_RESOURCE_NO_CHANGE, SHARECTL_BIN, SMBADM_BIN, SVCADM_BIN,
};
use common::types::{ApplyOpts, ApplySummary};
use os_types::GurpId;
use serde::Deserialize;
use std::process::{Command, Stdio};

#[derive(Deserialize, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct GurpMiscEnsure {
    #[serde(rename = "_id")]
    pub id: GurpId,
    #[serde(flatten)]
    pub desired_state: MiscState,
}

#[derive(Deserialize, Debug)]
#[cfg_attr(test, derive(PartialEq))]
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
            aggr += self.ensure_nfs_domain(domain, opts)?;
        }

        if let Some(user) = &self.desired_state.enable_smb {
            aggr += match self.enable_smb_share(user, opts) {
                Ok(summary) => summary,
                Err(e) => {
                    bail!("smbadm check: {}", e);
                }
            };
        }

        if let Some(class) = &self.desired_state.scheduler {
            aggr += match self.set_scheduler_class(class, opts) {
                Ok(summary) => summary,
                Err(e) => {
                    bail!("dispadmin error: {}", e);
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
        cmd_change_or_noop!(opts, SMBADM_BIN, "enable-user", username)
    }

    fn ensure_nfs_domain(
        &self,
        desired_domain: &str,
        opts: &ApplyOpts,
    ) -> anyhow::Result<ApplySummary> {
        tracing::debug!("calling misc/ensure_nfs_domain");

        let status = cmd_output!(SHARECTL_BIN, "get", "-p", "nfsmapid_domain", "nfs")
            .context("failed to get nfsmapid_domain")?;
        let chunks: Vec<_> = status.split('=').collect();

        ensure!(chunks.len() == 2, "unexpected sharectl output: {}", status);

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

        cmd_change_or_noop!(
            opts,
            SHARECTL_BIN,
            "set",
            "-p",
            format!("nfsmapid_domain={desired_domain}"),
            "nfs"
        )
        .with_context(|| format!("failed to set nfsmapid_domain to {desired_domain}"))
    }

    fn set_scheduler_class(
        &self,
        desired_class: &str,
        opts: &ApplyOpts,
    ) -> anyhow::Result<ApplySummary> {
        tracing::debug!("calling misc/set_scheduler_class");
        let mut get_cmd = Command::new(DISPADMIN_BIN);
        get_cmd.arg("-d").stderr(Stdio::piped());

        tracing::debug!(command = cmd::to_string(&get_cmd));

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

        let _ = cmd_change_or_noop!(opts, DISPADMIN_BIN, "-d", desired_class)
            .with_context(|| format!("failed to set scheduler class to {desired_class}"))?;

        cmd_change_or_noop!(opts, SVCADM_BIN, "refresh", "svc:/system/scheduler:default")
            .context("failed to refresh scheduler service")
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
    use tester::deserialized_example;

    #[test]
    fn test_misc_deserialize_ensure_nfs_domain() {
        assert_eq!(
            GurpMiscEnsure {
                id: GurpId::new("/NO-ROLE/misc/nfs-domain-lan.id264.net").unwrap(),
                desired_state: MiscState {
                    nfs_domain: Some("lan.id264.net".to_owned()),
                    enable_smb: None,
                    scheduler: None,
                }
            },
            deserialized_example("misc/ensure-nfs-domain.janet")
        );
    }

    #[test]
    fn test_misc_deserialize_ensure_smb_user() {
        assert_eq!(
            GurpMiscEnsure {
                id: GurpId::new("/NO-ROLE/misc/enable-smb-rob").unwrap(),
                desired_state: MiscState {
                    nfs_domain: None,
                    enable_smb: Some("rob".to_owned()),
                    scheduler: None,
                }
            },
            deserialized_example("misc/ensure-smb-user.janet")
        );
    }

    #[test]
    fn test_misc_deserialize_ensure_scheduler_class() {
        assert_eq!(
            GurpMiscEnsure {
                id: GurpId::new("/NO-ROLE/misc/scheduler-FSS").unwrap(),
                desired_state: MiscState {
                    nfs_domain: None,
                    enable_smb: None,
                    scheduler: Some("FSS".to_owned()),
                }
            },
            deserialized_example("misc/ensure-scheduler-class.janet")
        );
    }
}
