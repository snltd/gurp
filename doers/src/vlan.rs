use anyhow::Context;
use common::constants::{DLADM_BIN, ONE_RESOURCE_NO_CHANGE};
use common::types::{ApplyOpts, ApplySummary, VlanID};
use os_types::GurpId;
use os_types::link_name::LinkName;
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt::Debug;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(test, derive(PartialEq))]
pub struct GurpVlanEnsure {
    #[serde(rename = "_id")]
    pub id: GurpId,
    pub name: LinkName,
    pub over: LinkName,
    pub vlan_tag: VlanID,
}

#[derive(Deserialize, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct GurpVlanRemove {
    #[serde(rename = "_id")]
    pub id: GurpId,
    pub name: LinkName,
}

type VlanMap = HashMap<LinkName, VlanInfo>;

// I don't think the flags will be useful to us
#[derive(Debug, PartialEq)]
struct VlanInfo {
    over: LinkName,
    vlan_tag: VlanID,
}

impl GurpVlanEnsure {
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        let extant_vlans = vlan_map(&raw_vlan_info()?)?;

        if let Some(extant_vlan) = extant_vlans.get(&self.name) {
            if extant_vlan.over == self.over && extant_vlan.vlan_tag == self.vlan_tag {
                tracing::debug!("VLAN {} exists and is correct", self.name);
                Ok(ONE_RESOURCE_NO_CHANGE)
            } else {
                tracing::info!("recreating VLAN {}", self.name);
                let _ = delete_vlan(&self.name, opts)?;
                self.create_vlan(opts)
            }
        } else {
            self.create_vlan(opts)
        }
    }

    fn create_vlan(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        tracing::info!("creating VLAN {}", self.name);

        cmd_change_or_noop!(
            opts,
            DLADM_BIN,
            "create-vlan",
            "-l",
            &self.over.to_string(),
            "-v",
            &self.vlan_tag.to_string(),
            &self.name.to_string(),
        )
        .with_context(|| format!("failed to create VLAN object {}", self.name))
    }
}

impl GurpVlanRemove {
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        let extant_vlans = vlan_map(&raw_vlan_info()?)?;

        if extant_vlans.contains_key(&self.name) {
            delete_vlan(&self.name, opts)
        } else {
            tracing::debug!("{} does not exist", self.name);
            Ok(ONE_RESOURCE_NO_CHANGE)
        }
    }
}

fn delete_vlan(name: &LinkName, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
    tracing::info!("removing VLAN {name}");
    cmd_change_or_noop!(opts, DLADM_BIN, "delete-vlan", name.to_string())
        .with_context(|| format!("failed to delete VLAN object {name}"))
}

fn vlan_map(raw: &str) -> anyhow::Result<VlanMap> {
    raw.lines().map(|l| parse_vlan_line(l.trim())).collect()
}

fn parse_vlan_line(line: &str) -> anyhow::Result<(LinkName, VlanInfo)> {
    let mut chunks = line.split(':');

    let name = chunks
        .next()
        .with_context(|| format!("vlan line {line} does not have name field"))?;

    let vlan_id_field = chunks
        .next()
        .with_context(|| format!("vlan line {line} does not have vlan id field"))?;

    let over = chunks
        .next()
        .with_context(|| format!("vlan line {line} does not have over field"))?;

    let vlan_id = vlan_id_field
        .parse::<u16>()
        .with_context(|| format!("invalid vlan id '{vlan_id_field}'"))?;

    Ok((
        LinkName::new(name)?,
        VlanInfo {
            over: LinkName::new(over)?,
            vlan_tag: vlan_id,
        },
    ))
}

fn raw_vlan_info() -> anyhow::Result<String> {
    cmd_output!(DLADM_BIN, "show-vlan", "-p", "-olink,vid,over")
        .context("failed to list VLAN objects")
}

#[cfg(test)]
mod test {
    use super::*;
    use tester::deserialized_example;

    #[test]
    fn test_deserialize_ensure_vlan_10() {
        assert_eq!(
            GurpVlanEnsure {
                id: GurpId::new("/NO-ROLE/vlan/e1000g10000").unwrap(),
                name: LinkName::new("e1000g10000").unwrap(),
                over: LinkName::new("e1000g0").unwrap(),
                vlan_tag: 10,
            },
            deserialized_example("vlan/ensure-vlan-10.janet")
        );
    }

    #[test]
    fn test_deserialize_remove_vlan() {
        assert_eq!(
            GurpVlanRemove {
                id: GurpId::new("/NO-ROLE/vlan/e1000g10000").unwrap(),
                name: LinkName::new("e1000g10000").unwrap(),
            },
            deserialized_example("vlan/remove-vlan.janet")
        );
    }

    #[test]
    fn test_parse_vlans() {
        let input = indoc::indoc! { "
            e1000g4000:4:e1000g0
            gibbus0:5:e1000g0
        "};

        let expected: VlanMap = HashMap::from([
            (
                LinkName::new("e1000g4000").unwrap(),
                VlanInfo {
                    over: LinkName::new("e1000g0").unwrap(),
                    vlan_tag: 4,
                },
            ),
            (
                LinkName::new("gibbus0").unwrap(),
                VlanInfo {
                    over: LinkName::new("e1000g0").unwrap(),
                    vlan_tag: 5,
                },
            ),
        ]);

        assert_eq!(expected, vlan_map(input).unwrap());
    }
}
