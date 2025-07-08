use crate::common::constants::{
    ONE_RESOURCE_NO_CHANGE, ONE_RESOURCE_NOOP, ONE_RESOURCE_ONE_CHANGE,
};
use crate::common::types::{ApplySummary, Opts};
use crate::utils::helpers;
use anyhow::bail;
// use serde::Deserialize;
use crate::doers::zone::control;
use crate::doers::zone::control::ZoneadmState;
use serde::Deserialize;
use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::sync::LazyLock;

// THINGS TO KNOW / THINGS TO DO.

const ZONECFG_BIN: &str = "/usr/sbin/zonecfg";
const ZONEADM_BIN: &str = "/usr/sbin/zoneadm";
const ZONEADM_FIELDS: usize = 8;

static CURRENT_ZONE_LIST: LazyLock<ZoneadmZones> = LazyLock::new(|| {
    parse_zone_list(&zone_list().expect("Could not get zone list"))
        .expect("Could not parse zone list")
});

type ZoneName = String;
type ZoneadmZones = HashMap<ZoneName, ZoneadmState>;

#[derive(Debug, Deserialize)]
pub struct GurpZoneEnsure {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct GurpZoneRemove {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
}

impl GurpZoneEnsure {
    pub fn apply(&self, opts: &Opts) -> anyhow::Result<ApplySummary> {
        if CURRENT_ZONE_LIST.contains_key(&self.name) {
            println!("ZONE {} already exists", self.name);
        } else {
            println!("CREATE ZONE {}", &self.name);
        }

        Ok(ONE_RESOURCE_NO_CHANGE)
    }
}

impl GurpZoneRemove {
    pub fn apply(&self, opts: &Opts) -> anyhow::Result<ApplySummary> {
        if CURRENT_ZONE_LIST.contains_key(&self.name) {
            tracing::info!("zone {}: remove", self.name);
            if opts.noop {
                Ok(ONE_RESOURCE_NOOP)
            } else {
                control::remove_zone(&self.name)
            }
        } else {
            tracing::debug!("zone {}: not found", self.name);
            Ok(ONE_RESOURCE_NO_CHANGE)
        }
    }
}

fn zone_list() -> anyhow::Result<String> {
    let mut cmd = Command::new(ZONEADM_BIN);
    cmd.arg("list").arg("-cp");
    tracing::debug!(command = helpers::command_to_string(&cmd));
    let result = cmd.output()?;
    Ok(String::from_utf8_lossy(&result.stdout).to_string())
}

fn parse_zone_list(raw: &str) -> anyhow::Result<ZoneadmZones> {
    fn chunks_to_struct(chunks: &[&str]) -> anyhow::Result<(ZoneName, ZoneadmState)> {
        if chunks.len() != ZONEADM_FIELDS {
            bail!(
                "expected {} zoneadm fields. Got {}",
                ZONEADM_FIELDS,
                chunks.len()
            );
        }

        Ok((
            chunks[1].to_owned(),
            ZoneadmState {
                status: chunks[2].into(),
                path: chunks[3].into(),
                brand: chunks[5].into(),
                ip: chunks[6].into(),
            },
        ))
    }

    raw.lines()
        .map(|line| chunks_to_struct(&line.split(":").collect::<Vec<_>>()))
        .collect::<anyhow::Result<HashMap<_, _>>>()
}

#[cfg(test)]
mod test {
    use super::*;
    use camino::Utf8PathBuf;
    use indoc::indoc;

    #[test]
    fn test_zone_list() {
        let raw = indoc!(
        "0:global:running:/::ipkg:shared:0
        -:clean-zone:installed:/zones/clean-zone:311a4f36-779f-4d14-bc9d-c85cb9817327:lipkg:excl:216");

        let expected: ZoneadmZones = HashMap::from([
            (
                "global".to_owned(),
                ZoneadmState {
                    status: "running".to_owned(),
                    path: Utf8PathBuf::from("/"),
                    brand: "ipkg".to_owned(),
                    ip: "shared".to_owned(),
                },
            ),
            (
                "clean-zone".to_owned(),
                ZoneadmState {
                    status: "installed".to_owned(),
                    path: Utf8PathBuf::from("/zones/clean-zone"),
                    brand: "lipkg".to_owned(),
                    ip: "excl".to_owned(),
                },
            ),
        ]);

        assert_eq!(expected, parse_zone_list(raw).unwrap());
    }
}
