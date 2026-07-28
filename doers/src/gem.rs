use crate::types::ApplyResult;
use anyhow::{Context, ensure};
use camino::Utf8PathBuf;
use common::cmd;
use common::constants::{
    GEM_BIN, GEM_BIN_DIR, NO_RESOURCES_TO_CHANGE, ONE_RESOURCE_NO_CHANGE, ONE_RESOURCE_ONE_CHANGE,
};
use common::types::{ApplyOpts, ApplySummary, ChangedIds};
use os_types::GurpId;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::process::{Command, Stdio};

// This is keyed on the gem binary which reported the gem
type InstalledGems = HashMap<Utf8PathBuf, Vec<InstalledGem>>;
type EnsureList = Vec<GurpGemEnsure>;
type RemoveList = Vec<GurpGemRemove>;
type GemName = String;

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(rename_all = "kebab-case")]
pub struct GurpGemEnsure {
    #[serde(rename = "_id")]
    pub id: GurpId,
    pub name: GemName,
    pub version: Option<String>,
    pub source: Option<String>,
    pub gem_path: Option<Utf8PathBuf>,
}

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct GurpGemRemove {
    #[serde(rename = "_id")]
    pub id: GurpId,
    pub name: GemName,
    pub gem_path: Option<Utf8PathBuf>,
}

#[derive(Debug, PartialEq)]
pub struct InstalledGem {
    pub name: GemName,
    pub versions: Vec<String>,
}

impl GurpGem for GurpGemEnsure {
    fn gem_path(&self) -> &Option<Utf8PathBuf> {
        &self.gem_path
    }
}

trait GurpGem {
    fn gem_path(&self) -> &Option<Utf8PathBuf>;

    fn gem_bin_path(&self) -> Utf8PathBuf {
        self.gem_path()
            .clone()
            .unwrap_or_else(|| Utf8PathBuf::from(GEM_BIN))
    }
}

impl GurpGem for GurpGemRemove {
    fn gem_path(&self) -> &Option<Utf8PathBuf> {
        &self.gem_path
    }
}

// To try to minimize remote calls, and therefore run time, bundles (no pun intended) together
// calls which are directly to Rubygems using the default gem binary and without a specific
// version. If the user has specified a version, a different source or gem binary for any gems,
// they are dealt with individually.
//
pub fn collect_and_ensure(gem_list: &EnsureList, opts: &ApplyOpts) -> ApplyResult {
    let mut changed_ids = ChangedIds::default();

    if gem_list.is_empty() {
        return Ok((NO_RESOURCES_TO_CHANGE, changed_ids));
    }

    let mut summary = ApplySummary::default();
    let mut install_list = Vec::new();
    let installed_gems = installed_gems(gem_list);
    let batch_gem_bin = Utf8PathBuf::from(GEM_BIN);

    for gem in gem_list {
        if gem.version.is_some() || gem.source.is_some() || gem.gem_path.is_some() {
            changed_ids.insert(gem.id.clone());
            summary += install_specific(gem, &installed_gems, opts)?;
        } else if let Some(default_list) = installed_gems.get(&batch_gem_bin)
            && default_list.iter().any(|g| g.name == gem.name)
        {
            summary.resources += 1;
            tracing::debug!("gem {}: already installed", gem.name);
        } else {
            summary.resources += 1;
            summary.changes += 1;
            changed_ids.insert(gem.id.clone());
            install_list.push(gem.name.clone());
            tracing::debug!("gem {}: scheduled for install", gem.name);
        }
    }

    if install_list.is_empty() {
        tracing::debug!("no gems to batch install");
    } else {
        tracing::info!("batch installing: {}", install_list.join(", "));

        let mut cmd = Command::new(GEM_BIN);
        cmd.arg("install");
        cmd.arg("--bindir");
        cmd.arg(GEM_BIN_DIR);
        cmd.arg("--silent");
        cmd.arg("--no-document");
        cmd.args(&install_list);

        tracing::debug!(command = cmd::to_string(&cmd));

        if !opts.noop {
            run_cmd!(cmd).context("failed to run gem install")?;
        }
    }

    Ok((summary, changed_ids))
}

