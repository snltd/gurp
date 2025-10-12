use anyhow::{Context, ensure};
use common::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt::Debug;
use std::process::Command;
use util::deserializer::option_property_deserializer;

// THINGS TO KNOW / THINGS TO DO.

type AddrProps = HashMap<String, String>;

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
    pub properties: Option<AddrProps>,
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

fn describe_addrprops(address_name: &str) -> anyhow::Result<AddrProps> {
    let ipadm_output = cmd_output!(
        IPADM_BIN,
        "show-addrprop",
        "-c",
        "-o",
        "property,perm,current",
        address_name,
    )?;

    Ok(parse_addrprop_info(&ipadm_output))
}

impl GurpIpAddressEnsure {
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        let mut recreate = false;
        let mut create = false;
        let mut changed = false;

        if let Some(now) = describe_address(&self.name)? {
            tracing::debug!("{} exists", self.name);

            // If the address is wrong, I think the only way to fix it is to remove and re-create
            // the address.

            if now.address_type == self.address_type {
                if &self.address_type == "static"
                    && let Some(addr) = &self.address
                    && now.address != *addr
                {
                    tracing::info!(
                        "Changing {} address: {} -> {} (forces recreate)",
                        self.name,
                        now.address,
                        addr,
                    );

                    recreate = true;
                }
            } else {
                tracing::info!(
                    "Changing {} address type: {} -> {} (forces recreate)",
                    self.name,
                    now.address_type,
                    &self.address_type,
                );

                recreate = true;
            }
        } else {
            create = true
        }

        if recreate {
            tracing::info!("Deleting address {}", self.name);
            changed = true;
            self.delete_addr(opts)?;
        }

        if create || recreate {
            tracing::info!("Creating {}", self.name);
            changed = true;
            self.create_addr(opts)?;
        }

        if let Some(desired_props) = &self.properties {
            tracing::debug!("Examining address properties");

            for (prop, current_value) in describe_addrprops(&self.name)? {
                if let Some(desired_value) = desired_props.get(&prop) {
                    if *desired_value == current_value {
                        tracing::debug!("{}/{} already {}", self.name, prop, current_value);
                    } else {
                        tracing::info!(
                            "{}/{} change {} -> {}",
                            self.name,
                            prop,
                            current_value,
                            desired_value
                        );

                        changed = true;

                        if !opts.noop {
                            self.set_addrprop(&prop, desired_value)?;
                        }
                    }
                }
            }
        }

        if changed {
            Ok(ONE_RESOURCE_ONE_CHANGE)
        } else {
            Ok(ONE_RESOURCE_NO_CHANGE)
        }
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

    fn set_addrprop(&self, property: &str, value: &str) -> anyhow::Result<()> {
        cmd_output!(
            IPADM_BIN,
            "set-addrprop",
            "-p",
            format!("{property}={value}"),
            &self.name
        )?;

        Ok(())
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

fn parse_addrprop_info(raw: &str) -> AddrProps {
    let mut ret: AddrProps = HashMap::new();

    for line in raw.lines() {
        let chunks: Vec<_> = line.split(':').collect();

        if chunks.len() == 3 && chunks[1] == "rw" {
            ret.insert(chunks[0].to_owned(), chunks[2].to_owned());
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
    fn test_parse_addrprop_info() {
        let expected: AddrProps = HashMap::from([
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

        assert_eq!(expected, parse_addrprop_info(input));
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

        let expected_props: AddrProps = HashMap::from([
            ("prefixlen".to_owned(), "24".to_owned()),
            ("transmit".to_owned(), "on".to_owned()),
            ("private".to_owned(), "off".to_owned()),
        ]);

        let expected = GurpIpAddressEnsure {
            id: "/NO-ROLE/ip-address/test0_v4".to_owned(),
            name: "test0/v4".to_owned(),
            address_type: "static".to_owned(),
            address: Some("192.168.1.13/24".to_owned()),
            properties: Some(expected_props),
        };

        assert_eq!(expected, serde_json::from_str(&json_def).unwrap())
    }
}
