use crate::types::ApplyResult;
use common::cmd;
use common::constants::{APK_BIN, NO_RESOURCES_TO_CHANGE};
use common::types::{ApplyOpts, ApplySummary, ChangedIds};
use os_types::GurpId;
use regex::Regex;
use serde::Deserialize;
use std::process::Command;
use std::sync::LazyLock;

static CURRENT_APK_OUTPUT: LazyLock<String> =
    LazyLock::new(|| get_pkg_list().expect("Could not get apk list"));

type ApkName = String;
type InstalledApks = Vec<ApkName>;
type EnsureList = Vec<ApkEnsure>;
type RemoveList = Vec<ApkRemove>;

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct ApkEnsure {
    #[serde(rename = "_id")]
    pub id: GurpId,
    pub name: ApkName,
}

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct ApkRemove {
    #[serde(rename = "_id")]
    pub id: GurpId,
    pub name: ApkName,
}

pub fn collect_and_ensure(pkg_list: &EnsureList, opts: &ApplyOpts) -> ApplyResult {
    let mut changed_ids = ChangedIds::default();

    if pkg_list.is_empty() {
        return Ok((NO_RESOURCES_TO_CHANGE, changed_ids));
    }

    let resources = pkg_list.len() as u32;
    let installed_pkgs = parse_pkg_list(&CURRENT_APK_OUTPUT);
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

    tracing::debug!("ensure apk list: {}", install_list.join(" "));

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

        let mut cmd = Command::new(APK_BIN);
        cmd.arg("add");
        cmd.arg("--quiet");
        cmd.arg("--update-cache");

        if opts.noop {
            cmd.arg("--simulate");
        }

        cmd.args(&install_list);

        tracing::debug!(command = cmd::to_string(&cmd));

        run_cmd!(cmd)?;

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
    let installed_pkgs = parse_pkg_list(&CURRENT_APK_OUTPUT);
    let mut remove_list = Vec::new();

    for pkg in pkg_list {
        if installed_pkgs.contains(&pkg.name) {
            tracing::debug!("scheduled for removal: {}", pkg.name);
            remove_list.push(pkg.name.as_str());
            changed_ids.insert(pkg.id.clone());
        } else {
            tracing::debug!("not present: {}", pkg.name);
            continue;
        }
    }

    if remove_list.is_empty() {
        tracing::debug!("no packages to remove");
        Ok((NO_RESOURCES_TO_CHANGE, changed_ids))
    } else {
        tracing::info!("removing: {}", remove_list.join(", "));

        let mut cmd = Command::new(APK_BIN);
        cmd.arg("del");
        cmd.arg("--quiet");

        if opts.noop {
            cmd.arg("--simulate");
        }

        cmd.args(&remove_list);

        tracing::debug!(command = cmd::to_string(&cmd));

        run_cmd!(cmd)?;

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
    cmd_output!(APK_BIN, "list", "-I")
}

fn parse_pkg_list(output: &str) -> InstalledApks {
    let rx = Regex::new(r"-\d").unwrap();
    let mut installed: Vec<_> = Vec::new();

    for l in output.trim().lines() {
        let bits: Vec<_> = l.split_whitespace().collect();

        if bits.len() < 2 {
            continue;
        }

        let name_and_version = bits[0];
        let name_bits: Vec<_> = rx.split(name_and_version).collect();

        if name_bits.len() < 2 {
            continue;
        }

        installed.push(name_bits[0].to_owned());
    }

    installed
}

#[cfg(test)]
mod test {
    use super::*;
    use tester::deserialized_example;

    #[test]
    fn test_deserialize_apk_ensure_rust_package() {
        assert_eq!(
            ApkEnsure {
                id: GurpId::new("/NO-ROLE/apk/rust").unwrap(),
                name: "rust".to_owned(),
            },
            deserialized_example("apk/ensure-rust-package.janet")
        );
    }

    #[test]
    fn test_deserialize_apk_remove_go_package() {
        assert_eq!(
            ApkRemove {
                id: GurpId::new("/NO-ROLE/apk/go").unwrap(),
                name: "go".to_owned(),
            },
            deserialized_example("apk/remove-go-package.janet")
        );
    }

    #[test]
    fn test_parse_apk_output() {
        let sample_output = indoc::indoc! { r#")
            yaml-static-0.2.5-r2 x86_64 {yaml} (MIT))
            yamllint-1.35.1-r1 x86_64 {yamllint} (GPL-3.0-or-later))
            yamllint-pyc-1.35.1-r1 x86_64 {yamllint} (GPL-3.0-or-later))
            yank-1.3.0-r0 x86_64 {yank} (MIT)
            yank-doc-1.3.0-r0 x86_64 {yank} (MIT)
            yara-4.5.2-r0 x86_64 {yara} (BSD-3-Clause)
            yara-dev-4.5.2-r0 x86_64 {yara} (BSD-3-Clause)
            yara-doc-4.5.2-r0 x86_64 {yara} (BSD-3-Clause)
            yarn-1.22.22-r1 x86_64 {yarn} (BSD-2-Clause)
            yascreen-1.99-r0 x86_64 {yascreen} (GPL-3.0-or-later)
            yascreen-dev-1.99-r0 x86_64 {yascreen} (GPL-3.0-or-later)
        "#};

        let expected = vec![
            "yaml-static".to_owned(),
            "yamllint".to_owned(),
            "yamllint-pyc".to_owned(),
            "yank".to_owned(),
            "yank-doc".to_owned(),
            "yara".to_owned(),
            "yara-dev".to_owned(),
            "yara-doc".to_owned(),
            "yarn".to_owned(),
            "yascreen".to_owned(),
            "yascreen-dev".to_owned(),
        ];

        assert_eq!(expected, parse_pkg_list(sample_output));
    }
}
