use anyhow::{Context, ensure};
use common::constants::{SVCADM_BIN, SVCCFG_BIN};
use common::info;
use common::types::{ApplyOpts, ApplySummary};
use regex::Regex;
use serde::Deserialize;
use std::io::Write;
use std::thread::sleep;
use std::time::Duration;
use util::smf_builder::{
    PropertyGroupList, PropertyGroupMap, PropertyList, PropertyMap, PropertyStruct, PropertyValue,
    SvcProps,
};
use util::svcs;

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(rename_all = "lowercase")]
pub enum OnChangeAction {
    Restart,
    Refresh,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(test, derive(PartialEq))]
pub struct GurpSvcpropEnsure {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(rename = "name")]
    pub service: String,
    pub properties: PropertyMap,
    pub property_groups: Option<PropertyGroupMap>,
    pub on_change: Option<OnChangeAction>,
}

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct GurpSvcpropRemove {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(rename = "name")]
    pub service: String,
    pub properties: PropertyList,
    pub property_groups: Option<PropertyGroupList>,
}

#[derive(Debug, Default)]
struct SvcView {
    pub properties: SvcProps,
    pub property_groups: PropertyGroupList,
}

impl GurpSvcpropEnsure {
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        let svc = &self.service;
        ensure_instance(&self.service, opts)?;

        let all_values = current_svc_props(svc)?;
        let mut resources = self.properties.len() as u32;
        let mut changes = 0;
        let mut svccfg_script = String::new();

        if let Some(property_groups) = &self.property_groups {
            resources += property_groups.len() as u32;
            for (property_group, pgtype) in property_groups {
                tracing::debug!("{svc}: looking for pg '{property_group}' property group",);

                if all_values.property_groups.contains(property_group) {
                    tracing::debug!("{svc}: property group '{property_group}' exists",);
                } else {
                    changes += 1;
                    tracing::debug!("{svc}: adding property group '{property_group}'",);
                    svccfg_script.push_str(&format!("addpg {property_group} {pgtype}\n"));
                }
            }
        }

        for (property, desired) in &self.properties {
            tracing::debug!("{}: looking for '{}' property", self.service, property);

            if let Some(current_val) = all_values.properties.get(property) {
                tracing::debug!("{} found {}", self.service, property);
                if current_val.value == desired.value {
                    tracing::debug!("{svc} {property}: already {}", current_val.value);
                    continue;
                } else {
                    tracing::info!(
                        "{svc} {property}: {} -> {}",
                        current_val.value,
                        desired.value,
                    );
                    changes += 1;
                }
            } else {
                tracing::debug!("{svc} svcprop: did not find '{property}'");
            }

            tracing::info!(
                "setting {property} = {}: {}\n",
                desired.prop_type,
                desired.value
            );

            svccfg_script.push_str(&format!(
                "setprop {property} = {}: {}\n",
                desired.prop_type, desired.value
            ));

            changes += 1;
        }

        if svccfg_script.is_empty() {
            tracing::debug!("{svc} svcprop: no change");
        } else {
            tracing::debug!("{svc} svcprop: applying change file");

            if opts.output.dump_configs {
                println!(
                    "{}",
                    info::dump_config(&svccfg_script, Some("svccfg script"), &opts.output)
                );
            }

            if opts.noop {
                return Ok(ApplySummary { resources, changes });
            }

            let mut cmd = cmd_with_stdin!(SVCCFG_BIN, "-s", svc);
            let mut child = cmd.spawn()?;

            if let Some(stdin) = child.stdin.as_mut() {
                stdin.write_all(svccfg_script.as_bytes())?;
            }

            let output = child.wait_with_output().context("error running svccfg")?;

            ensure!(
                output.status.success(),
                String::from_utf8_lossy(&output.stderr).into_owned()
            );

            tracing::debug!("{svc} svcprop: applied svccfg successfully");

            if let Some(action) = &self.on_change {
                match action {
                    OnChangeAction::Refresh => self.apply_action("refresh", opts)?,
                    OnChangeAction::Restart => {
                        self.apply_action("refresh", opts)?;
                        self.apply_action("restart", opts)?;
                    }
                }
            }
        }

        Ok(ApplySummary { resources, changes })
    }

    fn apply_action(&self, action: &str, opts: &ApplyOpts) -> anyhow::Result<()> {
        tracing::debug!("{action}ing svc: {}", self.service);
        sleep(Duration::from_secs(1)); // I hate this, but it appears to make the difference

        if !opts.noop {
            cmd_output!(SVCADM_BIN, action, &self.service)
                .with_context(|| format!("failed to run {SVCADM_BIN} {action} {}", self.service))?;
        }

        Ok(())
    }
}

impl GurpSvcpropRemove {
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        let svc = &self.service;
        let current_state = current_svc_props(svc)?;
        let mut ret = ApplySummary::default();

