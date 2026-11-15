use anyhow::Context;
use common::cmd;
use common::constants::{DLADM_BIN, ONE_RESOURCE_NO_CHANGE, ONE_RESOURCE_ONE_CHANGE};
use common::types::{ApplyOpts, ApplySummary};
use os_types::{GurpId, LinkName};
use serde::Deserialize;
use std::collections::HashSet;
// use std::net::IpAddr;
use std::process::Command;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(test, derive(PartialEq))]
pub struct AggrEnsure {
    #[serde(rename = "_id")]
    pub id: GurpId,
    pub name: LinkName,
    pub links: HashSet<LinkName>,
    pub policy: String,
    pub lacp_mode: Option<String>,
    pub lacp_timer: Option<String>,
    pub mac_address: Option<String>,
}

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct ExtantAggr {
    pub name: LinkName,
    pub links: HashSet<LinkName>,
    pub policy: String,
    pub lacp_mode: String,
    pub lacp_timer: String,
    pub mac_address: Option<String>,
}

#[derive(Deserialize, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct AggrRemove {
    #[serde(rename = "_id")]
    pub id: GurpId,
    pub name: LinkName,
}

impl AggrEnsure {
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        if let Some(extant_aggr) = aggr_info(&self.name)? {
            tracing::debug!("{} already exists", self.name);
            self.align_aggr(&extant_aggr, opts)
        } else {
            self.create_aggr(opts)
        }
    }

    fn align_aggr(
        &self,
        extant_aggr: &ExtantAggr,
        opts: &ApplyOpts,
    ) -> anyhow::Result<ApplySummary> {
        if extant_aggr.links != self.links {
            tracing::info!("Recreating aggr {}", self.name);
            cmd_change_or_noop!(opts, DLADM_BIN, "delete-aggr", &self.name)?;
            return self.create_aggr(opts);
        }

        let mut args = Vec::new();

        if extant_aggr.policy != self.policy {
            tracing::info!(
                "aggr {} policy: {} -> {}",
                self.name,
                extant_aggr.policy,
                self.policy
            );
            args.extend(["-P", &self.policy]);
        }

        if let Some(lacp_mode) = &self.lacp_mode
            && lacp_mode != &extant_aggr.lacp_mode
        {
            tracing::info!(
                "aggr {} lacp_mode: {} -> {}",
                self.name,
                extant_aggr.lacp_mode,
                lacp_mode
            );
            args.extend(["-L", &self.policy]);
        }

        if let Some(lacp_timer) = &self.lacp_timer
            && lacp_timer != &extant_aggr.lacp_timer
        {
            tracing::info!(
                "aggr {} lacp_timer: {} -> {}",
                self.name,
                extant_aggr.lacp_timer,
                lacp_timer
            );
            args.extend(["-T", lacp_timer]);
        }

        if args.is_empty() {
            tracing::debug!("aggr {} is correct", self.name);
            Ok(ONE_RESOURCE_NO_CHANGE)
        } else {
            let mut cmd = Command::new(DLADM_BIN);
            cmd.arg("modify-aggr");
            cmd.args(args);
            cmd.arg(&self.name);

            tracing::debug!(command = cmd::to_string(&cmd));

            if !opts.noop {
                run_cmd!(cmd).with_context(|| format!("failed to modify aggr {}", self.name))?;
            }

            Ok(ONE_RESOURCE_ONE_CHANGE)
        }
    }

    fn create_aggr(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        tracing::info!("creating {}", self.name);

        let mut cmd = Command::new(DLADM_BIN);
        cmd.arg("create-aggr");
        cmd.args(["-P", &self.policy]);

        if let Some(mode) = &self.lacp_mode {
            cmd.args(["-L", mode]);
        }

        if let Some(timer) = &self.lacp_timer {
            cmd.args(["-T", timer]);
        }

        if let Some(mac_addr) = &self.mac_address {
            cmd.args(["-u", &mac_addr.to_string()]);
        }

        for link in &self.links {
            cmd.args(["-l", &link.to_string()]);
        }

        cmd.arg(&self.name);
        tracing::debug!(command = cmd::to_string(&cmd));

        if !opts.noop {
            run_cmd!(cmd).with_context(|| format!("failed to create aggr {}", self.name))?;
        }

        Ok(ONE_RESOURCE_ONE_CHANGE)
    }
}

