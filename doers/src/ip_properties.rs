// use anyhow::{Context, ensure};
use common::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt::Debug;
// use std::process::Command;
use util::deserializer;

// THINGS TO KNOW / THINGS TO DO.
// You can't use the ipadm set-prop +/-
// There is no remove resource.

type Protocol = String;
type ProtocolProps = HashMap<String, String>;
type Properties = HashMap<Protocol, ProtocolProps>;

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct GurpIpPropertiesEnsure {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
    #[serde(default, deserialize_with = "deserializer::hash_property_deserializer")]
    pub properties: HashMap<Protocol, ProtocolProps>,
}

impl GurpIpPropertiesEnsure {
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        let mut changes = 0;
        let mut resources = 0;
        let current_properties = self.current_properties()?;

        for (protocol, properties) in &self.properties {
            let empty_table = HashMap::new();
            let lookup_table = current_properties.get(protocol).unwrap_or(&empty_table);

            for (property, desired_value) in properties {
                resources += 1;
                if let Some(current_value) = lookup_table.get(property) {
                    if current_value == desired_value {
                        tracing::debug!("{}/{} already {}", protocol, property, current_value);
                    } else {
                        tracing::info!(
                            "{}/{} changing {} -> {}",
                            protocol,
                            property,
                            current_value,
                            desired_value
                        );

                        changes += 1;
                        if !opts.noop {
                            self.set_property(protocol, property, desired_value)?;
                        }
                    }
                } else {
                    tracing::info!("{}/{} setting to {}", protocol, property, desired_value);
                    changes += 1;

                    if !opts.noop {
                        self.set_property(protocol, property, desired_value)?;
                    }
                }
            }
        }

        Ok(ApplySummary { resources, changes })
    }

    fn set_property(&self, protocol: &str, property: &str, value: &str) -> anyhow::Result<()> {
        cmd_output!(
            IPADM_BIN,
            "set-prop",
            "-p",
            format!("{property}={value}"),
            protocol
        )?;
        Ok(())
    }

    fn current_properties(&self) -> anyhow::Result<Properties> {
        let ipadm_output =
            cmd_output!(IPADM_BIN, "show-prop", "-c", "-o", "proto,property,current")?;

        Ok(parse_prop_info(&ipadm_output))
    }
}

fn parse_prop_info(raw: &str) -> Properties {
    let mut ret: Properties = HashMap::new();

    for line in raw.lines() {
        let chunks: Vec<_> = line.split(':').collect();

        if chunks.len() == 3 {
            let protocol_hash = ret.entry(chunks[0].to_owned()).or_default();
            protocol_hash.insert(chunks[1].to_owned(), chunks[2].to_owned());
        }
    }

    ret
}

#[cfg(test)]
mod test {
    use super::*;
    use indoc::indoc;
    use tester::janet2json;

    #[test]
    fn test_parse_prop_info() {
        let expected_ipv4 = HashMap::from([("hostmodel".to_owned(), "weak".to_owned())]);

        let expected_icmp = HashMap::from([
            ("max_buf".to_owned(), "262144".to_owned()),
            ("recv_buf".to_owned(), "8192".to_owned()),
        ]);

        let expected_tcp = HashMap::from([("congestion_control".to_owned(), "sunreno".to_owned())]);

        let expected: Properties = HashMap::from([
            ("ipv4".to_owned(), expected_ipv4),
            ("icmp".to_owned(), expected_icmp),
            ("tcp".to_owned(), expected_tcp),
        ]);

        // read-only properties are ignored
        let input = indoc! { "
                ipv4:hostmodel:weak
                icmp:max_buf:262144
                icmp:recv_buf:8192
                tcp:congestion_control:sunreno
        "
        };

        assert_eq!(expected, parse_prop_info(input));
    }

    // #[test]
    // fn test_deserialize() {
    //     let json_def = janet2json(indoc! {r#"
    //        (ip-address/ensure "test0/v4"
    //                           :type "static"
    //                           :address "192.168.1.13/24"
    //                           :properties {:prefixlen 24
    //                                        :transmit true
    //                                        :private false})
    //       "#});

    //     let expected_props: AddrProps = HashMap::from([
    //         ("prefixlen".to_owned(), "24".to_owned()),
    //         ("transmit".to_owned(), "on".to_owned()),
    //         ("private".to_owned(), "off".to_owned()),
    //     ]);

    //     let expected = GurpIpAddressEnsure {
    //         id: "/NO-ROLE/ip-address/test0_v4".to_owned(),
    //         name: "test0/v4".to_owned(),
    //         address_type: "static".to_owned(),
    //         address: Some("192.168.1.13/24".to_owned()),
    //         properties: Some(expected_props),
    //     };

    //     assert_eq!(expected, serde_json::from_str(&json_def).unwrap())
    // }
}
