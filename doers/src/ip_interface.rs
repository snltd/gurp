use common::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt::Debug;
use util::deserializer;
use util::ip_protocols::{self, AlignIpPropArg, IpProtocolMap};

// THINGS TO KNOW / THINGS TO DO.

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct GurpIpInterfaceEnsure {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
    #[serde(default, deserialize_with = "deserializer::hash_property_deserializer")]
    pub protocols: IpProtocolMap,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
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

        for (protocol, properties) in &self.protocols {
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
    use indoc::indoc;
    use tester::janet2json;

    #[test]
    fn test_deserialize() {
        let json_def = janet2json(indoc! {r#"
           (ip-interface/ensure "test0"
                                {:ipv6 {:mtu 1500 :forwarding false}
                                {:ipv4 {:mtu 1505 :forwarding true})
          "#});

        let expected = GurpIpInterfaceEnsure {
            id: "/NO-ROLE/ip-interface/test0".to_owned(),
            name: "test0".to_owned(),
            protocols: HashMap::from([
                (
                    "ipv4".to_owned(),
                    HashMap::from([
                        ("mtu".to_owned(), "1505".to_owned()),
                        ("forwarding".to_owned(), "on".to_owned()),
                    ]),
                ),
                (
                    "ipv6".to_owned(),
                    HashMap::from([
                        ("mtu".to_owned(), "1500".to_owned()),
                        ("forwarding".to_owned(), "off".to_owned()),
                    ]),
                ),
            ]),
        };

        assert_eq!(expected, serde_json::from_str(&json_def).unwrap())
    }
}
