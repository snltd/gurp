use anyhow::{bail, ensure};
use common::cmd;
use common::constants::{DLADM_BIN, IPADM_BIN, ONE_RESOURCE_NO_CHANGE, ONE_RESOURCE_ONE_CHANGE};
use common::types::{ApplyOpts, ApplySummary, VlanID};
use serde::Deserialize;
use std::fmt::Debug;
use std::process::Command;

// THINGS TO KNOW / THINGS TO DO.

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct GurpVnicEnsure {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
    pub over: String,
    pub vlan_tag: Option<VlanID>,
    pub with_interface: bool,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct GurpVnicRemove {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
}

struct VnicInfo {
    over: String,
    vid: VlanID,
}

fn vnic_exists(vnic_name: &str) -> anyhow::Result<bool> {
    let dladm_output = cmd_output!(DLADM_BIN, "show-vnic", "-p", "-o", "link")?;
    Ok(dladm_output.lines().any(|l| l == vnic_name))
}

fn delete_vnic(vnic_name: &str) -> anyhow::Result<()> {
    cmd_output!(DLADM_BIN, "delete-vnic", vnic_name)?;
    Ok(())
}

impl GurpVnicEnsure {
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        if vnic_exists(&self.name)? {
            // dladm doesn't have a way to modify a VNIC, so if it's not up to spec, we must
            // remove it and re-create it.
            let mut recreate = false;

            // The VNIC may exist but be over the wrong link
            let vnic_info = self.vnic_info()?;

            if vnic_info.over != self.over {
                tracing::info!(
                    "{} is over {}, should be over {}: forces recreate",
                    self.name,
                    &vnic_info.over,
                    self.over,
                );

                recreate = true;
            }

            // Or it may have the wrong VLAN tag

            if let Some(desired_vid) = self.vlan_tag
                && desired_vid != vnic_info.vid
            {
                tracing::info!(
                    "{} has VLAN tag {}, should be {}: forces recreate",
                    self.name,
                    &vnic_info.vid,
                    desired_vid,
                );

                recreate = true;
            }

            if recreate {
                tracing::info!("Removing {}", self.name);

                if !opts.noop {
                    delete_vnic(&self.name)?;
                }
            } else {
                return Ok(ONE_RESOURCE_NO_CHANGE);
            }
        }

        tracing::info!("Creating {}", self.name);

        let mut cmd = Command::new(DLADM_BIN);
        cmd.arg("create-vnic");
        cmd.arg("-l");
        cmd.arg(&self.over);

        if let Some(vlan_tag) = &self.vlan_tag {
            cmd.arg("-v");
            cmd.arg(vlan_tag.to_string());
        }

        cmd.arg(&self.name);

        tracing::debug!(command = cmd::to_string(&cmd));

        if !opts.noop {
            let result = cmd.output()?;
            ensure!(
                result.status.success(),
                "error creating VNIC: {}",
                String::from_utf8_lossy(&result.stderr)
            );
        }

        if self.with_interface {
            tracing::info!("creating interface on {}", self.name);

            if !opts.noop {
                cmd_output!(IPADM_BIN, "create-if", &self.name)?;
            }
        }

        Ok(ONE_RESOURCE_ONE_CHANGE)
    }

    fn vnic_info(&self) -> anyhow::Result<VnicInfo> {
        let raw = cmd_output!(DLADM_BIN, "show-vnic", &self.name, "-p", "-o", "over,vid")?;

        let chunks: Vec<_> = raw.split(':').collect();

        if chunks.len() != 2 {
            bail!(format!("Bad output from show-vnic: {raw}"));
        }

        Ok(VnicInfo {
            over: chunks[0].to_owned(),
            vid: chunks[1].parse::<VlanID>()?,
        })
    }
}

impl GurpVnicRemove {
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        if vnic_exists(&self.name)? {
            tracing::info!("Removing {}", self.name);
            return_if_noop!(opts);

            delete_vnic(&self.name)?;
            Ok(ONE_RESOURCE_ONE_CHANGE)
        } else {
            tracing::debug!("{} does not exist", self.name);
            Ok(ONE_RESOURCE_NO_CHANGE)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
    use tester::deserialized_example;

    #[test]
    fn test_deserialize_vnic_ensure_01() {
        assert_eq!(
            GurpVnicEnsure {
                id: "/NO-ROLE/vnic/vnic0".to_owned(),
                name: "vnic0".to_owned(),
                over: "e1000g".to_owned(),
                vlan_tag: None,
                with_interface: false,
            },
            deserialized_example("vnic/ensure-01.janet")
        );
    }

    #[test]
    fn test_deserialize_vnic_remove_01() {
        assert_eq!(
            GurpVnicRemove {
                id: "/NO-ROLE/vnic/vnic1".to_owned(),
                name: "vnic1".to_owned(),
            },
            deserialized_example("vnic/remove-01.janet")
        );
    }
}
