use crate::zone::control::ZoneadmState;
use std::collections::HashMap;

pub type ZoneName = String;
pub type ZoneadmZones = HashMap<ZoneName, ZoneadmState>;
