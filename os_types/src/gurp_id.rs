use anyhow::{Context, ensure};
use serde_with::{DeserializeFromStr, SerializeDisplay};
use std::cmp::Ordering;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash, DeserializeFromStr, SerializeDisplay)]
pub struct GurpId(String);

impl GurpId {
    pub fn new(value: impl AsRef<str>) -> anyhow::Result<Self> {
        value.as_ref().parse()
    }
}

impl FromStr for GurpId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self> {
        let chopped = s
            .strip_prefix("/")
            .context("id '{s}' does not start with a /")?;

        let chunks: Vec<_> = chopped.split("/").collect();

        ensure!(
            chunks.len() == 3 && !chunks.iter().any(|c| c.is_empty()),
            "id '{s}' does not contain three valid chunks"
        );

        Ok(GurpId(s.to_owned()))
    }
}

impl fmt::Display for GurpId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl GurpId {
    pub fn role(&self) -> String {
        self.0.split("/").collect::<Vec<_>>()[0].to_owned()
    }

    pub fn resource_type(&self) -> String {
        self.0.split("/").collect::<Vec<_>>()[1].to_owned()
    }

    pub fn resource_name(&self) -> String {
        self.0.split("/").collect::<Vec<_>>()[2].to_owned()
    }
}

// We need this to insert into the changedIds BTreeSet
impl Ord for GurpId {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl PartialOrd for GurpId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_valid_id() {
        assert!(GurpId::new("/test-role/file/label").is_ok());
    }

    #[test]
    fn rejects_unqualified_name() {
        let err = GurpId::new("test-role/file/label").unwrap_err();
        assert!(err.to_string().contains("does not start with a /"));
    }

    #[test]
    fn rejects_two_chunks() {
        for id in ["/test-role/file", "/test-role/file/", "/test-role//file"] {
            let err = GurpId::new(id).unwrap_err();
            assert!(
                err.to_string()
                    .contains("does not contain three valid chunks")
            );
        }
    }

    #[test]
    fn display_round_trips_input() {
        let name = GurpId::new("/test-role/thing/label").unwrap();
        assert_eq!(name.to_string(), "/test-role/thing/label");
    }

    #[test]
    fn serde_round_trip() {
        let name = GurpId::new("/test-role/thing/label").unwrap();
        let json = serde_json::to_string(&name).unwrap();
        assert_eq!(json, "\"/test-role/thing/label\"");
        let back: GurpId = serde_json::from_str(&json).unwrap();
        assert_eq!(back, name);
    }

    #[test]
    fn serde_deserialize_rejects_invalid() {
        let err = serde_json::from_str::<GurpId>("\"rubbish\"").unwrap_err();
        assert!(err.to_string().contains("does not start with a /"));
    }
}
