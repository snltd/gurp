use crate::prelude::*;
use std::process::{Command, Stdio};

pub fn current_state(svc: &str) -> anyhow::Result<String> {
    let mut cmd = Command::new(SVCS_BIN);
    cmd.arg("-Ho").arg("state").arg(svc).stderr(Stdio::piped());

    tracing::debug!(command = helpers::command_to_string(&cmd));

    let output = cmd.output()?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
    } else {
        bail!(String::from_utf8_lossy(&output.stderr).into_owned())
    }
}

pub fn run_svcadm(svc: &str, action: &str) -> anyhow::Result<String> {
    let mut cmd = Command::new(SVCADM_BIN);
    cmd.arg(action).arg(svc).stderr(Stdio::piped());

    tracing::debug!(command = helpers::command_to_string(&cmd));
    let output = cmd.output()?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        bail!(String::from_utf8_lossy(&output.stderr).into_owned())
    }
}

pub fn run_svccfg(arg1: &str, arg2: &str) -> anyhow::Result<String> {
    let mut cmd = Command::new(SVCCFG_BIN);
    cmd.arg(arg1).arg(arg2).stderr(Stdio::piped());

    tracing::debug!(command = helpers::command_to_string(&cmd));
    let output = cmd.output()?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        bail!(String::from_utf8_lossy(&output.stderr).into_owned())
    }
}

pub fn exists(svc: &str) -> anyhow::Result<bool> {
    let mut cmd = Command::new(SVCS_BIN);
    cmd.arg(svc);
    tracing::debug!(command = helpers::command_to_string(&cmd));
    let output = cmd.output()?;

    if output.status.success() {
        Ok(true)
    } else if String::from_utf8_lossy(&output.stderr).contains("doesn't match any instances") {
        Ok(false)
    } else {
        bail!(String::from_utf8_lossy(&output.stderr).into_owned())
    }
}

pub fn set_state(svc: &str, current_state: &str, desired_state: &str) -> anyhow::Result<String> {
    let action = if current_state == "maintenance" {
        tracing::debug!("trying to clear svc: {}", svc);
        "clear"
    } else if desired_state == "online" {
        "enable"
    } else if desired_state == "disabled" {
        "disable"
    } else {
        bail!("unknown or unsupported state: {}", desired_state);
    };

    tracing::info!(
        "changing svc state: {} {} -> {}",
        svc,
        current_state,
        desired_state
    );
    run_svcadm(svc, action)
}
