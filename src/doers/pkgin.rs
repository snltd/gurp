use crate::prelude::*;
use serde::Deserialize;
use std::sync::LazyLock;

// THINGS TO KNOW / THINGS TO DO.
// You specify pkgs by name, so `openssl` rather than `openssl-3.3.2`. This means you
// can't request specific versions. I might change this, but I never pin to
// version, and I'm immediately only solving the problems I actually have.

static CURRENT_PKG_OUTPUT: LazyLock<String> =
    LazyLock::new(|| pkgin_output().expect("Could not get pkgin list"));

type PkginName = String;

struct GlobalPkgins {
    installed: Vec<PkginName>,
}

#[derive(Debug, Deserialize)]
pub struct GurpPkginEnsure {
    pub name: PkginName,
}

#[derive(Debug, Deserialize)]
pub struct GurpPkginRemove {
    pub name: PkginName,
}

type EnsureList = Vec<GurpPkginEnsure>;
type RemoveList = Vec<GurpPkginRemove>;

pub fn collect_and_ensure(pkg_list: &EnsureList, opts: &Opts) -> anyhow::Result<ApplySummary> {
    let resources = pkg_list.len() as u32;
    let global_pkgs = parse_pkg_output(&CURRENT_PKG_OUTPUT);
    let pkg_names: Vec<_> = pkg_list.iter().map(|r| &r.name).collect();
    let mut install_list = Vec::new();

    for pkg in &pkg_names {
        if global_pkgs.installed.contains(pkg) {
            tracing::debug!("already installed: {}", pkg);
        } else {
            install_list.push(pkg.as_str());
        }
    }

    tracing::debug!("ensure pkgin list: {}", install_list.join(" "));

    if install_list.is_empty() {
        tracing::debug!("no pkgsrc packages to install");
        Ok(ApplySummary {
            resources,
            errors: 0,
            changes: 0,
        })
    } else {
        tracing::info!("installing: {}", install_list.join(", "));

        let mut cmd = cmd!(PKGIN_BIN, "install");
        cmd.args(&install_list);

        return_if_noop!(opts);

        let output = cmd.output()?;

        if output.status.success() {
            Ok(ApplySummary {
                resources,
                errors: 0,
                changes: install_list.len() as u32,
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
        }
    }

    if remove_list.is_empty() {
        tracing::debug!("no packages to remove");
        Ok(NO_RESOURCES_TO_CHANGE)
    } else {
        tracing::info!("removing: {}", remove_list.join(", "));

        let mut cmd = cmd!(PKGIN_BIN, "remove");
        cmd.args(&remove_list);

        return_if_noop!(opts);

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

fn pkgin_output() -> anyhow::Result<String> {
    cmd_output!(PKGIN_BIN, "list")
}

fn parse_pkg_output(output: &str) -> GlobalPkgins {
    let mut installed: Vec<String> = Vec::new();

    for l in output.trim().lines() {
        let bits: Vec<_> = l.split_whitespace().collect();

        if bits.len() != 2 {
            continue;
        }

        let name_bits: Vec<_> = bits[0].rsplitn(2, "-").collect();

        if bits[1].starts_with('i') {
            installed.push(name_bits[0].to_owned());
        }
    }

    GlobalPkgins { installed }
}
