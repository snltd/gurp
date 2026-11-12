use common::constants::{IPADM_BIN, IPMPSTAT_BIN, ONE_RESOURCE_NO_CHANGE, ONE_RESOURCE_ONE_CHANGE};
use common::types::{ApplyOpts, ApplySummary};
use os_types::{GurpId, LinkName};
use serde::Deserialize;
use std::collections::HashSet;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(test, derive(PartialEq))]
pub struct IpmpEnsure {
    #[serde(rename = "_id")]
    pub id: GurpId,
    pub name: LinkName,
    pub interfaces: HashSet<LinkName>,
}

#[derive(Deserialize, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct IpmpRemove {
    #[serde(rename = "_id")]
    pub id: GurpId,
    pub name: LinkName,
}

impl IpmpEnsure {
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        if ipmp_exists(&self.name)? {
            tracing::debug!("{} already exists", self.name);
            align_interfaces(&self.name, &self.interfaces, opts)
        } else {
            self.create_ipmp(opts)
        }
    }

    fn create_ipmp(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        tracing::info!("creating {}", self.name);

        cmd_change_or_noop!(
            opts,
            IPADM_BIN,
            "create-ipmp",
            "-i",
            self.interfaces
                .iter()
                .map(|i| i.to_string())
                .collect::<Vec<_>>()
                .join(","),
            &self.name,
        )
    }
}

fn align_interfaces(
    group: &LinkName,
    desired_interfaces: &HashSet<LinkName>,
    opts: &ApplyOpts,
) -> anyhow::Result<ApplySummary> {
    let current_interfaces = ipmp_interfaces(&ipmpstat_output()?, group);

    if &current_interfaces == desired_interfaces {
        tracing::debug!("{group} has correct interface list");
        return Ok(ONE_RESOURCE_NO_CHANGE);
    }

    for iface in desired_interfaces.difference(&current_interfaces) {
        tracing::info!("adding {iface} to IPMP group {group}");
        cmd_change_or_noop!(opts, IPADM_BIN, "add-ipmp", "-i", iface, group)?;
    }

    for iface in current_interfaces.difference(desired_interfaces) {
        tracing::info!("removing {iface} from IPMP group {group}");
        cmd_change_or_noop!(opts, IPADM_BIN, "remove-ipmp", "-i", iface, group)?;
    }

    Ok(ONE_RESOURCE_ONE_CHANGE)
}

impl IpmpRemove {
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        if ipmp_exists(&self.name)? {
            tracing::info!("Removing {}", self.name);
            align_interfaces(&self.name, &HashSet::new(), opts)?;
            cmd_change_or_noop!(opts, IPADM_BIN, "delete-ipmp", &self.name)
        } else {
            tracing::debug!("{} does not exist", self.name);
            Ok(ONE_RESOURCE_NO_CHANGE)
        }
    }
}

fn ipmp_exists(name: &LinkName) -> anyhow::Result<bool> {
    let group_list = cmd_output!(IPMPSTAT_BIN, "-Pg", "-o", "group")?;
    Ok(group_list.lines().any(|l| l == name.to_string()))
}

fn ipmpstat_output() -> anyhow::Result<String> {
    cmd_output!(IPMPSTAT_BIN, "-Pg", "-o", "group,interfaces")
}

fn ipmp_interfaces(raw_output: &str, ipmp_group: &LinkName) -> HashSet<LinkName> {
    raw_output
        .lines()
        .find_map(|l| {
            let mut chunks = l.split(":");
            if let Some(group) = chunks.next()
                && let Some(interfaces) = chunks.next()
                && group == ipmp_group.to_string()
            {
                Some(
                    interfaces
                        .split(" ")
                        .filter_map(|s| LinkName::new(s).ok())
                        .collect::<HashSet<_>>(),
                )
            } else {
                None
            }
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod test {
    use super::*;
    use tester::{deserialized_example, janet2json};

    #[test]
    fn test_ensure_ipmp_deserialize() {
        let example_json = janet2json(r#"(ipmp/ensure "ipmp0" :interfaces["bge0" "e1000g0"])"#);
        let sut: IpmpEnsure = serde_json::from_str(&example_json).unwrap();

        assert_eq!(
            IpmpEnsure {
                id: GurpId::new("/NO-ROLE/ipmp/ipmp0").unwrap(),
                name: LinkName::new("ipmp0").unwrap(),
                interfaces: HashSet::from([
                    LinkName::new("bge0").unwrap(),
                    LinkName::new("e1000g0").unwrap()
                ])
            },
            sut
        );
    }

    #[test]
    fn test_remove_ipmp_deserialize() {
        assert_eq!(
            IpmpRemove {
                id: GurpId::new("/NO-ROLE/ipmp/ipmp1").unwrap(),
                name: LinkName::new("ipmp1").unwrap(),
            },
            deserialized_example("ipmp/remove-01.janet")
        );
    }
}
