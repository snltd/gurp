use common::helpers;
use common::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt::Debug;
use std::process::Command;

// THINGS TO KNOW / THINGS TO DO.

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct GurpNetworkFlowEnsure {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
    pub link: String,
    pub local_ip: Option<String>,
    pub remote_ip: Option<String>,
    pub protocol: Option<String>,
    pub local_port: Option<u16>,
    pub remote_port: Option<u16>,
    pub dsfield: Option<String>,
    pub maxbw: Option<String>,
    pub priority: Option<String>,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct GurpNetworkFlowRemove {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
}

type FlowName = String;
type ExtantFlows = HashMap<FlowName, ExtantFlow>;

#[derive(Debug, PartialEq)]
pub struct ExtantFlow {
    pub link: String,
    pub local_ip: Option<String>,
    pub remote_ip: Option<String>,
    pub transport: Option<String>,
    pub local_port: Option<u16>,
    pub remote_port: Option<u16>,
    pub dsfield: Option<String>,
}

// Flows only have these two properties
#[derive(Default, Debug, PartialEq)]
struct ExtantFlowprops {
    maxbw: Option<String>,
    priority: Option<String>,
}

impl GurpNetworkFlowEnsure {
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        let extant_flows = parse_flows(&get_flows()?);

        if let Some(extant_flow) = extant_flows.get(&self.name) {
            tracing::debug!("found existing flow {}", self.name);

            // if the flow isn't right, we must remove it and recreate it

            if self.flow_is_correct(extant_flow) {
                let flowprops = parse_flowprops(&get_flowprops(&self.name)?);

                if self.flowprops_are_correct(&flowprops) {
                    Ok(ONE_RESOURCE_NO_CHANGE)
                } else {
                    self.update_flowprops(opts)
                }
            } else {
                tracing::info!("must recreate flow {}: removing", self.name);
                cmd_output!(FLOWADM_BIN, "remove-flow", &self.name)?;
                self.create_flow(opts)
            }
        } else {
            self.create_flow(opts)
        }
    }

    fn create_flow(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        tracing::info!("creating flow {}", self.name);

        let mut cmd = self.build_command();
        tracing::debug!(command = common::helpers::command_to_string(&cmd));

        if !opts.noop {
            let status = cmd.status()?;

            if !status.success() {
                bail!("Error running flowadm command");
            }
        }
        Ok(ONE_RESOURCE_ONE_CHANGE)
    }

    fn flow_is_correct(&self, flow: &ExtantFlow) -> bool {
        self.link == flow.link
            && self.local_ip == flow.local_ip
            && self.remote_ip == flow.remote_ip
            && self.local_port == flow.local_port
            && self.remote_port == flow.remote_port
            && self.dsfield == flow.dsfield
    }

    fn flowprops_are_correct(&self, flowprops: &ExtantFlowprops) -> bool {
        // This is a pain. flowadm expects "1G" but outputs "1000"
        let numeric_bw = if let Some(maxbw) = &self.maxbw {
            if maxbw.ends_with('M') {
                Some(maxbw.replace('M', ""))
            } else if maxbw.ends_with('G') {
                let number = maxbw.replace('G', "");
                number.parse::<usize>().ok().map(|n| (n * 1000).to_string())
            } else {
                None
            }
        } else {
            None
        };

        numeric_bw == flowprops.maxbw && self.priority == flowprops.priority
    }

    fn update_flowprops(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        tracing::info!("resetting properties on {}", self.name);

        if !opts.noop {
            cmd_output!(FLOWADM_BIN, "reset-flowprop", &self.name)?;
        }

        let mut prop_args = Vec::new();

        if let Some(maxbw) = &self.maxbw {
            tracing::info!("setting maxbw to {maxbw}");
            prop_args.push(format!("maxbw={maxbw}"));
        }

        if let Some(priority) = &self.priority {
            tracing::info!("setting priority to {priority}");
            prop_args.push(format!("priority={priority}"));
        }

        let prop_arg: String = prop_args.join(",");

        if !opts.noop {
            cmd_output!(FLOWADM_BIN, "set-flowprop", "-p", prop_arg, &self.name)?;
        }

        Ok(ONE_RESOURCE_ONE_CHANGE)
    }

    // This builds the add commands, for when we don't have a flow
    fn build_command(&self) -> Command {
        let mut cmd = Command::new(FLOWADM_BIN);
        cmd.arg("add-flow");
        cmd.arg("-l");
        cmd.arg(&self.link);

        let mut attributes = Vec::new();

        if let Some(local_ip) = &self.local_ip {
            attributes.push(format!("local_ip={local_ip}"));
        }

        if let Some(remote_ip) = &self.remote_ip {
            attributes.push(format!("remote_ip={remote_ip}"));
        }

        if let Some(local_port) = &self.local_port {
            attributes.push(format!("local_port={local_port}"));
        }

        if let Some(remote_port) = &self.remote_port {
            attributes.push(format!("remote_port={remote_port}"));
        }

        if let Some(protocol) = &self.protocol {
            attributes.push(format!("transport={protocol}"));
        }

        if let Some(dsfield) = &self.dsfield {
            attributes.push(format!("dsfield={dsfield}"));
        }

        if !attributes.is_empty() {
            cmd.arg("-a");
            cmd.arg(attributes.join(","));
        }

        let mut properties = Vec::new();

        if let Some(maxbw) = &self.maxbw {
            properties.push(format!("maxbw={maxbw}"));
        }

        if let Some(priority) = &self.priority {
            properties.push(format!("priority={priority}"));
        }

        if !properties.is_empty() {
            cmd.arg("-p");
            cmd.arg(properties.join(","));
        }

        cmd.arg(&self.name);
        cmd
    }
}

