use crate::common::constants::ONE_RESOURCE_ONE_CHANGE;
use crate::common::types::ApplySummary;
use crate::doers::zone::cmd;
use anyhow::bail;
use camino::Utf8PathBuf;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub struct ZoneadmState {
    pub status: String,
    pub path: Utf8PathBuf,
    pub brand: String,
    pub ip: String,
}

const ZONEADM_FIELDS: usize = 8;

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
    let raw = cmd::run_zoneadm(zone_name, "list", ["-p"])?;
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

fn unmount_zone(zone: &str) -> anyhow::Result<String> {
    tracing::debug!("zone {}: halting", zone);
    cmd::run_zoneadm(zone, "unmount", std::iter::empty::<&str>())
}

// I've seen things (bhyve) get stuck here, but I can't reproduce anything right now
fn halt_zone(zone: &str) -> anyhow::Result<String> {
    tracing::debug!("zone {}: halting", zone);
    cmd::run_zoneadm(zone, "halt", std::iter::empty::<&str>())
}

fn uninstall_zone(zone: &str) -> anyhow::Result<String> {
    tracing::debug!("zone {}: uninstall", zone);
    cmd::run_zoneadm(zone, "uninstall", ["-F"])
}

// We may want to clean up ZFS filesystems here as well
fn delete_zone(zone: &str) -> anyhow::Result<String> {
    tracing::debug!("zone {}: delete", zone);
    cmd::run_zonecfg(zone, "delete", ["-F"])
}
