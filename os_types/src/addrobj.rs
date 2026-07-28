use super::LinkName;
use anyhow::{Context, bail, ensure};
use serde_with::{DeserializeFromStr, SerializeDisplay};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash, DeserializeFromStr, SerializeDisplay)]
pub struct AddrObj(String);

impl AddrObj {
    pub fn new(value: impl AsRef<str>) -> anyhow::Result<Self> {
        value.as_ref().parse()
    }
}

// It is made up of two
// parts, delimited by a ‘/’.  The first part is the name of the interface
// and the second part is an arbitrary string up to 32 alphanumeric
// characters long, where the first character must be alphabetic

impl FromStr for AddrObj {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self> {
        if let Some((link_name, suffix)) = s.split_once("/") {
            let _ = LinkName::new(link_name)
                .with_context(|| format!("could not parse link name component of addrobj '{s}'"))?;

            ensure!(
                suffix.len() <= 32,
                "addrobj '{s}' suffix exceeds 32 characters"
            );

            ensure!(
                suffix.chars().all(|c| c.is_ascii_alphanumeric()),
                "addrobject '{s}' suffix contains illegal chars"
            );

            ensure!(
                suffix.chars().next().unwrap().is_ascii_alphabetic(),
                "addrobject '{s}' suffix must begin with a letter"
            );
        } else {
            bail!("addrobj name must be two /-separated components");
        }

        Ok(AddrObj(s.to_owned()))
    }
}

impl fmt::Display for AddrObj {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_simple_name() {
        assert!(AddrObj::new("net0/v4").is_ok());
    }

    #[test]
    fn accepts_underscore_and_mixed_alnum() {
        assert!(AddrObj::new("my_link99/v6").is_ok());
    }

    #[test]
    fn rejects_empty_string() {
        assert!(AddrObj::new("").is_err());
    }

    #[test]
    fn rejects_no_suffix() {
        let err = AddrObj::new("net0").unwrap_err();
        assert!(err.to_string().contains("must be two"));
    }

    #[test]
    fn rejects_leading_number_in_suffix() {
        let err = AddrObj::new("net0/0").unwrap_err();
        assert!(err.to_string().contains("begin with a letter"));
    }

    #[test]
    fn accepts_max_length_suffix() {
        let s = format!("net0/{}", "a".repeat(32));
        assert!(AddrObj::new(s).is_ok());
    }

    #[test]
    fn rejects_too_long_suffix() {
        let s = format!("net0/{}", "a".repeat(33));
        let err = AddrObj::new(s).unwrap_err();
        assert!(err.to_string().contains("exceeds 32 characters"));
    }

    #[test]
    fn rejects_invalid_characters() {
        for bad in ["net0/v_4", "net0/v4!"] {
            let err = AddrObj::new(bad).unwrap_err();
            assert!(
                err.to_string().contains("contains illegal"),
                "expected char error for {bad:?}, got: {err}"
            );
        }
    }

    #[test]
    fn display_round_trips_input() {
        let name = AddrObj::new("net0/v4").unwrap();
        assert_eq!(name.to_string(), "net0/v4");
    }

    #[test]
    fn serde_round_trip() {
        let name = AddrObj::new("net0/v4").unwrap();
        let json = serde_json::to_string(&name).unwrap();
        assert_eq!(json, "\"net0/v4\"");
        let back: AddrObj = serde_json::from_str(&json).unwrap();
        assert_eq!(back, name);
    }

    #[test]
    fn serde_deserialize_rejects_invalid() {
        let err = serde_json::from_str::<AddrObj>("\"net0/123\"").unwrap_err();
        assert!(err.to_string().contains("must begin with a letter"));
    }
}
