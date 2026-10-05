use super::types::{Publisher, PublisherName};
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

            let current_state = parse::parse_publisher(&raw_publisher_info)
                .with_context(|| format!("failed to parse raw publisher info for {}", self.name))?;

            if self.desired_state == current_state {
                tracing::debug!("publisher {} is correct", self.name);
                Ok(ONE_RESOURCE_NO_CHANGE)
            } else {
                tracing::info!("modifying publisher {}", self.name);
                self.set_publisher(Some(current_state), opts)
            }
        } else {
            tracing::info!("creating publisher {}", self.name);
            self.set_publisher(None, opts)
        }
    }

    fn set_publisher(
        &self,
        current: Option<Publisher>,
        opts: &ApplyOpts,
    ) -> anyhow::Result<ApplySummary> {
        let origin_cmds = self.origin_cmds(current.as_ref());
        let mirror_cmds = self.mirror_cmds(current.as_ref());

        if !opts.noop {
            for mut cmd in origin_cmds {
                run_cmd!(cmd)?;
            }

            for mut cmd in mirror_cmds {
                run_cmd!(cmd)?;
            }
        }

        Ok(ONE_RESOURCE_ONE_CHANGE)
    }

    fn origin_cmds(&self, current: Option<&Publisher>) -> Vec<Command> {
        self.desired_state
            .origins
            .iter()
            .filter_map(|origin| {
                if let Some(c) = &current
                    && c.origins.contains(origin)
                {
                    tracing::debug!("found existing origin for {}: {}", self.name, origin.uri);
                    None
                } else {
                    let mut cmd_vec = vec!["-G", "*", "-g", origin.uri.as_str()];

                    if let Some(proxy) = &origin.proxy {
                        cmd_vec.push("--proxy");
                        cmd_vec.push(proxy.as_str());
                    }

                    cmd_vec.push(&self.name);
                    Some(self.prep_cmd(cmd_vec.as_ref()))
                }
            })
            .collect()
    }

    fn mirror_cmds(&self, current: Option<&Publisher>) -> Vec<Command> {
        self.desired_state
            .mirrors
            .iter()
            .flatten()
            .filter_map(|mirror| {
                if current
                    .as_ref()
                    .and_then(|c| c.mirrors.as_ref())
                    .is_some_and(|m| m.contains(mirror))
                {
                    tracing::debug!("found existing mirror for {}: {}", self.name, mirror.uri);
                    None
                } else {
                    let mut cmd_vec = vec!["-M", "*", "-m", mirror.uri.as_str()];

                    if let Some(proxy) = &mirror.proxy {
                        cmd_vec.push("--proxy");
                        cmd_vec.push(proxy.as_str());
                    }

                    cmd_vec.push(&self.name);
                    Some(self.prep_cmd(cmd_vec.as_ref()))
                }
            })
            .collect()
    }

    fn prep_cmd(&self, args: &[&str]) -> Command {
        let mut cmd = Command::new(PKG_BIN);
        cmd.arg("set-publisher");

        for arg in args {
            cmd.arg(arg);
        }

        tracing::debug!(command = cmd::to_string(&cmd));
        cmd
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::publisher::types::{Mirror, Origin, OriginOrMirror};
    use pretty_assertions::assert_eq;
    use tester::{deserialized_example, janet2json};
    use url::Url;

    #[test]
    fn test_change_origin_cmds_everything_is_correct() {
        let example_json = janet2json(indoc::indoc! {
                r#"
                (publisher/ensure "tester"
                    (publisher/origin "https://pkg.lan.id264.net"
                                      :proxy "http://10.0.2.11:3128"))
                "#
        });

        let sut: PublisherEnsure = serde_json::from_str(&example_json).unwrap();

        let existing_publisher = Publisher {
            origins: vec![OriginOrMirror {
                uri: Url::parse("https://pkg.lan.id264.net").unwrap(),
                proxy: Some(Url::parse("http://10.0.2.11:3128").unwrap()),
            }],
            mirrors: None,
        };

        assert!(sut.origin_cmds(Some(&existing_publisher)).is_empty());
        assert!(sut.mirror_cmds(Some(&existing_publisher)).is_empty());
    }

    #[test]
    fn test_change_origin_cmds_add_mirror() {
        let example_json = janet2json(indoc::indoc! {
                r#"
                (publisher/ensure "tester"
                    (publisher/origin "https://pkg.lan.id264.net"
                                      :proxy "http://10.0.2.11:3128")
                    (publisher/mirror "https://mirror.lan.id264.net"))
                "#
        });

        let sut: PublisherEnsure = serde_json::from_str(&example_json).unwrap();

        let existing_publisher = Publisher {
            origins: vec![OriginOrMirror {
                uri: Url::parse("https://pkg.lan.id264.net").unwrap(),
                proxy: Some(Url::parse("http://10.0.2.11:3128").unwrap()),
            }],
            mirrors: None,
        };

        assert!(sut.origin_cmds(Some(&existing_publisher)).is_empty());
        let m_result = sut.mirror_cmds(Some(&existing_publisher));

        assert_eq!(1, m_result.len());
        assert_eq!(
            "/bin/pkg set-publisher -M * -m https://mirror.lan.id264.net/ tester",
            cmd::to_string(&m_result[0])
        );
    }

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
