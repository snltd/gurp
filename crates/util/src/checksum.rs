use crate::hash::HashAlgorithm;
use crate::http;
use crate::url::WithSuffix;
use anyhow::Context;
use serde::Deserialize;
use url::Url;

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum HashValue {
    Literal(String),
    FromUrl(Url),
    Suffix(String),
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct Checksum {
    pub algorithm: HashAlgorithm,
    #[serde(flatten)]
    pub value: HashValue,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct ResolvedChecksum {
    pub algorithm: HashAlgorithm,
    pub value: String,
}

impl Checksum {
    pub fn resolve(&self, base_url: Option<&Url>) -> anyhow::Result<String> {
        Ok(match &self.value {
            HashValue::Literal(val) => val.to_owned(),
            HashValue::FromUrl(url) => http::url_to_string(url)?,
            HashValue::Suffix(suffix) => {
                let base_url = base_url.context("cannot use checksum suffix without base URL")?;
                http::url_to_string(&base_url.with_suffix(suffix)?)?
            }
        })
    }

    /// Resolve the expected checksum, NOT the actual one
    pub fn resolve_and_compare(
        &self,
        expected: &str,
        base_url: Option<&Url>,
    ) -> anyhow::Result<bool> {
        let actual = self.resolve(base_url)?;
        tracing::debug!("comparing checksums: expected={expected} actual={actual}");
        Ok(expected == actual)
    }
}
