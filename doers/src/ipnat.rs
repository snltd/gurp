use anyhow::{bail, ensure};
use common::prelude::*;
use serde::Deserialize;
use std::fs;
use std::io::Write;
use std::process::{Command, Output, Stdio};
use util::svcs;

// THINGS TO KNOW
// ipnat/remove removes ALL NAT rules

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct GurpIpnatEnsure {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
    pub from: Option<String>,
    pub content: Option<String>,
    pub flags: Option<Vec<String>>,
    pub in_zone: Option<String>,
    pub global_zone: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GurpIpnatRemove {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
}

impl GurpIpnatEnsure {
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        ensure_ipf_is_running()?;

        ensure!(
            !(self.content.is_some() && self.from.is_some()),
            "need exactly one of :from and :content"
        );

        let mut check_cmd = self.build_ipnat_cmd(true);

        match self.run_cmd(&mut check_cmd) {
            Ok(result) => {
                if result.status.success() {
                    tracing::debug!("ipnat config passed check")
                } else {
                    bail!(
                        "error checking ipnat config: {} {}",
                        String::from_utf8_lossy(&result.stdout),
                        String::from_utf8_lossy(&result.stderr),
                    )
                }
            }
            Err(e) => bail!("error checking ipnat config: {e}"),
        }

        let current_rules = parse_nat_table(&ipnat_output()?);

        let desired_rules = if let Some(path) = &self.from {
            fs::read_to_string(path)?
        } else if let Some(content) = &self.content {
            content.to_string()
        } else {
            bail!("require either :content or :from")
        };

        if let Some(cmd_flags) = &self.flags
            && cmd_flags.contains(&":remove".to_string())
        {
            let current_rule_lines: Vec<_> = current_rules.lines().collect();

            if desired_rules
                .lines()
                .all(|r| current_rule_lines.contains(&r))
            {
                tracing::debug!("no rules in current ruleset");
                return Ok(ONE_RESOURCE_NO_CHANGE);
            }
        } else if desired_rules == current_rules {
            tracing::debug!("ipnat rules are up to date");
            return Ok(ONE_RESOURCE_NO_CHANGE);
        }

        tracing::info!("updating NAT rules");

        let mut apply_cmd = self.build_ipnat_cmd(false);

        return_if_noop!(opts);

        self.run_cmd(&mut apply_cmd)?;

        Ok(ONE_RESOURCE_ONE_CHANGE)
    }

    fn build_ipnat_cmd(&self, check_only: bool) -> Command {
        let mut cmd = Command::new(IPNAT_BIN);

        if let Some(zone) = &self.in_zone {
            cmd.args(["-z", zone.as_str()]);
        } else if let Some(zone) = &self.global_zone {
            cmd.args(["-z", zone.as_str()]);
        }

        cmd.arg("-C");

        if let Some(flags) = &self.flags {
            for flag in flags {
                match flag.as_str() {
                    ":disable-resolution" => {
                        cmd.arg("-R");
                    }
                    ":remove" => {
                        cmd.arg("-r");
                    }
                    other => tracing::warn!("ignoring unknown ipnat flag '{other}'"),
                };
            }
        }

        if check_only {
            cmd.arg("-n");
        }

        if let Some(path) = &self.from {
            cmd.args(["-f", path]);
        } else if self.content.is_some() {
            cmd.args(["-f", "-"]);
            cmd.stdin(Stdio::piped());
        }

        cmd
    }

    fn run_cmd(&self, cmd: &mut Command) -> anyhow::Result<Output> {
        if let Some(content) = &self.content {
            let mut child = cmd.spawn()?;

            if let Some(stdin) = child.stdin.as_mut() {
                stdin.write_all(content.as_bytes())?;
            }
            Ok(child.wait_with_output()?)
        } else {
            Ok(cmd.output()?)
        }
    }
}

impl GurpIpnatRemove {
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        let ipnat_output = cmd_output!(IPNAT_BIN, "-l")?;

        if parse_nat_table(&ipnat_output).is_empty() {
            tracing::debug!("no NATs to clear");
            return Ok(ONE_RESOURCE_NO_CHANGE);
        }

        tracing::info!("clearing NAT table");
        let mut cmd = cmd!(IPNAT_BIN, "-C");

        return_if_noop!(opts);

        run_cmd!(cmd)?;
        Ok(ONE_RESOURCE_ONE_CHANGE)
    }
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
    use indoc::indoc;
    use tester::janet2json;

    #[test]
    fn test_build_ipnat_cmd_1() {
        let json_def = janet2json(indoc! { r#"
              (ipnat/ensure "test-1"
                            :from "/tmp/ipnat-test"
                            :flags [:disable-resolution]
                            :in-zone "test-zone")
            "# });

        let sut: GurpIpnatEnsure = serde_json::from_str(&json_def).unwrap();

        assert_eq!(
            "/usr/sbin/ipnat -z test-zone -C -R -f /tmp/ipnat-test",
            common::helpers::command_to_string(&sut.build_ipnat_cmd(false))
        );
    }

    #[test]
    fn test_build_ipnat_cmd_2() {
        let json_def = janet2json(indoc! { r#"
              (ipnat/ensure "test-1" :content "not important here")
            "# });

        let sut: GurpIpnatEnsure = serde_json::from_str(&json_def).unwrap();

        assert_eq!(
            "/usr/sbin/ipnat -C -f -",
            common::helpers::command_to_string(&sut.build_ipnat_cmd(false))
        );
    }

    #[test]
    fn test_nats_exist() {
        let raw = indoc! { "
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
