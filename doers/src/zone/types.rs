use crate::zone::config::ImageChecksum;
use crate::zone::control::ZoneadmState;
use std::collections::HashMap;

pub struct ZoneImage<'a> {
    pub user_string: Option<&'a str>,
    pub checksum: Option<&'a ImageChecksum>,
}

pub type ZoneName = String;
pub type ZoneadmZones = HashMap<ZoneName, ZoneadmState>;
