use anyhow::{Context, bail, ensure};
use common::cmd;
use common::constants::{
    IPADM_BIN, NETSTAT_BIN, ONE_RESOURCE_NO_CHANGE, ONE_RESOURCE_ONE_CHANGE, ROUTE_BIN,
};
use common::types::{ApplyOpts, ApplySummary};
use os_types::GurpId;
use serde::Deserialize;
use std::collections::HashMap;
use std::process::{Command, Stdio};
use util::deserializer;

#[derive(Debug)]
struct Route {
    destination: String,
    gateway: Option<String>,
    interface: Option<String>,
}

#[derive(Debug, PartialEq)]
struct ExtantRoute {
    destination: String,
    gateway: String,
    interface: Option<String>,
    flags: Vec<char>,
}

type Flags = HashMap<String, String>;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(test, derive(PartialEq))]
pub struct RouteEnsure {
    #[serde(rename = "_id")]
    pub id: GurpId,
    #[serde(rename = "name")]
    pub destination: String,
    pub gateway: Option<String>,
    pub interface: Option<String>,
    pub force_gateway: bool,
    #[serde(
        default,
        deserialize_with = "deserializer::option_property_deserializer"
    )]
    pub flags: Option<Flags>,
    #[serde(rename = "type")]
    pub route_type: Option<String>, // dropped in right after the "add"
}

#[derive(Deserialize, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct RouteRemove {
    #[serde(rename = "_id")]
    pub id: GurpId,
    #[serde(rename = "name")]
    pub destination: String,
    pub gateway: Option<String>,
    pub interface: Option<String>,
    #[serde(rename = "type")]
    pub route_type: Option<String>,
}

impl RouteEnsure {
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        let route = Route {
            destination: self.destination.clone(),
            gateway: self.gateway.clone(),
            interface: self.interface.clone(),
        };

        let route_table = current_routes()?;

        let route_type = if self.interface.is_some() {
            "interface"
        } else {
            "gateway"
        };

        let target = gateway_or_interface(&route)?;

        if route_exists(&route, &route_table) {
            tracing::debug!(
                "{} -> {route_type} {target} already exists",
                self.destination,
            );
            Ok(ONE_RESOURCE_NO_CHANGE)
        } else {
            tracing::info!("creating {} -> {route_type} {target}", self.destination);
            let mut cmd = self.build_add_route_cmd();
            tracing::debug!(command = cmd::to_string(&cmd));

            if !opts.noop {
                run_cmd!(cmd).context("error running route command")?;
            }

            Ok(ONE_RESOURCE_ONE_CHANGE)
        }
    }

    pub fn build_add_route_cmd(&self) -> Command {
        let mut cmd = Command::new(ROUTE_BIN);
        cmd.arg("-p");
        cmd.arg("add");

        if let Some(route_type) = &self.route_type {
            cmd.arg(format!("-{route_type}"));
        }

        cmd.arg(&self.destination);

        if let Some(flags) = &self.flags {
            for (k, v) in flags {
                cmd.arg(format!("-{k}"));
                if v.as_str() != "true" {
                    cmd.arg(v);
                }
            }
        }

        if let Some(interface) = &self.interface {
            cmd.arg("-interface");
            cmd.arg(interface);
        } else if let Some(gateway) = &self.gateway {
            if self.force_gateway {
                cmd.arg("-gateway");
            }

            cmd.arg(gateway);
        }

        cmd.stderr(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd
    }
}

impl RouteRemove {
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        let route = Route {
            destination: self.destination.clone(),
            gateway: self.gateway.clone(),
            interface: self.interface.clone(),
        };

        let route_table = current_routes()?;

        let route_type = if self.interface.is_some() {
            "interface"
        } else {
            "gateway"
        };

        let target = gateway_or_interface(&route)?;

