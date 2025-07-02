use crate::common::constants::NO_RESOURCES_TO_CHANGE;
use crate::common::types::{ApplySummary, Opts};
use crate::utils::helpers;
use anyhow::bail;
use serde::Deserialize;
use std::process::Command;
use std::sync::LazyLock;

// THINGS TO KNOW / THINGS TO DO.
// You specify pkgs by name, so `ooce/editor/helix` rather than
// `pkg://sysdef/ooce/editor/helix@25.1-151052.0:20250108t110907Z`. This means you
// can't request specific versions. I might change this, but I never pin to
// version, and I'm immediately only solving the problems I actually have.
// You need the full path as well: it isn't remotely smart and can't understand that "helix"
// is "ooce/editor/helix".

// Operating only on name makes the doer run faster, because it knows exactly
// what can and cannot be done, so runs `pkg(1)` in the most efficient way
// possible. `pkg(1)` is rather a slow tool.

static CURRENT_PKG_OUTPUT: LazyLock<String> =
    LazyLock::new(|| pkg_output().expect("Could not get pkg list"));

const PKG_BIN: &str = "/bin/pkg";

type PkgName = String;

struct GlobalPkgs {
    available: Vec<PkgName>,
    installed: Vec<PkgName>,
}

#[derive(Debug, Deserialize)]
pub struct GurpPkgEnsure {
    pub name: PkgName,
}

#[derive(Debug, Deserialize)]
pub struct GurpPkgRemove {
    pub name: PkgName,
}

type EnsureList = Vec<GurpPkgEnsure>;
type RemoveList = Vec<GurpPkgRemove>;

pub fn collect_and_ensure(pkg_list: &EnsureList, opts: &Opts) -> anyhow::Result<ApplySummary> {
    let resources = pkg_list.len() as u32;
    let global_pkgs = parse_pkg_output(&CURRENT_PKG_OUTPUT);
    let pkg_names: Vec<_> = pkg_list.iter().map(|r| &r.name).collect();
    let mut install_list = Vec::new();

    for pkg in &pkg_names {
        if global_pkgs.installed.contains(pkg) {
            tracing::debug!("already installed: {}", pkg);
            continue;
        }

        if global_pkgs.available.contains(pkg) {
            tracing::debug!("scheduled for install: {}", pkg);
            install_list.push(pkg.as_str());
        } else {
            tracing::warn!("not available: {}", pkg);
        }
    }

    tracing::debug!("ensure pkg list: {}", install_list.join(" "));

    if install_list.is_empty() {
        tracing::debug!("no packages to install");
        Ok(NO_RESOURCES_TO_CHANGE)
    } else {
        tracing::info!("installing: {}", install_list.join(", "));

        let mut cmd = Command::new(PKG_BIN);
        cmd.arg("install");

        if opts.noop {
            cmd.arg("-n");
        }

        cmd.args(&install_list);
        tracing::debug!(command = helpers::command_to_string(&cmd));
        let output = cmd.output()?;

        if output.status.success() {
            Ok(ApplySummary {
                resources,
                errors: 0,
                changes: install_list.len() as u32,
            })
        } else {
            // pkg doesn't always write to stderr on an error
            let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

            let error_message = if stderr.is_empty() {
                let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
                stdout
                    .lines()
                    .collect::<Vec<_>>()
                    .last()
                    .map_or("no output", |v| v)
                    .to_owned()
            } else {
                stderr.to_owned()
            };

            bail!(error_message)
        }
    }
}

pub fn collect_and_remove(pkg_list: &RemoveList, opts: &Opts) -> anyhow::Result<ApplySummary> {
    let resources = pkg_list.len() as u32;
    let global_pkgs = parse_pkg_output(&CURRENT_PKG_OUTPUT);
    let pkg_names: Vec<_> = pkg_list.iter().map(|r| &r.name).collect();
    let mut remove_list = Vec::new();

    for pkg in pkg_names {
        if global_pkgs.installed.contains(pkg) {
            tracing::debug!("scheduled for removal: {}", pkg);
            remove_list.push(pkg.as_str());
        } else {
            tracing::debug!("not present: {}", pkg);
            continue;
        }
    }

    if remove_list.is_empty() {
        tracing::debug!("no packages to remove");
        Ok(NO_RESOURCES_TO_CHANGE)
    } else {
        tracing::info!("removing: {}", remove_list.join(", "));

        let mut cmd = Command::new(PKG_BIN);
        cmd.arg("uninstall");
        cmd.arg("-q");

        if opts.noop {
            cmd.arg("-n");
        }

        cmd.args(&remove_list);
        tracing::debug!(command = helpers::command_to_string(&cmd));
        let output = cmd.output()?;

        if output.status.success() {
            Ok(ApplySummary {
                resources,
                errors: 0,
                changes: remove_list.len() as u32,
            })
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

            let error_message = if stderr.is_empty() {
                let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
                stdout
                    .lines()
                    .collect::<Vec<_>>()
                    .last()
                    .map_or("no output", |v| v)
                    .to_owned()
            } else {
                stderr.to_owned()
            };

            bail!(error_message)
        }
    }
}

fn pkg_output() -> anyhow::Result<String> {
    let cmd = Command::new(PKG_BIN)
        .arg("list")
        .arg("-aH")
        .arg("-o")
        .arg("name,flags")
        .output()?;

    Ok(String::from_utf8(cmd.stdout)?)
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
