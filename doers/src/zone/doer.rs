use crate::zone::config::GurpZoneConfig;
use crate::zone::control::{self, ZoneadmState};
use crate::zone::lx;
use anyhow::bail;
use common::prelude::*;
use fs_extra::dir::CopyOptions;
use serde::Deserialize;
use std::collections::HashMap;
use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::LazyLock;
use std::{env, fs};

// THINGS TO KNOW / THINGS TO DO.
// Creates and removes zones. Doesn't modify existing ones. Only supports some resources.

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
    fn recreate(&self) -> bool {
        if self.config.recreate == 0 {
            false
        } else {
            let num = rand::random_range(1..=self.config.recreate);
            tracing::debug!("zone recreate random: {} == {}", self.config.recreate, num);
            num == 1
        }
    }

    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        let config_input = self.config.to_zonecfg();

        if CURRENT_ZONE_LIST.contains_key(&self.name) {
            tracing::debug!("zone {}: already exists", self.name);

            if self.recreate() {
                tracing::info!("zone {}: remove", self.name);
                control::remove_zone(&self.name)?;
            } else {
                return Ok(ONE_RESOURCE_NO_CHANGE);
            }
        }

        if opts.dump_config {
            println!(
                "{}",
                helpers::dump_config(&config_input, "zonecfg config", opts)
            );
        }

        if opts.noop {
            Ok(ONE_RESOURCE_NOOP)
        } else {
            self.create_from_config(&config_input)?;
            if let Some(clone_source) = &self.config.clone_from {
                self.clone_zone(clone_source, opts)
            } else {
                self.install_zone(opts)
            }
        }
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

    fn install_zone(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        tracing::info!("installing {} [{}]", self.name, self.config.brand);
        match self.config.brand.as_str() {
            "lx" => {
                let img_path = self.install_zone_lx_path()?;
                cmd_output!(ZONEADM_BIN, "-z", &self.name, "install", "-s", img_path)?
            }
            _ => cmd_output!(ZONEADM_BIN, "-z", &self.name, "install")?,
        };

        tracing::debug!("zone {}: installed", self.name);
        self.boot_zone()?;
        self.postinstall_zone()?;
        self.copy_in()?;
        self.bootstrap_zone(opts)?;
        self.exec_in()?;
        Ok(ONE_RESOURCE_ONE_CHANGE)
    }

    fn postinstall_zone(&self) -> anyhow::Result<()> {
        if self.config.brand.as_str() == "lx"
            && let Some(dns_config) = &self.config.dns
        {
            lx::set_up_dns(&self.config.zonepath, dns_config)?;
        }
        Ok(())
    }

    fn install_zone_lx_path(&self) -> anyhow::Result<Utf8PathBuf> {
        if let Some(img_pattern) = &self.config.image {
            match lx::image_path(img_pattern)? {
                Some(path) => Ok(path),
                None => bail!("did not find a suitable LX image"),
            }
        } else {
            bail!("LX zones require an :image")
        }
    }

    fn clone_zone(&self, source_zone: &str, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        tracing::info!("zone {}: cloning from {}", self.name, source_zone);
        cmd_output!(ZONEADM_BIN, "-z", &self.name, "clone", source_zone)?;
        tracing::debug!("zone {}: cloned", self.name);
        self.boot_zone()?;
        self.postinstall_zone()?;
        self.copy_in()?;
        self.bootstrap_zone(opts)?;
        self.exec_in()?;
        Ok(ONE_RESOURCE_ONE_CHANGE)
    }

    fn boot_zone(&self) -> anyhow::Result<ApplySummary> {
        if self.config.boot_after_install {
            tracing::debug!("zone {}: booting", self.name);
            cmd_output!(ZONEADM_BIN, "-z", &self.name, "boot")?;
        }

        match self.config.brand.as_str() {
            "lx" => control::wait_for_readiness_lx(&self.name)?,
            _ => control::wait_for_readiness(&self.name)?,
        };

        Ok(ONE_RESOURCE_ONE_CHANGE)
    }

    fn exec_in(&self) -> anyhow::Result<()> {
        if let Some(cmds) = &self.config.exec_in {
            for cmd in cmds {
                tracing::debug!("zone {}; exec '{}'", self.name, cmd);
                run_zlogin_cmd(&self.name, cmd)?;
                tracing::debug!("zone {}; exec '{}' OK", self.name, cmd);
            }
        }

        Ok(())
    }

    fn copy_in(&self) -> anyhow::Result<()> {
        if let Some(files) = &self.config.copy_in {
            for (src, dest) in files {
                self.copy_to_zone(src, dest)?;
            }
        }

        Ok(())
    }

    fn copy_to_zone(&self, src: &Utf8PathBuf, dest: &str) -> anyhow::Result<()> {
        let zone_root = &self.config.zonepath.join("root");

        if !zone_root.exists() {
            bail!("cannot find zone root {}", zone_root);
        }

        let relative_dest = dest.trim_matches('/');
        let zone_dest = zone_root.join(relative_dest);
        tracing::info!("copying {} -> {}", src, zone_dest);

        if src.is_file() {
            fs::copy(src, zone_dest)?;
        } else if src.is_dir() {
            let mut options = CopyOptions::new();
            options.overwrite = true;
            options.copy_inside = true;
            fs_extra::dir::copy(src, zone_dest, &options)?;
        } else {
            bail!("{} is neither a file nor a directory", src);
        }

        Ok(())
    }

    fn bootstrap_zone(&self, opts: &ApplyOpts) -> anyhow::Result<()> {
        // Like everything else, this is super-minimal, at least for now, possibly for ever. Copy
        // our own executable into the zone, and trust the user that the file they gave us is
        // there, and can access all the roles and files it needs.
        //
        if let Some(host_config) = &self.config.bootstrap_from {
            tracing::info!("BEGIN BOOTSTRAP {} [{}]", self.name, host_config);
            let mut bootstrap_command = String::new();

            if let Some(log_level) = env::var_os("RUST_LOG") {
                bootstrap_command.push_str(&format!("RUST_LOG={} ", log_level.to_string_lossy()));
            }

            bootstrap_command.push_str("/var/tmp/gurp ");

            if opts.dump_config {
                bootstrap_command.push_str("--dump-config ");
            }

            if opts.colour {
                bootstrap_command.push_str("--colour ");
            }

            if opts.line_no {
                bootstrap_command.push_str("--line-no ");
            }

            bootstrap_command.push_str(&format!("apply {host_config}"));

            let this_exec =
                Utf8PathBuf::from_path_buf(env::current_exe()?).expect("can't get my path");

            self.copy_to_zone(&this_exec, "/var/tmp/gurp")?;
            run_zlogin_cmd(&self.name, &bootstrap_command)?;
            tracing::info!("END BOOTSTRAP {}", self.name);
        }

        Ok(())
    }
}

impl GurpZoneRemove {
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
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

fn run_zlogin_cmd(zone: &str, command: &str) -> anyhow::Result<()> {
    // Pass the RUST_LOG env var through, because we may be running an instance of this program
    let mut cmd = Command::new(ZLOGIN_BIN);
    cmd.arg(zone);
    cmd.args(command.split_whitespace().collect::<Vec<_>>());
    cmd.stdout(Stdio::inherit());
    cmd.stderr(Stdio::inherit());

    tracing::debug!(command = helpers::command_to_string(&cmd));

    let output = cmd.output()?;

    if output.status.success() {
        Ok(())
    } else {
        bail!(String::from_utf8_lossy(&output.stderr).into_owned())
    }
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
