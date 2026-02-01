use anyhow::ensure;
use byte_unit::Byte;
use camino::Utf8PathBuf;
use common::cmd;
use common::constants::{ONE_RESOURCE_NO_CHANGE, ONE_RESOURCE_ONE_CHANGE, ZFS_BIN, ZFS_LX_BIN};
use common::types::{ApplyOpts, ApplySummary};
use serde::Deserialize;
use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::sync::LazyLock;
use util::deserializer::option_property_deserializer;

// THINGS TO KNOW / THINGS TO DO.
// Destroy is recursive!

static ZFS_BIN_PATH: LazyLock<&'static str> = LazyLock::new(zfs_bin);

// We used to cache the ZFS output. Don't do that!

#[derive(Debug, Deserialize, PartialEq)]
pub struct GurpZfsEnsure {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
    pub size: Option<String>,
    #[serde(default, deserialize_with = "option_property_deserializer")]
    pub properties: Option<ZfsProperties>,
}

type ZfsProperties = HashMap<String, String>;

#[derive(Debug, Deserialize)]
pub struct GurpZfsRemove {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
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

fn zfs_output() -> anyhow::Result<Vec<String>> {
    Ok(cmd_output!(*ZFS_BIN_PATH, "list", "-H", "-o", "name")?
        .lines()
        .map(|s| s.to_owned())
        .collect())
}

fn zfs_state(name: &str) -> anyhow::Result<ZfsProperties> {
    let mut ret = HashMap::new();
    let prop_vals = cmd_output!(*ZFS_BIN_PATH, "get", "-pHo", "property,value", "all", name)?;

    for l in prop_vals.lines() {
        let bits: Vec<_> = l.split_whitespace().collect();

        if bits.len() != 2 {
            continue;
        }

        ret.insert(bits[0].to_owned(), bits[1].to_owned());
    }

    Ok(ret)
}

fn zfs_exists(name: &str) -> anyhow::Result<bool> {
    Ok(zfs_output()?.contains(&name.to_owned()))
}

impl GurpZfsEnsure {
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        if zfs_exists(&self.name)? {
            tracing::debug!("zfs: {} exists", &self.name);
            if let Some(state) = self.properties.as_ref() {
                let current_state = zfs_state(&self.name)?;
                let mut run_cmd = false;
                let mut cmd = Command::new(*ZFS_BIN_PATH);
                cmd.arg("set");

                for (property, desired_value) in state {
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
                                self.name,
                                current_value,
                                desired_value,
                            );
                            run_cmd = true;
                            cmd.arg(format!("{property}={desired_value}"));
                        }
                    }
                }

                if run_cmd {
                    cmd.arg(&self.name);
                    tracing::debug!(command = cmd::to_string(&cmd));
                    return_if_noop!(opts);

                    let output = cmd.output()?;

                    ensure!(
                        output.status.success(),
                        String::from_utf8_lossy(&output.stderr).into_owned()
                    );

                    Ok(ONE_RESOURCE_ONE_CHANGE)
                } else {
                    tracing::debug!("no change: {}", self.name);
                    Ok(ONE_RESOURCE_NO_CHANGE)
                }
            } else {
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

        if let Some(properties) = &self.properties {
            for (property, value) in properties {
                cmd.arg("-o");
                cmd.arg(format!("{property}={value}"));
            }
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

        return_if_noop!(opts);

        one_change_or_stderr!(cmd, "creating ZFS dataset")
    }
}

impl GurpZfsRemove {
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        tracing::debug!("zfs: looking for {}", self.name);
        if zfs_exists(&self.name)? {
            tracing::info!("removing filesystem: {}", self.name);
            return_if_noop!(opts);
            self.remove_filesystem(opts)
        } else {
            tracing::debug!("not present: {}", self.name);
            Ok(ONE_RESOURCE_NO_CHANGE)
        }
    }

    fn remove_filesystem(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        let mut cmd = cmd!(*ZFS_BIN_PATH, "destroy", "-r", &self.name);
        return_if_noop!(opts);
        one_change_or_stderr!(cmd)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use indoc::indoc;
    use tester::janet2json;

    #[test]
    fn test_deserialize() {
        let json_def = janet2json(indoc! {r#"
          (zfs/ensure "tank/test"
                      :label "test-zfs"
                      :properties {:compression "gzip9"
                                   :atime true
                                   :exec "off"
                                   :devices false})
          "#});

        let expected_props: ZfsProperties = HashMap::from([
            ("compression".to_owned(), "gzip9".to_owned()),
            ("atime".to_owned(), "on".to_owned()),
            ("exec".to_owned(), "off".to_owned()),
            ("devices".to_owned(), "off".to_owned()),
        ]);

        let expected = GurpZfsEnsure {
            id: "/NO-ROLE/zfs/test-zfs".to_owned(),
            name: "tank/test".to_owned(),
            size: None,
            properties: Some(expected_props),
        };

        assert_eq!(expected, serde_json::from_str(&json_def).unwrap())
    }
}
