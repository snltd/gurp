use anyhow::Result;
use ipnet::IpNet;
use serde_with::{DeserializeFromStr, SerializeDisplay};
use std::net::IpAddr;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, DeserializeFromStr, SerializeDisplay)]
pub struct FlowAddr(IpNet);

/// A wrapper type around IpAddr which lets use receive a bare IP address or a CIDR, both of which
/// are accepted by flowadm. If we get the bare address, we assume it's a host and stick /32 on it.
impl FlowAddr {
    pub fn new(value: impl AsRef<str>) -> anyhow::Result<Self> {
        value.as_ref().parse()
    }
}

impl FromStr for FlowAddr {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        if s.contains('/') {
            Ok(FlowAddr(s.parse()?))
        } else {
            let addr: IpAddr = s.parse()?;
            let prefix = if addr.is_ipv4() { 32 } else { 128 };
            Ok(FlowAddr(IpNet::new(addr, prefix)?))
        }
    }
}

impl std::fmt::Display for FlowAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_bare_ip() {
        assert!(FlowAddr::new("192.168.1.5").is_ok());
    }

    #[test]
    fn accepts_cidr() {
        assert!(FlowAddr::new("192.168.1.5/24").is_ok());
    }

    #[test]
    fn display_round_trips_input() {
        let name = FlowAddr::new("192.168.1.5").unwrap();
        assert_eq!(name.to_string(), "192.168.1.5/32");
    }

    #[test]
    fn serde_round_trip() {
        let name = FlowAddr::new("10.0.10.2").unwrap();
        let json = serde_json::to_string(&name).unwrap();
        assert_eq!(json, "\"10.0.10.2/32\"");
        let back: FlowAddr = serde_json::from_str(&json).unwrap();
        assert_eq!(back, name);
    }

    #[test]
    fn serde_deserialize_rejects_invalid() {
        let err = serde_json::from_str::<FlowAddr>("\"some junk\"").unwrap_err();
        assert!(err.to_string().contains("invalid IP address syntax"));
    }
}
