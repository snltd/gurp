use crate::zone::constants::*;
use anyhow::{Context, bail, ensure};
use camino::Utf8PathBuf;
use common::constants::{ONE_RESOURCE_ONE_CHANGE, SVCS_BIN, ZONEADM_BIN, ZONECFG_BIN};
use common::types::ApplySummary;
use std::str::FromStr;
use std::thread::sleep;
use std::time::Duration;

#[derive(Debug, PartialEq)]
pub struct ZoneadmState {
    pub status: String,
    pub path: Utf8PathBuf,
    pub brand: String,
    pub ip: String,
}

// State machine to handle zone cleanup
//
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ZoneState {
    Configured,
    Incomplete,
    Installed,
    Ready,
    Running,
    ShuttingDown,
    Mounted,
    Down,
    Halted,
    Unknown,
}

impl FromStr for ZoneState {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> anyhow::Result<Self> {
        match s {
            "configured" => Ok(ZoneState::Configured),
            "incomplete" => Ok(ZoneState::Incomplete),
            "installed" => Ok(ZoneState::Installed),
            "ready" => Ok(ZoneState::Ready),
            "running" => Ok(ZoneState::Running),
            "shutting_down" => Ok(ZoneState::ShuttingDown),
            "down" => Ok(ZoneState::Down),
            "halted" => Ok(ZoneState::Halted),
            "mounted" => Ok(ZoneState::Mounted),
            _ => bail!("Unknown zone state: {}", s),
        }
    }
}

fn zone_state(zone_name: &str) -> anyhow::Result<ZoneState> {
    let raw = cmd_output!(ZONEADM_BIN, "-z", zone_name, "list", "-p")
        .with_context(|| format!("failed to get state of zone {zone_name}"))?;
    let chunks: Vec<_> = raw.split(":").collect();

    ensure!(
        chunks.len() == ZONEADM_FIELDS,
        format!(
            "expected {} zoneadm fields. Got {}",
            ZONEADM_FIELDS,
            chunks.len()
        )
    );

    chunks[2].parse()
}

pub fn remove_zone(zone: &str) -> anyhow::Result<ApplySummary> {
    let mut state = zone_state(zone)?;

    while state != ZoneState::Unknown {
        match state {
            ZoneState::Mounted => {
                unmount_zone(zone)?;
            }
            ZoneState::Running | ZoneState::Ready | ZoneState::ShuttingDown => {
                halt_zone(zone)?;
            }
            ZoneState::Installed | ZoneState::Halted | ZoneState::Down => {
                uninstall_zone(zone)?;
            }
            ZoneState::Configured | ZoneState::Incomplete => {
                delete_zone(zone)?;
                break;
            }
            ZoneState::Unknown => {
                bail!("Unable to determine current zone state");
            }
        }

        std::thread::sleep(std::time::Duration::from_secs(1));
        state = zone_state(zone)?;
    }

    Ok(ONE_RESOURCE_ONE_CHANGE)
}

// I've seen things (bhyve) get stuck here, but I can't reproduce anything right now
pub fn halt_zone(zone: &str) -> anyhow::Result<()> {
    tracing::debug!("zone {}: halting", zone);
    cmd_output!(ZONEADM_BIN, "-z", zone, "halt")
        .with_context(|| format!("failed to halt zone {zone}"))?;
    wait_for_state(zone, ZoneState::Installed)
}

pub fn boot_zone(zone: &str) -> anyhow::Result<()> {
    tracing::debug!("zone {}: booting", zone);
    cmd_output!(ZONEADM_BIN, "-z", zone, "boot")
        .with_context(|| format!("failed to boot zone {zone}"))?;
    Ok(())
}

pub fn reboot_zone(zone: &str) -> anyhow::Result<()> {
    tracing::debug!("zone {}: rebooting", zone);
    cmd_output!(ZONEADM_BIN, "-z", zone, "reboot")
        .with_context(|| format!("failed to reboot zone {zone}"))?;
    Ok(())
}

fn unmount_zone(zone: &str) -> anyhow::Result<()> {
    tracing::debug!("zone {}: halting", zone);
    cmd_output!(ZONEADM_BIN, "-z", zone, "unmount")
        .with_context(|| format!("failed to unmount zone {zone}"))?;
    wait_for_state(zone, ZoneState::Halted)
}

fn uninstall_zone(zone: &str) -> anyhow::Result<()> {
    tracing::debug!("zone {}: uninstall", zone);
    cmd_output!(ZONEADM_BIN, "-z", zone, "uninstall", "-F")
        .with_context(|| format!("failed to uninstall zone {zone}"))?;
    wait_for_state(zone, ZoneState::Configured)
}

// We may want to clean up ZFS filesystems here as well
fn delete_zone(zone: &str) -> anyhow::Result<String> {
    tracing::debug!("zone {}: delete", zone);
    cmd_output!(ZONECFG_BIN, "-z", zone, "delete", "-F").with_context(|| {
        format!(
            "failed to delete
      ▍    ↪ zone {zone}"
        )
    })
}

fn wait_for_state(zone: &str, desired_state: ZoneState) -> anyhow::Result<()> {
    let elapsed = Duration::from_secs(0);

    loop {
        if zone_state(zone)? == desired_state {
            return Ok(());
        }

        sleep(STATE_WAIT_INTERVAL);
        let elapsed = elapsed + STATE_WAIT_INTERVAL;

        if elapsed >= STATE_WAIT_TIMEOUT {
            bail!(
                "Timed out waiting for {} to reach state '{:?}'",
                zone,
                desired_state
            )
        }
    }
}

pub fn wait_for_readiness(zone: &str) -> anyhow::Result<bool> {
    // This goes a bit further than waiting for the zone state. It checks it's up and in multi-user
    // mode. LX and Bhyve have their own versions of this.
    let elapsed = Duration::from_secs(0);
    loop {
        if is_ready(zone)? {
            return Ok(true);
        }

        sleep(READINESS_WAIT_INTERVAL);
        let elapsed = elapsed + READINESS_WAIT_INTERVAL;

        if elapsed >= READINESS_WAIT_TIMEOUT_NATIVE {
            bail!("Timed out waiting for {} be ready", zone)
        }
    }
}

fn is_ready(zone: &str) -> anyhow::Result<bool> {
    // LX and Bhyve provide their own versions of this
    let mut cmd = cmd!(SVCS_BIN, "-z", zone, "-Ho", "state", READY_SVC);

    let output = cmd
        .output()
        .with_context(|| format!("failed to get state of zone {zone}"))?;

    if output.status.success() {
        let status = String::from_utf8_lossy(&output.stdout);
        Ok(status.trim() == "online")
    } else {
        Ok(false)
    }
}
