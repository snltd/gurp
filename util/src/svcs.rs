use anyhow::{Context, bail};
use common::constants::{SVC_WAIT_INTERVAL, SVC_WAIT_TIMEOUT, SVCADM_BIN, SVCCFG_BIN, SVCS_BIN};
use common::types::ApplyOpts;
use std::thread;
use std::time::Duration;

const STATE_RETRIES: u64 = 5;

/// It is possible that a service from a recently installed package might not be available yet.
/// I've seen this with Squid. So, let's do this on a retry.
pub fn current_state(svc: &str) -> anyhow::Result<String> {
    let mut attempt = 1;

    loop {
        match cmd_output!(SVCS_BIN, "-Ho", "state", svc) {
            Ok(s) => return Ok(s),
            Err(e) => {
                if attempt == STATE_RETRIES {
                    bail!("failed to get state of service {svc}: {e}")
                } else {
                    let sleepy_time = attempt * attempt * 500;
                    tracing::debug!(
                        "attempt {attempt} failed to get state of {svc}: retrying in {sleepy_time}s"
                    );
                    thread::sleep(Duration::from_micros(sleepy_time));
                    attempt += 1;
                }
            }
        }
    }
}

pub fn run_svccfg(arg1: &str, arg2: &str) -> anyhow::Result<String> {
    cmd_output!(SVCCFG_BIN, arg1, arg2)
        .with_context(|| format!("failed to run {SVCCFG_BIN} {arg1} {arg2}"))
}

pub fn exists(svc: &str) -> anyhow::Result<bool> {
    let mut cmd = cmd!(SVCS_BIN, svc);
    let output = cmd
        .output()
        .with_context(|| format!("failed to run {SVCS_BIN} {svc}"))?;

    if output.status.success() {
        Ok(true)
    } else if String::from_utf8_lossy(&output.stderr).contains("doesn't match any instances") {
        Ok(false)
    } else {
        bail!(String::from_utf8_lossy(&output.stderr).into_owned())
    }
}

pub fn set_state(
    svc: &str,
    current_state: &str,
    desired_state: &str,
    opts: &ApplyOpts,
) -> anyhow::Result<String> {
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

        let mut cmd = cmd!(SVCADM_BIN, action, svc);

        if opts.noop {
            Ok("noop".to_owned())
        } else {
            run_cmd!(cmd).with_context(|| format!("failed to run {SVCADM_BIN} {action} {svc}"))
        }
    }
}

pub fn wait_for_state(svc: &str, state: &str) -> anyhow::Result<bool> {
    let elapsed = Duration::from_secs(0);

    loop {
        if current_state(svc)? == state {
            return Ok(true);
        }

        thread::sleep(SVC_WAIT_INTERVAL);
        let elapsed = elapsed + SVC_WAIT_INTERVAL;

        if elapsed >= SVC_WAIT_TIMEOUT {
            bail!("Timed out waiting for {} be {}", svc, state)
        }
    }
}
