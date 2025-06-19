use crate::common::constants::{
    NO_RESOURCES_TO_CHANGE, ONE_RESOURCE_ONE_CHANGE, ONE_RESOURCE_ONE_ERROR,
};
use crate::common::output::Output;
use crate::common::traits::Apply;
use crate::common::types::{Action, ApplyContext, ApplySummary, Opts, Resource};
use crate::utils::janet_helpers::JanetExt;
use crate::utils::janet_helpers::{self, JanetExt, JanetStructExt};
use crate::{debug, warn};
use anyhow::Context;
use colored::Colorize;
use janetrs::{Janet, JanetArray, JanetKeyword};
use paste::paste;
use std::collections::HashMap;
use std::process::Command;
use std::sync::LazyLock;

// THINGS TO KNOW / THINGS TO DO.

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
        let state =

        Ok(GurpZfs {
            name,
            id: data.get_field_string("_id")?,
            action,
            exists,
            desired_state:
            data.get_field_struct(options)?;
            doer: "zfs".to_owned(),
        })
    }
}
