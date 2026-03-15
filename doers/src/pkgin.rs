use crate::types::ApplyResult;
use common::cmd;
use common::constants::{NO_RESOURCES_TO_CHANGE, PKGIN_BIN};
use common::types::{ApplyOpts, ApplySummary, ChangedIds};
use serde::Deserialize;
use std::collections::BTreeSet;
use std::process::Command;
use std::sync::LazyLock;

static CURRENT_PKG_OUTPUT: LazyLock<String> =
    LazyLock::new(|| get_package_list().expect("Could not get pkgin list"));

type PkginName = String;
type InstalledPkgs = Vec<PkginName>;
type EnsureList = Vec<GurpPkginEnsure>;
type RemoveList = Vec<GurpPkginRemove>;

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct GurpPkginEnsure {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: PkginName,
}

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct GurpPkginRemove {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: PkginName,
}

pub fn collect_and_ensure(pkg_list: &EnsureList, opts: &ApplyOpts) -> ApplyResult {
    let mut changed_ids: ChangedIds = BTreeSet::new();

    if pkg_list.is_empty() {
        return Ok((NO_RESOURCES_TO_CHANGE, changed_ids));
    }

    let installed_pkgs = parse_pkg_list(&CURRENT_PKG_OUTPUT);
    let resources = pkg_list.len() as u32;
    let mut install_list = Vec::new();

    for pkg in pkg_list {
        if installed_pkgs.contains(&pkg.name) {
            tracing::debug!("already installed: {}", pkg.name);
        } else {
            tracing::debug!("scheduled for install: {}", pkg.name);
            install_list.push(pkg.name.as_str());
            changed_ids.insert(pkg.id.clone());
        }
    }

    tracing::debug!("ensure pkgin list: {}", install_list.join(" "));

    if install_list.is_empty() {
        tracing::debug!("no pkgsrc packages to install");
        Ok((
            ApplySummary {
                resources,
                changes: 0,
            },
            changed_ids,
        ))
    } else {
        tracing::info!("installing: {}", install_list.join(", "));

        let mut cmd = Command::new(PKGIN_BIN);
        cmd.arg("-y");
        cmd.arg("install");
        cmd.args(&install_list);

        tracing::debug!(command = cmd::to_string(&cmd));

        if !opts.noop {
            run_cmd!(cmd)?;
        }

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
    let installed_pkgs = parse_pkg_list(&CURRENT_PKG_OUTPUT);
    let mut remove_list = Vec::new();

    for pkg in pkg_list {
        if installed_pkgs.contains(&pkg.name) {
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

        let mut cmd = Command::new(PKGIN_BIN);
        cmd.arg("-y");
        cmd.arg("remove");
        cmd.args(&remove_list);

        tracing::debug!(command = cmd::to_string(&cmd));

        if !opts.noop {
            run_cmd!(cmd)?;
        }

        Ok((
            ApplySummary {
                resources,
                changes: remove_list.len() as u32,
            },
            changed_ids,
        ))
    }
}

fn get_package_list() -> anyhow::Result<String> {
    cmd_output!(PKGIN_BIN, "list")
}

fn parse_pkg_list(output: &str) -> InstalledPkgs {
    let mut installed: Vec<String> = Vec::new();

    for l in output.trim().lines() {
        let bits: Vec<_> = l.split_whitespace().collect();

        if bits.len() >= 2
            && let Some(pkg_name) = bits[0].rsplitn(2, "-").last()
        {
            installed.push(pkg_name.to_owned());
        }
    }

    installed
}

#[cfg(test)]
mod test {
    use super::*;
    use tester::deserialized_example;

    #[test]
    fn test_deserialize_pkgin_ensure_rust_package() {
        assert_eq!(
            GurpPkginEnsure {
                id: "/NO-ROLE/pkgin/rust".to_owned(),
                name: "rust".to_owned(),
            },
            deserialized_example("pkgin/ensure-rust-package.janet")
        );
    }

    #[test]
    fn test_deserialize_pkgin_remove_go_package() {
        assert_eq!(
            GurpPkginRemove {
                id: "/NO-ROLE/pkgin/go".to_owned(),
                name: "go".to_owned(),
            },
            deserialized_example("pkgin/remove-go-package.janet")
        );
    }

    #[test]
    fn test_parse_pkg_list() {
        let sample_output = indoc::indoc! { r#"
            libxml2-2.12.9nb2    XML parser library from the GNOME project
            ruby33-3.3.6         Ruby 3.3.6 release package
            ruby33-mini_portile2-2.8.7 Simple autoconf builder for developers
            Zlib-1.3.1           General purpose data compression library
        "#
        };

        assert_eq!(
            vec![
                "libxml2".to_owned(),
                "ruby33".to_owned(),
                "ruby33-mini_portile2".to_owned(),
                "Zlib".to_owned(),
            ],
            parse_pkg_list(sample_output)
        );
    }
}
