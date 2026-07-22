use serde::Deserialize;
use std::fmt;
use url::Url;

#[derive(Deserialize, Debug)]
pub struct Publisher {
    #[serde(rename = "origin")]
    pub origins: Vec<OriginOrMirror>,
    #[serde(rename = "mirror")]
    pub mirrors: Option<Vec<OriginOrMirror>>,
}

impl PartialEq for Publisher {
    fn eq(&self, other: &Self) -> bool {
        let mut self_mirrors: Vec<_> = self.mirrors.iter().flatten().collect();
        let mut other_mirrors: Vec<_> = other.mirrors.iter().flatten().collect();

        self_mirrors.sort_by(|a, b| a.uri.cmp(&b.uri));
        other_mirrors.sort_by(|a, b| a.uri.cmp(&b.uri));

        self.origins == other.origins && self_mirrors == other_mirrors
    }
}

#[derive(PartialEq, Deserialize, Debug)]
pub struct OriginOrMirror {
    #[serde(rename = "name")]
    pub uri: Url,
    pub proxy: Option<Url>,
}

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

pub type Origin = OriginOrMirror;
pub type Mirror = OriginOrMirror;
pub type PublisherName = String;
