use anyhow::{bail, ensure};
use common::cmd;
use common::constants::{DLADM_BIN, ONE_RESOURCE_NO_CHANGE, ONE_RESOURCE_ONE_CHANGE};
use common::types::{ApplyOpts, ApplySummary};
use serde::Deserialize;
use std::collections::BTreeSet;
use std::fmt::Debug;
use std::process::Command;

type Links = BTreeSet<String>;

#[derive(Deserialize, Debug)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(rename_all = "kebab-case")]
pub struct GurpBridgeEnsure {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
    #[serde(flatten)]
    pub desired_state: BridgeState,
}

struct RawBridgeState {
    state: String,
    links: String,
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct BridgeState {
    pub links: Option<Links>,
    pub protect: String,
    pub priority: u16,
    pub max_age: u8,
    pub hello_time: u8,
    pub forward_delay: u8,
    pub force_protocol: u8,
}

#[derive(Deserialize, Debug, PartialEq)]
// #[cfg_attr(test, derive(PartialEq))]
pub struct GurpBridgeRemove {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
}

fn bridge_exists(bridge: &str) -> anyhow::Result<bool> {
    let raw = cmd_output!(DLADM_BIN, "show-bridge", "-p")?;

    Ok(raw.lines().any(|l| l.starts_with(&format!("{bridge}:"))))
}

impl GurpBridgeEnsure {
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        if bridge_exists(&self.name)? {
            let current_state = parse_bridge(&self.describe_bridge()?)?;

            if current_state == self.desired_state {
                tracing::debug!("state of bridge {} is correct", self.name);
                Ok(ONE_RESOURCE_NO_CHANGE)
            } else {
                self.align_state(&current_state, opts)
            }
        } else {
            tracing::info!("creating bridge {}", self.name);
            return_if_noop!(opts);

            self.create_bridge(opts)
        }
    }

    fn create_bridge(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        let mut cmd = Command::new(DLADM_BIN);
        cmd.arg("create-bridge");
        cmd.args(["-P", &self.desired_state.protect]);
        cmd.args(["-p", &self.desired_state.priority.to_string()]);
        cmd.args(["-m", &self.desired_state.max_age.to_string()]);
        cmd.args(["-h", &self.desired_state.hello_time.to_string()]);
        cmd.args(["-d", &self.desired_state.forward_delay.to_string()]);
        cmd.args(["-f", &self.desired_state.force_protocol.to_string()]);

        if let Some(links) = &self.desired_state.links {
            for link in links {
                cmd.args(["-l", link]);
            }
        }

        cmd.arg(&self.name);

        tracing::debug!(command = cmd::to_string(&cmd));

        if !opts.noop {
            run_cmd!(cmd)?;
        }

        Ok(ONE_RESOURCE_ONE_CHANGE)
    }

    fn describe_bridge(&self) -> anyhow::Result<RawBridgeState> {
        Ok(RawBridgeState {
            state: cmd_output!(
                DLADM_BIN,
                "show-bridge",
                "-p",
                "-o",
                "protect,priority,bhellotime,bfwddelay,forceproto,bmaxage",
                &self.name
            )?,
            links: cmd_output!(
                DLADM_BIN,
                "show-bridge",
                "-l",
                "-p",
                "-o",
                "link",
                &self.name
            )?,
        })
    }

    fn modify_links(&self, action: &str, links: Vec<&str>, opts: &ApplyOpts) -> anyhow::Result<()> {
        let mut message = match action {
            "add-bridge" => format!("Adding link(s) to bridge {}: ", self.name),
            "remove-bridge" => format!("Removing link(s) from bridge {}: ", self.name),
            _ => bail!("unknown modify-links action"),
        };

        let mut cmd = Command::new(DLADM_BIN);
        cmd.arg(action);

        for link in links {
            cmd.args(["-l", link]);
            message.push_str(&format!("{link} "));
        }

        cmd.arg(&self.name);
        tracing::info!(message);
        tracing::debug!(command = cmd::to_string(&cmd));

        if !opts.noop {
            run_cmd!(cmd)?;
        }

        Ok(())
    }

    fn align_links(&self, current: &BridgeState, opts: &ApplyOpts) -> anyhow::Result<bool> {
        let mut change = false;
        let no_links: Links = BTreeSet::new();

        let desired_links: &Links = self.desired_state.links.as_ref().unwrap_or(&no_links);
        let current_links: &Links = current.links.as_ref().unwrap_or(&no_links);

        let add_links: Vec<_> = desired_links
            .difference(current_links)
            .map(|s| s.as_str())
            .collect();

        let remove_links: Vec<_> = current_links
            .difference(desired_links)
            .map(|s| s.as_str())
            .collect();

        if !add_links.is_empty() {
            self.modify_links("add-bridge", add_links, opts)?;
            change = true;
        }

        if !remove_links.is_empty() {
            self.modify_links("remove-bridge", remove_links, opts)?;
            change = true;
        }

        Ok(change)
    }

    fn flag_up<T: std::fmt::Display>(&self, thing: &str, from: T, to: T) {
        tracing::info!(
            "bridge {}: changing {}: {} -> {}",
            self.name,
            thing,
            from,
            to
        );
    }

