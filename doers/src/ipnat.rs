use anyhow::bail;
use camino::Utf8PathBuf;
use common::constants::{IPF_SVC, IPNAT_BIN, ONE_RESOURCE_NO_CHANGE, ONE_RESOURCE_ONE_CHANGE};
use common::info;
use common::types::{ApplyOpts, ApplySummary};
use serde::Deserialize;
use std::fs::{self, File};
use std::io::Write;
use std::process::{Command, Output, Stdio};
use util::svcs;

const NAT_CONF_FILE: &str = "/etc/ipf/ipnat.conf";

// We build a single big set of NAT rules from multiple sources, and apply it, clearing out whatever
// was already there. We also write the same rules to /etc/ipf/ipnat.conf. I don't see another
// way to assert persistent state.

#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct GurpIpnatEnsure {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
    pub from: Option<String>,
    pub content: Option<String>,
    pub priority: usize,
}

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct GurpIpnatRemove {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
}

type EnsureList = Vec<GurpIpnatEnsure>;

impl GurpIpnatRemove {
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        let nat_file = Utf8PathBuf::from(NAT_CONF_FILE);
        let mut ret = ONE_RESOURCE_NO_CHANGE;

        if parse_nat_table(&ipnat_output()?).is_empty() {
            tracing::debug!("no live NATs to clear");
        } else {
            tracing::info!("clearing live NAT table");
            let mut cmd = cmd!(IPNAT_BIN, "-C");

            if !opts.noop {
                run_cmd!(cmd)?;
            }

            ret = ONE_RESOURCE_ONE_CHANGE;
        }

        if nat_file.exists() {
            tracing::info!("clearing persistent NAT table");

            if !opts.noop {
                fs::remove_file(&nat_file)?;
            }

            ret = ONE_RESOURCE_ONE_CHANGE;
        } else {
            tracing::debug!("no persistent NATs to clear");
        }

        Ok(ret)
    }
}

fn ensure_persistent_rules(desired_rules: &str, opts: &ApplyOpts) -> anyhow::Result<bool> {
    let nat_file = Utf8PathBuf::from(NAT_CONF_FILE);

    let current_persistent_rules = if nat_file.exists() {
        fs::read_to_string(&nat_file)?
    } else {
        String::new()
    };

    if current_persistent_rules.trim() == desired_rules.trim() {
        tracing::debug!("no changes to persistent NAT rules");
        Ok(false)
    } else {
        tracing::info!("updating nat rules in {nat_file}");

        if opts.dump_diffs {
            println!(
                "{}",
                &info::dump_diff(
                    &current_persistent_rules,
                    desired_rules,
                    Some(&format!("IP NAT rules [{nat_file}]")),
                    opts.colour
                )
            );
        }

        let mut fh = File::create(nat_file)?;
        write!(fh, "{desired_rules}")?;
        Ok(true)
    }
}

fn ensure_live_rules(desired_rules: &str, opts: &ApplyOpts) -> anyhow::Result<bool> {
    let current_live_rules = parse_nat_table(&ipnat_output()?);

    if current_live_rules.trim() == desired_rules.trim() {
        tracing::debug!("no changes to live NAT rules");
        return Ok(false);
    }

    if opts.dump_diffs {
        println!(
            "{}",
            &info::dump_diff(
                &current_live_rules,
                desired_rules,
                Some("IP NAT rules [LIVE]"),
                opts.colour
            )
        );
    }

    let mut apply_cmd = build_ipnat_cmd(false);

    tracing::info!("updating live NAT rules");

    if !opts.noop {
        run_cmd(&mut apply_cmd, desired_rules)?;
    }

    Ok(true)
}

