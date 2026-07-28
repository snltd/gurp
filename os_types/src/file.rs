use anyhow::{Context, ensure};
use serde_with::{DeserializeFromStr, SerializeDisplay};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, DeserializeFromStr, SerializeDisplay)]
pub struct FileMode(u32);

impl FileMode {
    pub fn new(value: impl AsRef<str>) -> anyhow::Result<Self> {
        value.as_ref().parse()
    }
}

impl FromStr for FileMode {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self> {
        let digits = s.strip_prefix('0').unwrap_or(s);

        ensure!(
            !digits.is_empty() || digits.len() <= 4,
            "file mode '{s}' must be 3 or 4 octal digits"
        );

        ensure!(
            digits.chars().all(|c| ('0'..='7').contains(&c)),
            "file mode '{s}' contains non-octal digits"
        );

        let mode = u32::from_str_radix(digits, 8)
            .with_context(|| format!("file mode '{s}' is not valid octal"))?;

        Ok(FileMode(mode))
    }
}

impl Default for FileMode {
    fn default() -> Self {
        Self::new("0755").unwrap()
    }
}

impl fmt::Display for FileMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04o}", self.0)
    }
}

impl FileMode {
    pub fn as_u32(&self) -> u32 {
        self.0
    }

    pub fn from_u32(n: u32) -> Self {
        Self(n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_bare_three_digit() {
        let m = "755".parse::<FileMode>().unwrap();
        assert_eq!(m.as_u32(), 0o755);
    }

    #[test]
    fn parses_with_leading_zero() {
        let m = "0755".parse::<FileMode>().unwrap();
        assert_eq!(m.as_u32(), 0o755);
    }

    #[test]
    fn parses_four_digit_with_setuid() {
        let m = "4755".parse::<FileMode>().unwrap();
        assert_eq!(m.as_u32(), 0o4755);
    }

    #[test]
    fn parses_all_zero() {
        let m = "000".parse::<FileMode>().unwrap();
        assert_eq!(m.as_u32(), 0);
    }

    #[test]
    fn parses_all_permission_bits_set() {
        let m = "0777".parse::<FileMode>().unwrap();
        assert_eq!(m.as_u32(), 0o777);
    }

    #[test]
    fn rejects_non_octal_digit() {
        assert!("0789".parse::<FileMode>().is_err());
    }

    #[test]
    fn rejects_too_many_digits() {
        assert!("099999".parse::<FileMode>().is_err());
    }

    #[test]
    fn rejects_empty_string() {
        assert!("".parse::<FileMode>().is_err());
    }

    #[test]
    fn rejects_bare_leading_zero_only() {
        assert!("0".parse::<FileMode>().is_err());
    }

    #[test]
    fn display_round_trips_to_four_digit_octal() {
        let m: FileMode = "755".parse().unwrap();
        assert_eq!(m.to_string(), "0755");
    }

    #[test]
    fn display_pads_short_values() {
        let m: FileMode = "7".parse().unwrap();
        assert_eq!(m.to_string(), "0007");
    }

    #[test]
    fn serde_round_trip() {
        let m: FileMode = "0755".parse().unwrap();
        let json = serde_json::to_string(&m).unwrap();
        assert_eq!(json, "\"0755\"");
        let back: FileMode = serde_json::from_str(&json).unwrap();
        assert_eq!(back, m);
    }

    #[test]
    fn serde_deserialize_rejects_invalid() {
        assert!(serde_json::from_str::<FileMode>("\"899\"").is_err());
    }
}
