use anyhow::ensure;
use serde_with::{DeserializeFromStr, SerializeDisplay};
use std::ffi::OsStr;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash, DeserializeFromStr, SerializeDisplay)]
pub struct BridgeName(String);

impl BridgeName {
    pub fn new(value: impl AsRef<str>) -> anyhow::Result<Self> {
        value.as_ref().parse()
    }
}

impl FromStr for BridgeName {
    type Err = anyhow::Error;

    // The name may use any alphanumeric characters or the
    // underscore, (_), but must start and end with an alphabetic
    // character.  A bridge name can be at most 31 characters.  The
    // name `default' is reserved, as are all names starting with
    // `SUNW'.

    fn from_str(s: &str) -> anyhow::Result<Self> {
        ensure!(s.len() <= 31, "bridge name '{s}' exceeds 31 characters");

        let chars: Vec<_> = s.chars().collect();

        ensure!(
            chars.iter().all(|c| c.is_ascii_alphanumeric() || *c == '_'),
            "link name '{s}' contains illegal chars"
        );

        ensure!(
            chars.first().is_some_and(|c| c.is_ascii_alphabetic()),
            "bridge name '{s}' must start with a letter"
        );

        ensure!(
            chars.last().is_some_and(|c| c.is_ascii_alphabetic()),
            "bridge name '{s}' must end with a letter"
        );

        ensure!(
            s != "default" && !s.starts_with("SUNW"),
            "bridge name '{s}' is reserved"
        );

        Ok(BridgeName(s.to_owned()))
    }
}

impl fmt::Display for BridgeName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl AsRef<OsStr> for BridgeName {
    fn as_ref(&self) -> &OsStr {
        self.0.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_simple_name() {
        assert!(BridgeName::new("mybridge").is_ok());
    }

    #[test]
    fn rejects_empty_string() {
        assert!(BridgeName::new("").is_err());
    }

    #[test]
    fn accepts_max_length() {
        let s = "a".repeat(31).to_string();
        assert_eq!(s.len(), 31);
        assert!(BridgeName::new(s).is_ok());
    }

    #[test]
    fn rejects_too_long() {
        let s = "a".repeat(32).to_string();
        let err = BridgeName::new(s).unwrap_err();
        assert!(err.to_string().contains("exceeds 31 characters"));
    }

    #[test]
    fn rejects_invalid_first_and_last_chars() {
        let err = BridgeName::new("0bridge").unwrap_err();
        assert!(err.to_string().contains("must start with"));

        let err = BridgeName::new("bridge1").unwrap_err();
        assert!(err.to_string().contains("must end with"));
    }

    #[test]
    fn rejects_invalid_characters() {
        for bad in ["b-r-i-d-g-e", "LOUD_BRIDGE!", "my bridge", "zone/bridge"] {
            let err = BridgeName::new(bad).unwrap_err();
            assert!(
                err.to_string().contains("contains illegal"),
                "expected char error for {bad:?}, got: {err}"
            );
        }
    }
    #[test]
    fn display_round_trips_input() {
        let name = BridgeName::new("test_br").unwrap();
        assert_eq!(name.to_string(), "test_br");
    }

    #[test]
    fn serde_round_trip() {
        let name = BridgeName::new("testbridge").unwrap();
        let json = serde_json::to_string(&name).unwrap();
        assert_eq!(json, "\"testbridge\"");
        let back: BridgeName = serde_json::from_str(&json).unwrap();
        assert_eq!(back, name);
    }

    #[test]
    fn serde_deserialize_rejects_invalid() {
        let err = serde_json::from_str::<BridgeName>("\"9bad\"").unwrap_err();
        assert!(err.to_string().contains("must start with"));
    }
}
