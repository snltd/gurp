use crate::zone::control::{self};
use crate::zone::helpers;
use common::constants::{ONE_RESOURCE_NO_CHANGE, ONE_RESOURCE_ONE_CHANGE};
use common::types::{ApplyOpts, ApplySummary};
use os_types::GurpId;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GurpZoneRemove {
    #[serde(rename = "_id")]
    pub id: GurpId,
    pub name: String,
}

impl GurpZoneRemove {
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        if helpers::current_zone_list()?.contains_key(&self.name) {
            tracing::info!("zone {}: remove", self.name);

            if opts.noop {
                Ok(ONE_RESOURCE_ONE_CHANGE)
            } else {
                control::remove_zone(&self.name)
            }
        } else {
            tracing::debug!("zone {}: not found", self.name);
            Ok(ONE_RESOURCE_NO_CHANGE)
        }
    }
}
