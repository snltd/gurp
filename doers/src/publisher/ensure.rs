use crate::publisher::types::{Mirror, Origin, Publisher, PublisherName};
use crate::publisher::{functions, parse};
use anyhow::Context;
use common::constants::{ONE_RESOURCE_NO_CHANGE, ONE_RESOURCE_ONE_CHANGE, PKG_BIN};
use common::types::{ApplyOpts, ApplySummary};
use serde::Deserialize;

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

    fn align_publisher(
        &self,
        current: &Publisher,
        opts: &ApplyOpts,
    ) -> anyhow::Result<ApplySummary> {
        todo!()
    }

    fn create_publisher(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        tracing::info!("creating publisher {}", self.name);

        for origin in &self.desired_state.origins {
            self.add_origin(origin, opts)?;
        }

        for mirror in &self.desired_state.mirrors {
            self.add_origin(mirror, opts)?;
        }

        Ok(ONE_RESOURCE_ONE_CHANGE)
    }

    fn add_origin(&self, origin: &Origin, opts: &ApplyOpts) -> anyhow::Result<bool> {
        tracing::debug!("publisher {}: adding origin {}", self.name, origin.uri);
        todo!()
    }

    fn align_origin(&self, origin: &Origin, opts: &ApplyOpts) -> anyhow::Result<bool> {
        todo!()
    }

    fn remove_origin(&self, origin: &Origin, opts: &ApplyOpts) -> anyhow::Result<bool> {
        tracing::debug!("publisher {}: removing origin {}", self.name, origin.uri);
        todo!()
    }

    fn add_mirror(&self, mirror: &Mirror, opts: &ApplyOpts) -> anyhow::Result<bool> {
        tracing::debug!("publisher {}: adding mirror {}", self.name, mirror.uri);
        todo!()
    }

    fn align_origin(&self, mirror: &Mirror, opts: &ApplyOpts) -> anyhow::Result<bool> {
        todo!()
    }

    fn remove_mirror(&self, mirror: &Mirror, opts: &ApplyOpts) -> anyhow::Result<bool> {
        tracing::debug!("publisher {}: removing mirror {}", self.name, mirror.uri);
        todo!()
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
                        ssl_key: None,
                        ssl_cert: None,
                    }],
                    mirrors: vec![Mirror {
                        uri: "http://mirror.lan.id264.net".to_owned(),
                        proxy: None,
                        ssl_key: None,
                        ssl_cert: None,
                    }],
                }
            },
            deserialized_example("publisher/ensure-new-publisher.janet")
        );
    }
}
