use anyhow::bail;
use common::constants::{SVC_WAIT_INTERVAL, SVC_WAIT_TIMEOUT, SVCADM_BIN, SVCCFG_BIN, SVCS_BIN};
use std::thread::sleep;
use std::time::Duration;

pub fn current_state(svc: &str) -> anyhow::Result<String> {
    cmd_output!(SVCS_BIN, "-Ho", "state", svc)
}

pub fn run_svccfg(arg1: &str, arg2: &str) -> anyhow::Result<String> {
    cmd_output!(SVCCFG_BIN, arg1, arg2)
}

pub fn exists(svc: &str) -> anyhow::Result<bool> {
    let mut cmd = cmd!(SVCS_BIN, svc);
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

    if current_state == desired_state {
        tracing::debug!("{svc} already in state {desired_state}");
        Ok(String::new())
    } else {
        tracing::info!(
            "changing svc state: {} {} -> {}",
            svc,
            current_state,
            desired_state
        );

        cmd_output!(SVCADM_BIN, action, svc)
    }
}

pub fn wait_for_state(svc: &str, state: &str) -> anyhow::Result<bool> {
    let elapsed = Duration::from_secs(0);

    loop {
        if current_state(svc)? == state {
            return Ok(true);
        }

        sleep(SVC_WAIT_INTERVAL);
        let elapsed = elapsed + SVC_WAIT_INTERVAL;

        if elapsed >= SVC_WAIT_TIMEOUT {
            bail!("Timed out waiting for {} be {}", svc, state)
        }
    }
}
