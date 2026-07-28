use std::fmt;
use std::str::FromStr;

use anyhow::{Context, bail, ensure};
use serde_with::{DeserializeFromStr, SerializeDisplay};

#[derive(Debug, Clone, PartialEq, Eq, Hash, DeserializeFromStr, SerializeDisplay)]
pub struct LinkName(String);

impl LinkName {
    pub fn new(value: impl AsRef<str>) -> anyhow::Result<Self> {
        value.as_ref().parse()
    }
}

impl FromStr for LinkName {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self> {
        ensure!(s.len() <= 31, "link name '{s}' exceeds 31 characters");

        ensure!(
            s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'),
            "link name '{s}' contains illegal chars"
        );

        ensure!(
            s.chars().next().is_some_and(|c| c.is_ascii_alphabetic()),
            "link name '{s}' must start with a letter"
        );

        let digit_start = s
            .rfind(|c: char| !c.is_ascii_digit())
            .map(|i| i + 1)
            .unwrap_or(0);

        let num = &s[digit_start..];

        if num.is_empty() || (num.len() > 1 && num.starts_with('0')) {
            bail!("link name '{s}' must end with a number (no leading zero)");
        }

        num.parse::<u32>()
            .ok()
            .filter(|&n| n <= 4_294_967_294)
            .with_context(|| format!("link name '{s}' has an out-of-range trailing number"))?;

        Ok(LinkName(s.to_owned()))
    }
}

impl fmt::Display for LinkName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_simple_name() {
        assert!(LinkName::new("net0").is_ok());
    }

    #[test]
    fn accepts_underscore_and_mixed_alnum() {
        assert!(LinkName::new("my_link99").is_ok());
    }

    #[test]
    fn rejects_empty_string() {
        assert!(LinkName::new("").is_err());
    }

    #[test]
    fn accepts_trailing_zero() {
        assert!(LinkName::new("net0").is_ok());
    }

    #[test]
    fn accepts_large_trailing_number_within_range() {
        assert!(LinkName::new("net4294967294").is_ok());
    }

    #[test]
    fn rejects_no_trailing_number() {
        let err = LinkName::new("net").unwrap_err();
        assert!(err.to_string().contains("must end with a number"));
    }

    #[test]
    fn rejects_leading_zero_in_trailing_number() {
        let err = LinkName::new("net01").unwrap_err();
        assert!(err.to_string().contains("must end with a number"));
    }

    #[test]
    fn rejects_out_of_range_trailing_number() {
        let err = LinkName::new("net4294967295").unwrap_err();
        assert!(err.to_string().contains("out-of-range"));
    }

    #[test]
    fn accepts_max_length() {
        let s = format!("{}0", "a".repeat(30));
        assert_eq!(s.len(), 31);
        assert!(LinkName::new(s).is_ok());
    }

    #[test]
    fn rejects_too_long() {
        let s = format!("{}0", "a".repeat(31));
        let err = LinkName::new(s).unwrap_err();
        assert!(err.to_string().contains("exceeds 31 characters"));
    }

    #[test]
    fn rejects_invalid_first_char() {
        let err = LinkName::new("9net").unwrap_err();
        assert!(err.to_string().contains("must start with"));

        let err = LinkName::new("_net0").unwrap_err();
        assert!(err.to_string().contains("must start with"));
    }

    #[test]
    fn rejects_invalid_characters() {
        for bad in ["net-0", "net.0", "net 0", "net/0"] {
            let err = LinkName::new(bad).unwrap_err();
            assert!(
                err.to_string().contains("contains illegal"),
                "expected char error for {bad:?}, got: {err}"
            );
        }
    }

    #[test]
    fn display_round_trips_input() {
        let name = LinkName::new("net0").unwrap();
        assert_eq!(name.to_string(), "net0");
    }

    #[test]
    fn serde_round_trip() {
        let name = LinkName::new("net0").unwrap();
        let json = serde_json::to_string(&name).unwrap();
        assert_eq!(json, "\"net0\"");
        let back: LinkName = serde_json::from_str(&json).unwrap();
        assert_eq!(back, name);
    }

    #[test]
    fn serde_deserialize_rejects_invalid() {
        let err = serde_json::from_str::<LinkName>("\"9bad\"").unwrap_err();
        assert!(err.to_string().contains("must start with"));
    }
}