        if route_exists(&route, &route_table) {
            tracing::info!("removing {} -> {} {}", self.destination, route_type, target);
            let mut cmd = Command::new(ROUTE_BIN);
            cmd.arg("-p");
            cmd.arg("delete");

            if let Some(route_type) = &self.route_type {
                cmd.arg(format!("-{route_type}"));
            }

            cmd.arg(&self.destination);
            cmd.arg(gateway_or_interface(&route)?);

            tracing::debug!(command = common::cmd::to_string(&cmd));

            if !opts.noop {
                cmd.stderr(Stdio::piped());
                let status = cmd.status().context("error running route delete")?;
                ensure!(status.success(), "error running route command");
            }

            Ok(ONE_RESOURCE_ONE_CHANGE)
        } else {
            tracing::debug!(
                "{} -> {route_type} {target} does not exist",
                self.destination,
            );
            Ok(ONE_RESOURCE_NO_CHANGE)
        }
    }
}

fn current_routes() -> anyhow::Result<Vec<ExtantRoute>> {
    let raw_netstat =
        cmd_output!(NETSTAT_BIN, "-rn", "-f", "inet").context("error listing routes")?;
    let raw_ip =
        cmd_output!(IPADM_BIN, "show-addr", "-po", "addr").context("error listing ip-addresses")?;
    let local_ip_list = parse_local_addrs(&raw_ip);

    Ok(parse_route_table(&raw_netstat, &local_ip_list))
}

fn route_exists(needle: &Route, haystack: &[ExtantRoute]) -> bool {
    for route in haystack {
        let destination_without_mask = needle.destination.split('/').next().unwrap();

        if route.destination != destination_without_mask {
            continue;
        }

        if let Some(gateway) = &needle.gateway
            && gateway != &route.gateway
        {
            continue;
        }

        return true;
    }

    false
}

fn gateway_or_interface(route: &Route) -> anyhow::Result<String> {
    if let Some(interface) = &route.interface {
        Ok(interface.to_string())
    } else if let Some(gateway) = &route.gateway {
        Ok(gateway.to_string())
    } else {
        bail!("No interface or gateway for {}", route.destination);
    }
}

fn parse_route_table(raw: &str, local_addrs: &[String]) -> Vec<ExtantRoute> {
    let mut ret: Vec<ExtantRoute> = Vec::new();
    let mut in_table = false;

    for line in raw.lines() {
        if line.starts_with("-------") {
            in_table = true;
            continue;
        }

        if !in_table {
            continue;
        }

        // We expect six fields
        let fields: Vec<_> = line.split_whitespace().collect();

        if fields.len() < 5 {
            continue;
        }

        let interface = fields.get(5).map(|f| f.to_string());

        if fields[0] == fields[1] {
            continue;
        }

        if local_addrs.iter().any(|a| a == fields[1]) && fields[2] == "U" {
            continue;
        }

        ret.push(ExtantRoute {
            destination: fields[0].to_string(), // netmasks are lost
            gateway: fields[1].to_string(),
            flags: fields[2].chars().collect::<Vec<char>>(),
            interface,
        })
    }

    ret
}

