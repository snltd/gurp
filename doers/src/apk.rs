use anyhow::bail;
use common::cmd;
use common::constants::{APK_BIN, NO_RESOURCES_TO_CHANGE};
use common::types::{ApplyOpts, ApplySummary};
use regex::Regex;
use serde::Deserialize;
use std::process::Command;
use std::sync::LazyLock;

static CURRENT_APK_OUTPUT: LazyLock<String> =
    LazyLock::new(|| apk_output().expect("Could not get apk list"));

type ApkName = String;
type InstalledApks = Vec<ApkName>;

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct GurpApkEnsure {
    pub name: ApkName,
}

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct GurpApkRemove {
    pub name: ApkName,
}

type EnsureList = Vec<GurpApkEnsure>;
type RemoveList = Vec<GurpApkRemove>;

pub fn collect_and_ensure(apk_list: &EnsureList, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
    let resources = apk_list.len() as u32;
    let installed_apks = parse_apk_output(&CURRENT_APK_OUTPUT);
    let apk_names: Vec<_> = apk_list.iter().map(|r| &r.name).collect();
    let mut install_list = Vec::new();

    for apk in &apk_names {
        if installed_apks.contains(apk) {
            tracing::debug!("already installed: {}", apk);
            continue;
        } else {
            tracing::debug!("scheduled for install: {}", apk);
            install_list.push(apk.as_str());
        }
    }

    tracing::debug!("ensure apk list: {}", install_list.join(" "));

    if install_list.is_empty() {
        tracing::debug!("no packages to install");
        Ok(ApplySummary {
            resources,
            changes: 0,
        })
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
        let output = cmd.output()?;

        if output.status.success() {
            Ok(ApplySummary {
                resources,
                changes: install_list.len() as u32,
            })
        } else {
            bail!(String::from_utf8_lossy(&output.stderr).into_owned())
        }
    }
}

pub fn collect_and_remove(apk_list: &RemoveList, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
    let resources = apk_list.len() as u32;
    let installed_apks = parse_apk_output(&CURRENT_APK_OUTPUT);
    let apk_names: Vec<_> = apk_list.iter().map(|r| &r.name).collect();
    let mut remove_list = Vec::new();

    for apk in apk_names {
        if installed_apks.contains(apk) {
            tracing::debug!("scheduled for removal: {}", apk);
            remove_list.push(apk.as_str());
        } else {
            tracing::debug!("not present: {}", apk);
            continue;
        }
    }

    if remove_list.is_empty() {
        tracing::debug!("no packages to remove");
        Ok(NO_RESOURCES_TO_CHANGE)
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
        let output = cmd.output()?;

        if output.status.success() {
            Ok(ApplySummary {
                resources,
                changes: remove_list.len() as u32,
            })
        } else {
            bail!(String::from_utf8_lossy(&output.stderr).into_owned())
        }
    }
}

fn apk_output() -> anyhow::Result<String> {
    cmd_output!(APK_BIN, "list", "-I")
}

fn parse_apk_output(output: &str) -> InstalledApks {
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
            GurpApkEnsure {
                name: "rust".to_owned(),
            },
            deserialized_example("apk/ensure-rust-package.janet")
        );
    }

    #[test]
    fn test_deserialize_apk_remove_go_package() {
        assert_eq!(
            GurpApkRemove {
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

        assert_eq!(expected, parse_apk_output(sample_output));
    }
}
