use crate::common::constants::{
    ONE_RESOURCE_NO_CHANGE, ONE_RESOURCE_NOOP, ONE_RESOURCE_ONE_CHANGE,
};
use crate::common::types::{ApplySummary, Opts};
use crate::debug;
use crate::doers::zone::config::GurpZoneConfig;
use crate::doers::zone::control::ZoneadmState;
use crate::doers::zone::{cmd, control};
use crate::utils::helpers;
use anyhow::bail;
use serde::Deserialize;
use std::collections::HashMap;
use std::io::Write;
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
    #[serde(flatten)]
    pub config: GurpZoneConfig,
}

#[derive(Debug, Deserialize)]
pub struct GurpZoneRemove {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
}

impl GurpZoneEnsure {
    pub fn apply(&self, opts: &Opts) -> anyhow::Result<ApplySummary> {
        let config_input = self.config.to_zonecfg();

        if CURRENT_ZONE_LIST.contains_key(&self.name) {
            tracing::debug!("zone {}: already exists", self.name);

            if self.config.recreate == "always" {
                tracing::info!("zone {}: remove", self.name);
                control::remove_zone(&self.name)?;
            } else {
                return self.modify_from_config(&config_input);
            }
        }

        debug!(
            opts,
            "zone/create", "raw zonecfg config follows:\n{}", &config_input
        );

        if opts.noop {
            Ok(ONE_RESOURCE_NOOP)
        } else {
            self.create_from_config(&config_input)?;
            if let Some(clone_source) = &self.config.clone_from {
                self.clone_zone(clone_source)
            } else {
                self.install_zone()
            }
        }
    }

    fn modify_from_config(&self, _config: &str) -> anyhow::Result<ApplySummary> {
        Ok(ONE_RESOURCE_ONE_CHANGE)
    }

    fn create_from_config(&self, config: &str) -> anyhow::Result<()> {
        let mut cmd = Command::new(ZONECFG_BIN);
        cmd.arg("-z")
            .arg(&self.name)
            .stdin(Stdio::piped())
            .stderr(Stdio::piped());

        tracing::debug!(command = helpers::command_to_string(&cmd));

        let mut child = cmd.spawn()?;

        if let Some(stdin) = child.stdin.as_mut() {
            stdin.write_all(config.as_bytes())?;
        }

        let output = child.wait_with_output()?;

        if output.status.success() {
            tracing::debug!("zone {}: configured successfully", self.name);
            Ok(())
        } else {
            bail!(String::from_utf8_lossy(&output.stderr).into_owned())
        }
    }

    fn install_zone(&self) -> anyhow::Result<ApplySummary> {
        tracing::info!("zone {}: installing", self.name);
        cmd::run_zoneadm(&self.name, "install", std::iter::empty::<&str>())?;
        tracing::debug!("zone {}: installed", self.name);
        self.boot_zone()
        // self.bootstrap_zone()
    }

    fn clone_zone(&self, source_zone: &str) -> anyhow::Result<ApplySummary> {
        tracing::info!("zone {}: installing", self.name);
        cmd::run_zoneadm(&self.name, "clone", [source_zone])?;
        tracing::debug!("zone {}: installed", self.name);
        self.boot_zone()
        // self.bootstrap_zone()
    }

    fn boot_zone(&self) -> anyhow::Result<ApplySummary> {
        if self.config.boot_after_install {
            tracing::debug!("zone {}: booting", self.name);
            cmd::run_zoneadm(&self.name, "boot", std::iter::empty::<&str>())?;
        }

        Ok(ONE_RESOURCE_ONE_CHANGE)
    }
    /*
    fn boostrap_zone(&self) -> anyhow::Result<ApplySummary> {
        let zone_dir = &self.config.zonepath.join("root").join("var").join("tmp");

        if !zone_dir.exists() {
            bail!("bootstrapper cannot find {}", zone_dir);
        }

        let bootstrap_gurp = zone_dir.join("gurp");
        // let bootstrap_conf =

        fs::copy(env::current_exe(), )
    }
    */
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
    use pretty_assertions::assert_eq;

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
