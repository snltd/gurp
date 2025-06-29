use crate::common::constants::{
    NO_RESOURCES_TO_CHANGE, ONE_RESOURCE_NOOP, ONE_RESOURCE_ONE_CHANGE,
};
use crate::common::types::{ApplyContext, ApplySummary, Opts };
use serde::Deserialize;
use std::process::Command;
use std::sync::LazyLock;

// THINGS TO KNOW / THINGS TO DO.
// This is the dictionary definition of MVP. It makes sure a gem is installed or not installed. That
// is it. Nothing else. `gem install` takes many options, and this code does not let the user
// specify any of them, not even the version. It only uses the OmniOS system `gem`.

type GemName = String;
type InstalledGems = Vec<GemName>;

const GEM_BIN: &str = "/opt/ooce/bin/gem";

static CURRENT_GEM_OUTPUT: LazyLock<String> =
    LazyLock::new(|| gem_output().expect("Could not get gem list"));

#[derive(Debug, Deserialize)]
pub struct GurpGemEnsure {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: GemName,
}

#[derive(Debug, Deserialize)]
pub struct GurpGemRemove {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: GemName,
}

/*
impl GurpGem {
    // Because they're all done in one shot, we consider any number of changes to be a
    // single change. You could _MAYBE_ justify this as saying "it's one change to the gem
    // state" but I know in my heart that's cheating. I might make it smarter in the future.
    //
    fn apply_ensure(
        &self,
        _apply_context: &ApplyContext,
        opts: &Opts,
    ) -> anyhow::Result<ApplySummary> {
        if self.gem_list.is_empty() {
            tracing::info!("no change: {}", self.gem_list.join(", "));
            return Ok(NO_RESOURCES_TO_CHANGE);
        }

        tracing::info!("installing gems: {}", self.gem_list.join(", "));

        let mut cmd = Command::new(GEM_BIN);
        cmd.arg("install");
        cmd.arg("--silent");
        cmd.arg("--no-document");

        if opts.noop {
            cmd.arg("--explain");
        }

        cmd.args(&self.gem_list);
        tracing::debug!(command = helpers::command_to_string(&cmd));
        let result = cmd.output()?;

        if result.status.success() {
            Ok(ONE_RESOURCE_ONE_CHANGE)
        } else {
            bail!(String::from_utf8_lossy(&result.stderr).into_owned())
        }
    }

    fn apply_remove(
        &self,
        _apply_context: &ApplyContext,
        opts: &Opts,
    ) -> anyhow::Result<ApplySummary> {
        if self.gem_list.is_empty() {
            tracing::info!("no change: {}", self.gem_list.join(", "));
            return Ok(NO_RESOURCES_TO_CHANGE);
        }

        tracing::info!("removing gems: {}", self.gem_list.join(", "));

        if opts.noop {
            return Ok(ONE_RESOURCE_NOOP);
        }

        let mut cmd = Command::new(GEM_BIN);
        cmd.arg("uninstall");
        cmd.arg("--silent");
        cmd.arg("--executables");
        cmd.arg("--all");
        cmd.args(&self.gem_list);

        tracing::debug!(command = helpers::command_to_string(&cmd));
        let result = cmd.output()?;

        if result.status.success() {
            Ok(ONE_RESOURCE_ONE_CHANGE)
        } else {
            bail!(String::from_utf8_lossy(&result.stderr).into_owned())
        }
    }
}

// Receive a list of gems, but return a single element vec which will be applied.
pub fn unpack_ensure_list(
    resource_list: &JanetArray,
    _opts: &Opts,
) -> anyhow::Result<Vec<Resource>> {
    let installed_gems = parse_gem_output(&CURRENT_GEM_OUTPUT);
    let mut install_list = Vec::new();

    for candidate in resource_list {
        let candidate_struct = candidate
            .extract_struct()
            .context("failed to extract gem struct")?;
        let name = candidate_struct
            .get(JanetKeyword::from("name"))
            .context("gem struct missing 'name' field")?
            .to_string();

        if installed_gems.contains(&name) {
            tracing::debug!("gem {}: already installed", name);
            continue;
        }

        tracing::debug!("gem {}: scheduled for install", name);
        install_list.push(name);
    }

    tracing::debug!("installing gems: {}", install_list.join(", "));

    Ok(vec![Resource::Gem(GurpGem {
        id: "/aggr/gem/all".to_owned(),
        action: Action::Ensure,
        gem_list: install_list,
    })])
}

// Receive a list of gems, but return a single element vec which will be applied.
pub fn unpack_remove_list(
    resource_list: &JanetArray,
    _opts: &Opts,
) -> anyhow::Result<Vec<Resource>> {
    let installed_gems = parse_gem_output(&CURRENT_GEM_OUTPUT);
    let mut remove_list = Vec::new();

    for candidate_struct in resource_list {
        let candidate_struct = candidate_struct.extract_struct()?;
        if let Some(candidate) = candidate_struct.get(JanetKeyword::from("name")) {
            let name = candidate.unwrap().to_string();

            if installed_gems.contains(&name) {
                tracing::debug!("gem {}: scheduled for removal", name);
                remove_list.push(name);
            } else {
                tracing::debug!("gem {}: not installed", name);
            }
        }
    }

    Ok(vec![Resource::Gem(GurpGem {
        id: "/aggr/gem/all".to_owned(),
        action: Action::Remove,
        gem_list: remove_list,
    })])
}
*/

fn gem_output() -> anyhow::Result<String> {
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
