use anyhow::Context;
use common::cmd;
use common::constants::{DLADM_BIN, ONE_RESOURCE_NO_CHANGE, ONE_RESOURCE_ONE_CHANGE};
use common::types::{ApplyOpts, ApplySummary};
use os_types::{GurpId, LinkName};
use serde::Deserialize;
use std::collections::HashSet;
use std::net::IpAddr;
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
    pub address: Option<IpAddr>,
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
        if aggr_exists(&self.name)? {
            tracing::debug!("{} already exists", self.name);
            // align_interfaces(&self.name, &self.interfaces, opts)
            Ok(ONE_RESOURCE_NO_CHANGE)
        } else {
            self.create_aggr(opts)
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

        if let Some(addr) = &self.address {
            cmd.args(["-u", &addr.to_string()]);
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

// fn align_interfaces(
//     group: &LinkName,
//     desired_interfaces: &HashSet<LinkName>,
//     opts: &ApplyOpts,
// ) -> anyhow::Result<ApplySummary> {
//     let current_interfaces = aggr_interfaces(&aggrstat_output()?, group);

//     if &current_interfaces == desired_interfaces {
//         tracing::debug!("{group} has correct interface list");
//         return Ok(ONE_RESOURCE_NO_CHANGE);
//     }

//     for iface in desired_interfaces.difference(&current_interfaces) {
//         tracing::info!("adding {iface} to aggr group {group}");
//         cmd_change_or_noop!(opts, IPADM_BIN, "add-aggr", "-i", iface, group)?;
//     }

//     for iface in current_interfaces.difference(desired_interfaces) {
//         tracing::info!("removing {iface} from aggr group {group}");
//         cmd_change_or_noop!(opts, IPADM_BIN, "remove-aggr", "-i", iface, group)?;
//     }

//     Ok(ONE_RESOURCE_ONE_CHANGE)
// }

impl AggrRemove {
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        if aggr_exists(&self.name)? {
            tracing::info!("Removing {}", self.name);
            // align_interfaces(&self.name, &HashSet::new(), opts)?;
            cmd_change_or_noop!(opts, DLADM_BIN, "delete-aggr", &self.name)
        } else {
            tracing::debug!("{} does not exist", self.name);
            Ok(ONE_RESOURCE_NO_CHANGE)
        }
    }
}

fn aggr_exists(name: &LinkName) -> anyhow::Result<bool> {
    // let group_list = cmd_output!(aggrSTAT_BIN, "-Pg", "-o", "group")?;
    // Ok(group_list.lines().any(|l| l == name.to_string()))
    Ok(false)
}

// fn aggrstat_output() -> anyhow::Result<String> {
//     cmd_output!(aggrSTAT_BIN, "-Pg", "-o", "group,interfaces")
// }

// fn aggr_interfaces(raw_output: &str, aggr_group: &LinkName) -> HashSet<LinkName> {
//     raw_output
//         .lines()
//         .find_map(|l| {
//             let mut chunks = l.split(":");
//             if let Some(group) = chunks.next()
//                 && let Some(interfaces) = chunks.next()
//                 && group == aggr_group.to_string()
//             {
//                 Some(
//                     interfaces
//                         .split(" ")
//                         .filter_map(|s| LinkName::new(s).ok())
//                         .collect::<HashSet<_>>(),
//                 )
//             } else {
//                 None
//             }
//         })
//         .unwrap_or_default()
// }

// #[cfg(test)]
// mod test {
//     use super::*;
//     use tester::{deserialized_example, janet2json};

//     #[test]
//     fn test_ensure_aggr_deserialize() {
//         let example_json = janet2json(r#"(aggr/ensure "aggr0" :interfaces["bge0" "e1000g0"])"#);
//         let sut: AggrEnsure = serde_json::from_str(&example_json).unwrap();

//         assert_eq!(
//             AggrEnsure {
//                 id: GurpId::new("/NO-ROLE/aggr/aggr0").unwrap(),
//                 name: LinkName::new("aggr0").unwrap(),
//                 interfaces: HashSet::from([
//                     LinkName::new("bge0").unwrap(),
//                     LinkName::new("e1000g0").unwrap()
//                 ])
//             },
//             sut
//         );
//     }

//     #[test]
//     fn test_remove_aggr_deserialize() {
//         assert_eq!(
//             AggrRemove {
//                 id: GurpId::new("/NO-ROLE/aggr/aggr1").unwrap(),
//                 name: LinkName::new("aggr1").unwrap(),
//             },
//             deserialized_example("aggr/remove-01.janet")
//         );
//     }
// }
