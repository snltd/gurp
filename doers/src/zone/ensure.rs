use crate::zone::config::{Brand, ZoneConfig};
use crate::zone::{bhyve, container, control, helpers};
use anyhow::bail;
use common::constants::{ONE_RESOURCE_NO_CHANGE, ONE_RESOURCE_ONE_CHANGE, ZONECFG_BIN};
use common::types::{ApplyOpts, ApplySummary};
use common::{cmd, info};
use os_types::GurpId;
use serde::Deserialize;
use std::io::Write;
use std::process::{Command, Stdio};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct ZoneEnsure {
    #[serde(rename = "_id")]
    pub id: GurpId,
    pub name: String,
    #[serde(flatten)]
    pub config: ZoneConfig,
}

impl ZoneEnsure {
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        let zone = &self.name;

        // This is used for Bhyve Cloudinit
        let uuid = Uuid::new_v4();

        let config_input = self.config.to_zonecfg(&uuid);

        if helpers::current_zone_list()?.contains_key(zone) {
            tracing::debug!("zone {zone}: already exists");

            if self.recreate() {
                tracing::info!("zone {zone}: remove");
                control::remove_zone(zone)?;
            } else {
                return Ok(ONE_RESOURCE_NO_CHANGE);
            }
        }

        tracing::info!("Must create zone {zone}");

        if opts.output.dump_configs {
            println!(
                "{}",
                info::dump_config(&config_input, Some("zonecfg config"), &opts.output)
            );
        }

        if opts.noop {
            Ok(ONE_RESOURCE_ONE_CHANGE)
        } else {
            self.create_from_config(&config_input)?;

            match self.config.brand {
                Brand::Bhyve => bhyve::build_zone(&self.name, &self.config, &uuid, opts),
                _ => container::build_zone(&self.name, &self.config, opts),
            }?;

            self.set_final_state()?;

            Ok(ONE_RESOURCE_ONE_CHANGE)
        }
    }

    fn set_final_state(&self) -> anyhow::Result<()> {
        match self.config.final_state.as_deref() {
            Some("reboot") => control::reboot_zone(&self.name),
            Some("installed") => control::halt_zone(&self.name),
            Some(_) => bail!("Only supported final states are 'reboot', 'installed'"),
            None => Ok(()),
        }
    }

    fn create_from_config(&self, config: &str) -> anyhow::Result<()> {
        let mut cmd = Command::new(ZONECFG_BIN);
        cmd.arg("-z")
            .arg(&self.name)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        tracing::debug!(command = cmd::to_string(&cmd));

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

    fn recreate(&self) -> bool {
        if self.config.recreate == 0 {
            false
        } else {
            let num = rand::random_range(1..=self.config.recreate);
            tracing::debug!("zone recreate random: {} == {}", self.config.recreate, num);
            num == 1
        }
    }
}
