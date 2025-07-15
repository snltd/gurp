use crate::prelude::*;
use serde::Deserialize;
use std::process::Command;
use std::sync::LazyLock;

// THINGS TO KNOW / THINGS TO DO.
// This is the dictionary definition of MVP. It makes sure a gem is installed or not installed. That
// is it. Nothing else. `gem install` takes many options, and this code does not let the user
// specify any of them, not even the version. It only uses the OmniOS system `gem`.

type GemName = String;
type InstalledGems = Vec<GemName>;
type EnsureList = Vec<GurpGemEnsure>;
type RemoveList = Vec<GurpGemRemove>;

static CURRENT_GEM_OUTPUT: LazyLock<String> =
    LazyLock::new(|| gem_output().expect("Could not get gem list"));

#[derive(Debug, Deserialize)]
pub struct GurpGemEnsure {
    pub name: GemName,
}

#[derive(Debug, Deserialize)]
pub struct GurpGemRemove {
    pub name: GemName,
}

pub fn collect_and_ensure(gem_list: &EnsureList, opts: &Opts) -> anyhow::Result<ApplySummary> {
    if gem_list.is_empty() {
        return Ok(NO_RESOURCES_TO_CHANGE);
    }
    let resources = gem_list.len() as u32;
    let installed_gems = parse_gem_output(&CURRENT_GEM_OUTPUT);
    let gem_names: Vec<_> = gem_list.iter().map(|r| &r.name).collect();
    let mut install_list = Vec::new();

    for gem in gem_names {
        if installed_gems.contains(gem) {
            tracing::debug!("gem {}: already installed", gem);
            continue;
        }

        tracing::debug!("gem {}: scheduled for install", gem);
        install_list.push(gem.as_str());
    }

    tracing::debug!("ensure gem list: {}", install_list.join(" "));

    if install_list.is_empty() {
        tracing::debug!("no gems to install");
        Ok(NO_RESOURCES_TO_CHANGE)
    } else {
        tracing::info!("installing: {}", install_list.join(", "));

        let mut cmd = Command::new(GEM_BIN);
        cmd.arg("install")
            .arg("--bindir")
            .arg("/opts/ooce/bin")
            .arg("--silent")
            .arg("--no-document");

        let changes = if opts.noop {
            cmd.arg("--explain");
            0
        } else {
            install_list.len() as u32
        };

        cmd.args(&install_list);
        tracing::debug!(command = helpers::command_to_string(&cmd));
        let output = cmd.output()?;

        if output.status.success() {
            Ok(ApplySummary {
                resources,
                errors: 0,
                changes,
            })
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
    let installed_gems = parse_gem_output(&CURRENT_GEM_OUTPUT);
    let gem_names: Vec<_> = gem_list.iter().map(|r| &r.name).collect();
    let mut install_list = Vec::new();

    for gem in gem_names {
        if installed_gems.contains(gem) {
            tracing::debug!("gem {}: already installed", gem);
            continue;
        }

        tracing::debug!("gem {}: scheduled for install", gem);
        install_list.push(gem.as_str());
    }

    tracing::debug!("ensure gem list: {}", install_list.join(" "));

    if install_list.is_empty() {
        tracing::debug!("no gems to install");
        Ok(NO_RESOURCES_TO_CHANGE)
    } else {
        tracing::info!("installing: {}", install_list.join(", "));
        let mut cmd = Command::new(GEM_BIN);
        cmd.arg("uninstall");
        cmd.arg("--silent");
        cmd.arg("--executables");
        cmd.arg("--all");

        let changes = if opts.noop {
            cmd.arg("--explain");
            0
        } else {
            install_list.len() as u32
        };

        cmd.args(&install_list);
        tracing::debug!(command = helpers::command_to_string(&cmd));
        let output = cmd.output()?;

        if output.status.success() {
            Ok(ApplySummary {
                resources,
                errors: 0,
                changes,
            })
        } else {
            bail!(String::from_utf8_lossy(&output.stderr).into_owned());
        }
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
    let mut installed: Vec<String> = Vec::new();

    for l in output.trim().lines() {
        let bits: Vec<_> = l.split_whitespace().collect();

        if bits.len() < 2 {
            continue;
        }

        installed.push(bits[0].to_owned());
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
        assert_eq!(365, result.len());
    }
}
