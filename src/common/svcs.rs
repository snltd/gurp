use crate::common::types::Opts;
use crate::utils::helpers;
use crate::{debug, verbose};
use anyhow::bail;
use std::process::{Command, Stdio};

const SVCCFG_BIN: &str = "/usr/sbin/svccfg";
const SVCADM_BIN: &str = "/usr/sbin/svcadm";
const SVCS_BIN: &str = "/bin/svcs";

pub fn current_state(svc: &str, opts: &Opts) -> anyhow::Result<String> {
    let mut cmd = Command::new(SVCS_BIN);
    cmd.arg("-Ho").arg("state").arg(svc).stderr(Stdio::piped());

    debug!(opts, "common/svcs", "{}", helpers::command_to_string(&cmd));

    let output = cmd.output()?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
    } else {
        bail!(String::from_utf8_lossy(&output.stderr).into_owned())
    }
}

pub fn run_svcadm(svc: &str, action: &str, opts: &Opts) -> anyhow::Result<String> {
    let mut cmd = Command::new(SVCADM_BIN);
    cmd.arg(action).arg(svc).stderr(Stdio::piped());

    debug!(opts, "common/svcs", "{}", helpers::command_to_string(&cmd));
    let output = cmd.output()?;

    if output.status.success() {
        Ok(String::from_utf8(output.stdout)?)
    } else {
        bail!(String::from_utf8(output.stderr)?)
    }
}

pub fn run_svccfg(arg1: &str, arg2: &str, opts: &Opts) -> anyhow::Result<String> {
    let mut cmd = Command::new(SVCCFG_BIN);
    cmd.arg(arg1).arg(arg2).stderr(Stdio::piped());

    debug!(opts, "common/svcs", "{}", helpers::command_to_string(&cmd));
    let output = cmd.output()?;

    if output.status.success() {
        Ok(String::from_utf8(output.stdout)?)
    } else {
        bail!(String::from_utf8(output.stderr)?)
    }
}

pub fn exists(svc: &str, opts: &Opts) -> anyhow::Result<bool> {
    let mut cmd = Command::new(SVCS_BIN);
    cmd.arg(svc);
    debug!(opts, "common/svcs", "{}", helpers::command_to_string(&cmd));
    let output = cmd.output()?;

    if output.status.success() {
        Ok(true)
    } else if String::from_utf8_lossy(&output.stderr).contains("doesn't match any instances") {
        Ok(false)
    } else {
        bail!(String::from_utf8(output.stderr)?)
    }
}

pub fn set_state(
    svc: &str,
    current_state: &str,
    desired_state: &str,
    opts: &Opts,
) -> anyhow::Result<String> {
    let action = if current_state == "maintenance" {
        debug!(opts, "doer/svc", "Trying to clear {}", svc);
        "clear"
    } else if desired_state == "online" {
        "enable"
    } else if desired_state == "disabled" {
        "disable"
    } else {
        bail!("unknown or unsupported state: {}", desired_state);
    };

    verbose!(opts, "transitioning {} to {}", svc, desired_state);
    run_svcadm(svc, action, opts)
}