        for property in &self.properties {
            if let Some(current) = current_state.properties.get(property) {
                tracing::info!(
                    "{svc} svcprop: removing '{property}' (was'{}')",
                    current.value,
                );
                ret += cmd_change_or_noop!(opts, SVCCFG_BIN, "-s", svc, "delprop", property)
                    .with_context(|| format!("failed to delete property {property} from {svc}"))?;
            } else {
                tracing::debug!("{svc} svcprop: no '{property}' property");
            }
        }

        if let Some(property_groups) = &self.property_groups {
            for pg in property_groups {
                tracing::debug!("{svc}: looking for '{pg}' property group");

                if current_state.property_groups.contains(pg) {
                    tracing::debug!("{}: removing property group '{pg}'", self.service,);
                    ret += cmd_change_or_noop!(opts, SVCCFG_BIN, "-s", &self.service, "delpg", pg)
                        .with_context(|| {
                            format!("failed to delete property_group {pg} from {svc}")
                        })?;
                } else {
                    tracing::debug!("{svc}: property group '{pg}' exists");
                }
            }
        }

        Ok(ret)
    }
}

// We inspect only the directly referenced service/instance. No composition.
fn svc_property_values(svc: &str) -> anyhow::Result<String> {
    cmd_output!(SVCCFG_BIN, "-s", svc, "listprop")
        .with_context(|| format!("failed to list properties for {svc}"))
}

// We inspect only the directly referenced service/instance. No composition.
fn svc_property_groups(svc: &str) -> anyhow::Result<String> {
    cmd_output!(SVCCFG_BIN, "-s", svc, "listpg")
        .with_context(|| format!("failed to list property groups for {svc}"))
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
fn ensure_instance(svc: &str, opts: &ApplyOpts) -> anyhow::Result<()> {
    let chunks: Vec<_> = svc.rsplitn(2, ":").collect();

    let instance = chunks
        .first()
        .context(format!("could not get service of {svc}"))?;

    let service = chunks
        .last()
        .context(format!("could not get instance of {svc}"))?;

    if svcs::exists(svc)? {
        tracing::debug!("svc instance {svc} exists");
    } else {
        tracing::debug!("adding instance '{instance}' to service '{service}'");
        let mut cmd = cmd_with_stdin!(SVCCFG_BIN, "-s", &service);

        if opts.noop {
            return Ok(());
        }

        let mut child = cmd.spawn()?;

        if let Some(stdin) = child.stdin.as_mut() {
            stdin.write_all(format!("add {instance}").as_bytes())?;
        }

        let output = child
            .wait_with_output()
            .with_context(|| format!("failed to add instance {instance} to svc {service}"))?;

        ensure!(
            output.status.success(),
            String::from_utf8_lossy(&output.stderr).into_owned()
        );

        tracing::debug!("created instance {}", svc);
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::collections::{BTreeMap, BTreeSet};
    use tester::deserialized_example;

    #[test]
    fn test_deserialize_svcprop_ensure_application_props() {
        assert_eq!(
            GurpSvcpropEnsure {
                service: "example/svc_1".to_owned(),
                id: "/NO-ROLE/svcprop/example_svc_1".to_owned(),
                on_change: Some(OnChangeAction::Restart),
                property_groups: Some(BTreeMap::from([(
                    "application".to_owned(),
                    "application".to_owned()
                ),])),
                properties: BTreeMap::from([
                    (
                        "application/datadir".to_owned(),
                        PropertyStruct {
                            value: PropertyValue::String("/data".to_owned()),
                            prop_type: "astring".to_owned(),
                        }
                    ),
                    (
                        "application/active".to_owned(),
                        PropertyStruct {
                            value: PropertyValue::Bool(true),
                            prop_type: "boolean".to_owned(),
                        }
                    ),
                    (
                        "application/timeout".to_owned(),
                        PropertyStruct {
                            value: PropertyValue::Int(50),
                            prop_type: "integer".to_owned(),
                        }
                    )
                ])
            },
            deserialized_example("svcprop/ensure-application-props.janet")
        );
    }

    #[test]
    fn test_deserialize_svcprop_ensure_group_and_properties() {
        assert_eq!(
            GurpSvcpropEnsure {
                service: "example/svc_1".to_owned(),
                id: "/NO-ROLE/svcprop/example_svc_1".to_owned(),
                on_change: None,
                property_groups: Some(BTreeMap::from([(
                    "application".to_owned(),
                    "application".to_owned()
                ),])),
                properties: BTreeMap::from([(
                    "application/datadir".to_owned(),
                    PropertyStruct {
                        value: PropertyValue::String("/data".to_owned()),
                        prop_type: "astring".to_owned(),
                    }
                ),])
            },
            deserialized_example("svcprop/ensure-group-and-properties.janet")
        );
    }

    #[test]
    fn test_deserialize_svcprop_remove_properties() {
        assert_eq!(
            GurpSvcpropRemove {
                id: "/NO-ROLE/svcprop/example_svc_3".to_owned(),
                service: "example/svc_3".to_owned(),
                properties: BTreeSet::from(["application/thing".to_owned()]),
                property_groups: None,
            },
            deserialized_example("svcprop/remove-properties.janet")
        );
    }

    #[test]
    fn test_process_property_groups() {
        let input = indoc::indoc! { r#"
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