pub fn collect_and_ensure(nat_list: &EnsureList, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
    ensure_ipf_is_running()?;
    svcs::wait_for_state(IPF_SVC, "online")?;

    let mut nat_list = nat_list.clone();
    nat_list.sort_by_key(|r| r.priority);

    let mut desired_rules = String::new();

    for nat in nat_list {
        if let Some(content) = nat.content {
            desired_rules.push_str(&content);
        } else if let Some(path) = nat.from {
            desired_rules.push_str(&fs::read_to_string(path)?);
        } else {
            tracing::warn!("neither :from nor :content for rul {}", nat.id)
        }

        if !desired_rules.ends_with('\n') {
            desired_rules.push('\n');
        }
    }

    if opts.dump_config {
        info::dump_config(&desired_rules, Some("NAT rules"), opts);
    }

    let mut check_cmd = build_ipnat_cmd(true);
    check_nat_rules_are_valid(&mut check_cmd, &desired_rules)?;

    let persistent_change = ensure_persistent_rules(&desired_rules, opts)?;
    let live_change = ensure_live_rules(&desired_rules, opts)?;

    if persistent_change || live_change {
        Ok(ONE_RESOURCE_ONE_CHANGE)
    } else {
        Ok(ONE_RESOURCE_NO_CHANGE)
    }
}

fn check_nat_rules_are_valid(check_cmd: &mut Command, config: &str) -> anyhow::Result<()> {
    match run_cmd(check_cmd, config) {
        Ok(result) => {
            if result.status.success() {
                tracing::debug!("ipnat config passed check");
                Ok(())
            } else {
                bail!(
                    "error running ipnat check config command: {} {}",
                    String::from_utf8_lossy(&result.stdout),
                    String::from_utf8_lossy(&result.stderr),
                )
            }
        }
        Err(e) => bail!("error checking ipnat config: {e}"),
    }
}

fn build_ipnat_cmd(check_only: bool) -> Command {
    let mut cmd = Command::new(IPNAT_BIN);
    cmd.arg("-C");

    if check_only {
        cmd.arg("-n");
    }

    cmd.args(["-f", "-"]);
    cmd.stdin(Stdio::piped());
    cmd.stderr(Stdio::piped());
    cmd.stdout(Stdio::piped());
    cmd
}

fn run_cmd(cmd: &mut Command, config: &str) -> anyhow::Result<Output> {
    let mut child = cmd.spawn()?;

    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(config.as_bytes())?;
    }
    Ok(child.wait_with_output()?)
}

fn ipnat_output() -> anyhow::Result<String> {
    Ok(cmd_output!(IPNAT_BIN, "-l")?)
}

fn parse_nat_table(raw: &str) -> String {
    raw.lines()
        .filter(|l| !l.starts_with("List") || l.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

fn ensure_ipf_is_running() -> anyhow::Result<()> {
    let ipf_state = svcs::current_state(IPF_SVC)?;
    svcs::set_state(IPF_SVC, &ipf_state, "online")?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
    use tester::deserialized_example;

    #[test]
    fn test_ipnat_deserialize_ensure_02() {
        assert_eq!(
            GurpIpnatEnsure {
                name: "rules-in-config".to_owned(),
                id: "/NO-ROLE/ipnat/rules-in-config".to_owned(),
                priority: 1,
                from: None,
                content: Some("rdr le0 203.1.2.3/32 port 80 -> 203.1.2.3,203.1.2.4 port 80 tcp round-robin\nrdr le0 203.1.2.3/32 port 80 -> 203.1.2.5 port 80 tcp round-robin".to_owned())
            },

            deserialized_example::<GurpIpnatEnsure>("ipnat/ensure-02.janet")
        );
    }

    #[test]
    fn test_ipnat_deserialize_remove_01() {
        assert_eq!(
            GurpIpnatRemove {
                name: "removes-all-rules".to_owned(),
                id: "/NO-ROLE/ipnat/removes-all-rules".to_owned(),
            },
            deserialized_example::<GurpIpnatRemove>("ipnat/remove-01.janet")
        );
    }

    #[test]
    fn test_nats_exist() {
        let raw = indoc::indoc! { "
            List of active MAP/Redirect filters:
            map route10 10.10.1.0/24 -> 192.168.1.0/24

            List of active sessions:
            " };

        assert_eq!(
            "map route10 10.10.1.0/24 -> 192.168.1.0/24".to_string(),
            parse_nat_table(raw)
        );
    }
}
