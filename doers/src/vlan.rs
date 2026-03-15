use common::constants::{DLADM_BIN, ONE_RESOURCE_NO_CHANGE, ONE_RESOURCE_ONE_CHANGE};
use common::types::{ApplyOpts, ApplySummary, VlanID};
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt::Debug;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(test, derive(PartialEq))]
pub struct GurpVlanEnsure {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
    pub over: String,
    pub vlan_tag: VlanID,
}

#[derive(Deserialize, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct GurpVlanRemove {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
}

type Vlans = HashMap<String, VlanInfo>;

// I don't think the flags will be useful to us
#[derive(Debug, PartialEq)]
struct VlanInfo {
    over: String,
    vlan_tag: VlanID,
}

impl GurpVlanEnsure {
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        let extant_vlans = parse_vlans(&get_vlans()?);

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

        let mut cmd = cmd!(
            DLADM_BIN,
            "create-vlan",
            "-l",
            &self.over,
            "-v",
            &self.vlan_tag.to_string(),
            &self.name
        );

        return_if_noop!(opts);

        run_cmd!(cmd)?;
        Ok(ONE_RESOURCE_ONE_CHANGE)
    }
}

impl GurpVlanRemove {
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        let extant_vlans = parse_vlans(&get_vlans()?);

        if extant_vlans.contains_key(&self.name) {
            delete_vlan(&self.name, opts)
        } else {
            tracing::debug!("{} does not exist", self.name);
            Ok(ONE_RESOURCE_NO_CHANGE)
        }
    }
}

fn delete_vlan(name: &str, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
    tracing::info!("removing VLAN {name}");
    let mut cmd = cmd!(DLADM_BIN, "delete-vlan", name);

    return_if_noop!(opts);

    run_cmd!(cmd)?;

    Ok(ONE_RESOURCE_ONE_CHANGE)
}

fn parse_vlans(raw: &str) -> Vlans {
    raw.lines()
        .filter_map(|l| {
            let mut chunks = l.trim().split(':');

            if let Some(name) = chunks.next()
                && let Some(vid_str) = chunks.next()
                && let Some(over) = chunks.next()
                && let Some(vid) = vid_str.parse::<u16>().ok()
            {
                Some((
                    name.to_string(),
                    VlanInfo {
                        over: over.to_string(),
                        vlan_tag: vid,
                    },
                ))
            } else {
                None
            }
        })
        .collect()
}

fn get_vlans() -> anyhow::Result<String> {
    cmd_output!(DLADM_BIN, "show-vlan", "-p", "-olink,vid,over")
}

#[cfg(test)]
mod test {
    use super::*;
    use tester::deserialized_example;

    #[test]
    fn test_deserialize_ensure_vlan_10() {
        assert_eq!(
            GurpVlanEnsure {
                id: "/NO-ROLE/vlan/e1000g010".to_owned(),
                name: "e1000g010".to_owned(),
                over: "e1000g0".to_owned(),
                vlan_tag: 10,
            },
            deserialized_example("vlan/ensure-vlan-10.janet")
        );
    }

    #[test]
    fn test_deserialize_remove_vlan() {
        assert_eq!(
            GurpVlanRemove {
                id: "/NO-ROLE/vlan/e1000g010".to_owned(),
                name: "e1000g010".to_owned(),
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

        let expected: Vlans = HashMap::from([
            (
                "e1000g4000".to_owned(),
                VlanInfo {
                    over: "e1000g0".to_owned(),
                    vlan_tag: 4,
                },
            ),
            (
                "gibbus0".to_owned(),
                VlanInfo {
                    over: "e1000g0".to_owned(),
                    vlan_tag: 5,
                },
            ),
        ]);

        assert_eq!(expected, parse_vlans(input));
    }
}
