use anyhow::{Context, ensure};
use common::cmd;
use common::constants::{FLOWADM_BIN, ONE_RESOURCE_NO_CHANGE, ONE_RESOURCE_ONE_CHANGE};
use common::types::{ApplyOpts, ApplySummary};
use os_types::{FlowAddr, GurpId, LinkName};
use serde::Deserialize;
use std::fmt::Debug;
use std::process::Command;

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct GurpNetworkFlowEnsure {
    #[serde(rename = "_id")]
    pub id: GurpId,
    pub name: String,
    pub link: LinkName,
    pub local_ip: Option<FlowAddr>,
    pub remote_ip: Option<FlowAddr>,
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
    pub id: GurpId,
    pub name: String,
}

#[derive(Debug, PartialEq)]
pub struct ExtantFlow {
    pub name: String,
    pub link: LinkName,
    pub local_ip: Option<FlowAddr>,
    pub remote_ip: Option<FlowAddr>,
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
        let extant_flows = parse_flows(&get_flows()?).context("cannot get list of flows")?;

        if let Some(extant_flow) = extant_flows.iter().find(|f| f.name == self.name) {
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
                cmd_output!(FLOWADM_BIN, "remove-flow", &self.name)
                    .with_context(|| format!("failed to remove network-flow {}", self.name))?;
                self.create_flow(opts)
            }
        } else {
            self.create_flow(opts)
        }
    }

    fn create_flow(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        tracing::info!("creating flow {}", self.name);

        let mut cmd = self.build_command();
        tracing::debug!(command = cmd::to_string(&cmd));

        if !opts.noop {
            let status = cmd
                .status()
                .with_context(|| format!("failed to create network-flow {}", self.name))?;

            ensure!(status.success(), "Error running flowadm command");
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
            cmd_output!(FLOWADM_BIN, "reset-flowprop", &self.name)
                .with_context(|| format!("failed to reset flowprops for {}", self.name))?;
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
            cmd_output!(FLOWADM_BIN, "set-flowprop", "-p", prop_arg, &self.name)
                .with_context(|| format!("failed to set flowprops for {}", self.name))?;
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
        let extant_flows = parse_flows(&get_flows()?).context("cannot get list of flows")?;

        if extant_flows.iter().find(|f| f.name == self.name).is_some() {
            tracing::info!("removing flow {}", self.name);
            cmd_change_or_noop!(opts, FLOWADM_BIN, "remove-flow", &self.name)
                .with_context(|| format!("failed to remote network-flow {}", self.name))
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
    .context("failed to get network-flows")
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
    .with_context(|| format!("failed to get flowprops for network-flow {flow_name}"))
}

fn extract_cidr(raw: &str) -> anyhow::Result<Option<FlowAddr>> {
    raw.trim()
        .split(':')
        .next_back()
        .filter(|addr| !addr.is_empty())
        .map(|addr| {
            addr.parse::<FlowAddr>()
                .with_context(|| format!("cannot parse flow IP CIDR {raw}"))
        })
        .transpose()
}

fn parse_flow(raw: &str) -> anyhow::Result<ExtantFlow> {
    let chunks = split_unescaped_colon(raw.trim());

    let [name, link, ip, transport, lport, rport, dsfield]: [String; 7] = chunks
        .try_into()
        .map_err(|_| anyhow::anyhow!("expected 7 fields in {raw}"))?;

    let (remote_ip, local_ip) = if ip.starts_with("RMT") {
        (extract_cidr(&ip)?, None)
    } else if ip.starts_with("LCL") {
        (None, extract_cidr(&ip)?)
    } else {
        (None, None)
    };

    Ok(ExtantFlow {
        name,
        link: LinkName::new(link)?,
        local_ip,
        remote_ip,
        transport: (!transport.is_empty()).then_some(transport),
        local_port: (!lport.is_empty()).then(|| lport.parse()).transpose()?,
        remote_port: (!rport.is_empty()).then(|| rport.parse()).transpose()?,
        dsfield: (!dsfield.is_empty()).then_some(dsfield),
    })
}

fn parse_flows(raw: &str) -> anyhow::Result<Vec<ExtantFlow>> {
    raw.lines().map(|l| parse_flow(l.trim())).collect()
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

fn split_unescaped_colon(s: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut escaped = false;

    for c in s.chars() {
        if escaped {
            current.push(c);
            escaped = false;
        } else if c == '\\' {
            escaped = true;
        } else if c == ':' {
            parts.push(current);
            current = String::new();
        } else {
            current.push(c);
        }
    }

    parts.push(current);
    parts
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
    use tester::deserialized_example;

    #[test]
    fn test_deserialize_network_flow_ensure_tcp_443_throttle() {
        assert_eq!(
            GurpNetworkFlowEnsure {
                name: "tls-throttle".to_owned(),
                id: GurpId::new("/NO-ROLE/network-flow/tls-throttle").unwrap(),
                link: LinkName::new("vnic1").unwrap(),
                protocol: Some("tcp".to_owned()),
                remote_ip: None,
                remote_port: Some(443),
                maxbw: Some("10M".to_owned()),
                dsfield: None,
                priority: None,
                local_ip: None,
                local_port: None,
            },
            deserialized_example("network-flow/ensure-tcp-443-throttle.janet")
        );
    }

    #[test]
    fn test_deserialize_network_flow_ensure_ssh_local_throttle() {
        assert_eq!(
            GurpNetworkFlowEnsure {
                name: "ssh-flow".to_owned(),
                id: GurpId::new("/NO-ROLE/network-flow/ssh-flow").unwrap(),
                link: LinkName::new("vnic1").unwrap(),
                protocol: Some("tcp".to_owned()),
                remote_ip: None,
                remote_port: None,
                maxbw: Some("1200K".to_owned()),
                dsfield: None,
                priority: None,
                local_ip: None,
                local_port: Some(22),
            },
            deserialized_example("network-flow/ensure-ssh-local-throttle.janet")
        );
    }

    #[test]
    fn test_deserialize_network_flow_remove_flow() {
        assert_eq!(
            GurpNetworkFlowRemove {
                name: "unwanted".to_owned(),
                id: GurpId::new("/NO-ROLE/network-flow/unwanted").unwrap(),
            },
            deserialized_example("network-flow/remove-flow.janet")
        );
    }

    macro_rules! assert_cmd_call {
        ($sut_type:ident, $janet_resource:expr, $expected_command:expr$(,)?) => {
            let json_def = tester::janet2json($janet_resource);
            let sut: $sut_type = serde_json::from_str(&json_def).unwrap();

            assert_eq!(
                $expected_command,
                common::cmd::to_string(&sut.build_command())
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

        let expected = vec![
            ExtantFlow {
                name: "testflow1".to_owned(),
                link: LinkName::new("e1000g0").unwrap(),
                local_ip: None,
                remote_ip: None,
                transport: Some("tcp".to_owned()),
                local_port: Some(443),
                remote_port: None,
                dsfield: None,
            },
            ExtantFlow {
                name: "testflow2".to_owned(),
                link: LinkName::new("e1000g0").unwrap(),
                local_ip: None,
                remote_ip: None,
                transport: Some("tcp".to_owned()),
                local_port: Some(80),
                remote_port: None,
                dsfield: None,
            },
            ExtantFlow {
                name: "testflow3".to_owned(),
                link: LinkName::new("ws_net0").unwrap(),
                local_ip: None,
                remote_ip: Some("5.6.7.8/32".parse().unwrap()),
                transport: None,
                local_port: None,
                remote_port: None,
                dsfield: None,
            },
            ExtantFlow {
                name: "testflow4".to_owned(),
                link: LinkName::new("gurp_net0").unwrap(),
                local_ip: Some("1.2.3.4/32".parse().unwrap()),
                remote_ip: None,
                transport: None,
                local_port: None,
                remote_port: None,
                dsfield: None,
            },
            ExtantFlow {
                name: "testflow6".to_owned(),
                link: LinkName::new("media_net0").unwrap(),
                local_ip: None,
                remote_ip: None,
                transport: Some("tcp".to_owned()),
                local_port: None,
                remote_port: Some(12345),
                dsfield: None,
            },
        ];

        assert_eq!(expected, parse_flows(input).unwrap());
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
                                    :remote-ip "203.0.113.4/32"
                                    :remote-port 443
                                    :maxbw "10M") "#,
            "/usr/sbin/flowadm add-flow -l vnic2 -a remote_ip=203.0.113.4/32,remote_port=443,transport=tcp -p maxbw=10M tls_shaper",
        );
    }
}
