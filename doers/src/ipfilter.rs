use crate::types::ApplyResult;
use anyhow::{Context, bail};
use camino::Utf8PathBuf;
use common::constants::{
    IPF_BIN, IPF_SVC, IPFSTAT_BIN, NO_RESOURCES_TO_CHANGE, ONE_RESOURCE_NO_CHANGE,
    ONE_RESOURCE_ONE_CHANGE,
};
use common::info;
use common::types::{ApplyOpts, ApplySummary, ChangedIds};
use serde::Deserialize;
use std::fs::{self, File};
use std::io::Write;
use util::svcs;

const IPF_CONF: &str = "/etc/ipf/ipf.conf";

type EnsureList = Vec<GurpIpfilterEnsure>;

// We build a single big set of filter rules from multiple sources, check its validity, and ensure
// its contents align with those of /etc/ipf/ipf.conf. If the file has changed, or if any resource
// used to build the content has :always-reloaded true, the contents of the file become the current
// firewall configuration.

#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(rename_all = "kebab-case")]
pub struct GurpIpfilterEnsure {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
    pub from: Option<String>,
    pub content: Option<String>,
    pub priority: usize,
    pub always_reload: bool,
}

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct GurpIpfilterRemove {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
}

pub fn collect_and_ensure(filter_list: &EnsureList, opts: &ApplyOpts) -> ApplyResult {
    let mut changed_ids = ChangedIds::default();

    if filter_list.is_empty() {
        return Ok((NO_RESOURCES_TO_CHANGE, changed_ids));
    }

    ensure_ipf_is_running(opts)?;
    svcs::wait_for_state(IPF_SVC, "online").context("error waiting ipf service to come online")?;
    let mut force_reload = false;

    let mut filter_list = filter_list.clone();
    filter_list.sort_by_key(|r| r.priority);

    let mut desired_rules = String::new();

    for filter in filter_list {
        if let Some(content) = filter.content {
            desired_rules.push_str(&content);
        } else if let Some(path) = filter.from {
            desired_rules.push_str(
                &fs::read_to_string(&path)
                    .with_context(|| format!("error reading rules form {path}"))?,
            );
        } else {
            tracing::warn!("neither :from nor :content for rule {}", filter.id)
        }

        if !desired_rules.ends_with('\n') {
            desired_rules.push('\n');
        }

        if filter.always_reload {
            force_reload = true;
        }

        changed_ids.insert(filter.id);
    }

    if opts.output.dump_configs {
        info::dump_config(&desired_rules, Some("ipfilter rules"), &opts.output);
    }

    check_filter_rules_are_valid(&desired_rules)?;

    let persistent_change = ensure_persistent_rules(&desired_rules, opts)?;

    if persistent_change || force_reload {
        tracing::debug!("forcing reload of ipfilter rules from {IPF_CONF}");
        let mut reload_cmd = cmd!(IPF_BIN, "-Fa", "-f", IPF_CONF);

        if opts.noop {
            Ok((ONE_RESOURCE_ONE_CHANGE, changed_ids))
        } else {
            let before_change =
                cmd_output!(IPFSTAT_BIN, "-io").context("failed to get ipfstat status")?;

            run_cmd!(reload_cmd).context("failed to reload ipfilter rules")?;

            let after_change =
                cmd_output!(IPFSTAT_BIN, "-io").context("failed to get ipfstat status")?;

            if before_change == after_change {
                tracing::debug!("reloading ipfilter conf did not change live rules");
                Ok((ONE_RESOURCE_NO_CHANGE, changed_ids))
            } else {
                tracing::info!("reloading ipfilter conf produced new live rules");
                Ok((ONE_RESOURCE_ONE_CHANGE, changed_ids))
            }
        }
    } else {
        Ok((ONE_RESOURCE_NO_CHANGE, changed_ids))
    }
}

impl GurpIpfilterRemove {
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        let filter_file = Utf8PathBuf::from(IPF_CONF);
        let mut ret = ONE_RESOURCE_NO_CHANGE;

        if there_are_rules()? {
            tracing::info!("clearing live ipf rules");
            let mut cmd = cmd!(IPF_BIN, "-Fa");

            if !opts.noop {
                run_cmd!(cmd).context("failed to flush ipfilter rules")?;
            }

            ret = ONE_RESOURCE_ONE_CHANGE;
        } else {
            tracing::debug!("no live ipf rules to clear");
        }

