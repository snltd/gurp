use anyhow::{bail, ensure};
use common::cmd;
use common::constants::{
    IPADM_BIN, NETSTAT_BIN, ONE_RESOURCE_NO_CHANGE, ONE_RESOURCE_ONE_CHANGE, ROUTE_BIN,
};
use common::types::{ApplyOpts, ApplySummary};
use serde::Deserialize;
use std::collections::HashMap;
use std::process::{Command, Stdio};

// THINGS TO KNOW / THINGS TO DO.
// The route command is messy legacy, and it takes all manner of commands. This is a best-
// guess attempt to provide something useful
// We only add persistent routes.
// We only support IPv4
// Flags only get set when a route is created. We can't change them on an existing route.

#[derive(Debug, PartialEq)]
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

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct GurpRouteEnsure {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(rename = "name")]
    pub destination: String,
    pub gateway: Option<String>,
    pub interface: Option<String>,
    pub force_gateway: bool,
    pub flags: Option<Flags>,
    #[serde(rename = "type")]
    pub route_type: Option<String>, // dropped in right after the "add"
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct GurpRouteRemove {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(rename = "name")]
    pub destination: String,
    pub gateway: Option<String>,
    pub interface: Option<String>,
}

impl GurpRouteEnsure {
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
                let status = cmd.status()?;
                ensure!(status.success(), "Error running route command");
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

impl GurpRouteRemove {
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
            let mut cmd = cmd!(
                ROUTE_BIN,
                "-p",
                "delete",
                &self.destination,
                gateway_or_interface(&route)?,
            );

            if !opts.noop {
                cmd.stderr(Stdio::piped());
                let status = cmd.status()?;
                ensure!(status.success(), "Error running route command");
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
    let raw_netstat = cmd_output!(NETSTAT_BIN, "-rn", "-f", "inet")?;
    let raw_ip = cmd_output!(IPADM_BIN, "show-addr", "-po", "addr")?;
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
    use tester::janet2json;

    #[test]
    fn test_build_add_route_cmd() {
        // default route
        let json_def = janet2json(r#"(route/ensure "default" :gateway "192.168.1.1")"#);
        let sut: GurpRouteEnsure = serde_json::from_str(&json_def).unwrap();

        assert_eq!(
            "/usr/sbin/route -p add default 192.168.1.1",
            cmd::to_string(&sut.build_add_route_cmd())
        );

        // normal route
        let json_def = janet2json(r#"(route/ensure "10.0.0.0/16" :gateway "10.0.0.2")"#);
        let sut: GurpRouteEnsure = serde_json::from_str(&json_def).unwrap();

        assert_eq!(
            "/usr/sbin/route -p add 10.0.0.0/16 10.0.0.2",
            cmd::to_string(&sut.build_add_route_cmd())
        );

        // interface route
        let json_def = janet2json(r#"(route/ensure "10.0.0.0/16" :interface "e1000g0")"#);
        let sut: GurpRouteEnsure = serde_json::from_str(&json_def).unwrap();

        assert_eq!(
            "/usr/sbin/route -p add 10.0.0.0/16 -interface e1000g0",
            cmd::to_string(&sut.build_add_route_cmd())
        );

        // reject route
        let json_def =
            janet2json(r#"(route/ensure "203.0.113.0/24" :gateway "127.0.0.1" :type "reject")"#);
        let sut: GurpRouteEnsure = serde_json::from_str(&json_def).unwrap();

        assert_eq!(
            "/usr/sbin/route -p add -reject 203.0.113.0/24 127.0.0.1",
            cmd::to_string(&sut.build_add_route_cmd())
        );

        // blackhole route
        let json_def =
            janet2json(r#"(route/ensure "203.0.113.0/24" :gateway "127.0.0.1" :type "blackhole")"#);
        let sut: GurpRouteEnsure = serde_json::from_str(&json_def).unwrap();

        assert_eq!(
            "/usr/sbin/route -p add -blackhole 203.0.113.0/24 127.0.0.1",
            cmd::to_string(&sut.build_add_route_cmd())
        );

        // host route
        let json_def =
            janet2json(r#"(route/ensure "10.11.12.13" :gateway "192.168.1.10" :type "host")"#);
        let sut: GurpRouteEnsure = serde_json::from_str(&json_def).unwrap();

        assert_eq!(
            "/usr/sbin/route -p add -host 10.11.12.13 192.168.1.10",
            cmd::to_string(&sut.build_add_route_cmd())
        );

        // gateway route
        let json_def =
            janet2json(r#"(route/ensure "10.11.12.13" :gateway "router" :force-gateway true)"#);
        let sut: GurpRouteEnsure = serde_json::from_str(&json_def).unwrap();

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
