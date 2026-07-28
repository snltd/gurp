use anyhow::Context;
use ipnet::IpNet;
use std::net::IpAddr;

// Returns an IpNet, whether the given string has a prefix or not
pub fn parse_addr_or_cidr(raw: &str) -> anyhow::Result<IpNet> {
    if raw.contains('/') {
        raw.parse::<IpNet>()
            .with_context(|| format!("cannot parse CIDR {raw}"))
    } else {
        let addr: IpAddr = raw
            .parse()
            .with_context(|| format!("cannot parse IP address {raw}"))?;

        IpNet::new(addr, if addr.is_ipv4() { 32 } else { 128 })
            .with_context(|| format!("cannot construct IP address from {raw}"))
    }
}
