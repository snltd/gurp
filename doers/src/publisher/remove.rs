use crate::publisher::functions;
use crate::publisher::types::PublisherName;
use common::constants::{ONE_RESOURCE_NO_CHANGE, PKG_BIN};
use common::types::{ApplyOpts, ApplySummary};
use os_types::GurpId;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct GurpPublisherRemove {
    #[serde(rename = "_id")]
    pub id: GurpId,
    pub name: PublisherName,
}

impl GurpPublisherRemove {
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        if functions::publisher_exists(&self.name)? {
            tracing::info!("removing publisher: {}", self.name);
            cmd_change_or_noop!(opts, PKG_BIN, "unset-publisher", &self.name)
        } else {
            tracing::debug!("publisher {} does not exist", self.name);
            Ok(ONE_RESOURCE_NO_CHANGE)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
    use tester::deserialized_example;

    #[test]
    fn test_deserialize_publisher_remove_old_publisher() {
        assert_eq!(
            GurpPublisherRemove {
                id: GurpId::new("/NO-ROLE/publisher/old_publisher").unwrap(),
                name: "old_publisher".to_owned(),
            },
            deserialized_example("publisher/remove-old-publisher.janet")
        );
    }
}
