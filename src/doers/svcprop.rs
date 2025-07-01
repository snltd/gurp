use crate::common::constants::{
    ONE_RESOURCE_NO_CHANGE, ONE_RESOURCE_NOOP, ONE_RESOURCE_ONE_CHANGE,
};
use crate::common::types::{ApplySummary, Opts};
use crate::utils::helpers;
use anyhow::bail;
use serde::Deserialize;
use std::collections::HashMap;
use std::process::{Command, Stdio};
// use std::sync::LazyLock;

// THINGS TO KNOW / THINGS TO DO.
// As always, extremely limited. Just sets a service property.

const SVCPROP_BIN: &str = "/usr/sbin/svcprop";
const SVCCFG_BIN: &str = "/usr/sbin/svccfg";

// static CURRENT_SVCCFG_OUTPUT: LazyLock<Vec<String>> =
//     LazyLock::new(|| svcprop_output().expect("Could not get svcprop list"));

// A chunk of text from svcprop(8).
// fn svcprop_output() -> anyhow::Result<Vec<String>> {
//     let mut cmd = Command::new(SVCCFG_BIN);
//     cmd.arg("list").arg("-H").arg("-o").arg("name");

//     tracing::debug!(command = helpers::command_to_string(&cmd));
//     let result = cmd.output()?;

//     Ok(String::from_utf8_lossy(&result.stdout)
//         .lines()
//         .map(|s| s.to_owned())
//         .collect())
// }

//    2  - name: Register data directory property
//    3    ansible.builtin.shell: "/usr/bin/svcprop -p application/datadir {{ svc }}"
//    4    register: current_data_dir
//    5    changed_when: false
//    6
//    7  - name: Set data directory property
//    8    when: current_data_dir.stdout != data_dir
//    9    ansible.builtin.command: "/usr/sbin/svccfg -s {{ svc }} setprop application/datadir={{ data_dir }}"
//   10    notify:
//   11    ╎ - Refresh MariaDB
//   12    ╎ - Restart MariaDB

type PropertyMap = HashMap<String, String>;

#[derive(Debug, Deserialize)]
pub struct GurpSvcpropEnsure {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(rename = "name")]
    pub service: String,
    pub values: PropertyMap,
}

// fn svcprop_state(name: &str) -> anyhow::Result<SvcpropState> {
//     let mut ret = HashMap::new();
//     let mut cmd = Command::new(SVCCFG_BIN);
//     cmd.arg("get")
//         .arg("-pH")
//         .arg("-o")
//         .arg("property,value")
//         .arg("all")
//         .arg(name);

//     tracing::debug!(command = helpers::command_to_string(&cmd));

//     let result = cmd.output()?;

//     for l in String::from_utf8_lossy(&result.stdout).lines() {
//         let bits: Vec<_> = l.split_whitespace().collect();

//         if bits.len() != 2 {
//             continue;
//         }

//         ret.insert(bits[0].to_owned(), bits[1].to_owned());
//     }

//     Ok(ret)
// }

// fn svcprop_exists(name: &str) -> bool {
//     CURRENT_SVCCFG_OUTPUT.contains(&name.to_owned())
// }

type SvcProps = HashMap<String, SvcPropVal>;

struct SvcPropVal {
    prop_type: String,
    value: String,
}

