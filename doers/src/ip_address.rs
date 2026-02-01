use anyhow::{Context, bail, ensure};
use common::constants::{IPADM_BIN, ONE_RESOURCE_NO_CHANGE, ONE_RESOURCE_ONE_CHANGE};
use common::helpers;
use common::types::{ApplyOpts, ApplySummary};
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt::Debug;
use std::process::Command;
use util::deserializer::option_property_deserializer;
use util::ip_protocols::{self, AlignIpPropArg};

// THINGS TO KNOW / THINGS TO DO.

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct GurpIpAddressEnsure {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub address_type: String,
    pub address: Option<String>,
    #[serde(default, deserialize_with = "option_property_deserializer")]
    pub properties: Option<IpAddressPropMap>,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct GurpIpAddressRemove {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
}

#[derive(Debug, PartialEq)]
struct IpAddressObject {
    name: String,
    address_type: String,
    state: String,
    address: String,
}

type IpAddressPropMap = HashMap<String, String>;

impl GurpIpAddressEnsure {
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        let mut recreate_interface = false;
        let mut create_interface = false;
        let mut summary = ApplySummary::default();

        // The address

        if let Some(current) = describe_address(&self.name)? {
            tracing::debug!("{} exists", self.name);

            // If the IP address is wrong, I think the only way to fix it is to remove and re-create
            // the address object.

            if current.address_type == self.address_type {
                if &self.address_type == "static"
                    && let Some(desired_address) = &self.address
                    && current.address != *desired_address
                {
                    tracing::info!(
                        "Changing {} address: {} -> {} (forces recreate)",
                        self.name,
                        current.address,
                        desired_address,
                    );

                    recreate_interface = true;
                }
            } else {
                tracing::info!(
                    "Changing {} address type: {} -> {} (forces recreate)",
                    self.name,
                    current.address_type,
                    &self.address_type,
                );

                recreate_interface = true;
            }
        } else {
            create_interface = true
        }

        if recreate_interface {
            tracing::info!("Deleting address {}", self.name);
            self.delete_addr(opts)?;
        }

        if create_interface || recreate_interface {
            tracing::info!("Creating {}", self.name);
            summary = ONE_RESOURCE_ONE_CHANGE;
            self.create_addr(opts)?;
        }

        // The properties

        if let Some(desired_props) = &self.properties {
            tracing::debug!("Examining address properties");
            let raw = self.current_properties_raw()?;
            let current_values = parse_address_props(&raw);

            for (property, desired_value) in desired_props {
                if ip_protocols::align_property(AlignIpPropArg {
                    ipadm_cmd: "set-addrprop",
                    protocol: None,
                    property,
                    current_value: current_values.get(property).map(String::as_str),
                    desired_value,
                    pass_protocol_to_ipadm: false,
                    ipadm_final_arg: None,
                    opts,
                })? {
                    summary = ONE_RESOURCE_ONE_CHANGE
                }
            }
        }

        Ok(summary)
    }

    fn delete_addr(&self, opts: &ApplyOpts) -> anyhow::Result<()> {
        if !opts.noop {
            cmd_output!(IPADM_BIN, "delete-addr", &self.name)?;
        }

        Ok(())
    }

    fn create_addr(&self, opts: &ApplyOpts) -> anyhow::Result<()> {
        let mut cmd = Command::new(IPADM_BIN);
        cmd.arg("create-addr");
        cmd.arg("-T");

        match self.address_type.as_str() {
            "static" => {
                cmd.arg("static");
                cmd.arg("-a");
                cmd.arg(format!(
                    "local={}",
                    self.address.as_ref().context("static IP but no address")?
                ));
            }
            "dhcp" => {
                cmd.arg("dhcp");
            }
            other => bail!("unknown address type: {other}"),
        }

        cmd.arg(&self.name);

        tracing::debug!(command = helpers::command_to_string(&cmd));

        if !opts.noop {
            let result = cmd.output()?;

            if !result.status.success() {
                bail!(String::from_utf8_lossy(&result.stderr).into_owned())
            }
        }

        Ok(())
    }

    fn current_properties_raw(&self) -> anyhow::Result<String> {
        cmd_output!(
            IPADM_BIN,
            "show-addrprop",
            "-c",
            "-o",
            "property,perm,current",
            &self.name,
        )
    }
}