        if filter_file.exists() {
            tracing::info!("clearing persistent ipf rules");

            if !opts.noop {
                fs::remove_file(&filter_file)
                    .with_context(|| format!("failed to remove ipf config at {filter_file}"))?;
            }

            ret = ONE_RESOURCE_ONE_CHANGE;
        } else {
            tracing::debug!("no persistent ipf rules to clear");
        }

        Ok(ret)
    }
}

// this is not 'Nam Smokey,
fn there_are_rules() -> anyhow::Result<bool> {
    Ok(!cmd_output!(IPFSTAT_BIN, "-io")
        .context("failed to get ipf status")?
        .is_empty())
}

fn check_filter_rules_are_valid(rules: &str) -> anyhow::Result<()> {
    tracing::debug!("checking ipf rules");
    let mut cmd = cmd_with_stdin!(IPF_BIN, "-n", "-f", "-");
    let mut child = cmd.spawn()?;

    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(rules.as_bytes())?;
    }

    let output = child.wait_with_output()?;

    if output.status.success() {
        tracing::debug!("rules validated successfully");
    } else {
        bail!(format!(
            "failed to validate rules: {}",
            String::from_utf8_lossy(&output.stderr).into_owned()
        ))
    }

    Ok(())
}

fn ensure_persistent_rules(desired_rules: &str, opts: &ApplyOpts) -> anyhow::Result<bool> {
    let filter_file = Utf8PathBuf::from(IPF_CONF);

    let current_persistent_rules = if filter_file.exists() {
        fs::read_to_string(&filter_file)
            .with_context(|| format!("failed to read filter rules from {filter_file}"))?
    } else {
        String::new()
    };

    if current_persistent_rules.trim() == desired_rules.trim() {
        tracing::debug!("no changes to persistent ipfilter rules");
        Ok(false)
    } else {
        tracing::info!("updating ipfilter rules in {filter_file}");

        if opts.output.dump_diffs {
            println!(
                "{}",
                &info::dump_diff(
                    &current_persistent_rules,
                    desired_rules,
                    Some(&format!("IP filter rules [{filter_file}]")),
                    &opts.output
                )
            );
        }

        if !opts.noop {
            let mut fh = File::create(&filter_file)
                .with_context(|| format!("failed to open {filter_file}"))?;
            write!(fh, "{desired_rules}")
                .with_context(|| format!("failed to write ipfilter rules to {filter_file}"))?;
        }

        Ok(true)
    }
}

fn ensure_ipf_is_running(opts: &ApplyOpts) -> anyhow::Result<()> {
    let ipf_state = svcs::current_state(IPF_SVC)?;
    svcs::set_state(IPF_SVC, &ipf_state, "online", opts).context("failed to start ipf service")?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
    use tester::deserialized_example;

    #[test]
    fn test_ipfilter_deserialize_ensure_from_config() {
        assert_eq!(
            GurpIpfilterEnsure {
                name: "rules-from-config".to_owned(),
                id: "/NO-ROLE/ipfilter/rules-from-config".to_owned(),
                priority: 0,
                from: None,
                content: Some("block in log all\nblock out all".to_owned()),
                always_reload: true,
            },
            deserialized_example("ipfilter/ensure-from-config.janet")
        );
    }

    #[test]
    fn test_ipfilter_deserialize_ensure_from_file() {
        assert_eq!(
            GurpIpfilterEnsure {
                name: "rules-from-file".to_owned(),
                id: "/NO-ROLE/ipfilter/rules-from-file".to_owned(),
                priority: 1,
                from: Some("test/ipfilter-test".to_owned()),
                content: None,
                always_reload: false,
            },
            deserialized_example("ipfilter/ensure-from-file.janet")
        );
    }

    #[test]
    fn test_ipfilter_deserialize_remove_all_rules() {
        assert_eq!(
            GurpIpfilterRemove {
                name: "removes-all-rules".to_owned(),
                id: "/NO-ROLE/ipfilter/removes-all-rules".to_owned(),
            },
            deserialized_example("ipfilter/remove-all-rules.janet")
        );
    }
}
