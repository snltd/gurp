use common::constants::{IPADM_BIN, ONE_RESOURCE_NO_CHANGE, ONE_RESOURCE_ONE_CHANGE};
use common::types::{ApplyOpts, ApplySummary};
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt::Debug;
use util::deserializer;
use util::ip_protocols::{self, AlignIpPropArg, IpProtocolMap};

// THINGS TO KNOW / THINGS TO DO.

#[derive(Deserialize, Debug)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(rename_all = "kebab-case")]
pub struct GurpIpInterfaceEnsure {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
    #[serde(
        default,
        deserialize_with = "deserializer::option_hash_property_deserializer"
    )]
    pub protocols: Option<IpProtocolMap>,
}

#[derive(Deserialize, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct GurpIpInterfaceRemove {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
}

impl GurpIpInterfaceEnsure {
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        let mut summary = ONE_RESOURCE_NO_CHANGE;

        // The interface

        if interface_exists(&self.name)? {
            tracing::debug!("{} exists", self.name);
        } else {
            tracing::info!("creating {}", self.name);

            if !opts.noop {
                cmd_output!(IPADM_BIN, "create-if", &self.name)?;
            }

            summary = ONE_RESOURCE_ONE_CHANGE;
        }

        // The properties

        let raw = self.current_properties_raw()?;
        let current_properties = ip_protocols::parse_ipadm_props(&raw);

        if let Some(protocols) = &self.protocols {
            for (protocol, properties) in protocols {
                let no_values = HashMap::new();
                let current_values = current_properties.get(protocol).unwrap_or(&no_values);

                for (property, desired_value) in properties {
                    if ip_protocols::align_property(AlignIpPropArg {
                        ipadm_cmd: "set-prop",
                        protocol: Some(protocol),
                        property,
                        current_value: current_values.get(property.as_str()).map(String::as_str),
                        desired_value,
                        pass_protocol_to_ipadm: true,
                        ipadm_final_arg: Some(&self.name),
                        opts,
                    })? {
                        summary = ONE_RESOURCE_ONE_CHANGE
                    }
                }
            }
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
            &self.name,
        )
    }
}

impl GurpIpInterfaceRemove {
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        if interface_exists(&self.name)? {
            tracing::info!("removing {}", self.name);
            return_if_noop!(opts);

            cmd_output!(IPADM_BIN, "delete-if", &self.name)?;
            Ok(ONE_RESOURCE_ONE_CHANGE)
        } else {
            tracing::debug!("{} does not exist", self.name);
            Ok(ONE_RESOURCE_NO_CHANGE)
        }
    }
}

fn interface_exists(interface_name: &str) -> anyhow::Result<bool> {
    let ipadm_output = cmd_output!(IPADM_BIN, "show-if", "-p", "-o", "ifname")?;
    Ok(ipadm_output.lines().any(|l| l == interface_name))
}

#[cfg(test)]
mod test {
    use super::*;
    use tester::{deserialized_example, propmap};

    #[test]
    fn test_ip_interface_deserialize_ensure_01() {
        assert_eq!(
            GurpIpInterfaceEnsure {
                name: "example0".to_owned(),
                id: "/NO-ROLE/ip-interface/example0".to_owned(),
                protocols: None,
            },
            deserialized_example::<GurpIpInterfaceEnsure>("ip-interface/ensure-01.janet")
        );
    }

    #[test]
    fn test_ip_interface_deserialize_ensure_02() {
        assert_eq!(
            GurpIpInterfaceEnsure {
                name: "example1".to_owned(),
                id: "/NO-ROLE/ip-interface/example-interface".to_owned(),
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
            deserialized_example::<GurpIpInterfaceEnsure>("ip-interface/ensure-02.janet")
        );
    }

    #[test]
    fn test_ip_interface_deserialize_remove_01() {
        assert_eq!(
            GurpIpInterfaceRemove {
                name: "example2".to_owned(),
                id: "/NO-ROLE/ip-interface/example2".to_owned(),
            },
            deserialized_example::<GurpIpInterfaceRemove>("ip-interface/remove-01.janet")
        );
    }
}