    fn align_properties(&self, current: &BridgeState, opts: &ApplyOpts) -> anyhow::Result<bool> {
        let desired = &self.desired_state;
        let mut change = false;
        let mut cmd = Command::new(DLADM_BIN);

        cmd.arg("modify-bridge");

        if current.protect != desired.protect {
            self.flag_up("protect", &current.protect, &desired.protect);
            cmd.args(["-P", &desired.protect]);
            change = true;
        }

        if current.priority != desired.priority {
            self.flag_up("priority", current.priority, desired.priority);
            cmd.args(["-p", &desired.priority.to_string()]);
            change = true;
        }

        if current.max_age != desired.max_age {
            self.flag_up("max_age", current.max_age, desired.max_age);
            cmd.args(["-m", &desired.max_age.to_string()]);
            change = true;
        }

        if current.hello_time != desired.hello_time {
            self.flag_up("hello_time", current.hello_time, desired.hello_time);
            cmd.args(["-h", &desired.hello_time.to_string()]);
            change = true;
        }

        if current.forward_delay != desired.forward_delay {
            self.flag_up(
                "forward_delay",
                current.forward_delay,
                desired.forward_delay,
            );
            cmd.args(["-d", &desired.forward_delay.to_string()]);
            change = true;
        }

        if current.force_protocol != desired.force_protocol {
            self.flag_up(
                "force_protocol",
                current.force_protocol,
                desired.force_protocol,
            );
            cmd.args(["-f", &desired.force_protocol.to_string()]);
            change = true;
        }

        if change {
            cmd.arg(&self.name);
            tracing::debug!(command = cmd::to_string(&cmd));

            if !opts.noop {
                run_cmd!(cmd)?;
            }
        }
        Ok(change)
    }

    fn align_state(&self, current: &BridgeState, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        let mut change = self.align_links(current, opts)?;

        if self.align_properties(current, opts)? {
            change = true;
        }

        if change {
            Ok(ONE_RESOURCE_ONE_CHANGE)
        } else {
            Ok(ONE_RESOURCE_NO_CHANGE)
        }
    }
}

impl GurpBridgeRemove {
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        if bridge_exists(&self.name)? {
            tracing::info!("removing bridge {}", self.name);
            self.delete_bridge(opts)
        } else {
            tracing::debug!("bridge {} does not exist", self.name);
            Ok(ONE_RESOURCE_NO_CHANGE)
        }
    }

    fn delete_bridge(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        let mut cmd = cmd!(DLADM_BIN, "delete-bridge", &self.name);

        if !opts.noop {
            run_cmd!(cmd)?;
        }

        Ok(ONE_RESOURCE_ONE_CHANGE)
    }
}

fn parse_bridge(raw: &RawBridgeState) -> anyhow::Result<BridgeState> {
    let state_chunks: Vec<_> = raw.state.split(":").collect();
    ensure!(
        state_chunks.len() == 6,
        "cannot parse bridge state: {}",
        raw.state
    );

    let links = if raw.links.is_empty() {
        None
    } else {
        let lines: Links = raw.links.lines().map(|l| l.to_owned()).collect();

        Some(lines)
    };

    Ok(BridgeState {
        protect: state_chunks[0].to_owned(),
        priority: state_chunks[1].parse::<u16>()?,
        hello_time: state_chunks[2].parse::<u8>()?,
        forward_delay: state_chunks[3].parse::<u8>()?,
        force_protocol: state_chunks[4].parse::<u8>()?,
        max_age: state_chunks[5].parse::<u8>()?,
        links,
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use tester::deserialized_example;

    #[test]
    fn test_ensure_bridge_deserialize_01() {
        assert_eq!(
            GurpBridgeEnsure {
                name: "basic".to_owned(),
                id: "/NO-ROLE/bridge/basic".to_owned(),
                desired_state: BridgeState {
                    protect: "stp".to_owned(),
                    priority: 32768,
                    hello_time: 2,
                    forward_delay: 15,
                    force_protocol: 3,
                    max_age: 20,
                    links: None,
                },
            },
            deserialized_example::<GurpBridgeEnsure>("bridge/ensure-01.janet")
        );
    }

    #[test]
    fn test_ensure_bridge_deserialize_02() {
        assert_eq!(
            GurpBridgeEnsure {
                name: "with_links".to_owned(),
                id: "/NO-ROLE/bridge/with_links".to_owned(),
                desired_state: BridgeState {
                    protect: "stp".to_owned(),
                    priority: 4096,
                    hello_time: 2,
                    forward_delay: 15,
                    force_protocol: 3,
                    max_age: 30,
                    links: Some(BTreeSet::from([
                        "vnic0".to_owned(),
                        "stub0".to_owned(),
                        "e1000g0".to_owned()
                    ])),
                },
            },
            deserialized_example::<GurpBridgeEnsure>("bridge/ensure-02.janet")
        );
    }

    #[test]
    fn test_remove_bridge_deserialize() {
        assert_eq!(
            GurpBridgeRemove {
                name: "unwanted".to_owned(),
                id: "/NO-ROLE/bridge/unwanted".to_owned(),
            },
            deserialized_example::<GurpBridgeRemove>("bridge/remove-01.janet")
        );
    }

    #[test]
    fn test_parse_bridge() {
        assert_eq!(
            BridgeState {
                links: None,
                protect: "stp".to_owned(),
                priority: 4096,
                hello_time: 2,
                forward_delay: 16,
                force_protocol: 3,
                max_age: 20,
            },
            parse_bridge(&RawBridgeState {
                state: "stp:4096:2:16:3:20".to_owned(),
                links: String::new(),
            })
            .unwrap()
        );

        assert_eq!(
            BridgeState {
                links: Some(BTreeSet::from(["stub0".to_owned(), "stub2".to_owned()])),
                protect: "stp".to_owned(),
                priority: 32768,
                hello_time: 2,
                forward_delay: 15,
                force_protocol: 3,
                max_age: 20,
            },
            parse_bridge(&RawBridgeState {
                state: "stp:32768:2:15:3:20".to_owned(),
                links: "stub0\nstub2".to_owned(),
            })
            .unwrap()
        );
    }
}
