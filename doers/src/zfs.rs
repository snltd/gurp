use byte_unit::Byte;
use common::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::sync::LazyLock;

// THINGS TO KNOW / THINGS TO DO.
// Very limited in what it can do. Create, destroy, align get/set properties. Can't do fixed
// sizes.

static CURRENT_ZFS_OUTPUT: LazyLock<Vec<String>> =
    LazyLock::new(|| zfs_output().expect("Could not get zfs list"));

fn zfs_output() -> anyhow::Result<Vec<String>> {
    Ok(cmd_output!(ZFS_BIN, "list", "-H", "-o", "name")?
        .lines()
        .map(|s| s.to_owned())
        .collect())
}

#[derive(Debug, Deserialize)]
pub struct GurpZfsEnsure {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
    pub properties: Option<ZfsProperties>,
}

type ZfsProperties = HashMap<String, String>;

#[derive(Debug, Deserialize)]
pub struct GurpZfsRemove {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
}

fn zfs_state(name: &str) -> anyhow::Result<ZfsProperties> {
    let mut ret = HashMap::new();
    let prop_vals = cmd_output!(ZFS_BIN, "get", "-pHo", "property,value", "all", name)?;

    for l in prop_vals.lines() {
        let bits: Vec<_> = l.split_whitespace().collect();

        if bits.len() != 2 {
            continue;
        }

        ret.insert(bits[0].to_owned(), bits[1].to_owned());
    }

    Ok(ret)
}

fn zfs_exists(name: &str) -> bool {
    CURRENT_ZFS_OUTPUT.contains(&name.to_owned())
}

impl GurpZfsEnsure {
    pub fn apply(&self, opts: &Opts) -> anyhow::Result<ApplySummary> {
        if zfs_exists(&self.name) {
            tracing::debug!("zfs: {} exists", &self.name);
            if let Some(state) = self.properties.as_ref() {
                let current_state = zfs_state(&self.name)?;
                let mut run_cmd = false;
                let mut cmd = Command::new(ZFS_BIN);
                cmd.arg("set");

                for (property, desired_value) in state {
                    if let Some(current_value) = current_state.get(property) {
                        if current_value == desired_value {
                            tracing::debug!("{}: already {}", property, desired_value);
                        } else {
                            // Catch size properties. Putting the iB is a nasty, but it works
                            if let Ok(desired_bytes) =
                                Byte::parse_str(format!("{desired_value}iB"), true)
                                && desired_value.ends_with(['M', 'G', 'k', 'E'])
                                && desired_bytes.to_string() == *current_value
                            {
                                break;
                            }

                            tracing::info!(
                                "change zfs {}: [{}] {} -> {}",
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
                    tracing::debug!("no change: {}", self.name);
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

        let mut cmd = Command::new(ZFS_BIN);
        cmd.arg("create");

        if let Some(properties) = &self.properties {
            for (property, value) in properties {
                cmd.arg("-o");
                cmd.arg(format!("{property}={value}"));
            }
        }

        if opts.noop {
            cmd.arg("-n");
        }

        cmd.arg(&self.name).stderr(Stdio::piped());
        tracing::debug!(command = helpers::command_to_string(&cmd));
        let output = cmd.output()?;

        if output.status.success() {
            return_if_noop!(opts);
            Ok(ONE_RESOURCE_ONE_CHANGE)
        } else {
            bail!(String::from_utf8_lossy(&output.stderr).into_owned())
        }
    }
}

impl GurpZfsRemove {
    pub fn apply(&self, opts: &Opts) -> anyhow::Result<ApplySummary> {
        tracing::debug!("zfs: looking for {}", self.name);
        if zfs_exists(&self.name) {
            tracing::info!("removing filesystem: {}", self.name);
            return_if_noop!(opts);
            self.remove_filesystem(opts)
        } else {
            tracing::debug!("not present: {}", self.name);
            Ok(ONE_RESOURCE_NO_CHANGE)
        }
    }

    fn remove_filesystem(&self, opts: &Opts) -> anyhow::Result<ApplySummary> {
        let mut cmd = cmd!(ZFS_BIN, "destroy", "-r", &self.name);
        return_if_noop!(opts);
        one_change_or_stderr!(cmd)
    }
}
