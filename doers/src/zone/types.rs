use crate::zone::config::{ImageChecksum, ImageSource};
use crate::zone::control::ZoneadmState;
use std::collections::HashMap;

pub struct ZoneImage<'a> {
    pub image_source: Option<&'a ImageSource>,
    pub checksum: Option<&'a ImageChecksum>,
}

pub type ZoneName = String;
pub type ZoneadmZones = HashMap<ZoneName, ZoneadmState>;
