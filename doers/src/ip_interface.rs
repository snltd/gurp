use common::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt::Debug;
// use std::process::Command;

// THINGS TO KNOW / THINGS TO DO.

type IfProperties = HashMap<String, String>;

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct GurpIpInterfaceEnsure {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
    pub over: String,
    pub properties: Option<IfProperties>,
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
            changed = true;
        }

        if let Some(desired_properties) = &self.properties {
            // We will ignore any properties ipadm doesn't know about
            for (prop, val) in &self.current_ifprops()? {
                if let Some(cv) = desired_properties.get(prop) {
                    if cv == val {
                        tracing::debug!("{}:{} already {}", self.name, prop, val);
                    } else {
                        changed = true;
                        tracing::info!("{}:{} changing {} -> {}", self.name, prop, val, cv);

                        if !opts.noop {
                            self.set_property(prop, cv)?;
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

    fn set_property(&self, property: &str, value: &str) -> anyhow::Result<()> {
        cmd_output!(
            IPADM_BIN,
            "set-ifprop",
            "-p",
            &format!("{property}={value}"),
            &self.name
        )?;
        Ok(())
    }

    fn current_ifprops(&self) -> anyhow::Result<IfProperties> {
        let ipadm_output = cmd_output!(
            IPADM_BIN,
            "show-ifprop",
            "-c",
            "-o",
            "property,current",
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
