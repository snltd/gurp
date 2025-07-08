use crate::common::constants::ONE_RESOURCE_ONE_CHANGE;
use crate::common::types::{ApplySummary, Opts};
use crate::utils::helpers;
use anyhow::bail;
use camino::Utf8PathBuf;
use std::ffi::OsStr;
use std::{
    process::{Command, Stdio},
    str::FromStr,
};

#[derive(Debug, PartialEq)]
pub struct ZoneadmState {
    pub status: String,
    pub path: Utf8PathBuf,
    pub brand: String,
    pub ip: String,
}

const ZONECFG_BIN: &str = "/usr/sbin/zonecfg";
const ZONEADM_BIN: &str = "/usr/sbin/zoneadm";
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
    let raw = run_zoneadm(zone_name, "list", &["-p"])?;
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

fn run_zoneadm<I, S>(zone: &str, subcommand: &str, extra_args: I) -> anyhow::Result<String>
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

fn run_zonecfg<I, S>(zone: &str, subcommand: &str, extra_args: I) -> anyhow::Result<String>
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

fn unmount_zone(zone: &str) -> anyhow::Result<String> {
    tracing::debug!("zone {}: halting", zone);
    run_zoneadm(zone, "unmount", std::iter::empty::<&str>())
}

// I've seen things (bhyve) get stuck here, but I can't reproduce anything right now
fn halt_zone(zone: &str) -> anyhow::Result<String> {
    tracing::debug!("zone {}: halting", zone);
    run_zoneadm(zone, "halt", std::iter::empty::<&str>())
}

fn uninstall_zone(zone: &str) -> anyhow::Result<String> {
    tracing::debug!("zone {}: uninstall", zone);
    run_zoneadm(zone, "uninstall", &["-F"])
}

// We may want to clean up ZFS filesystems here as well
fn delete_zone(zone: &str) -> anyhow::Result<String> {
    tracing::debug!("zone {}: delete", zone);
    run_zonecfg(zone, "delete", &["-F"])
}