impl GurpNetworkFlowRemove {
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        let extant_flows = parse_flows(&get_flows()?);

        if extant_flows.contains_key(&self.name) {
            tracing::info!("removing flow {}", self.name);
            return_if_noop!(opts);

            cmd_output!(FLOWADM_BIN, "remove-flow", &self.name)?;
            Ok(ONE_RESOURCE_ONE_CHANGE)
        } else {
            tracing::debug!("flow {} does not exist", self.name);
            Ok(ONE_RESOURCE_NO_CHANGE)
        }
    }
}

fn get_flows() -> anyhow::Result<String> {
    cmd_output!(
        FLOWADM_BIN,
        "show-flow",
        "-p",
        "-o",
        "flow,link,ipaddr,proto,lport,rport,dsfld"
    )
}

fn get_flowprops(flow_name: &str) -> anyhow::Result<String> {
    cmd_output!(
        FLOWADM_BIN,
        "show-flowprop",
        "-c",
        "-o",
        "property,value",
        flow_name,
    )
}

fn string_field_or_none(bits: &[String], index: usize) -> Option<String> {
    bits.get(index)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
}

fn u16_field_or_none(bits: &[String], index: usize) -> Option<u16> {
    if let Some(bit) = bits.get(index) {
        bit.parse::<u16>().ok()
    } else {
        None
    }
}

fn extract_ip(raw: &str) -> String {
    if let Some(addr) = raw.trim().split(':').next_back() {
        addr.to_string()
    } else {
        String::new()
    }
}

fn parse_flows(raw: &str) -> ExtantFlows {
    raw.lines()
        .filter_map(|l| {
            let bits: Vec<_> = helpers::split_unescaped_colon(l.trim());

            if bits.len() == 7 {
                let (remote_ip, local_ip) = if bits[2].starts_with("RMT") {
                    (Some(extract_ip(&bits[2])), None)
                } else if bits[2].starts_with("LCL") {
                    (None, Some(extract_ip(&bits[2])))
                } else {
                    (None, None)
                };

                Some((
                    bits[0].clone(),
                    ExtantFlow {
                        link: bits[1].clone(),
                        local_ip,
                        remote_ip,
                        transport: string_field_or_none(&bits, 3),
                        local_port: u16_field_or_none(&bits, 4),
                        remote_port: u16_field_or_none(&bits, 5),
                        dsfield: string_field_or_none(&bits, 6),
                    },
                ))
            } else {
                None
            }
        })
        .collect()
}

