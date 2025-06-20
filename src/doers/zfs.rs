use crate::common::constants::{
    ONE_RESOURCE_NO_CHANGE, ONE_RESOURCE_NOOP, ONE_RESOURCE_ONE_CHANGE,
};
use crate::common::output::Output;
use crate::common::traits::Apply;
use crate::common::types::{Action, ApplyContext, ApplySummary, Opts, Resource};
use crate::debug;
use crate::utils::helpers;
use crate::utils::janet_helpers::JanetExt;
use crate::utils::janet_helpers::{self, JanetStructExt};
use anyhow::bail;
use janetrs::{Janet, JanetArray};
use paste::paste;
use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::sync::LazyLock;

// THINGS TO KNOW / THINGS TO DO.
// Very limited in what it can do. Create, destroy, align get/set options. Can't do fixed
// sizes.

const ZFS_BIN: &str = "/usr/sbin/zfs";

static CURRENT_ZFS_OUTPUT: LazyLock<Vec<String>> =
    LazyLock::new(|| zfs_output().expect("Could not get zfs list"));

// A chunk of text from zfs(8).
fn zfs_output() -> anyhow::Result<Vec<String>> {
    let cmd = Command::new(ZFS_BIN)
        .arg("list")
        .arg("-H")
        .arg("-o")
        .arg("name")
        .output()?;

    Ok(String::from_utf8_lossy(&cmd.stdout)
        .lines()
        .map(|s| s.to_owned())
        .collect())
}

pub struct GurpZfs {
    pub action: Action,
    pub exists: bool,
    pub id: String,
    pub name: String,
    pub desired_state: Option<ZfsState>,
    pub doer: String,
}

type ZfsState = HashMap<String, String>;

impl TryFrom<&Janet> for GurpZfs {
    type Error = anyhow::Error;

    fn try_from(value: &Janet) -> anyhow::Result<Self> {
        let data = value.extract_struct()?;
        let action = janet_helpers::action_as_enum(&data)?;
        let name = data.get_field_string("name")?;
        let exists = CURRENT_ZFS_OUTPUT.contains(&name);

        let state = match action {
            Action::Ensure => {
                let state_struct = data.get_field_struct("options")?;
                Some(janet_helpers::struct_to_hash(&state_struct))
            }
            Action::Remove => None,
        };

        Ok(GurpZfs {
            name,
            id: data.get_field_string("_id")?,
            action,
            exists,
            desired_state: state,
            doer: "zfs".to_owned(),
        })
    }
}

crate::unpack_fn!(ensure_list, Zfs, GurpZfs);
crate::unpack_fn!(remove_list, Zfs, GurpZfs);
crate::impl_apply!(GurpZfs);

impl GurpZfs {
    fn current_state(&self) -> anyhow::Result<ZfsState> {
        let cmd = Command::new(ZFS_BIN)
            .arg("get")
            .arg("-pH")
            .arg("-o")
            .arg("property,value")
            .arg("all")
            .arg(&self.name)
            .output()?;

        let mut ret = HashMap::new();

        for l in String::from_utf8_lossy(&cmd.stdout).lines() {
            let bits: Vec<_> = l.split_whitespace().collect();

            if bits.len() != 2 {
                continue;
            }

            ret.insert(bits[0].to_owned(), bits[1].to_owned());
        }

        Ok(ret)
    }

    fn apply_ensure(
        &self,
        _apply_context: &ApplyContext,
        opts: &Opts,
        output: &Output,
    ) -> anyhow::Result<ApplySummary> {
        if self.exists {
            if let Some(state) = self.desired_state.as_ref() {
                let current_state = self.current_state()?;
                let mut run_cmd = false;
                let mut cmd = Command::new(ZFS_BIN);
                cmd.arg("set");

                for (k, v) in state {
                    if let Some(current_val) = current_state.get(k) {
                        if current_val == v {
                            debug!(opts, "doer/zfs", "{} already {}", k, v);
                        } else {
                            output.change(&self.name, current_val, v);
                            run_cmd = true;
                            cmd.arg(format!("{}={}", k, v));
                        }
                    }
                }

                if run_cmd {
                    cmd.arg(&self.name);
                    debug!(opts, "doer/zfs", "{}", helpers::command_to_string(&cmd));

                    let output = cmd.output()?;

                    if output.status.success() {
                        Ok(ONE_RESOURCE_ONE_CHANGE)
                    } else {
                        bail!(String::from_utf8_lossy(&output.stderr).into_owned())
                    }
                } else {
                    output.no_change(&self.name);
                    Ok(ONE_RESOURCE_NO_CHANGE)
                }
            } else {
                Ok(ONE_RESOURCE_NO_CHANGE)
            }
        } else {
            self.create_filesystem(opts, output)
        }
    }

    fn apply_remove(
        &self,
        _apply_context: &ApplyContext,
        opts: &Opts,
        output: &Output,
    ) -> anyhow::Result<ApplySummary> {
        if self.exists {
            output.removing(&self.name);

            if opts.noop {
                Ok(ONE_RESOURCE_NOOP)
            } else {
                self.remove_filesystem(opts)
            }
        } else {
            output.not_present(&self.name);
            Ok(ONE_RESOURCE_NO_CHANGE)
        }
    }

    fn create_filesystem(&self, opts: &Opts, output: &Output) -> anyhow::Result<ApplySummary> {
        output.creating(&self.name);

        let mut cmd = Command::new(ZFS_BIN);
        cmd.arg("create");

        for (property, value) in self.desired_state.as_ref().unwrap() {
            cmd.arg("-o");
            cmd.arg(format!("{}={}", property, value));
        }

        if opts.noop {
            cmd.arg("-n");
        }

        cmd.arg(&self.name).stderr(Stdio::piped());

        debug!(opts, "doer/zfs", "{}", helpers::command_to_string(&cmd));

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

    fn remove_filesystem(&self, opts: &Opts) -> anyhow::Result<ApplySummary> {
        let mut cmd = Command::new(ZFS_BIN);
        cmd.arg("destroy")
            .arg("-r")
            .arg(&self.name)
            .stderr(Stdio::piped());

        debug!(opts, "doer/zfs", "{}", helpers::command_to_string(&cmd));

        let output = cmd.output()?;

        if output.status.success() {
            Ok(ONE_RESOURCE_ONE_CHANGE)
        } else {
            bail!(String::from_utf8_lossy(&output.stderr).into_owned())
        }
    }
}
