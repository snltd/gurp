use crate::utils::helpers;
use anyhow::bail;
use std::ffi::OsStr;
use std::process::{Command, Stdio};

const ZONECFG_BIN: &str = "/usr/sbin/zonecfg";
const ZONEADM_BIN: &str = "/usr/sbin/zoneadm";
const ZLOGIN_BIN: &str = "/usr/sbin/zlogin";

pub fn run_zoneadm<I, S>(zone: &str, subcommand: &str, extra_args: I) -> anyhow::Result<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut cmd = Command::new(ZONEADM_BIN);
    cmd.arg("-z").arg(zone).arg(subcommand);
    cmd.args(extra_args);
    cmd.stderr(Stdio::piped());

    tracing::debug!(command = helpers::command_to_string(&cmd));
    let output = cmd.output()?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        bail!(String::from_utf8_lossy(&output.stderr).into_owned())
    }
}

pub fn run_zonecfg<I, S>(zone: &str, subcommand: &str, extra_args: I) -> anyhow::Result<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut cmd = Command::new(ZONECFG_BIN);
    cmd.arg("-z").arg(zone).arg(subcommand);
    cmd.args(extra_args);
    cmd.stderr(Stdio::piped());

    tracing::debug!(command = helpers::command_to_string(&cmd));
    let output = cmd.output()?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        bail!(String::from_utf8_lossy(&output.stderr).into_owned())
    }
}

pub fn run_zlogin_cmd(zone: &str, command: &str) -> anyhow::Result<String> {
    let mut cmd = Command::new(ZLOGIN_BIN);
    cmd.arg(zone);
    cmd.arg(command);
    cmd.stderr(Stdio::piped());

    tracing::debug!(command = helpers::command_to_string(&cmd));
    let output = cmd.output()?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        bail!(String::from_utf8_lossy(&output.stderr).into_owned())
    }
}
