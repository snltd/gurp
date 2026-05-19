use serde::Deserialize;
use std::fmt;

#[derive(Deserialize, Debug, PartialEq)]
pub struct Publisher {
    #[serde(rename = "origin")]
    pub origins: Vec<OriginOrMirror>,
    #[serde(rename = "mirror")]
    pub mirrors: Option<Vec<OriginOrMirror>>,
}

#[derive(PartialEq, Default, Deserialize, Debug)]
pub struct OriginOrMirror {
    #[serde(rename = "name")]
    pub uri: PublisherUri,
    pub proxy: Option<PublisherUri>,
}

pub type Origin = OriginOrMirror;
pub type Mirror = OriginOrMirror;

pub type PublisherName = String;
pub type PublisherUri = String;

#[derive(Deserialize, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TargetType {
    Origin,
    Mirror,
}

impl fmt::Display for TargetType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TargetType::Origin => "origin",
                TargetType::Mirror => "mirror",
            }
        )
    }
}
