use anyhow::Context;
use common::constants::{IPADM_BIN, ONE_RESOURCE_NO_CHANGE, ONE_RESOURCE_ONE_CHANGE};
use common::types::{ApplyOpts, ApplySummary};
use os_types::GurpId;
use os_types::link_name::LinkName;
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt::Debug;
use util::deserializer;
use util::ip_protocols::{self, AlignIpPropArg, IpObjType, IpProtocolMap};

#[derive(Deserialize, Debug)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(rename_all = "kebab-case")]
pub struct IpInterfaceEnsure {
    #[serde(rename = "_id")]
    pub id: GurpId,
    pub name: LinkName,
    #[serde(
        default,
        deserialize_with = "deserializer::option_hash_property_deserializer"
    )]
    pub protocols: Option<IpProtocolMap>,
}

#[derive(Deserialize, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct IpInterfaceRemove {
    #[serde(rename = "_id")]
    pub id: GurpId,
    pub name: LinkName,
}

impl IpInterfaceEnsure {
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        let mut summary = ONE_RESOURCE_NO_CHANGE;

        // The interface

        if interface_exists(&self.name)? {
            tracing::debug!("{} exists", self.name);
        } else {
            tracing::info!("creating {}", self.name);
            summary = cmd_change_or_noop!(opts, IPADM_BIN, "create-if", &self.name.to_string())
                .with_context(|| format!("failed to create ip-interface {}", self.name))?;
        }

        // The properties can only be considered if the interface exists. If we're in the middle
        // of a no-op, it won't

        if interface_exists(&self.name)? {
            let raw = self.current_properties_raw()?;
            let current_properties = ip_protocols::parse_ipadm_props(&raw);

            if let Some(protocols) = &self.protocols {
                for (protocol, properties) in protocols {
                    let no_values = HashMap::new();
                    let current_values = current_properties.get(protocol).unwrap_or(&no_values);

                    for (property, desired_value) in properties {
                        if ip_protocols::align_property(
                            &AlignIpPropArg {
                                ipadm_cmd: "set-ifprop",
                                protocol: Some(protocol),
                                property,
                                current_value: current_values
                                    .get(property.as_str())
                                    .map(String::as_str),
                                desired_value,
                                protocol_requires_flag: true,
                                ip_object: Some(&IpObjType::Link(&self.name)),
                            },
                            opts,
                        )? {
                            summary = ONE_RESOURCE_ONE_CHANGE
                        }
                    }
                }
            }
        } else if opts.noop {
            tracing::info!("cannot consider new interface props in a no-op");
        }

        Ok(summary)
    }

    fn current_properties_raw(&self) -> anyhow::Result<String> {
        cmd_output!(
            IPADM_BIN,
            "show-ifprop",
            "-c",
            "-o",
            "proto,property,current",
            &self.name.to_string(),
        )
        .with_context(|| format!("failed to get properties of ip-interface {}", self.name))
    }
}

impl IpInterfaceRemove {
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        if interface_exists(&self.name)? {
            tracing::info!("removing {}", self.name);
            cmd_change_or_noop!(opts, IPADM_BIN, "delete-if", &self.name.to_string())
                .with_context(|| format!("failed to delete ip-interface {}", self.name))
        } else {
            tracing::debug!("{} does not exist", self.name);
            Ok(ONE_RESOURCE_NO_CHANGE)
        }
    }
}

fn interface_exists(interface_name: &LinkName) -> anyhow::Result<bool> {
    let ipadm_output = cmd_output!(IPADM_BIN, "show-if", "-p", "-o", "ifname")
        .with_context(|| format!("failed to get state of ip-interface {interface_name}"))?;

    Ok(ipadm_output
        .lines()
        .any(|l| l == interface_name.to_string()))
}

#[cfg(test)]
mod test {
    use super::*;
    use tester::{deserialized_example, propmap};

    #[test]
    fn test_ip_interface_deserialize_ensure_interface() {
        assert_eq!(
            IpInterfaceEnsure {
                name: LinkName::new("example0").unwrap(),
                id: GurpId::new("/NO-ROLE/ip-interface/example0").unwrap(),
                protocols: None,
            },
            deserialized_example("ip-interface/ensure-interface.janet")
        );
    }

    #[test]
    fn test_ip_interface_deserialize_ensure_interface_with_options_and_label() {
        assert_eq!(
            IpInterfaceEnsure {
                name: LinkName::new("example1").unwrap(),
                id: GurpId::new("/NO-ROLE/ip-interface/example-interface").unwrap(),
                protocols: Some(HashMap::from([
                    (
                        "ipv4".to_owned(),
                        propmap! {
                            "mtu" => "1500",
                            "forwarding" => "on",

                        }
                    ),
                    (
                        "ipv6".to_owned(),
                        propmap! {
                            "mtu" => "1500",
                            "forwarding" => "off",

                        }
                    )
                ]))
            },
            deserialized_example("ip-interface/ensure-interface-with-options-and-label.janet")
        );
    }

    #[test]
    fn test_ip_interface_deserialize_remove_interface() {
        assert_eq!(
            IpInterfaceRemove {
                name: LinkName::new("example2").unwrap(),
                id: GurpId::new("/NO-ROLE/ip-interface/example2").unwrap(),
            },
            deserialized_example("ip-interface/remove-interface.janet")
        );
    }
}