pub fn collect_and_remove(gem_list: &RemoveList, opts: &ApplyOpts) -> ApplyResult {
    let mut changed_ids = ChangedIds::default();

    if gem_list.is_empty() {
        return Ok((NO_RESOURCES_TO_CHANGE, changed_ids));
    }

    let resources = gem_list.len() as u32;
    let installed_gems = installed_gems(gem_list);
    let default_gem_bin = Utf8PathBuf::from(GEM_BIN);
    let mut changes = 0;
    let mut remove_hash: HashMap<&Utf8PathBuf, Vec<&str>> = HashMap::new();

    for gem in gem_list {
        if let Some(gem_bin) = &gem.gem_path {
            if let Some(inst) = installed_gems.get(gem_bin)
                && inst.iter().any(|g| g.name == gem.name)
            {
                tracing::debug!("{}: scheduled for removal", gem.name);
                changes += 1;
                changed_ids.insert(gem.id.clone());
                remove_hash.entry(gem_bin).or_default().push(&gem.name);
            }
        } else if let Some(inst) = installed_gems.get(&default_gem_bin) {
            if inst.iter().any(|g| g.name == gem.name) {
                changes += 1;
                changed_ids.insert(gem.id.clone());
                tracing::debug!("{}: scheduled for removal", gem.name);
                remove_hash
                    .entry(&default_gem_bin)
                    .or_default()
                    .push(&gem.name);
            }
        } else {
            tracing::debug!("{}: not installed", gem.name);
        }
    }

    let ret = ApplySummary { resources, changes };

    for (gem_bin, remove_list) in remove_hash {
        tracing::info!("Removing {} [{}]", remove_list.join(", "), gem_bin);
        let mut cmd = Command::new(gem_bin);
        cmd.arg("uninstall");
        cmd.arg("--silent");
        cmd.arg("--executables");
        cmd.arg("--all");
        cmd.args(&remove_list);
        cmd.stderr(Stdio::piped());

        tracing::debug!(command = cmd::to_string(&cmd));

        if !opts.noop {
            let _ = run_cmd!(cmd).context("failed to run gem uninstall")?;
        }
    }

    Ok((ret, changed_ids))
}

fn gem_output(gem_bin: &Utf8PathBuf) -> anyhow::Result<String> {
    ensure!(gem_bin.exists(), format!("No gem binary at {gem_bin}"));
    cmd_output!(gem_bin, "list", "-l").context("failed to list gems")
}

fn parse_gem_output(output: &str) -> Vec<InstalledGem> {
    let mut installed: Vec<_> = Vec::new();

    for l in output.trim().lines() {
        let bits: Vec<_> = l.split_whitespace().collect();

        if bits.len() < 2 {
            continue;
        }

        let name = bits[0].to_owned();

        let versions: Vec<_> = bits[1..]
            .iter()
            .filter_map(|b| {
                let trimmed = b.trim_matches([',', '(', ')']).to_owned();

                if trimmed == "default:" {
                    None
                } else {
                    Some(trimmed)
                }
            })
            .collect();

        installed.push(InstalledGem { name, versions });
    }

    installed
}

fn install_specific(
    gem: &GurpGemEnsure,
    installed_gems: &InstalledGems,
    opts: &ApplyOpts,
) -> anyhow::Result<ApplySummary> {
    tracing::debug!("installing specific gem {}", gem.name);
    let gem_path: Utf8PathBuf;

    if let Some(path) = &gem.gem_path {
        gem_path = path.clone();
    } else {
        gem_path = Utf8PathBuf::from(GEM_BIN)
    };

    let installed_gem_list = installed_gems
        .get(&gem_path)
        .context(format!("no installed gems for {gem_path}"))?;

    if let Some(desired_version) = &gem.version {
        // If a version is specified, no change if that version is installed, regardless of source
        if installed_gem_list
            .iter()
            .any(|g| g.name == gem.name && g.versions.contains(desired_version))
        {
            return Ok(ONE_RESOURCE_NO_CHANGE);
        }
    } else if installed_gem_list.iter().any(|g| g.name == gem.name) {
        // No version specified but alternate source. If any version of the gem is installed, that's
        // near enough
        return Ok(ONE_RESOURCE_NO_CHANGE);
    }

    // If we're still here, we need to install something

    let mut cmd = Command::new(gem_path);
    cmd.arg("install");
    cmd.arg("--bindir");
    cmd.arg(GEM_BIN_DIR);
    cmd.arg("--silent");
    cmd.arg("--no-document");

    if let Some(source) = &gem.source {
        cmd.arg("--source");
        cmd.arg(source);
    }

    cmd.arg(&gem.name);

    if let Some(desired_version) = &gem.version {
        cmd.arg("--version");
        cmd.arg(desired_version);
    }

    tracing::debug!(command = cmd::to_string(&cmd));

    if !opts.noop {
        run_cmd!(cmd).context("failed to install specific gem")?;
    }

    Ok(ONE_RESOURCE_ONE_CHANGE)
}

