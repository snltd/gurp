use common::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt::Debug;
use util::deserializer;
use util::ip_protocols::{self, AlignIpPropArg, IpProtocolMap};

// THINGS TO KNOW / THINGS TO DO.
// You can't use the ipadm set-prop +/-
// There is no remove resource.

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct GurpIpPropertiesEnsure {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
    #[serde(default, deserialize_with = "deserializer::hash_property_deserializer")]
    pub protocols: IpProtocolMap,
}

impl GurpIpPropertiesEnsure {
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        let raw = self.current_properties_raw()?;
        let current_properties = ip_protocols::parse_ipadm_props(&raw);
        let mut changes = 0;

        for (protocol, properties) in &self.protocols {
            let no_values = HashMap::new();
            let current_values = current_properties.get(protocol).unwrap_or(&no_values);

            for (property, desired_value) in properties {
                if ip_protocols::align_property(AlignIpPropArg {
                    ipadm_cmd: "set-prop",
                    protocol: Some(protocol),
                    property,
                    current_value: current_values.get(property).map(String::as_str),
                    desired_value,
                    pass_protocol_to_ipadm: false,
                    ipadm_final_arg: None,
                    opts,
                })? {
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
    }
}