// impl GurpSvcpropEnsure {
    fn svc_property_values(&self, svc: &str) -> anyhow::Result<String> {
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

    fn process_svc_properties(&self, raw: &str) -> SvcProps {
        raw.lines()
            .filter_map(|l| {
                let chunks: Vec<_> = l.split_whitespace().collect();
                if chunks.len() == 3 {
                    // Empty string values show as "". That *might* be a problem one day
                    let value = if chunks[2] == "\"\"" { "" } else { chunks[2] }.to_owned();

                    Some((
                        chunks[0].to_owned(),
                        SvcPropVal {
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

    pub fn apply(&self, opts: &Opts) -> anyhow::Result<ApplySummary> {
        let all_values = &self.process_svc_properties(&self.svc_property_values(&self.service)?);

        let mut svccfg_script = String::new();
        
        for (property, propval) in &self.values {
            if let Some(current_val) = all_values.get(property) {
                if current_val.value == propval {
                    tracing::debug!("{} svcprop {} already {}", &self.service, property, propval);
                    continue;
                }

                svccfg_script.push_str(format!("setprop {property} = 
            }
            
        if svcprop_exists(&self.name) {
            if let Some(state) = self.options.as_ref() {
                let current_state = svcprop_state(&self.name)?;
                let mut run_cmd = false;
                let mut cmd = Command::new(SVCCFG_BIN);
                cmd.arg("set");

                for (property, desired_value) in state {
                    if let Some(current_value) = current_state.get(property) {
                        if current_value == desired_value {
                            tracing::debug!("{}: already {}", property, desired_value);
                        } else {
                            tracing::info!(
                                "change svcprop {}: [{}] {} -> {}",
                                property,
                                self.name,
                                current_value,
                                desired_value,
                            );
                            run_cmd = true;
                            cmd.arg(format!("{property}={desired_value}"));
                        }
                    }
                }

                if run_cmd {
                    cmd.arg(&self.name);
                    tracing::debug!(command = helpers::command_to_string(&cmd));

                    let output = cmd.output()?;

                    if output.status.success() {
                        Ok(ONE_RESOURCE_ONE_CHANGE)
                    } else {
                        bail!(String::from_utf8_lossy(&output.stderr).into_owned())
                    }
                } else {
                    tracing::info!("no change: {}", self.name);
                    Ok(ONE_RESOURCE_NO_CHANGE)
                }
            } else {
                Ok(ONE_RESOURCE_NO_CHANGE)
            }
        } else {
            self.create_filesystem(opts)
        }
    }

    fn create_filesystem(&self, opts: &Opts) -> anyhow::Result<ApplySummary> {
        tracing::info!("creating filesystem: {}", self.name);

        let mut cmd = Command::new(SVCCFG_BIN);
        cmd.arg("create");

        for (property, value) in self.options.as_ref().unwrap() {
            cmd.arg("-o");
            cmd.arg(format!("{property}={value}"));
        }

        if opts.noop {
            cmd.arg("-n");
        }

        cmd.arg(&self.name).stderr(Stdio::piped());
        tracing::debug!(command = helpers::command_to_string(&cmd));
        let output = cmd.output()?;

        if output.status.success() {
            if opts.noop {
                Ok(ONE_RESOURCE_NOOP)
            } else {
                Ok(ONE_RESOURCE_ONE_CHANGE)
            }
        } else {
            bail!(String::from_utf8_lossy(&output.stderr).into_owned())
        }
    }
}

impl GurpSvcpropRemove {
    pub fn apply(&self, opts: &Opts) -> anyhow::Result<ApplySummary> {
        if svcprop_exists(&self.name) {
            tracing::info!("removing filesystem: {}", self.name);

            if opts.noop {
                Ok(ONE_RESOURCE_NOOP)
            } else {
                self.remove_filesystem()
            }
        } else {
            tracing::debug!("not present: {}", self.name);
            Ok(ONE_RESOURCE_NO_CHANGE)
        }
    }

    fn remove_filesystem(&self) -> anyhow::Result<ApplySummary> {
        let mut cmd = Command::new(SVCCFG_BIN);
        cmd.arg("destroy")
            .arg("-r")
            .arg(&self.name)
            .stderr(Stdio::piped());

        tracing::debug!(command = helpers::command_to_string(&cmd));
        let output = cmd.output()?;

        if output.status.success() {
            Ok(ONE_RESOURCE_ONE_CHANGE)
        } else {
            bail!(String::from_utf8_lossy(&output.stderr).into_owned())
        }
    }
}
