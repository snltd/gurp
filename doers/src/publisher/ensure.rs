use super::types::{OriginOrMirror, Publisher, PublisherName, TargetType};
use super::{functions, parse};
use anyhow::Context;
use common::cmd;
use common::constants::{ONE_RESOURCE_NO_CHANGE, ONE_RESOURCE_ONE_CHANGE, PKG_BIN};
use common::types::{ApplyOpts, ApplySummary};
use os_types::GurpId;
use serde::Deserialize;
use std::process::Command;

#[derive(Deserialize, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct PublisherEnsure {
    #[serde(rename = "_id")]
    pub id: GurpId,
    pub name: PublisherName,
    #[serde(flatten)]
    pub desired_state: Publisher,
}

impl PublisherEnsure {
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        if functions::publisher_exists(&self.name)? {
            let raw_publisher_info = cmd_output!(PKG_BIN, "publisher", &self.name)
                .with_context(|| format!("cannot get info for publisher: {}", self.name))?;

            let current_state =
                parse::parse_publisher(&raw_publisher_info).context("failed to parse publisher")?;

            if self.desired_state == current_state {
                tracing::debug!("publisher {} is correct", self.name);
                Ok(ONE_RESOURCE_NO_CHANGE)
            } else {
                self.align_publisher(&current_state, opts)
            }
        } else {
            self.create_publisher(opts)
        }
    }

    fn add_items(&self, mut cmd: Command, items: &[OriginOrMirror], ttype: TargetType) -> Command {
        for item in items {
            cmd.arg(match ttype {
                TargetType::Origin => "-g",
                TargetType::Mirror => "-m",
            });

            cmd.arg(item.uri.as_str());

            if let Some(proxy) = &item.proxy {
                tracing::info!(
                    "publisher {} origin {}: adding proxy {proxy}",
                    self.name,
                    item.uri
                );
                cmd.args(["--proxy", proxy.as_str()]);
            }
        }

        cmd
    }

    fn create_publisher(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        tracing::info!("creating publisher {}", self.name);

        let mut cmd = Command::new(PKG_BIN);
        cmd.arg("set-publisher");
        cmd = self.add_items(cmd, self.desired_state.origins.as_ref(), TargetType::Origin);

        if let Some(mirrors) = &self.desired_state.mirrors {
            cmd = self.add_items(cmd, mirrors, TargetType::Mirror);
        }

        cmd.arg(&self.name);

        tracing::debug!(command = cmd::to_string(&cmd));

        if !opts.noop {
            run_cmd!(cmd).with_context(|| format!("failed to create publisher {}", self.name))?;
        }

        Ok(ONE_RESOURCE_ONE_CHANGE)
    }

    fn set_origin_or_mirror(
        &self,
        target_type: TargetType,
        target: &OriginOrMirror,
        opts: &ApplyOpts,
    ) -> anyhow::Result<()> {
        let mut cmd = Command::new(PKG_BIN);
        cmd.arg("set-publisher");

        tracing::info!(
            "publisher {}: setting {} {}",
            self.name,
            target_type,
            target.uri
        );

        cmd.arg("-p");
        cmd.arg(target.uri.as_str());

        if let Some(proxy) = &target.proxy {
            tracing::info!(
                "publisher {} origin {}: adding proxy {proxy}",
                self.name,
                target.uri
            );
            cmd.args(["--proxy", proxy.as_str()]);
        }

        cmd.arg(&self.name);

        tracing::debug!(command = cmd::to_string(&cmd));

        if !opts.noop {
            run_cmd!(cmd).with_context(|| format!("failed to create publisher {}", self.name))?;
        }

        Ok(())
    }

    fn align_publisher(
        &self,
        current: &Publisher,
        opts: &ApplyOpts,
    ) -> anyhow::Result<ApplySummary> {
        tracing::info!("modifying publisher {}", self.name);

        // The pkg interface is a bit clunky, and though you can add an origin and one or more
        // mirrors in the same command, you can only add one proxy per command. So, to be on the
        // safe side, we'll issue separate commands for each action. Slow, but you probably only
        // ever do it once per host.
        for origin in &self.desired_state.origins {
            if !current.origins.contains(origin) {
                self.set_origin_or_mirror(TargetType::Origin, origin, opts)?;
            }
        }

        for mirror in self.desired_state.mirrors.iter().flatten() {
            if !current.mirrors.as_ref().is_some_and(|m| m.contains(mirror)) {
                self.set_origin_or_mirror(TargetType::Mirror, mirror, opts)?;
            }
        }

        for origin in &current.origins {
            if !self.desired_state.origins.contains(origin) {
                tracing::info!("publisher {}: removing origin {}", self.name, origin.uri);
                let _ = cmd_change_or_noop!(
                    opts,
                    PKG_BIN,
                    "set-publisher",
                    "-G",
                    &origin.uri.as_str(),
                    &self.name
                )?;
            }
        }

        for mirror in current.mirrors.iter().flatten() {
            if !self
                .desired_state
                .mirrors
                .as_ref()
                .is_some_and(|m| m.contains(mirror))
            {
                tracing::info!("publisher {}: removing mirror {}", self.name, mirror.uri);
                let _ = cmd_change_or_noop!(
                    opts,
                    PKG_BIN,
                    "set-publisher",
                    "-M",
                    &mirror.uri.as_str(),
                    &self.name
                )?;
            }
        }

        Ok(ONE_RESOURCE_ONE_CHANGE)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::publisher::types::{Mirror, Origin};
    use pretty_assertions::assert_eq;
    use tester::deserialized_example;
    use url::Url;

    #[test]
    fn test_deserialize_publisher_ensure_new_publisher() {
        assert_eq!(
            PublisherEnsure {
                id: GurpId::new("/NO-ROLE/publisher/example").unwrap(),
                name: "example".to_owned(),
                desired_state: Publisher {
                    origins: vec![Origin {
                        uri: Url::parse("http://pkg.lan.id264.net").unwrap(),
                        proxy: Some(Url::parse("http://10.2.0.20/1837").unwrap()),
                    }],
                    mirrors: Some(vec![Mirror {
                        uri: Url::parse("http://mirror.lan.id264.net").unwrap(),
                        proxy: None,
                    }]),
                }
            },
            deserialized_example("publisher/ensure-new-publisher.janet")
        );
    }
}
