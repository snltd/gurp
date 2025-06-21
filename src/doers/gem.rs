use crate::common::constants::{
    NO_RESOURCES_TO_CHANGE, ONE_RESOURCE_NOOP, ONE_RESOURCE_ONE_CHANGE, ONE_RESOURCE_ONE_ERROR,
};
use crate::common::output::Output;
use crate::common::traits::Apply;
use crate::common::types::{Action, ApplyContext, ApplySummary, Opts, Resource};
use crate::debug;
use crate::utils::helpers;
use crate::utils::janet_helpers::JanetExt;
use anyhow::Context;
use janetrs::{JanetArray, JanetKeyword};
use paste::paste;
use std::process::Command;
use std::sync::LazyLock;

// THINGS TO KNOW / THINGS TO DO.
// This is the dictionary definition of MVP. It makes sure a gem is installed or not installed. That
// is it. Nothing else. `gem install` takes many options, and this code does not let the user
// specify any of them, not even the version. It only uses the OmniOS system `gem`.

const GEM_BIN: &str = "/opt/ooce/bin/gem";

// A chunk of text from `gem list`.
fn gem_output() -> anyhow::Result<String> {
    let cmd = Command::new(GEM_BIN).arg("list").arg("-l").output()?;
    Ok(String::from_utf8_lossy(&cmd.stdout).into_owned())
}

type GemName = String;
type InstalledGems = Vec<GemName>;

static CURRENT_GEM_OUTPUT: LazyLock<String> =
    LazyLock::new(|| gem_output().expect("Could not get gem list"));

pub struct GurpGem {
    pub action: Action,
    pub id: String,
    pub gem_list: Vec<String>,
    pub doer: String,
}

crate::impl_apply!(GurpGem);

impl GurpGem {
    // Because they're all done in one shot, we consider any number of changes to be a
    // single change. You could _MAYBE_ justify this as saying "it's one change to the gem
    // state" but I know in my heart that's cheating. I might make it smarter in the future.
    //
    fn apply_ensure(
        &self,
        _apply_context: &ApplyContext,
        opts: &Opts,
        output: &Output,
    ) -> anyhow::Result<ApplySummary> {
        if self.gem_list.is_empty() {
            output.no_change("gems");
            return Ok(NO_RESOURCES_TO_CHANGE);
        }

        output.creating(self.gem_list.join(", "));

        let mut cmd = Command::new(GEM_BIN);
        cmd.arg("install");
        cmd.arg("--silent");
        cmd.arg("--no-document");

        if opts.noop {
            cmd.arg("--explain");
        }

        cmd.args(&self.gem_list);
        debug!(opts, "doer/gem", "{}", helpers::command_to_string(&cmd));
        let result = cmd.status()?;

        if result.success() {
            Ok(ONE_RESOURCE_ONE_CHANGE)
        } else {
            Ok(ONE_RESOURCE_ONE_ERROR)
        }
    }

    fn apply_remove(
        &self,
        _apply_context: &ApplyContext,
        opts: &Opts,
        output: &Output,
    ) -> anyhow::Result<ApplySummary> {
        if self.gem_list.is_empty() {
            output.no_change("gems");
            return Ok(NO_RESOURCES_TO_CHANGE);
        }

        output.removing(self.gem_list.join(", "));

        if opts.noop {
            return Ok(ONE_RESOURCE_NOOP);
        }

        let mut cmd = Command::new(GEM_BIN);
        cmd.arg("uninstall");
        cmd.arg("--silent");
        cmd.arg("--executables");
        cmd.arg("--all");
        cmd.args(&self.gem_list);

        debug!(opts, "doer/gem", "{}", helpers::command_to_string(&cmd));
        let result = cmd.status()?;

        if result.success() {
            Ok(ONE_RESOURCE_ONE_CHANGE)
        } else {
            Ok(ONE_RESOURCE_ONE_ERROR)
        }
    }
}

// Receive a list of gems, but return a single element vec which will be applied.
pub fn unpack_ensure_list(
    resource_list: &JanetArray,
    opts: &Opts,
) -> anyhow::Result<Vec<Resource>> {
    let installed_gems = parse_gem_output(&CURRENT_GEM_OUTPUT);
    let mut install_list = Vec::new();

    for candidate in resource_list {
        let candidate_struct = candidate
            .extract_struct()
            .context("Failed to extract gem struct")?;
        let name = candidate_struct
            .get(JanetKeyword::from("name"))
            .context("gem struct missing 'name' field")?
            .to_string();

        if installed_gems.contains(&name) {
            debug!(opts, "doer/gem", "gem {} already installed", name);
            continue;
        }

        debug!(opts, "doer/gem", "gem {} scheduled for install", name);
        install_list.push(name);
    }

    debug!(
        opts,
        "doer/gem", "ensure gem list follows:\n{:?}", install_list
    );

    Ok(vec![Resource::Gem(GurpGem {
        id: "/aggr/gem/all".to_owned(),
        action: Action::Ensure,
        gem_list: install_list,
        doer: "gem".to_owned(),
    })])
}

// Receive a list of gems, but return a single element vec which will be applied.
pub fn unpack_remove_list(
    resource_list: &JanetArray,
    opts: &Opts,
) -> anyhow::Result<Vec<Resource>> {
    let installed_gems = parse_gem_output(&CURRENT_GEM_OUTPUT);
    let mut remove_list = Vec::new();

    for candidate_struct in resource_list {
        let candidate_struct = candidate_struct.extract_struct()?;
        if let Some(candidate) = candidate_struct.get(JanetKeyword::from("name")) {
            let candidate = candidate.unwrap().to_string();

            if installed_gems.contains(&candidate) {
                debug!(opts, "doer/gem", "gem: {} scheduled for removal", candidate);
                remove_list.push(candidate);
            } else {
                debug!(opts, "doer/gem", "gem: {} is not installed", candidate);
            }
        }
    }

    Ok(vec![Resource::Gem(GurpGem {
        id: "/aggr/gem/all".to_owned(),
        action: Action::Remove,
        gem_list: remove_list,
        doer: "gem".to_owned(),
    })])
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