fn installed_gems<T: GurpGem>(gem_list: &[T]) -> InstalledGems {
    let mut installed_gems: InstalledGems = HashMap::new();
    let ooce_bin = Utf8PathBuf::from(GEM_BIN);

    if ooce_bin.exists() {
        installed_gems.insert(
            ooce_bin.clone(),
            parse_gem_output(&gem_output(&ooce_bin).expect("Could not get gem list")),
        );
    }

    let alternate_gem_bins: HashSet<_> = gem_list.iter().map(|g| g.gem_bin_path()).collect();

    for path in alternate_gem_bins {
        installed_gems.insert(
            path.clone(),
            parse_gem_output(&gem_output(&path).expect("Could not get gem list")),
        );
    }

    installed_gems
}

#[cfg(test)]
mod test {
    use super::*;
    use indoc::indoc;
    use tester::deserialized_example;

    #[test]
    fn test_gem_deserialize_ensure_rubygem() {
        assert_eq!(
            GurpGemEnsure {
                id: GurpId::new("/NO-ROLE/gem/wavefront-cli").unwrap(),
                name: "wavefront-cli".to_owned(),
                version: None,
                source: None,
                gem_path: None,
            },
            deserialized_example("gem/ensure-rubygem.janet")
        );
    }

    #[test]
    fn test_gem_deserialize_ensure_version_with_source_and_gempath() {
        assert_eq!(
            GurpGemEnsure {
                id: GurpId::new("/NO-ROLE/gem/my-gem").unwrap(),
                name: "my-gem".to_owned(),
                version: Some("1.2.3".to_owned()),
                source: Some("https://my-gem-repo.com".to_owned()),
                gem_path: Some(Utf8PathBuf::from("/opt/pkgin/bin/gem")),
            },
            deserialized_example("gem/ensure-version-with-source-and-gempath.janet")
        );
    }

    #[test]
    fn test_gem_deserialize_remove_gem() {
        assert_eq!(
            GurpGemRemove {
                id: GurpId::new("/NO-ROLE/gem/webscale").unwrap(),
                name: "webscale".to_owned(),
                gem_path: None,
            },
            deserialized_example("gem/remove-gem.janet")
        );
    }

    #[test]
    fn test_parse_gem_output() {
        let sample_output = indoc! { r#"
            un (default: 0.3.0)
            unicode-display_width (3.1.4, 2.6.0, 2.5.0)
            unicode-emoji (4.0.4)
            uri (1.0.3, default: 0.13.1)
            yaml (default: 0.3.0)
            "#
        };

        assert_eq!(
            vec![
                InstalledGem {
                    name: "un".to_owned(),
                    versions: vec!["0.3.0".to_owned()],
                },
                InstalledGem {
                    name: "unicode-display_width".to_owned(),
                    versions: vec!["3.1.4".to_owned(), "2.6.0".to_owned(), "2.5.0".to_owned()],
                },
                InstalledGem {
                    name: "unicode-emoji".to_owned(),
                    versions: vec!["4.0.4".to_owned()],
                },
                InstalledGem {
                    name: "uri".to_owned(),
                    versions: vec!["1.0.3".to_owned(), "0.13.1".to_owned()],
                },
                InstalledGem {
                    name: "yaml".to_owned(),
                    versions: vec!["0.3.0".to_owned()],
                },
            ],
            parse_gem_output(sample_output)
        );
    }
}
