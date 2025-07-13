use crate::utils::helpers;
use anyhow::bail;
use std::env;
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

pub fn run_zlogin_cmd(zone: &str, command: &str) -> anyhow::Result<()> {
    // Pass the RUST_LOG env var through, because we may be running an instance of this program
    let mut cmd = Command::new(ZLOGIN_BIN);
    cmd.arg(zone);
    cmd.env("RUST_LOG", env::var_os("RUST_LOG").unwrap_or_default());
    cmd.arg(command);
    cmd.stdout(Stdio::inherit());
    cmd.stderr(Stdio::inherit());

    tracing::debug!(command = helpers::command_to_string(&cmd));

    let status = cmd.status()?;

    if !status.success() {
        anyhow::bail!("command failed with status: {}", status);
    }

    let output = cmd.output()?;

    if output.status.success() {
        Ok(())
    } else {
        bail!(String::from_utf8_lossy(&output.stderr).into_owned())
    }
}