impl AggrRemove {
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        if aggr_info(&self.name)?.is_some() {
            tracing::info!("Removing {}", self.name);
            cmd_change_or_noop!(opts, DLADM_BIN, "delete-aggr", &self.name)
        } else {
            tracing::debug!("{} does not exist", self.name);
            Ok(ONE_RESOURCE_NO_CHANGE)
        }
    }
}

fn extract_info(name: &LinkName, raw_info: &str, raw_x_info: &str) -> Option<ExtantAggr> {
    if let Some(aggr_row) = raw_info
        .lines()
        .find(|l| l.starts_with(&format!("{name}:")))
    {
        let mut chunks = aggr_row.split(":");
        chunks.next();

        if let Some(policy) = chunks.next()
            && let Some(_addr_policy) = chunks.next()
            && let Some(lacp_mode) = chunks.next()
            && let Some(lacp_timer) = chunks.next()
        {
            let links = raw_x_info
                .lines()
                .filter_map(|l| {
                    let mut chunks = l.split(":");

                    if let Some(_aggr) = chunks.next()
                        && let Some(link) = chunks.next()
                        && !link.is_empty()
                        && let Some(link_name) = LinkName::new(link).ok()
                    {
                        Some(LinkName::new(link_name.to_string()).unwrap())
                    } else {
                        None
                    }
                })
                .collect::<HashSet<_>>();

            Some(ExtantAggr {
                name: name.to_owned(),
                links,
                policy: policy.to_owned(),
                lacp_mode: lacp_mode.to_owned(),
                lacp_timer: lacp_timer.to_owned(),
                mac_address: None,
            })
        } else {
            None
        }
    } else {
        None
    }
}

fn aggr_info(name: &LinkName) -> anyhow::Result<Option<ExtantAggr>> {
    let raw_info = cmd_output!(
        DLADM_BIN,
        "show-aggr",
        "-po",
        "link,policy,addrpolicy,lacpactivity,lacptimer"
    )
    .context("cannot get list of aggrs")?;

    let raw_x_info = cmd_output!(DLADM_BIN, "show-aggr", "-xpo", "link,port")
        .context("cannot get aggr link info")?;

    Ok(extract_info(name, &raw_info, &raw_x_info))
}

#[cfg(test)]
mod test {
    use super::*;
    use indoc::indoc;
    use tester::deserialized_example;

    #[test]
    fn test_extract_info() {
        assert_eq!(
            ExtantAggr {
                name: LinkName::new("aggr0").unwrap(),
                links: HashSet::from([
                    LinkName::new("vioif1").unwrap(),
                    LinkName::new("vioif2").unwrap()
                ]),
                policy: "L4".to_owned(),
                lacp_timer: "short".to_owned(),
                lacp_mode: "off".to_owned(),
                mac_address: None,
            },
            extract_info(
                &LinkName::new("aggr0").unwrap(),
                indoc! {
                "aggr0:L4:auto:off:short
                 aggr1:L4:auto:off:short",

                 },
                indoc! {
                "aggr0:
                      aggr0:vioif2
                      aggr0:vioif1"
                  }
            )
            .unwrap()
        );
    }

    #[test]
    fn test_ensure_aggr_deserialize() {
        assert_eq!(
            AggrEnsure {
                id: GurpId::new("/NO-ROLE/aggr/aggr0").unwrap(),
                name: LinkName::new("aggr0").unwrap(),
                policy: "L2,L3".to_owned(),
                lacp_timer: Some("long".to_owned()),
                lacp_mode: None,
                mac_address: None,
                links: HashSet::from([
                    LinkName::new("vioif1").unwrap(),
                    LinkName::new("vioif2").unwrap(),
                    LinkName::new("vioif3").unwrap(),
                ])
            },
            deserialized_example("aggr/ensure-01.janet")
        );
    }

    #[test]
    fn test_remove_aggr_deserialize() {
        assert_eq!(
            AggrRemove {
                id: GurpId::new("/NO-ROLE/aggr/aggr1").unwrap(),
                name: LinkName::new("aggr1").unwrap(),
            },
            deserialized_example("aggr/remove-01.janet")
        );
    }
}