fn parse_flowprops(raw: &str) -> ExtantFlowprops {
    let mut ret = ExtantFlowprops::default();

    for l in raw.lines() {
        let bits: Vec<_> = l.trim().split(':').collect();

        if let Some(v) = bits.get(1) {
            let val = if *v == "--" {
                None
            } else {
                Some(v.trim().to_string())
            };

            if l.starts_with("maxbw:") {
                ret.maxbw = val;
            } else if l.starts_with("priority:") {
                ret.priority = val;
            }
        }
    }

    ret
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    macro_rules! assert_cmd_call {
        ($sut_type:ident, $janet_resource:expr, $expected_command:expr$(,)?) => {
            let json_def = tester::janet2json($janet_resource);
            let sut: $sut_type = serde_json::from_str(&json_def).unwrap();

            assert_eq!(
                $expected_command,
                common::helpers::command_to_string(&sut.build_command())
            );
        };
    }

    #[test]
    fn test_parse_flowprop() {
        let input = indoc::indoc! { r#"
            maxbw:--
            priority:high
            "# };

        assert_eq!(
            ExtantFlowprops {
                maxbw: None,
                priority: Some("high".to_owned()),
            },
            parse_flowprops(input)
        );

        let input = indoc::indoc! { r#"
            maxbw: 1000
            priority:high
            "# };

        assert_eq!(
            ExtantFlowprops {
                maxbw: Some("1000".to_owned()),
                priority: Some("high".to_owned()),
            },
            parse_flowprops(input)
        );
    }

    #[test]
    fn test_parse_flows() {
        // The machine-parseable output is probably less machine-parseable than the normal!
        let input = indoc::indoc! { r#"
            testflow1:e1000g0::tcp:443::
            testflow2:e1000g0::tcp:80::
            testflow3:ws_net0:RMT\:5.6.7.8/32  ::::
            testflow4:gurp_net0:LCL\:1.2.3.4/32  ::::
            testflow6:media_net0::tcp::12345:
            "# };

        let expected = HashMap::from([
            (
                "testflow1".to_owned(),
                ExtantFlow {
                    link: "e1000g0".to_owned(),
                    local_ip: None,
                    remote_ip: None,
                    transport: Some("tcp".to_owned()),
                    local_port: Some(443),
                    remote_port: None,
                    dsfield: None,
                },
            ),
            (
                "testflow2".to_owned(),
                ExtantFlow {
                    link: "e1000g0".to_owned(),
                    local_ip: None,
                    remote_ip: None,
                    transport: Some("tcp".to_owned()),
                    local_port: Some(80),
                    remote_port: None,
                    dsfield: None,
                },
            ),
            (
                "testflow3".to_owned(),
                ExtantFlow {
                    link: "ws_net0".to_owned(),
                    local_ip: None,
                    remote_ip: Some("5.6.7.8/32".to_owned()),
                    transport: None,
                    local_port: None,
                    remote_port: None,
                    dsfield: None,
                },
            ),
            (
                "testflow4".to_owned(),
                ExtantFlow {
                    link: "gurp_net0".to_owned(),
                    local_ip: Some("1.2.3.4/32".to_owned()),
                    remote_ip: None,
                    transport: None,
                    local_port: None,
                    remote_port: None,
                    dsfield: None,
                },
            ),
            (
                "testflow6".to_owned(),
                ExtantFlow {
                    link: "media_net0".to_owned(),
                    local_ip: None,
                    remote_ip: None,
                    transport: Some("tcp".to_owned()),
                    local_port: None,
                    remote_port: Some(12345),
                    dsfield: None,
                },
            ),
        ]);

        assert_eq!(expected, parse_flows(input));
    }

    #[test]
    fn test_build_command() {
        assert_cmd_call!(
            GurpNetworkFlowEnsure,
            r#"(network-flow/ensure "web_flow"
                                    :link "vnic1"
                                    :protocol "tcp"
                                    :local-port 80
                                    :maxbw "10M") "#,
            "/usr/sbin/flowadm add-flow -l vnic1 -a local_port=80,transport=tcp -p maxbw=10M web_flow",
        );

        assert_cmd_call!(
            GurpNetworkFlowEnsure,
            r#"(network-flow/ensure "tls_shaper"
                                    :link "vnic2"
                                    :protocol "tcp"
                                    :remote-ip "203.0.113.4"
                                    :remote-port 443
                                    :maxbw "10M") "#,
            "/usr/sbin/flowadm add-flow -l vnic2 -a remote_ip=203.0.113.4,remote_port=443,transport=tcp -p maxbw=10M tls_shaper",
        );
    }
}
