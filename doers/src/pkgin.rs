use anyhow::bail;
use common::cmd;
use common::constants::{NO_RESOURCES_TO_CHANGE, PKGIN_BIN};
use common::types::{ApplyOpts, ApplySummary};
use serde::Deserialize;
use std::process::Command;
use std::sync::LazyLock;

static CURRENT_PKG_OUTPUT: LazyLock<String> =
    LazyLock::new(|| pkgin_output().expect("Could not get pkgin list"));

type PkginName = String;

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
struct GlobalPkgins {
    installed: Vec<PkginName>,
}

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct GurpPkginEnsure {
    pub name: PkginName,
}

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct GurpPkginRemove {
    pub name: PkginName,
}

type EnsureList = Vec<GurpPkginEnsure>;
type RemoveList = Vec<GurpPkginRemove>;

pub fn collect_and_ensure(pkg_list: &EnsureList, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
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
            changes: 0,
        })
    } else {
        tracing::info!("installing: {}", install_list.join(", "));

        let mut cmd = Command::new(PKGIN_BIN);
        cmd.arg("-y");
        cmd.arg("install");
        cmd.args(&install_list);

        tracing::debug!(command = cmd::to_string(&cmd));

        return_if_noop!(opts);

        let output = cmd.output()?;

        if output.status.success() {
            Ok(ApplySummary {
                resources,
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

pub fn collect_and_remove(pkg_list: &RemoveList, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
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

        let mut cmd = Command::new(PKGIN_BIN);
        cmd.arg("-y");
        cmd.arg("remove");
        cmd.args(&remove_list);

        tracing::debug!(command = cmd::to_string(&cmd));

        return_if_noop!(opts);

        let output = cmd.output()?;

        if output.status.success() {
            Ok(ApplySummary {
                resources,
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

        if bits.len() >= 2
            && let Some(pkg_name) = bits[0].rsplitn(2, "-").last()
        {
            installed.push(pkg_name.to_owned());
        }
    }

    GlobalPkgins { installed }
}

#[cfg(test)]
mod test {
    use super::*;
    use tester::deserialized_example;

    #[test]
    fn test_deserialize_pkgin_ensure_rust_package() {
        assert_eq!(
            GurpPkginEnsure {
                name: "rust".to_owned(),
            },
            deserialized_example("pkgin/ensure-rust-package.janet")
        );
    }

    #[test]
    fn test_deserialize_pkgin_remove_go_package() {
        assert_eq!(
            GurpPkginRemove {
                name: "go".to_owned(),
            },
            deserialized_example("pkgin/remove-go-package.janet")
        );
    }

    #[test]
    fn test_parse_gem_output() {
        let sample_output = indoc::indoc! { r#"
            libxml2-2.12.9nb2    XML parser library from the GNOME project
            ruby33-3.3.6         Ruby 3.3.6 release package
            ruby33-mini_portile2-2.8.7 Simple autoconf builder for developers
            Zlib-1.3.1           General purpose data compression library
        "#
        };

        assert_eq!(
            GlobalPkgins {
                installed: vec![
                    "libxml2".to_owned(),
                    "ruby33".to_owned(),
                    "ruby33-mini_portile2".to_owned(),
                    "Zlib".to_owned(),
                ]
            },
            parse_pkg_output(sample_output)
        );
    }
}
