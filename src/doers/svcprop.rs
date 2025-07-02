use crate::common::types::{ApplySummary, Opts};
use crate::debug;
use crate::utils::helpers;
use anyhow::bail;
use serde::Deserialize;
use std::collections::HashMap;
use std::io::Write;
use std::process::{Command, Stdio};

// THINGS TO KNOW / THINGS TO DO.
// As always, extremely limited. Just sets and removes service properties.

const SVCPROP_BIN: &str = "/usr/bin/svcprop";
const SVCCFG_BIN: &str = "/usr/sbin/svccfg";

#[derive(Debug, Deserialize)]
pub struct GurpSvcpropEnsure {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(rename = "name")]
    pub service: String,
    pub properties: PropertyMap,
}

type PropertyMap = HashMap<String, PropertyStruct>;

#[derive(Debug, Deserialize)]
pub struct PropertyStruct {
    pub value: String,
    #[serde(rename = "type")]
    pub prop_type: String,
}

#[derive(Debug, Deserialize)]
pub struct GurpSvcpropRemove {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(rename = "name")]
    pub service: String,
    pub properties: Vec<String>,
}

type SvcProps = HashMap<String, PropertyStruct>;

fn svc_property_values(svc: &str) -> anyhow::Result<String> {
    let mut cmd = Command::new(SVCPROP_BIN);
    cmd.arg(svc).stderr(Stdio::piped());

    tracing::debug!(command = helpers::command_to_string(&cmd));
    let output = cmd.output()?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        bail!(String::from_utf8_lossy(&output.stderr).into_owned())
    }
}

fn process_svc_properties(raw: &str) -> SvcProps {
    raw.lines()
        .filter_map(|l| {
            let chunks: Vec<_> = l.splitn(3, ' ').collect();
            if chunks.len() == 3 {
                // Empty string values show as "". That *might* be a problem one day
                let value = if chunks[2] == "\"\"" { "" } else { chunks[2] }.to_owned();

                let value = value.replace("\\ ", " ");

                // svcprop escapes spaces
                //

                Some((
                    chunks[0].to_owned(),
                    PropertyStruct {
                        prop_type: chunks[1].to_owned(),
                        value,
                    },
                ))
            } else {
                None
            }
        })
        .collect()
}

impl GurpSvcpropEnsure {
    pub fn apply(&self, opts: &Opts) -> anyhow::Result<ApplySummary> {
        let all_values = process_svc_properties(&svc_property_values(&self.service)?);
        let resources = self.properties.len() as u32;
        let mut changes = 0;
        let mut svccfg_script = String::new();

        for (property, desired) in &self.properties {
            tracing::debug!("{} svcprop: looking for '{}'", self.service, property);

            if let Some(current_val) = all_values.get(property) {
                tracing::debug!("{} svcprop: found '{}'", self.service, property);
                if current_val.value == desired.value {
                    tracing::debug!(
                        "{} svcprop: '{}' already '{}'",
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
            debug!(opts, "doer/svcprop", "{}", svccfg_script);

            let mut cmd = Command::new(SVCCFG_BIN);
            cmd.arg("-s")
                .arg(&self.service)
                .stdin(Stdio::piped())
                .stderr(Stdio::piped());

            tracing::debug!(command = helpers::command_to_string(&cmd));

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
        let all_values = process_svc_properties(&svc_property_values(&self.service)?);
        let resources = self.properties.len() as u32;
        let mut changes = 0;
        let mut errors = 0;
        let mut to_remove = Vec::new();

        for property in &self.properties {
            if let Some(current) = all_values.get(property) {
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
                let mut cmd = Command::new(SVCCFG_BIN);
                cmd.arg("-s")
                    .arg(&self.service)
                    .arg("delprop")
                    .arg(property)
                    .stderr(Stdio::piped());

                tracing::debug!(command = helpers::command_to_string(&cmd));

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
