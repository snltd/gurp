use crate::prelude::*;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::io::Write;

// THINGS TO KNOW / THINGS TO DO.
// As always, limited. Sets and removes service properties and property groups. You can't change
// the type of an existing property group.

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct GurpSvcpropEnsure {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(rename = "name")]
    pub service: String,
    pub properties: PropertyMap,
    pub property_groups: PropertyGroupMap,
}

#[derive(Debug, Deserialize)]
pub struct GurpSvcpropRemove {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(rename = "name")]
    pub service: String,
    pub properties: PropertyList,
    pub property_groups: PropertyGroupList,
}

type PropertyName = String;
type PropertyGroupName = String;
type PropertyGroupType = String;
type PropertyList = Vec<PropertyName>;
type PropertyMap = HashMap<String, PropertyStruct>;
type PropertyGroupMap = HashMap<PropertyGroupName, PropertyGroupType>;
type PropertyGroupList = HashSet<PropertyGroupName>;
type SvcProps = HashMap<PropertyName, PropertyStruct>;

#[derive(Debug, Default)]
struct SvcView {
    pub properties: SvcProps,
    pub property_groups: PropertyGroupList,
}

#[derive(Debug, Deserialize)]
pub struct PropertyStruct {
    pub value: String,
    #[serde(rename = "type")]
    pub prop_type: String,
}

fn svc_property_values(svc: &str) -> anyhow::Result<String> {
    cmd_output!(SVCCFG_BIN, "-s", svc, "listprop")
}

fn svc_property_groups(svc: &str) -> anyhow::Result<String> {
    cmd_output!(SVCCFG_BIN, "-s", svc, "listpg")
}

fn process_property_groups(raw: &str) -> PropertyGroupList {
    raw.lines()
        .filter_map(|l| {
            let chunks: Vec<_> = l.split_whitespace().collect();
            if chunks.len() >= 2 {
                Some(chunks[0].to_owned())
            } else {
                None
            }
        })
        .collect()
}

fn process_svc_properties(raw: &str) -> SvcProps {
    raw.lines()
        .filter_map(|l| {
            let chunks: Vec<_> = l.splitn(3, ' ').collect();
            if chunks.len() == 3 {
                // Empty string values show as "". That *might* be a problem one day
                let value = if chunks[2] == "\"\"" { "" } else { chunks[2] }.to_owned();
                Some((
                    chunks[0].to_owned(),
                    PropertyStruct {
                        prop_type: chunks[1].to_owned(),
                        value: value.replace("\\ ", " "), // svcprop escapes spaces
                    },
                ))
            } else {
                None
            }
        })
        .collect()
}

fn current_svc_props(svc: &str) -> anyhow::Result<SvcView> {
    let raw_properties = svc_property_values(svc)?;
    let raw_property_groups = svc_property_groups(svc)?;

    Ok(SvcView {
        properties: process_svc_properties(&raw_properties),
        property_groups: process_property_groups(&raw_property_groups),
    })
}

impl GurpSvcpropEnsure {
    pub fn apply(&self, opts: &Opts) -> anyhow::Result<ApplySummary> {
        let all_values = current_svc_props(&self.service)?;
        let resources = self.properties.len() as u32 + self.property_groups.len() as u32;
        let mut changes = 0;
        let mut svccfg_script = String::new();

        for (property_group, pgtype) in &self.property_groups {
            tracing::debug!(
                "{}: looking for '{}' property group",
                self.service,
                property_group
            );

            if all_values.property_groups.contains(property_group) {
                tracing::debug!(
                    "{}: property group '{}' exists",
                    self.service,
                    property_group
                );
            } else {
                changes += 1;
                tracing::debug!(
                    "{}: adding property group '{}'",
                    self.service,
                    property_group
                );
                svccfg_script.push_str(&format!("addpg {property_group} {pgtype}\n"));
            }
        }

        for (property, desired) in &self.properties {
            tracing::debug!("{}: looking for '{}' property", self.service, property);

            if let Some(current_val) = all_values.properties.get(property) {
                tracing::debug!("{} found '{}'", self.service, property);
                if current_val.value == desired.value {
                    tracing::debug!(
                        "{}: '{}' already '{}'",
                        &self.service,
                        property,
                        current_val.value
                    );
                    continue;
                }
            } else {
                tracing::debug!("{} svcprop: did not find '{}'", self.service, property);
            }

            let value = if desired.prop_type == "astring" {
                &format!("\"{}\"", desired.value)
            } else {
                &desired.value
            };

            tracing::info!(
                "{} svcprop: setting '{}' to '{}'",
                self.service,
                property,
                value,
            );

            svccfg_script.push_str(&format!(
                "setprop {} = {}: {}\n",
                property, desired.prop_type, value
            ));

            changes += 1;
        }

        if svccfg_script.is_empty() {
            tracing::debug!("{} svcprop: no change", self.service);
        } else {
            tracing::debug!("{} svcprop: applying change file", self.service);
            debug!(
                opts,
                "doer/svcprop", "svccfg input follows:\n{}", svccfg_script
            );

            let mut cmd = cmd_with_stdin!(SVCCFG_BIN, "-s", &self.service);

            if opts.noop {
                return Ok(ApplySummary {
                    resources,
                    changes: 0,
                    errors: 0,
                });
            }

            let mut child = cmd.spawn()?;

            if let Some(stdin) = child.stdin.as_mut() {
                stdin.write_all(svccfg_script.as_bytes())?;
            }

            let output = child.wait_with_output()?;

            if output.status.success() {
                tracing::debug!("{} svcprop: applied successfully", self.service);
            } else {
                bail!(String::from_utf8_lossy(&output.stderr).into_owned())
            }

            sleep(Duration::from_secs(1));
            tracing::debug!("{}: refreshing svc", self.service);
            cmd_output!(SVCADM_BIN, "refresh", &self.service)?;
        }

        Ok(ApplySummary {
            resources,
            changes,
            errors: 0,
        })
    }
}

impl GurpSvcpropRemove {
    pub fn apply(&self, opts: &Opts) -> anyhow::Result<ApplySummary> {
        let all_values = current_svc_props(&self.service)?;
        let resources = self.properties.len() as u32;
        let mut changes = 0;
        let mut errors = 0;
        let mut to_remove = Vec::new();

        for property in &self.properties {
            if let Some(current) = all_values.properties.get(property) {
                tracing::info!(
                    "{} svcprop: removing '{}' (was'{}')",
                    self.service,
                    property,
                    current.value,
                );
                to_remove.push(property);
            } else {
                tracing::debug!("{} svcprop: no '{}' property", self.service, property);
            }

            if to_remove.is_empty() {
                return Ok(ApplySummary {
                    resources,
                    changes: 0,
                    errors: 0,
                });
            }

            for property in &to_remove {
                let mut cmd = cmd!(SVCCFG_BIN, "-s", &self.service, "delprop", property);

                if opts.noop {
                    continue;
                }

                let output = cmd.output()?;

                if output.status.success() {
                    tracing::debug!("{} svcprop: removed '{}'", self.service, property);
                    changes += 1;
                } else {
                    tracing::error!(
                        "error from svccfg: {}",
                        String::from_utf8_lossy(&output.stderr).into_owned()
                    );
                    errors += 1;
                }
            }
        }

        Ok(ApplySummary {
            resources,
            changes,
            errors,
        })
    }
}
