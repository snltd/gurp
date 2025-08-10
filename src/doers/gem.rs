use crate::prelude::*;
use serde::Deserialize;
use std::process::Command;
use std::sync::LazyLock;

// THINGS TO KNOW / THINGS TO DO.
// Tries to minimise the calls to `gem install`. The only options the user can pass are `version`
// and `source`. All gems with both of these unset are installed in a single shot. Gems with either
// of these values are handled individually. Only version numbers are supported, so `latest` won't
// work.
//
// `gem/remove` takes no options, so removes all versions of the given gem.

type GemName = String;
type InstalledGems = Vec<InstalledGem>;
type EnsureList = Vec<GurpGemEnsure>;
type RemoveList = Vec<GurpGemRemove>;

static INSTALLED_GEMS: LazyLock<InstalledGems> =
    LazyLock::new(|| parse_gem_output(&gem_output().expect("Could not get gem list")));

#[derive(Debug, PartialEq)]
pub struct InstalledGem {
    pub name: GemName,
    pub versions: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct GurpGemEnsure {
    pub name: GemName,
    pub version: Option<String>,
    pub source: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GurpGemRemove {
    pub name: GemName,
}

fn install_specific(gem: &GurpGemEnsure, opts: &Opts) -> anyhow::Result<ApplySummary> {
    if let Some(desired_version) = &gem.version {
        // If a version is specified, no change if that version is installed, regardless of source
        if INSTALLED_GEMS
            .iter()
            .any(|g| g.name == gem.name && g.versions.contains(desired_version))
        {
            return Ok(ONE_RESOURCE_NO_CHANGE);
        }
    } else if INSTALLED_GEMS.iter().any(|g| g.name == gem.name) {
        // No version specified but alternate source. If any version of the gem is installed, that's
        // near enough
        return Ok(ONE_RESOURCE_NO_CHANGE);
    }

    return_if_noop!(opts);

    // If we're still here, we need to install something

    let mut cmd = cmd!(
        GEM_BIN,
        "install",
        "--bindir",
        GEM_BIN_DIR,
        "--silent",
        "--no-document"
    );

    if let Some(desired_version) = &gem.version {
        cmd.args(["--version", desired_version]);
    }

    if let Some(source) = &gem.source {
        cmd.args(["--source", source]);
    }

    one_change_or_stderr!(cmd, format!("failed to install gem {}", gem.name))
}

// Makes a single call to RubyGems to install things which don't specify an alternate source or
// specific version
pub fn collect_and_ensure(gem_list: &EnsureList, opts: &Opts) -> anyhow::Result<ApplySummary> {
    if gem_list.is_empty() {
        return Ok(NO_RESOURCES_TO_CHANGE);
    }

    let mut summary = ApplySummary::default();

    let mut install_list = Vec::new();

    for gem in gem_list {
        if gem.version.is_some() || gem.source.is_some() {
            summary = summary + install_specific(gem, opts)?;
        } else if INSTALLED_GEMS.iter().any(|g| g.name == gem.name) {
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

        let mut cmd = cmd!(
            GEM_BIN,
            "install",
            "--bindir",
            GEM_BIN_DIR,
            "--silent",
            "--no-document"
        );

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

pub fn collect_and_remove(gem_list: &RemoveList, opts: &Opts) -> anyhow::Result<ApplySummary> {
    if gem_list.is_empty() {
        return Ok(NO_RESOURCES_TO_CHANGE);
    }

    let resources = gem_list.len() as u32;
    let mut changes = 0;
    let installed_gems: Vec<_> = INSTALLED_GEMS.iter().map(|g| &g.name).collect();
    let gem_names: Vec<_> = gem_list.iter().map(|r| &r.name).collect();
    let mut remove_list = Vec::new();

    for gem in gem_names {
        if installed_gems.contains(&gem) {
            tracing::debug!("gem {}: not installed", gem);
            continue;
        }

        tracing::debug!("gem {}: scheduled for removal", gem);
        changes += 1;
        remove_list.push(gem.as_str());
    }

    tracing::debug!("ensure gem list: {}", remove_list.join(" "));

    if remove_list.is_empty() {
        tracing::debug!("no gems to remove");
        return Ok(NO_RESOURCES_TO_CHANGE);
    }

    let ret = ApplySummary {
        resources,
        errors: 0,
        changes,
    };

    if opts.noop {
        return Ok(ret);
    }

    tracing::info!("removing: {}", remove_list.join(", "));

    let mut cmd = cmd!(GEM_BIN, "uninstall", "--silent", "--executables", "--all");
    cmd.args(remove_list);
    let output = cmd.output()?;

    if output.status.success() {
        Ok(ret)
    } else {
        bail!(String::from_utf8_lossy(&output.stderr).into_owned());
    }
}

fn gem_output() -> anyhow::Result<String> {
    if !Utf8PathBuf::from(GEM_BIN).exists() {
        bail!("No gem binary at {}", GEM_BIN);
    }
    let cmd = Command::new(GEM_BIN).arg("list").arg("-l").output()?;
    Ok(String::from_utf8_lossy(&cmd.stdout).into_owned())
}

fn parse_gem_output(output: &str) -> InstalledGems {
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
    use crate::test_utils::spec_helper::load_fixture;

    #[test]
    fn test_parse_gem_output() {
        let result = parse_gem_output(&load_fixture("doers/gem/gem-output"));

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
            result
        );
    }
}
