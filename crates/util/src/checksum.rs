use crate::hash::HashAlgorithm;
use serde::Deserialize;
use url::Url;

#[derive(Deserialize, Debug)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(rename_all = "lowercase")]
pub enum HashValue {
    Literal(String),
    FromUrl(Url),
    Suffix(String),
}

#[derive(Deserialize, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Checksum {
    pub algorithm: HashAlgorithm,
    #[serde(flatten)]
    pub value: HashValue,
}

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct ResolvedChecksum {
    pub algorithm: HashAlgorithm,
    pub value: String,
}
