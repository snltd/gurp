use crate::types::ApplyResult;
use anyhow::Context;
use common::cmd;
use common::constants::{NO_RESOURCES_TO_CHANGE, PKG_BIN};
use common::types::{ApplyOpts, ApplySummary, ChangedIds};
use serde::Deserialize;
use std::process::Command;
use std::sync::LazyLock;

static CURRENT_PKG_OUTPUT: LazyLock<String> =
    LazyLock::new(|| get_pkg_list().expect("Could not get pkg list"));

type PkgName = String;

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
struct AllPkgs {
    available: Vec<PkgName>,
    installed: Vec<PkgName>,
}

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct GurpPkgEnsure {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: PkgName,
}

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct GurpPkgRemove {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: PkgName,
}

type EnsureList = Vec<GurpPkgEnsure>;
type RemoveList = Vec<GurpPkgRemove>;

pub fn collect_and_ensure(pkg_list: &EnsureList, opts: &ApplyOpts) -> ApplyResult {
    let mut changed_ids = ChangedIds::default();

    if pkg_list.is_empty() {
        return Ok((NO_RESOURCES_TO_CHANGE, changed_ids));
    }

    let resources = pkg_list.len() as u32;
    let all_pkgs = parse_pkg_list(&CURRENT_PKG_OUTPUT);
    let mut install_list = Vec::new();

    for pkg in pkg_list {
        if all_pkgs.installed.contains(&pkg.name) {
            tracing::debug!("already installed: {}", pkg.name);
        } else if all_pkgs.available.contains(&pkg.name) {
            tracing::debug!("scheduled for install: {}", pkg.name);
            install_list.push(pkg.name.as_str());
            changed_ids.insert(pkg.id.clone());
        } else {
            tracing::warn!("not available: {}", pkg.name);
        }
    }

    tracing::debug!("ensure pkg list: {}", install_list.join(" "));

    if install_list.is_empty() {
        tracing::debug!("no packages to install");
        Ok((
            ApplySummary {
                resources,
                changes: 0,
            },
            changed_ids,
        ))
    } else {
        tracing::info!("installing: {}", install_list.join(", "));

        let mut cmd = Command::new(PKG_BIN);
        cmd.arg("install");

        if opts.noop {
            cmd.arg("-n");
        }

        cmd.args(&install_list);
        tracing::debug!(command = cmd::to_string(&cmd));

        run_cmd!(cmd).context("failed to install packages")?;

        Ok((
            ApplySummary {
                resources,
                changes: install_list.len() as u32,
            },
            changed_ids,
        ))
    }
}

pub fn collect_and_remove(pkg_list: &RemoveList, opts: &ApplyOpts) -> ApplyResult {
    let mut changed_ids = ChangedIds::default();

    if pkg_list.is_empty() {
        return Ok((NO_RESOURCES_TO_CHANGE, changed_ids));
    }

    let resources = pkg_list.len() as u32;
    let all_pkgs = parse_pkg_list(&CURRENT_PKG_OUTPUT);
    let mut remove_list = Vec::new();

    for pkg in pkg_list {
        if all_pkgs.installed.contains(&pkg.name) {
            tracing::debug!("scheduled for removal: {}", pkg.name);
            remove_list.push(pkg.name.as_str());
            changed_ids.insert(pkg.id.clone());
        } else {
            tracing::debug!("not present: {}", pkg.name);
        }
    }

    if remove_list.is_empty() {
        tracing::debug!("no packages to remove");
        Ok((
            ApplySummary {
                resources,
                changes: 0,
            },
            changed_ids,
        ))
    } else {
        tracing::info!("removing: {}", remove_list.join(", "));

        let mut cmd = Command::new(PKG_BIN);
        cmd.arg("uninstall");
        cmd.arg("-q");

        if opts.noop {
            cmd.arg("-n");
        }

        cmd.args(&remove_list);
        tracing::debug!(command = cmd::to_string(&cmd));

        run_cmd!(cmd).context("failed to remove packages")?;

        Ok((
            ApplySummary {
                resources,
                changes: remove_list.len() as u32,
            },
            changed_ids,
        ))
    }
}

fn get_pkg_list() -> anyhow::Result<String> {
    cmd_output!(PKG_BIN, "list", "-aHo", "name,flags").context("failed to get package list")
}

fn parse_pkg_list(output: &str) -> AllPkgs {
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

    AllPkgs {
        available,
        installed,
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use tester::deserialized_example;

    #[test]
    fn test_deserialize_pkg_ensure_rust_package() {
        assert_eq!(
            GurpPkgEnsure {
                id: "/NO-ROLE/pkg/ooce_developer_rust".to_owned(),
                name: "ooce/developer/rust".to_owned(),
            },
            deserialized_example("pkg/ensure-rust-package.janet")
        );
    }

    #[test]
    fn test_deserialize_pkg_remove_go_package() {
        assert_eq!(
            GurpPkgRemove {
                id: "/NO-ROLE/pkg/ooce_developer_go".to_owned(),
                name: "ooce/developer/go".to_owned(),
            },
            deserialized_example("pkg/remove-go-package.janet")
        );
    }

    #[test]
    fn test_parse_pkg_list() {
        let sample_output = indoc::indoc! { r#"
            ooce/extra-build-tools                          im-
            ooce/file/acltool                               ---
            ooce/file/lsof                                  i--
            ooce/file/tree                                  ---
            ooce/fonts/liberation                           i--
            ooce/library/apr                                i--
        "#};

        let expected = AllPkgs {
            available: vec!["ooce/file/acltool".to_owned(), "ooce/file/tree".to_owned()],
            installed: vec![
                "ooce/extra-build-tools".to_owned(),
                "ooce/file/lsof".to_owned(),
                "ooce/fonts/liberation".to_owned(),
                "ooce/library/apr".to_owned(),
            ],
        };

        assert_eq!(expected, parse_pkg_list(sample_output));
    }
}
