use common::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::process::{Command, Stdio};

// THINGS TO KNOW / THINGS TO DO.
// The route command is messy legacy, and it takes all manner of commands. This is a best-
// guess attempt to provide something useful
// We only add persistent routes.
// Flags only get set when a route is created. We can't change them on an existing route.

#[derive(Debug, PartialEq)]
struct Route {
    destination: String,
    gateway: String,
}

type Routes = Vec<Route>;
type Flags = HashMap<String, String>;

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct GurpRouteEnsure {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(rename = "name")]
    pub destination: String,
    pub gateway: String,
    pub flags: Option<Flags>,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct GurpRouteRemove {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(rename = "name")]
    pub destination: String,
    pub gateway: String,
}

fn current_routes() -> anyhow::Result<Routes> {
    let raw = cmd_output!(ROUTE_BIN, "-p", "show")?;
    Ok(parse_route_table(&raw))
}

fn route_exists(desired: &Route) -> anyhow::Result<bool> {
    let routes: Routes = current_routes()?;
    println!("comparing {:?}", desired);
    println!("{:#?}", routes);
    Ok(routes.contains(desired))
}

impl GurpRouteEnsure {
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        let route = Route {
            destination: self.destination.clone(),
            gateway: self.gateway.clone(),
        };

        if route_exists(&route)? {
            tracing::debug!("{} -> {} already exists", self.destination, self.gateway);
            Ok(ONE_RESOURCE_NO_CHANGE)
        } else {
            tracing::info!("creating {} -> {}", self.destination, self.gateway);

            let mut cmd = Command::new(ROUTE_BIN);
            cmd.arg("-p");
            cmd.arg("add");
            cmd.arg(&self.destination);

            if let Some(flags) = &self.flags {
                for (k, v) in flags {
                    cmd.arg(format!("-{k}"));
                    if v.as_str() != "true" {
                        cmd.arg(v);
                    }
                }
            }

            cmd.arg(&self.gateway);
            cmd.stderr(Stdio::piped());

            tracing::debug!(command = common::helpers::command_to_string(&cmd));

            if !opts.noop {
                let status = cmd.status()?;

                if !status.success() {
                    bail!("Error running route command");
                }
            }

            Ok(ONE_RESOURCE_ONE_CHANGE)
        }
    }
}

impl GurpRouteRemove {
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        let route = Route {
            destination: self.destination.clone(),
            gateway: self.gateway.clone(),
        };

        if route_exists(&route)? {
            tracing::info!("removing {} -> {}", self.destination, self.gateway);
            let mut cmd = cmd!(ROUTE_BIN, "-p", "delete", &self.gateway, &self.destination);

            if !opts.noop {
                cmd.stderr(Stdio::piped());
                let status = cmd.status()?;

                if !status.success() {
                    bail!("Error running route command");
                }
            }

            Ok(ONE_RESOURCE_ONE_CHANGE)
        } else {
            tracing::debug!("{} -> {} does not exist", self.destination, self.gateway);
            Ok(ONE_RESOURCE_NO_CHANGE)
        }
    }
}

fn parse_route_table(raw: &str) -> Routes {
    let mut ret: Routes = Vec::new();

    for line in raw.lines().filter(|l| l.starts_with("persistent: route")) {
        let fields: Vec<_> = line.split_whitespace().collect();

        if let Some(destination) = fields.get(3)
            && let Some(gateway) = fields.get(fields.len() - 1)
        {
            ret.push(Route {
                destination: destination.to_string(),
                gateway: gateway.to_string(),
            })
        }
    }

    ret
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_default_route_exists() {}

    #[test]
    fn test_parse_route_table() {
        let input = indoc::indoc! { "
            persistent: route add default 192.168.1.1
            persistent: route add 10.0.0.0/16 -gateway 10.0.0.2
            "
        };

        assert_eq!(
            vec![
                Route {
                    destination: "default".to_owned(),
                    gateway: "192.168.1.1".to_owned(),
                },
                Route {
                    destination: "10.0.0.0/16".to_owned(),
                    gateway: "10.0.0.2".to_owned(),
                }
            ],
            parse_route_table(input)
        );
    }
}
