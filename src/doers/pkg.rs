// You specify pkgs by name, so `ooce/editor/helix` rather than
// `pkg://sysdef/ooce/editor/helix@25.1-151052.0:20250108t110907Z`. This means you
// can't request specific versions. I might change this, but I never pin to
// version, and I'm immediately only solving the problems I actually have.
// You need the full path as well: it isn't remotely smart and can't understand that "helix"
// is "ooce/editor/helix".

// Operating only on name makes the doer run faster, because it knows exactly
// what can and cannot be done, so runs `pkg(1)` in the most efficient way
// possible. `pkg(1)` is rather a slow tool.

use crate::common::constants::{
    NO_RESOURCES_TO_CHANGE, ONE_RESOURCE_ONE_CHANGE, ONE_RESOURCE_ONE_ERROR,
};
use crate::common::output::Output;
use crate::common::traits::Apply;
use crate::common::types::{Action, ApplyContext, ApplySummary, Opts, Resource};
use crate::utils::janet_helpers::JanetExt;
use crate::{debug, warn};
use anyhow::Context;
use colored::Colorize;
use janetrs::{JanetArray, JanetKeyword};
use paste::paste;
use std::process::Command;
use std::sync::LazyLock;

// A chunk of text from pkg(1). This is expensive, so do it once and parse the output twice.
fn pkg_output() -> anyhow::Result<String> {
    let cmd = Command::new("/bin/pkg")
        .arg("list")
        .arg("-aH")
        .arg("-o")
        .arg("name,flags")
        .output()?;

    Ok(String::from_utf8(cmd.stdout)?)
}

type PkgName = String;

static CURRENT_PKG_OUTPUT: LazyLock<String> =
    LazyLock::new(|| pkg_output().expect("Could not get pkg list"));

// TODO this needs a better name
struct GlobalPkgs {
    available: Vec<PkgName>,
    installed: Vec<PkgName>,
}

pub struct GurpPkg {
    pub action: Action,
    pub id: String,
    pub pkg_list: Vec<String>,
    pub doer: String,
}

crate::impl_apply!(GurpPkg);

impl GurpPkg {
    // Because they're all done in one shot, we consider any number of package changes to be a
    // single change. You could _MAYBE_ justify this as saying "it's one change to the package
    // state" but I know in my heart that's cheating. I might make it smarter in the future.
    //
    fn apply_ensure(
        &self,
        _apply_context: &ApplyContext,
        opts: &Opts,
        output: &Output,
    ) -> anyhow::Result<ApplySummary> {
        if self.pkg_list.is_empty() {
            output.no_change("pkgs");
            return Ok(NO_RESOURCES_TO_CHANGE);
        }

        output.creating(self.pkg_list.join(", "));

        let mut cmd = Command::new("/bin/pkg");
        cmd.arg("install");
        cmd.arg("-q");

        if opts.noop {
            cmd.arg("-n");
        }

        cmd.args(&self.pkg_list);
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
        if self.pkg_list.is_empty() {
            output.no_change("pkgs");
            return Ok(NO_RESOURCES_TO_CHANGE);
        }

        output.removing(self.pkg_list.join(", "));

        let mut cmd = Command::new("/bin/pkg");
        cmd.arg("uninstall");
        cmd.arg("-q");

        if opts.noop {
            cmd.arg("-n");
        }

        cmd.args(&self.pkg_list);
        let result = cmd.status()?;

        if result.success() {
            Ok(ONE_RESOURCE_ONE_CHANGE)
        } else {
            Ok(ONE_RESOURCE_ONE_ERROR)
        }
    }
}

// Receive a list of pkgs, but return a single element vec which will be applied.
pub fn unpack_ensure_list(
    resource_list: &JanetArray,
    opts: &Opts,
) -> anyhow::Result<Vec<Resource>> {
    let global_pkgs = parse_pkg_output(&CURRENT_PKG_OUTPUT);

    let mut install_list = Vec::new();

    for candidate in resource_list {
        let candidate_struct = candidate
            .extract_struct()
            .context("Failed to extract package struct")?;
        let name = candidate_struct
            .get(JanetKeyword::from("name"))
            .context("Package struct missing 'name' field")?
            .to_string();

        if global_pkgs.installed.contains(&name) {
            debug!(opts, "doer/pkg", "pkg {} already installed", name);
            continue;
        }

        if global_pkgs.available.contains(&name) {
            debug!(opts, "doer/pkg", "pkg {} scheduled for install", name);
            install_list.push(name);
        } else {
            warn!(opts, "doer/pkg", "pkg {} not available", name);
        }
    }

    Ok(vec![Resource::Pkg(GurpPkg {
        id: "/aggr/pkg/all".to_owned(),
        action: Action::Ensure,
        pkg_list: install_list,
        doer: "pkg".to_owned(),
    })])
}

// Receive a list of pkgs, but return a single element vec which will be applied.
pub fn unpack_remove_list(
    resource_list: &JanetArray,
    opts: &Opts,
) -> anyhow::Result<Vec<Resource>> {
    let global_pkgs = parse_pkg_output(&CURRENT_PKG_OUTPUT);

    let mut remove_list = Vec::new();

    for candidate_struct in resource_list {
        let candidate_struct = candidate_struct.extract_struct()?;
        if let Some(candidate) = candidate_struct.get(JanetKeyword::from("name")) {
            let candidate = candidate.unwrap().to_string();

            if global_pkgs.installed.contains(&candidate) {
                debug!(opts, "doer/pkg", "pkg: {} scheduled for removal", candidate);
                remove_list.push(candidate);
            } else {
                debug!(opts, "doer/pkg", "pkg: {} is not installed", candidate);
            }
        }
    }

    Ok(vec![Resource::Pkg(GurpPkg {
        id: "/aggr/pkg/all".to_owned(),
        action: Action::Remove,
        pkg_list: remove_list,
        doer: "pkg".to_owned(),
    })])
}

fn parse_pkg_output(output: &str) -> GlobalPkgs {
    let mut installed: Vec<String> = Vec::new();
    let mut available: Vec<String> = Vec::new();

    for l in output.trim().lines() {
        let bits: Vec<_> = l.split_whitespace().collect();

        if bits.len() != 2 {
            continue;
        }

        if bits[1].starts_with('i') {
            installed.push(bits[0].to_owned());
        } else {
            available.push(bits[0].to_owned());
        }
    }

    GlobalPkgs {
        available,
        installed,
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils::spec_helper::load_fixture;

    #[test]
    fn test_parse_pkg_output() {
        let result = parse_pkg_output(&load_fixture("doers/pkg/pkg-output"));
        assert_eq!(613, result.installed.len());
        assert_eq!(521, result.available.len());
    }
}
