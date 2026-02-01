use common::constants::{ONE_RESOURCE_NO_CHANGE, PKG_BIN};
use common::types::{ApplyOpts, ApplySummary};
use serde::Deserialize;
use std::sync::LazyLock;

// THINGS TO KNOW / THINGS TO DO.
// Only handles origins. I use it to add my own repo. Not tested beyond that.

static CURRENT_PKG_OUTPUT: LazyLock<Vec<Publisher>> =
    LazyLock::new(|| parse_pkg_output(&pkg_output().expect("Could not get publisher list")));

const PKG_PUBLISHER_FIELDS: usize = 5;

type PublisherName = String;
type PublisherUri = String;

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct GurpPublisherEnsure {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: PublisherName,
    pub uri: PublisherUri,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct GurpPublisherRemove {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: PublisherName,
}

// We don't care about anything else, for now at least
struct Publisher {
    name: PublisherName,
    uri: PublisherUri,
}

impl GurpPublisherEnsure {
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        let current_publishers = &CURRENT_PKG_OUTPUT;

        if let Some(existing) = &current_publishers.iter().find(|p| p.name == self.name) {
            if existing.uri == self.uri {
                tracing::debug!("no change to {} publisher", &self.name);
                return Ok(ONE_RESOURCE_NO_CHANGE);
            }

            tracing::info!(
                "change {} publisher URI: {} -> {}",
                self.name,
                existing.uri,
                self.uri,
            );
        } else {
            tracing::info!("add publisher {}", self.name,);
        }

        let mut cmd = cmd!(PKG_BIN, "set-publisher", "-g", &self.uri, &self.name);
        return_if_noop!(opts);
        one_change_or_stderr!(cmd, format!("error setting '{}'; publisher", self.name))
    }
}

impl GurpPublisherRemove {
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        let current_publishers = &CURRENT_PKG_OUTPUT;

        if current_publishers.iter().any(|p| p.name == self.name) {
            let mut cmd = cmd!(PKG_BIN, "unset-publisher", &self.name);
            return_if_noop!(opts);
            one_change_or_stderr!(cmd, format!("error unsetting '{}'; publisher", self.name))
        } else {
            tracing::debug!("publisher {} does not exist", self.name);
            Ok(ONE_RESOURCE_NO_CHANGE)
        }
    }
}

fn pkg_output() -> anyhow::Result<String> {
    tracing::debug!("looking up publishers");
    cmd_output!(PKG_BIN, "publisher", "-H")
}

fn parse_pkg_output(output: &str) -> Vec<Publisher> {
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
