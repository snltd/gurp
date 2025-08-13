use anyhow::Context;
use common::prelude::*;
use regex::Regex;
use serde::Deserialize;
use std::io::Write;
use std::thread::sleep;
use std::time::Duration;
use util::svcs;

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

#[derive(Debug, Default)]
struct SvcView {
    pub properties: SvcProps,
    pub property_groups: PropertyGroupList,
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

pub fn parse_svccfg_line(line: &str) -> Option<(String, PropertyStruct)> {
    let rx = Regex::new(r"\s+").unwrap();
    let mut chunks = rx.splitn(line, 3);

    let key = chunks.next()?.trim().to_string();
    let prop_type = chunks.next()?.trim();
    let raw_value = chunks.next()?.trim();

    // I can't see that you'd ever want to compare times or counts
    let value = match prop_type {
        "boolean" => match raw_value {
            "true" => PropertyValue::Bool(true),
            "false" => PropertyValue::Bool(false),
            _ => return None,
        },
        "integer" => raw_value.parse::<i64>().ok().map(PropertyValue::Int)?,
        "astring" => {
            let stripped = raw_value.trim_matches('"').to_string();
            PropertyValue::String(stripped)
        }
        _ => return None,
    };

    Some((
        key,
        PropertyStruct {
            value,
            prop_type: prop_type.to_owned(),
        },
    ))
}

fn process_properties(raw: &str) -> SvcProps {
    raw.lines().filter_map(parse_svccfg_line).collect()
}

fn current_svc_props(svc: &str) -> anyhow::Result<SvcView> {
    let raw_properties = svc_property_values(svc)?;
    let raw_property_groups = svc_property_groups(svc)?;

    Ok(SvcView {
        properties: process_properties(&raw_properties),
        property_groups: process_property_groups(&raw_property_groups),
    })
}

// It's possible we're being asked to set properties on a service instance which does not yet exist.
fn ensure_instance(svc: &str, opts: &Opts) -> anyhow::Result<()> {
    let chunks: Vec<_> = svc.rsplitn(2, ":").collect();

    let instance = chunks
        .first()
        .context(format!("could not get service of {svc}"))?;

    let service = chunks
        .last()
        .context(format!("could not get instance of {svc}"))?;

    if svcs::exists(svc)? {
        tracing::debug!("svc instance {} exists", svc);
    } else {
        tracing::debug!("adding instance '{}' to service '{}'", instance, service);
        let mut cmd = cmd_with_stdin!(SVCCFG_BIN, "-s", &service);

        if opts.noop {
            return Ok(());
        }

        let mut child = cmd.spawn()?;

        if let Some(stdin) = child.stdin.as_mut() {
            stdin.write_all(format!("add {instance}").as_bytes())?;
        }

        let output = child.wait_with_output()?;

        if output.status.success() {
            tracing::debug!("created instance {}", svc);
        } else {
            bail!(String::from_utf8_lossy(&output.stderr).into_owned())
        }
    }

    Ok(())
}

impl GurpSvcpropEnsure {
    pub fn apply(&self, opts: &Opts) -> anyhow::Result<ApplySummary> {
        ensure_instance(&self.service, opts)?;

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
                tracing::debug!("{} found {}", self.service, property);
                if current_val.value == desired.value {
                    tracing::debug!(
                        "{} {}: already {}",
                        &self.service,
                        property,
                        current_val.value
                    );
                    continue;
                } else {
                    tracing::info!(
                        "{} {}: {} -> {}",
                        self.service,
                        property,
                        current_val.value,
                        desired.value,
                    );
                }
            } else {
                tracing::debug!("{} svcprop: did not find '{}'", self.service, property);
            }

            svccfg_script.push_str(&format!(
                "setprop {} = {}: {}\n",
                property, desired.prop_type, desired.value
            ));

            changes += 1;
        }

        if svccfg_script.is_empty() {
            tracing::debug!("{} svcprop: no change", self.service);
        } else {
            tracing::debug!("{} svcprop: applying change file", self.service);
            helpers::dump_config(&svccfg_script, "svccfg input", opts);

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

            sleep(Duration::from_secs(1)); // I hate this, but it appears to make the difference
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

#[cfg(test)]
mod test {
    use super::*;
    use indoc::indoc;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_process_property_groups() {
        let input = indoc! { r#"
            general                            framework
            general/enabled                    boolean  true
            restarter                          framework	NONPERSISTENT
            restarter/logfile                  astring  /var/svc/log/system-console-login:default.log
            restarter/start_method_waitstatus  integer  0
            restarter/auxiliary_state          astring  dependencies_satisfied
            restarter/start_pid                count    5508
            restarter/state                    astring  online
            restarter/start_method_timestamp   time     1752853169.630361000
            "#
        };

        let propmap = SvcProps::from([
            (
                "restarter/auxiliary_state".into(),
                PropertyStruct {
                    value: PropertyValue::String("dependencies_satisfied".to_owned()),
                    prop_type: "astring".to_owned(),
                },
            ),
            (
                "general/enabled".to_owned(),
                PropertyStruct {
                    value: PropertyValue::Bool(true),
                    prop_type: "boolean".to_owned(),
                },
            ),
            (
                "restarter/start_method_waitstatus".to_owned(),
                PropertyStruct {
                    value: PropertyValue::Int(0),
                    prop_type: "integer".to_owned(),
                },
            ),
            (
                "restarter/logfile".into(),
                PropertyStruct {
                    value: PropertyValue::String(
                        "/var/svc/log/system-console-login:default.log".to_owned(),
                    ),
                    prop_type: "astring".to_owned(),
                },
            ),
            (
                "restarter/state".into(),
                PropertyStruct {
                    value: PropertyValue::String("online".to_owned()),
                    prop_type: "astring".to_owned(),
                },
            ),
        ]);

        assert_eq!(propmap, process_properties(input));
    }
}
