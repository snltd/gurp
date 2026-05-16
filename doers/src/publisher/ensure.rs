use crate::publisher::types::{Publisher, PublisherName};
use crate::publisher::{functions, parse};
use anyhow::Context;
use common::cmd;
use common::constants::{ONE_RESOURCE_NO_CHANGE, ONE_RESOURCE_ONE_CHANGE, PKG_BIN};
use common::types::{ApplyOpts, ApplySummary};
use serde::Deserialize;
use std::process::Command;

#[derive(Deserialize, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct GurpPublisherEnsure {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: PublisherName,
    #[serde(flatten)]
    pub desired_state: Publisher,
}

impl GurpPublisherEnsure {
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        if functions::publisher_exists(&self.name)? {
            let raw_publisher_info = cmd_output!(PKG_BIN, "publisher", &self.name)
                .with_context(|| format!("cannot get info for publisher: {}", &self.name))?;

            let current_state = parse::parse_publisher(&raw_publisher_info);

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

    fn create_publisher(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        tracing::info!("creating publisher {}", self.name);

        let mut cmd = Command::new(PKG_BIN);
        cmd.arg("set-publisher");

        for origin in &self.desired_state.origins {
            cmd.args(["-g", &origin.uri]);
        }

        for mirror in self.desired_state.mirrors.iter().flatten() {
            cmd.args(["-m", &mirror.uri]);
        }

        cmd.arg(&self.name);

        tracing::debug!(command = cmd::to_string(&cmd));

        if !opts.noop {
            run_cmd!(cmd).with_context(|| format!("failed to create publisher {}", self.name))?;
        }

        Ok(ONE_RESOURCE_ONE_CHANGE)
    }

    fn align_publisher(
        &self,
        current: &Publisher,
        opts: &ApplyOpts,
    ) -> anyhow::Result<ApplySummary> {
        tracing::info!("modifying publisher {}", self.name);

        let mut cmd = Command::new(PKG_BIN);
        cmd.arg("set-publisher");

        for origin in &self.desired_state.origins {
            if !current.origins.contains(origin) {
                tracing::info!("publisher {}: adding origin {}", self.name, origin.uri);
                cmd.args(["-g", &origin.uri]);
            }
        }

        for mirror in self.desired_state.mirrors.iter().flatten() {
            if !current.mirrors.as_ref().is_some_and(|m| m.contains(mirror)) {
                tracing::info!("publisher {}: adding mirror {}", self.name, mirror.uri);
                cmd.args(["-m", &mirror.uri]);
            }
        }

        for origin in &current.origins {
            if !self.desired_state.origins.contains(origin) {
                tracing::info!("publisher {}: removing origin {}", self.name, origin.uri);
                cmd.args(["-G", &origin.uri]);
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
                cmd.args(["-M", &mirror.uri]);
            }
        }

        cmd.arg(&self.name);

        tracing::debug!(command = cmd::to_string(&cmd));

        if !opts.noop {
            run_cmd!(cmd).with_context(|| format!("failed to create publisher {}", self.name))?;
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

    #[test]
    fn test_deserialize_publisher_ensure_new_publisher() {
        assert_eq!(
            GurpPublisherEnsure {
                id: "/NO-ROLE/publisher/example".to_owned(),
                name: "example".to_owned(),
                desired_state: Publisher {
                    origins: vec![Origin {
                        uri: "http://pkg.lan.id264.net".to_owned(),
                        proxy: Some("http://10.2.0.20/1837".to_owned()),
                    }],
                    mirrors: Some(vec![Mirror {
                        uri: "http://mirror.lan.id264.net".to_owned(),
                        proxy: None,
                    }]),
                }
            },
            deserialized_example("publisher/ensure-new-publisher.janet")
        );
    }
}