fn parse_local_addrs(raw: &str) -> Vec<String> {
    raw.lines()
        .filter(|l| !l.starts_with(':')) // ignore IPv6
        .map(|l| l.split('/').next().unwrap())
        .map(|l| l.to_string())
        .collect()
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
    use tester::{deserialized_example, janet2json, propmap};

    #[test]
    fn test_deserialize_route_ensure_blackhole() {
        assert_eq!(
            RouteEnsure {
                id: GurpId::new("/NO-ROLE/route/203.0.113.0_24").unwrap(),
                destination: "203.0.113.0/24".to_owned(),
                gateway: Some("127.0.0.1".to_owned()),
                interface: None,
                force_gateway: false,
                flags: None,
                route_type: Some("blackhole".to_owned()),
            },
            deserialized_example("route/ensure-blackhole.janet")
        );
    }

    #[test]
    fn test_deserialize_route_ensure_default_route() {
        assert_eq!(
            RouteEnsure {
                id: GurpId::new("/NO-ROLE/route/default").unwrap(),
                destination: "default".to_owned(),
                gateway: Some("192.168.1.1".to_owned()),
                interface: None,
                force_gateway: false,
                flags: None,
                route_type: None,
            },
            deserialized_example("route/ensure-default-route.janet")
        );
    }

    #[test]
    fn test_deserialize_route_ensure_network_with_mtu() {
        assert_eq!(
            RouteEnsure {
                id: GurpId::new("/NO-ROLE/route/10.0.5.0_24").unwrap(),
                destination: "10.0.5.0/24".to_owned(),
                gateway: Some("10.0.5.150".to_owned()),
                interface: None,
                force_gateway: false,
                flags: Some(propmap! {"mtu" => "1500"}),
                route_type: None,
            },
            deserialized_example("route/ensure-network-with-mtu.janet")
        );
    }

    #[test]
    fn test_deserialize_route_remove_blackhole() {
        assert_eq!(
            RouteRemove {
                id: GurpId::new("/NO-ROLE/route/203.0.113.0_24").unwrap(),
                destination: "203.0.113.0/24".to_owned(),
                gateway: Some("127.0.0.1".to_owned()),
                interface: None,
                route_type: Some("blackhole".to_owned()),
            },
            deserialized_example("route/remove-blackhole.janet")
        );
    }

    #[test]
    fn test_deserialize_route_remove_net_route() {
        assert_eq!(
            RouteRemove {
                id: GurpId::new("/NO-ROLE/route/10.0.5.0_24").unwrap(),
                destination: "10.0.5.0/24".to_owned(),
                gateway: Some("10.0.5.150".to_owned()),
                interface: None,
                route_type: None,
            },
            deserialized_example("route/remove-net-route.janet")
        );
    }

    #[test]
    fn test_build_add_route_cmd() {
        // default route
        let json_def = janet2json(r#"(route/ensure "default" :gateway "192.168.1.1")"#);
        let sut: RouteEnsure = serde_json::from_str(&json_def).unwrap();

        assert_eq!(
            "/usr/sbin/route -p add default 192.168.1.1",
            cmd::to_string(&sut.build_add_route_cmd())
        );

        // normal route
        let json_def = janet2json(r#"(route/ensure "10.0.0.0/16" :gateway "10.0.0.2")"#);
        let sut: RouteEnsure = serde_json::from_str(&json_def).unwrap();

        assert_eq!(
            "/usr/sbin/route -p add 10.0.0.0/16 10.0.0.2",
            cmd::to_string(&sut.build_add_route_cmd())
        );

        // interface route
        let json_def = janet2json(r#"(route/ensure "10.0.0.0/16" :interface "e1000g0")"#);
        let sut: RouteEnsure = serde_json::from_str(&json_def).unwrap();

        assert_eq!(
            "/usr/sbin/route -p add 10.0.0.0/16 -interface e1000g0",
            cmd::to_string(&sut.build_add_route_cmd())
        );

        // reject route
        let json_def =
            janet2json(r#"(route/ensure "203.0.113.0/24" :gateway "127.0.0.1" :type "reject")"#);
        let sut: RouteEnsure = serde_json::from_str(&json_def).unwrap();

        assert_eq!(
            "/usr/sbin/route -p add -reject 203.0.113.0/24 127.0.0.1",
            cmd::to_string(&sut.build_add_route_cmd())
        );

        // blackhole route
        let json_def =
            janet2json(r#"(route/ensure "203.0.113.0/24" :gateway "127.0.0.1" :type "blackhole")"#);
        let sut: RouteEnsure = serde_json::from_str(&json_def).unwrap();

        assert_eq!(
            "/usr/sbin/route -p add -blackhole 203.0.113.0/24 127.0.0.1",
            cmd::to_string(&sut.build_add_route_cmd())
        );

        // host route
        let json_def =
            janet2json(r#"(route/ensure "10.11.12.13" :gateway "192.168.1.10" :type "host")"#);
        let sut: RouteEnsure = serde_json::from_str(&json_def).unwrap();

        assert_eq!(
            "/usr/sbin/route -p add -host 10.11.12.13 192.168.1.10",
            cmd::to_string(&sut.build_add_route_cmd())
        );

        // gateway route
        let json_def =
            janet2json(r#"(route/ensure "10.11.12.13" :gateway "router" :force-gateway true)"#);
        let sut: RouteEnsure = serde_json::from_str(&json_def).unwrap();

        assert_eq!(
            "/usr/sbin/route -p add 10.11.12.13 -gateway router",
            cmd::to_string(&sut.build_add_route_cmd())
        );
    }

    #[test]
    fn test_route_exists() {
        let needle = Route {
            destination: "default".to_owned(),
            gateway: Some("192.168.1.1".to_owned()),
            interface: None,
        };

        assert!(route_exists(&needle, &sample_routes()));

        let needle = Route {
            destination: "10.0.0.0".to_owned(),
            gateway: Some("192.168.1.1".to_owned()),
            interface: None,
        };

        assert!(!route_exists(&needle, &sample_routes()));

        let needle = Route {
            destination: "203.0.113.0/24".to_owned(),
            gateway: Some("127.0.0.1".to_owned()),
            interface: None,
        };

        assert!(route_exists(&needle, &sample_routes()));

        let needle = Route {
            destination: "10.0.0.0/16".to_owned(),
            gateway: Some("192.168.1.250".to_owned()),
            interface: None,
        };

        assert!(route_exists(&needle, &sample_routes()));
    }

    #[test]
    fn test_parse_route_table() {
        let input = indoc::indoc! { "
            Routing Table: IPv4
              Destination            Gateway          Flags  Ref     Use     Interface
            -------------------- -------------------- ----- ----- ---------- ---------
            default              192.168.1.1          UGZ       1          0 test_net0
            10.0.0.0             192.168.1.250        UG        1          0
            10.0.0.0             10.0.0.2             U         2          0 test_net1
            127.0.0.1            127.0.0.1            UH        2          0 lo0
            192.168.1.0          192.168.1.16         U         5       1911 test_net0
            192.168.1.33         192.168.1.16         UH        1          0 test_net0
            203.0.113.0          127.0.0.1            URB       1          0 lo0
            "
        };

        assert_eq!(
            sample_routes(),
            parse_route_table(input, &sample_local_addrs())
        );
    }

    #[test]
    fn test_parse_local_addrs() {
        let input = indoc::indoc! { "
            127.0.0.1/8
            192.168.1.16/24
            10.0.0.2/8
            ::1/128
            "
        };

        assert_eq!(sample_local_addrs(), parse_local_addrs(input));
    }

    fn sample_local_addrs() -> Vec<String> {
        vec![
            "127.0.0.1".to_owned(),
            "192.168.1.16".to_owned(),
            "10.0.0.2".to_owned(),
        ]
    }

    fn sample_routes() -> Vec<ExtantRoute> {
        vec![
            ExtantRoute {
                destination: "default".to_owned(),
                gateway: "192.168.1.1".to_owned(),
                interface: Some("test_net0".to_owned()),
                flags: vec!['U', 'G', 'Z'],
            },
            ExtantRoute {
                destination: "10.0.0.0".to_owned(),
                gateway: "192.168.1.250".to_owned(),
                interface: None,
                flags: vec!['U', 'G'],
            },
            ExtantRoute {
                destination: "192.168.1.33".to_owned(),
                gateway: "192.168.1.16".to_owned(),
                interface: Some("test_net0".to_owned()),
                flags: vec!['U', 'H'],
            },
            ExtantRoute {
                destination: "203.0.113.0".to_owned(),
                gateway: "127.0.0.1".to_owned(),
                interface: Some("lo0".to_owned()),
                flags: vec!['U', 'R', 'B'],
            },
        ]
    }
}
