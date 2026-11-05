use crate::zone::config::ImageSource;
use crate::zone::control::ZoneadmState;
use std::collections::HashMap;
use util::checksum::Checksum;

pub struct ZoneImage<'a> {
    pub image_source: Option<&'a ImageSource>,
    pub checksum: Option<&'a Checksum>,
}

pub type ZoneName = String;
pub type ZoneadmZones = HashMap<ZoneName, ZoneadmState>;
