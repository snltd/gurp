use serde::Deserialize;

#[derive(Deserialize, Debug, PartialEq)]
pub struct Publisher {
    #[serde(rename = "origin")]
    pub origins: Vec<OriginOrMirror>,
    #[serde(rename = "mirror")]
    pub mirrors: Vec<OriginOrMirror>,
}

#[derive(PartialEq, Default, Deserialize, Debug)]
pub struct OriginOrMirror {
    #[serde(rename = "name")]
    pub uri: PublisherUri,
    pub proxy: Option<PublisherUri>,
    pub ssl_key: Option<String>,
    pub ssl_cert: Option<String>,
}

pub type Origin = OriginOrMirror;
pub type Mirror = OriginOrMirror;

pub type PublisherName = String;
pub type PublisherUri = String;

#[derive(Deserialize, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PublisherType {
    Origin,
    Mirror,
}