impl GurpIpAddressRemove {
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        if describe_address(&self.name)?.is_some() {
            tracing::info!("removing {}", self.name);
            return_if_noop!(opts);

            cmd_output!(IPADM_BIN, "delete-addr", &self.name)?;
            Ok(ONE_RESOURCE_ONE_CHANGE)
        } else {
            tracing::debug!("{} does not exist", self.name);
            Ok(ONE_RESOURCE_NO_CHANGE)
        }
    }
}

fn describe_address(address_name: &str) -> anyhow::Result<Option<IpAddressObject>> {
    let ipadm_output = cmd_output!(
        IPADM_BIN,
        "show-addr",
        "-p",
        "-o",
        "addrobj,type,state,addr"
    )?;

    let info = ipadm_output
        .lines()
        .filter_map(|l| parse_addr_info(l).ok())
        .find(|l| l.name == address_name);

    Ok(info)
}

fn parse_addr_info(raw: &str) -> anyhow::Result<IpAddressObject> {
    let chunks: Vec<_> = raw.split(':').collect();

    ensure!(
        chunks.len() == 4,
        format!("Expected four components in ipadm output {raw}")
    );

    Ok(IpAddressObject {
        name: chunks[0].to_owned(),
        address_type: chunks[1].to_owned(),
        state: chunks[2].to_owned(),
        address: chunks[3].to_owned(),
    })
}

pub fn parse_address_props(raw: &str) -> IpAddressPropMap {
    let mut ret = HashMap::new();

    for line in raw.lines() {
        let mut chunks = line.split(':');

        if let Some(property) = chunks.next()
            && let Some(perms) = chunks.next()
            && let Some(value) = chunks.next()
            && perms == "rw"
        {
            ret.insert(property.to_owned(), value.to_owned());
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
    fn test_parse_addr_info() {
        let expected = IpAddressObject {
            name: "e1000g0/v4".to_owned(),
            address_type: "static".to_owned(),
            state: "ok".to_owned(),
            address: "192.168.1.5/24".to_owned(),
        };

        assert_eq!(
            expected,
            parse_addr_info("e1000g0/v4:static:ok:192.168.1.5/24").unwrap()
        );
    }

    #[test]
    fn test_parse_address_props() {
        let expected = HashMap::from([
            ("deprecated".to_owned(), "off".to_owned()),
            ("prefixlen".to_owned(), "24".to_owned()),
        ]);

        // read-only properties are ignored
        let input = indoc! { "
            broadcast:r-:192.168.1.255
            deprecated:rw:off
            prefixlen:rw:24
            primary:r-:
        "
        };

        assert_eq!(expected, parse_address_props(input));
    }

    #[test]
    fn test_deserialize() {
        let json_def = janet2json(indoc! {r#"
           (ip-address/ensure "test0/v4"
                              :type "static"
                              :address "192.168.1.13/24"
                              :properties {:prefixlen 24
                                           :transmit true
                                           :private false})
          "#});

        let expected = GurpIpAddressEnsure {
            id: "/NO-ROLE/ip-address/test0_v4".to_owned(),
            name: "test0/v4".to_owned(),
            address_type: "static".to_owned(),
            address: Some("192.168.1.13/24".to_owned()),
            properties: Some(HashMap::from([
                ("prefixlen".to_owned(), "24".to_owned()),
                ("transmit".to_owned(), "on".to_owned()),
                ("private".to_owned(), "off".to_owned()),
            ])),
        };

        assert_eq!(expected, serde_json::from_str(&json_def).unwrap())
    }
}
