use common::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt::Debug;
use util::deserializer;

// THINGS TO KNOW / THINGS TO DO.

type Protocols = HashMap<String, IfProperties>;
type IfProperties = HashMap<String, String>;

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct GurpIpInterfaceEnsure {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
    #[serde(default, deserialize_with = "deserializer::hash_property_deserializer")]
    pub protocols: Protocols,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct GurpIpInterfaceRemove {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
}

fn interface_exists(interface_name: &str) -> anyhow::Result<bool> {
    let ipadm_output = cmd_output!(IPADM_BIN, "show-if", "-p", "-o", "ifname")?;
    Ok(ipadm_output.lines().any(|l| l == interface_name))
}

fn parse_ifprop(raw: &str) -> IfProperties {
    let mut ret = HashMap::new();

    for l in raw.lines() {
        let mut chunks = l.split(':');

        if let Some(key) = chunks.next()
            && let Some(value) = chunks.next()
        {
            ret.insert(key.to_owned(), value.to_owned());
        }
    }

    ret
}

impl GurpIpInterfaceEnsure {
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        let mut changed = false;

        // Create if necessary
        if interface_exists(&self.name)? {
            tracing::debug!("{} exists", self.name);
        } else {
            tracing::info!("creating {}", self.name);

            if !opts.noop {
                cmd_output!(IPADM_BIN, "create-if", &self.name)?;
            }

            changed = true;
        }

        for (protocol, properties) in &self.protocols {
            // We will ignore any properties ipadm doesn't know about
            for (prop, current_val) in &self.current_ifprops(protocol)? {
                if let Some(desired_val) = properties.get(prop) {
                    println!("desired={:?} current={:?}", desired_val, current_val);
                    if desired_val == current_val {
                        tracing::debug!("{}:{} already {}", self.name, prop, current_val);
                    } else {
                        changed = true;
                        tracing::info!(
                            "{}:{}/{} changing {} -> {}",
                            self.name,
                            prop,
                            protocol,
                            current_val,
                            desired_val
                        );

                        if !opts.noop {
                            self.set_property(prop, protocol, desired_val)?;
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

    fn set_property(&self, property: &str, protocol: &str, value: &str) -> anyhow::Result<()> {
        cmd_output!(
            IPADM_BIN,
            "set-ifprop",
            "-p",
            &format!("{property}={value}"),
            "-m",
            protocol,
            &self.name
        )?;
        Ok(())
    }

    fn current_ifprops(&self, protocol: &str) -> anyhow::Result<IfProperties> {
        let ipadm_output = cmd_output!(
            IPADM_BIN,
            "show-ifprop",
            "-c",
            "-o",
            "property,current",
            "-m",
            &protocol,
            &self.name,
        )?;

        Ok(parse_ifprop(&ipadm_output))
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

#[cfg(test)]
mod test {
    use super::*;
    use indoc::indoc;
    use tester::janet2json;

    #[test]
    fn test_deserialize() {
        let json_def = janet2json(indoc! {r#"
           (ip-interface/ensure "test0"
                                (ip-interface-protocol "ipv6"
                                                       :mtu 1500
                                                       :forwarding false)
                                (ip-interface-protocol "ipv4"
                                                       :mtu 1505
                                                       :forwarding true))
          "#});

        let expected_ipv4: IfProperties = HashMap::from([
            ("mtu".to_owned(), "1505".to_owned()),
            ("forwarding".to_owned(), "on".to_owned()),
        ]);

        let expected_ipv6: IfProperties = HashMap::from([
            ("mtu".to_owned(), "1500".to_owned()),
            ("forwarding".to_owned(), "off".to_owned()),
        ]);

        let expected = GurpIpInterfaceEnsure {
            id: "/NO-ROLE/ip-interface/test0".to_owned(),
            name: "test0".to_owned(),
            protocols: HashMap::from([
                ("ipv4".to_owned(), expected_ipv4),
                ("ipv6".to_owned(), expected_ipv6),
            ]),
        };

        assert_eq!(expected, serde_json::from_str(&json_def).unwrap())
    }

    #[test]
    fn test_parse_ifprop() {
        let input = indoc! { "
                    arp:on
                    metric:0
                    standby:off"
        };

        let expected: IfProperties = HashMap::from([
            ("arp".to_owned(), "on".to_owned()),
            ("metric".to_owned(), "0".to_owned()),
            ("standby".to_owned(), "off".to_owned()),
        ]);

        assert_eq!(expected, parse_ifprop(input));
    }
}
