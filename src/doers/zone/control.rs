use crate::prelude::*;
use std::str::FromStr;
use std::thread::sleep;
use std::time::Duration;

const SVCS_BIN: &str = "/bin/svcs";
const ZONEADM_FIELDS: usize = 8;
const READY_SVC: &str = "svc:/milestone/multi-user-server:default";
const STATE_WAIT_INTERVAL: Duration = Duration::from_secs(1);
const STATE_WAIT_TIMEOUT: Duration = Duration::from_secs(60);
const READINESS_WAIT_INTERVAL: Duration = Duration::from_secs(2);
const READINESS_WAIT_TIMEOUT: Duration = Duration::from_secs(60);

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
    let raw = cmd_output!(ZONEADM_BIN, "-z", zone_name, "list", "-p")?;
    let chunks: Vec<_> = raw.split(":").collect();

    if chunks.len() != ZONEADM_FIELDS {
        bail!(
            "expected {} zoneadm fields. Got {}",
            ZONEADM_FIELDS,
            chunks.len()
        );
    }

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

fn unmount_zone(zone: &str) -> anyhow::Result<()> {
    tracing::debug!("zone {}: halting", zone);
    cmd_output!(ZONEADM_BIN, "-z", zone, "unmount")?;
    wait_for_state(zone, ZoneState::Halted)
}

// I've seen things (bhyve) get stuck here, but I can't reproduce anything right now
fn halt_zone(zone: &str) -> anyhow::Result<()> {
    tracing::debug!("zone {}: halting", zone);
    cmd_output!(ZONEADM_BIN, "-z", zone, "halt")?;
    wait_for_state(zone, ZoneState::Installed)
}

fn uninstall_zone(zone: &str) -> anyhow::Result<()> {
    tracing::debug!("zone {}: uninstall", zone);
    cmd_output!(ZONEADM_BIN, "-z", zone, "uninstall", "-F")?;
    wait_for_state(zone, ZoneState::Configured)
}

// We may want to clean up ZFS filesystems here as well
fn delete_zone(zone: &str) -> anyhow::Result<String> {
    tracing::debug!("zone {}: delete", zone);
    cmd_output!(ZONECFG_BIN, "-z", zone, "delete", "-F")
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

pub fn wait_for_readiness(zone: &str) -> anyhow::Result<()> {
    // This goes a bit further than waiting for the zone state. It checks it's up and in multi-user
    // mode
    let elapsed = Duration::from_secs(0);
    loop {
        if is_ready(zone)? {
            return Ok(());
        }

        sleep(READINESS_WAIT_INTERVAL);
        let elapsed = elapsed + READINESS_WAIT_INTERVAL;

        if elapsed >= READINESS_WAIT_TIMEOUT {
            bail!("Timed out waiting for {} be ready", zone)
        }
    }
}

pub fn wait_for_readiness_lx(zone: &str) -> anyhow::Result<()> {
    // Because there are a bunch of possible images, it's hard to know what to look for here. For
    // starters I'm going to try, "are you running half-a-dozen processes"?
    //
    let elapsed = Duration::from_secs(0);
    loop {
        if is_ready_lx(zone)? {
            return Ok(());
        }

        sleep(READINESS_WAIT_INTERVAL);
        let elapsed = elapsed + READINESS_WAIT_INTERVAL;

        if elapsed >= READINESS_WAIT_TIMEOUT {
            bail!("Timed out waiting for {} be ready", zone)
        }
    }
}

fn is_ready(zone: &str) -> anyhow::Result<bool> {
    let mut cmd = cmd!(SVCS_BIN, "-z", zone, "-Ho", "state", READY_SVC);

    let output = cmd.output()?;

    if output.status.success() {
        let status = String::from_utf8_lossy(&output.stdout);
        Ok(status.trim() == "online")
    } else {
        Ok(false)
    }
}

fn is_ready_lx(zone: &str) -> anyhow::Result<bool> {
    let ps_output = cmd_output!(PS_BIN, "-e", "-z", zone)?;
    Ok(ps_output.lines().count() > 5)
}
