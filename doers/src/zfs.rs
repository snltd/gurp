use anyhow::Context;
use byte_unit::Byte;
use camino::Utf8PathBuf;
use common::cmd;
use common::constants::{ONE_RESOURCE_NO_CHANGE, ONE_RESOURCE_ONE_CHANGE, ZFS_BIN, ZFS_LX_BIN};
use common::types::{ApplyOpts, ApplySummary};
use serde::Deserialize;
use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::sync::LazyLock;
use util::deserializer::property_deserializer;

static ZFS_BIN_PATH: LazyLock<&'static str> = LazyLock::new(zfs_bin);

// We used to cache the ZFS output. Don't do that!

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct GurpZfsEnsure {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
    pub size: Option<String>,
    #[serde(default, deserialize_with = "property_deserializer")]
    pub properties: ZfsProperties,
}

type ZfsProperties = HashMap<String, String>;

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct GurpZfsRemove {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
}

impl GurpZfsEnsure {
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        let fs = &self.name;
        if zfs_exists(fs)? {
            tracing::debug!("zfs: {fs} exists");
            let current_state = zfs_state(fs)?;
            let mut run_cmd = false;
            let mut cmd = Command::new(*ZFS_BIN_PATH);
            cmd.arg("set");

            for (property, desired_value) in &self.properties {
                if let Some(current_value) = current_state.get(property) {
                    if current_value == desired_value {
                        tracing::debug!("{}: already {}", property, desired_value);
                    } else {
                        // Catch size properties. Putting the iB is a nasty, but it works
                        if let Ok(desired_bytes) =
                            Byte::parse_str(format!("{desired_value}iB"), true)
                            && desired_value.ends_with(['M', 'G', 'k', 'E'])
                            && desired_bytes.to_string() == *current_value
                        {
                            break;
                        }

                        tracing::info!(
                            "change zfs {}: [{}] {} -> {}",
                            property,
                            fs,
                            current_value,
                            desired_value,
                        );
                        run_cmd = true;
                        cmd.arg(format!("{property}={desired_value}"));
                    }
                }
            }

            if run_cmd {
                cmd.arg(fs);
                tracing::debug!(command = cmd::to_string(&cmd));

                if !opts.noop {
                    run_cmd!(cmd)
                        .with_context(|| format!("failed to modify ZFS filesystem {fs}"))?;
                }

                Ok(ONE_RESOURCE_ONE_CHANGE)
            } else {
                tracing::debug!("no change: {fs}");
                Ok(ONE_RESOURCE_NO_CHANGE)
            }
        } else {
            self.create_filesystem(opts)
        }
    }

    fn create_filesystem(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        tracing::info!("creating filesystem: {}", self.name);

        let mut cmd = Command::new(*ZFS_BIN_PATH);
        cmd.arg("create");

        for (property, value) in &self.properties {
            cmd.arg("-o");
            cmd.arg(format!("{property}={value}"));
        }

        if let Some(size) = &self.size {
            cmd.arg("-V");
            cmd.arg(size);
        }

        if opts.noop {
            cmd.arg("-n");
        }

        cmd.arg(&self.name).stderr(Stdio::piped());
        tracing::debug!(command = cmd::to_string(&cmd));

        if !opts.noop {
            run_cmd!(cmd)
                .with_context(|| format!("failed to create ZFS filesystem {}", self.name))?;
        }

        Ok(ONE_RESOURCE_ONE_CHANGE)
    }
}

impl GurpZfsRemove {
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        tracing::debug!("zfs: looking for {}", self.name);

        if zfs_exists(&self.name)? {
            tracing::info!("removing filesystem: {}", self.name);
            remove_filesystem(&self.name, opts)
        } else {
            tracing::debug!("not present: {}", self.name);
            Ok(ONE_RESOURCE_NO_CHANGE)
        }
    }
}

pub fn remove_filesystem(name: &str, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
    cmd_change_or_noop!(opts, *ZFS_BIN_PATH, "destroy", "-r", name)
        .with_context(|| format!("failed to destroy ZFS filesystem {}", name))
}

fn zfs_bin() -> &'static str {
    if Utf8PathBuf::from(ZFS_BIN).exists() {
        ZFS_BIN
    } else if Utf8PathBuf::from(ZFS_LX_BIN).exists() {
        ZFS_LX_BIN
    } else {
        panic!("No ZFS binary");
    }
}

fn zfs_state(name: &str) -> anyhow::Result<ZfsProperties> {
    let mut ret = HashMap::new();
    let prop_vals = cmd_output!(*ZFS_BIN_PATH, "get", "-pHo", "property,value", "all", name)
        .with_context(|| format!("failed to get ZFS properties for {name}"))?;

    for l in prop_vals.lines() {
        let bits: Vec<_> = l.split_whitespace().collect();

        if bits.len() != 2 {
            continue;
        }

        ret.insert(bits[0].to_owned(), bits[1].to_owned());
    }

    Ok(ret)
}

pub fn zfs_exists(name: &str) -> anyhow::Result<bool> {
    cmd_success!(*ZFS_BIN_PATH, "list", "-Ho", "name", name)
}

#[cfg(test)]
mod test {
    use super::*;
    use tester::{deserialized_example, propmap};

    #[test]
    fn test_deserialize_zfs_ensure_filesystem_with_properties() {
        assert_eq!(
            GurpZfsEnsure {
                id: "/NO-ROLE/zfs/zfs-example-1".to_owned(),
                name: "rpool/example/filesystem".to_owned(),
                size: None,
                properties: propmap! {
                    "compression" => "gzip-9",
                    "dedup" => "on",
                    "mountpoint" => "/example/mountpoint",
                    "devices" => "off",
                },
            },
            deserialized_example("zfs/ensure-filesystem-with-properties.janet")
        );
    }

    #[test]
    fn test_deserialize_zfs_ensure_volume_with_label() {
        assert_eq!(
            GurpZfsEnsure {
                id: "/NO-ROLE/zfs/example-zfs-vol".to_owned(),
                name: "rpool/example/volume".to_owned(),
                size: Some("10G".to_owned()),
                properties: propmap! {},
            },
            deserialized_example("zfs/ensure-volume-with-label.janet")
        );
    }

    #[test]
    fn test_deserialize_zfs_remove_dataset() {
        assert_eq!(
            GurpZfsRemove {
                id: "/NO-ROLE/zfs/rpool_old_filesystem".to_owned(),
                name: "rpool/old/filesystem".to_owned(),
            },
            deserialized_example("zfs/remove-dataset.janet")
        );
    }
}
