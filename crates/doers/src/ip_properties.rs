use anyhow::Context;
use common::constants::IPADM_BIN;
use common::types::{ApplyOpts, ApplySummary};
use os_types::GurpId;
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt::Debug;
use util::deserializer;
use util::ip_protocols::{self, AlignIpPropArg, IpProtocolMap};

#[derive(Deserialize, Debug)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(rename_all = "kebab-case")]
pub struct IpPropertiesEnsure {
    #[serde(rename = "_id")]
    pub id: GurpId,
    pub name: String,
    #[serde(default, deserialize_with = "deserializer::hash_property_deserializer")]
    pub protocols: IpProtocolMap,
}

impl IpPropertiesEnsure {
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        let raw = self.current_properties_raw()?;
        let current_properties = ip_protocols::parse_ipadm_props(&raw);
        let mut changes = 0;

        for (protocol, properties) in &self.protocols {
            let no_values = HashMap::new();
            let current_values = current_properties.get(protocol).unwrap_or(&no_values);
            for (property, desired_value) in properties {
                if property == "extra_priv_ports" {
                    if ip_protocols::align_list_property(
                        AlignIpPropArg {
                            ipadm_cmd: "set-prop",
                            protocol: Some(protocol),
                            property,
                            current_value: current_values.get(property).map(String::as_str),
                            desired_value,
                            protocol_requires_flag: false,
                            ip_object: None,
                        },
                        opts,
                    )? {
                        changes += 1;
                    }
                } else if ip_protocols::align_property(
                    &AlignIpPropArg {
                        ipadm_cmd: "set-prop",
                        protocol: Some(protocol),
                        property,
                        current_value: current_values.get(property).map(String::as_str),
                        desired_value,
                        protocol_requires_flag: false,
                        ip_object: None,
                    },
                    opts,
                )? {
                    changes += 1;
                }
            }
        }

        Ok(ApplySummary {
            resources: self.protocols.len() as u32,
            changes,
        })
    }

    fn current_properties_raw(&self) -> anyhow::Result<String> {
        cmd_output!(IPADM_BIN, "show-prop", "-c", "-o", "proto,property,current")
            .context("failed to get ip-properties")
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
    use tester::{deserialized_example, propmap};

    #[test]
    fn test_ip_properties_deserialize_ensure_properties() {
        assert_eq!(
            IpPropertiesEnsure {
                name: "general".to_owned(),
                id: GurpId::new("/NO-ROLE/ip-properties/general").unwrap(),
                protocols: HashMap::from([
                    (
                        "ipv4".to_owned(),
                        propmap! {
                            "forwarding" => "on",

                        }
                    ),
                    (
                        "ipv6".to_owned(),
                        propmap! {
                            "hoplimit" => "250",
                        }
                    ),
                    (
                        "icmp".to_owned(),
                        propmap! {
                            "max_buf" => "262000",
                        }
                    ),
                    (
                        "tcp".to_owned(),
                        propmap! {
                            "sack" => "passive",
                        }
                    ),
                    (
                        "sctp".to_owned(),
                        propmap! {
                            "max_buf" => "1048000",
                        }
                    ),
                    (
                        "udp".to_owned(),
                        propmap! {
                            "extra_priv_ports" => "2050,4040",
                        }
                    ),
                ])
            },
            deserialized_example("ip-properties/ensure-properties.janet")
        );
    }
}
