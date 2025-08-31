use crate::constants::GEM_BIN_DIR;
use anyhow::Context;
use common::prelude::*;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::process::{Command, Stdio};

// THINGS TO KNOW / THINGS TO DO.
// Tries to minimise the calls to `gem install`. The only options the user can pass are `version`
// and `source`. All gems with both of these unset are installed in a single shot. Gems with either
// of these values are handled individually. Only version numbers are supported, so `latest` won't
// work.
//
// `gem/remove` takes no options, so removes all versions of the given gem.

type GemName = String;
type InstalledGems = HashMap<Utf8PathBuf, Vec<InstalledGem>>;
type EnsureList = Vec<GurpGemEnsure>;
type RemoveList = Vec<GurpGemRemove>;

#[derive(Debug, PartialEq)]
pub struct InstalledGem {
    pub name: GemName,
    pub versions: Vec<String>,
}

trait GurpGem {
    fn gem_path(&self) -> &Option<Utf8PathBuf>;

    fn gem_bin_path(&self) -> Utf8PathBuf {
        self.gem_path()
            .clone()
            .unwrap_or_else(|| Utf8PathBuf::from(GEM_BIN))
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct GurpGemEnsure {
    pub name: GemName,
    pub version: Option<String>,
    pub source: Option<String>,
    pub gem_path: Option<Utf8PathBuf>,
}

impl GurpGem for GurpGemEnsure {
    fn gem_path(&self) -> &Option<Utf8PathBuf> {
        &self.gem_path
    }
}

#[derive(Debug, Deserialize)]
pub struct GurpGemRemove {
    pub name: GemName,
    pub gem_path: Option<Utf8PathBuf>,
}

impl GurpGem for GurpGemRemove {
    fn gem_path(&self) -> &Option<Utf8PathBuf> {
        &self.gem_path
    }
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

    tracing::debug!(command = helpers::command_to_string(&cmd));

    return_if_noop!(opts);

    one_change_or_stderr!(cmd, format!("failed to install gem {}", gem.name))
}

// Makes a single call to RubyGems to install things which don't specify an alternate source or
// specific version
pub fn collect_and_ensure(gem_list: &EnsureList, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
    if gem_list.is_empty() {
        return Ok(NO_RESOURCES_TO_CHANGE);
    }

    let mut summary = ApplySummary::default();
    let mut install_list = Vec::new();
    let installed_gems = installed_gems(gem_list);
    let default_gem_bin = Utf8PathBuf::from(GEM_BIN);

    let default_gem_list = installed_gems.get(&default_gem_bin);

    for gem in gem_list {
        if gem.version.is_some() || gem.source.is_some() || gem.gem_path.is_some() {
            summary = summary + install_specific(gem, &installed_gems, opts)?;
        } else if let Some(default_list) = default_gem_list
            && default_list.iter().any(|g| g.name == gem.name)
        {
            summary.resources += 1;
            tracing::debug!("gem {}: already installed", gem.name);
        } else {
            summary.resources += 1;
            summary.changes += 1;

            tracing::debug!("gem {}: scheduled for install", gem.name);
            install_list.push(gem.name.clone());
        }
    }

    if install_list.is_empty() {
        tracing::debug!("no gems to batch install");
        Ok(summary)
    } else {
        tracing::info!("batch installing: {}", install_list.join(", "));

        let mut cmd = Command::new(GEM_BIN);
        cmd.arg("install");
        cmd.arg("--bindir");
        cmd.arg(GEM_BIN_DIR);
        cmd.arg("--silent");
        cmd.arg("--no-document");
        cmd.args(&install_list);

        tracing::debug!(command = helpers::command_to_string(&cmd));

        let output = cmd.output()?;

        if output.status.success() {
            Ok(summary)
        } else {
            bail!(String::from_utf8_lossy(&output.stderr).into_owned());
        }
    }
}

pub fn collect_and_remove(gem_list: &RemoveList, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
    if gem_list.is_empty() {
        return Ok(NO_RESOURCES_TO_CHANGE);
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
                remove_hash.entry(gem_bin).or_default().push(&gem.name);
            }
        } else if let Some(inst) = installed_gems.get(&default_gem_bin) {
            if inst.iter().any(|g| g.name == gem.name) {
                changes += 1;
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

    if opts.noop {
        return Ok(ret);
    }

    for (gem_bin, remove_list) in remove_hash {
        tracing::info!("Removing {} [{}]", remove_list.join(", "), gem_bin);
        let mut cmd = Command::new(gem_bin);
        cmd.arg("uninstall");
        cmd.arg("--silent");
        cmd.arg("--executables");
        cmd.arg("--all");
        cmd.args(&remove_list);
        cmd.stderr(Stdio::piped());

        tracing::debug!(command = helpers::command_to_string(&cmd));

        let output = cmd.output()?;

        if !output.status.success() {
            bail!(String::from_utf8_lossy(&output.stderr).into_owned());
        }
    }

    Ok(ret)
}

fn gem_output(gem_bin: &Utf8PathBuf) -> anyhow::Result<String> {
    if !gem_bin.exists() {
        bail!("No gem binary at {}", gem_bin);
    }

    let cmd = Command::new(gem_bin).arg("list").arg("-l").output()?;
    Ok(String::from_utf8_lossy(&cmd.stdout).into_owned())
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

#[cfg(test)]
mod test {
    use super::*;
    use indoc::indoc;

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
