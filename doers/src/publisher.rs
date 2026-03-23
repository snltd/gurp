use anyhow::Context;
use common::constants::{ONE_RESOURCE_NO_CHANGE, PKG_BIN};
use common::types::{ApplyOpts, ApplySummary};
use serde::Deserialize;
use std::sync::LazyLock;

static CURRENT_PKG_OUTPUT: LazyLock<Vec<Publisher>> = LazyLock::new(|| {
    parse_publisher_list(&list_publishers().expect("Could not get publisher list"))
});

const PKG_PUBLISHER_FIELDS: usize = 5;

type PublisherName = String;
type PublisherUri = String;

#[derive(Deserialize, Debug)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(rename_all = "lowercase")]
pub enum PublisherType {
    Origin,
    Mirror,
}

#[derive(Deserialize, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct GurpPublisherEnsure {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: PublisherName,
    pub uri: PublisherUri,
    #[serde(rename = "type")]
    pub publisher_type: PublisherType,
}

#[derive(Deserialize, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct GurpPublisherRemove {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: PublisherName,
    pub mirror: Option<PublisherUri>,
}

// We don't care about anything else, for now at least
struct Publisher {
    name: PublisherName,
    uri: PublisherUri,
}

impl GurpPublisherEnsure {
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        let current_publishers = &CURRENT_PKG_OUTPUT;

        let desired_uri = if self.uri.ends_with("/") {
            &self.uri
        } else {
            &format!("{}/", self.uri)
        };

        if let Some(existing) = &current_publishers.iter().find(|p| p.name == self.name) {
            if &existing.uri == desired_uri {
                tracing::debug!("no change to {} publisher", &self.name);
                return Ok(ONE_RESOURCE_NO_CHANGE);
            }

            tracing::info!(
                "change {} publisher URI: {} -> {}",
                self.name,
                existing.uri,
                desired_uri,
            );
        } else {
            tracing::info!("add publisher {}", self.name,);
        }

        let type_flag = match self.publisher_type {
            PublisherType::Origin => "-g",
            PublisherType::Mirror => "-m",
        };
        cmd_change_or_noop!(
            opts,
            PKG_BIN,
            "set-publisher",
            type_flag,
            &desired_uri,
            &self.name
        )
        .with_context(|| format!("failed to set publisher {} -> {desired_uri}", self.name))
    }
}

impl GurpPublisherRemove {
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        if let Some(mirror) = &self.mirror {
            // Remove the mirror from the publisher
            if publisher_exists(&self.name)? {
                if publisher_has_mirror(&self.name, mirror)? {
                    let mut cmd = cmd!(
                        PKG_BIN,
                        "set-publisher",
                        "--remove-mirror",
                        mirror,
                        &self.name
                    );
                    return_if_noop!(opts);

        if current_publishers.iter().any(|p| p.name == self.name) {
            cmd_change_or_noop!(opts, PKG_BIN, "unset-publisher", &self.name)
                .with_context(|| format!("failed to unset publisher {}", self.name))
                    one_change_or_stderr!(
                        cmd,
                        format!(
                            "error removing mirror '{mirror}' from publisher {}",
                            self.name
                        )
                    )
                } else {
                    tracing::debug!("publisher {} has no mirror {}", self.name, mirror);
                    Ok(ONE_RESOURCE_NO_CHANGE)
                }
            } else {
                tracing::warn!(
                    "publisher {} does not exist, so cannot remove mirror",
                    self.name
                );
                Ok(ONE_RESOURCE_NO_CHANGE)
            }
        } else {
            // Remove the publisher
            if publisher_exists(&self.name)? {
                let mut cmd = cmd!(PKG_BIN, "unset-publisher", &self.name);
                return_if_noop!(opts);

                one_change_or_stderr!(cmd, format!("error unsetting '{}'; publisher", self.name))
            } else {
                tracing::debug!("publisher {} does not exist", self.name);
                Ok(ONE_RESOURCE_NO_CHANGE)
            }
        }
    }
}

fn publisher_has_mirror(publisher: &str, mirror: &str) -> anyhow::Result<bool> {
    let output = cmd_output!(PKG_BIN, "publisher", publisher)?;
    let pattern = format!("Mirror URI: {mirror}");

    Ok(output.lines().any(|l| l.trim() == pattern))
}

fn publisher_exists(name: &str) -> anyhow::Result<bool> {
    Ok(CURRENT_PKG_OUTPUT.iter().any(|p| p.name == name))
}

fn list_publishers() -> anyhow::Result<String> {
    tracing::debug!("looking up publishers");
    cmd_output!(PKG_BIN, "publisher", "-H").context("failed to list publishers")
}

fn parse_publisher_list(output: &str) -> Vec<Publisher> {
    output
        .trim()
        .lines()
        .filter_map(|l| {
            let bits: Vec<_> = l.split_whitespace().collect();

            if bits.len() != PKG_PUBLISHER_FIELDS {
                None
            } else if bits[1] == "origin" {
                Some(Publisher {
                    name: bits[0].to_string(),
                    uri: bits[4].to_string(),
                })
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
    use tester::deserialized_example;

    #[test]
    fn test_deserialize_publisher_ensure_new_publisher() {
        assert_eq!(
            GurpPublisherEnsure {
                id: "/NO-ROLE/publisher/new_publisher".to_owned(),
                name: "new_publisher".to_owned(),
                uri: "http://pkg.lan.id264.net".to_owned(),
                publisher_type: PublisherType::Origin,
            },
            deserialized_example("publisher/ensure-new-publisher.janet")
        );
    }

    #[test]
    fn test_deserialize_publisher_remove_old_publisher() {
        assert_eq!(
            GurpPublisherRemove {
                id: "/NO-ROLE/publisher/old_publisher".to_owned(),
                name: "old_publisher".to_owned(),
                mirror: None,
            },
            deserialized_example("publisher/remove-old-publisher.janet")
        );
    }
}
