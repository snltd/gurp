use anyhow::Context;
use common::constants::IPADM_BIN;
use common::types::ApplyOpts;
use os_types::{AddrObj, LinkName};
use std::collections::{BTreeSet, HashMap};
use std::process::Command;

pub type IpProtocol = String;
pub type IpadmPropertyMap = HashMap<String, String>;
pub type IpProtocolMap = HashMap<IpProtocol, IpadmPropertyMap>;

pub enum IpObjType<'a> {
    Link(&'a LinkName),
    AddrObj(&'a AddrObj),
}

pub struct AlignIpPropArg<'a> {
    pub ipadm_cmd: &'a str,
    pub protocol: Option<&'a str>,
    pub property: &'a str,
    pub current_value: Option<&'a str>,
    pub desired_value: &'a str,
    pub protocol_requires_flag: bool,
    pub ip_object: Option<&'a IpObjType<'a>>,
}

fn modify_list_property(
    property: &str,
    value: &str,
    protocol: Option<&str>,
    operation: &str,
    opts: &ApplyOpts,
) -> anyhow::Result<()> {
    let mut cmd = cmd!(
        IPADM_BIN,
        "set-prop",
        "-p",
        format!("{property}{operation}={value}"),
    );

    if let Some(protocol) = protocol {
        cmd.arg(protocol);
    }

    if !opts.noop {
        let _stdout = run_cmd!(cmd)
            .with_context(|| format!("failed to set ip property {property} to {value}"))?;
    }

    Ok(())
}

// extra_priv_ports is a list. We need to align the list with the user's list, and we can
// only change one element of the list at a time
pub fn align_list_property(args: AlignIpPropArg, opts: &ApplyOpts) -> anyhow::Result<bool> {
    if property_alignment_notification(&args) {
        let current_props: BTreeSet<&str> = if let Some(current) = args.current_value {
            current.split(",").collect()
        } else {
            BTreeSet::new()
        };

        let desired_props: BTreeSet<&str> = args.desired_value.split(",").collect();

        for value in desired_props.difference(&current_props) {
            modify_list_property(args.property, value, args.protocol, "+", opts)?;
        }

        for value in current_props.difference(&desired_props) {
            modify_list_property(args.property, value, args.protocol, "-", opts)?;
        }

        Ok(true)
    } else {
        Ok(false)
    }
}

fn property_alignment_notification(args: &AlignIpPropArg) -> bool {
    let mut resource = if let Some(protocol) = args.protocol {
        format!("{protocol}/{}", args.property)
    } else {
        args.property.to_owned()
    };

    if let Some(ip_object) = args.ip_object {
        resource = match ip_object {
            IpObjType::Link(link) => format!("{link}:{resource}"),
            IpObjType::AddrObj(addrobj) => format!("{addrobj}:{resource}"),
        };
    }

    if let Some(current_value) = args.current_value {
        if current_value == args.desired_value {
            tracing::debug!("{resource} already {current_value}");
            return false;
        }

        tracing::info!(
            "{resource} changing {current_value} -> {}",
            args.desired_value
        );
    } else {
        tracing::info!(
            "{resource}/{} setting to {}",
            args.property,
            args.desired_value
        );
    }

    true
}

pub fn align_property(args: &AlignIpPropArg, opts: &ApplyOpts) -> anyhow::Result<bool> {
    if property_alignment_notification(args) {
        let mut cmd = construct_ipadm_prop_cmd(args);

        if !opts.noop {
            let _stdout = run_cmd!(cmd).context("failed to run ipadm")?;
        }

        Ok(true)
    } else {
        Ok(false)
    }
}

fn construct_ipadm_prop_cmd(args: &AlignIpPropArg) -> Command {
    let mut cmd = cmd!(
        IPADM_BIN,
        args.ipadm_cmd,
        "-p",
        format!("{}={}", args.property, args.desired_value),
    );

    if let Some(protocol) = args.protocol {
        if args.protocol_requires_flag {
            cmd.arg("-m");
        }
        cmd.arg(protocol);
    }

    if let Some(ip_object) = args.ip_object {
        cmd.arg(match ip_object {
            IpObjType::Link(link) => link.to_string(),
            IpObjType::AddrObj(addrobj) => addrobj.to_string(),
        });
    };

    tracing::debug!(command = common::cmd::to_string(&cmd));
    cmd
}

pub fn parse_ipadm_props(raw: &str) -> IpProtocolMap {
    let mut ret: IpProtocolMap = HashMap::new();

    for line in raw.lines() {
        let mut chunks = line.split(':');

        if let Some(protocol) = chunks.next()
            && let Some(property) = chunks.next()
            && let Some(value) = chunks.next()
        {
            let prop_map = ret.entry(protocol.to_owned()).or_default();
            prop_map
                .entry(property.to_owned())
                .or_insert(value.to_owned());
        }
    }

    ret
}

#[cfg(test)]
mod test {
    use super::*;
    use indoc::indoc;

    #[test]
    fn test_construct_ipadm_prop_cmd_ifprop() {
        let input = AlignIpPropArg {
            ipadm_cmd: "set-ifprop",
            protocol: Some("ipv6"),
            property: "nud",
            current_value: Some("off"), // not relevant here
            desired_value: "on",
            protocol_requires_flag: true,
            ip_object: Some(&IpObjType::Link(&LinkName::new("mvnic1").unwrap())),
        };

        let cmd = construct_ipadm_prop_cmd(&input);

        assert_eq!(
            "/usr/sbin/ipadm set-ifprop -p nud=on -m ipv6 mvnic1",
            common::cmd::to_string(&cmd)
        );
    }

    #[test]
    fn test_construct_ipadm_prop_cmd_prop() {
        let input = AlignIpPropArg {
            ipadm_cmd: "set-prop",
            protocol: Some("ipv6"),
            property: "hostmodel",
            current_value: Some("weak"), // not relevant here
            desired_value: "strong",
            protocol_requires_flag: false,
            ip_object: None,
        };

        let cmd = construct_ipadm_prop_cmd(&input);

        assert_eq!(
            "/usr/sbin/ipadm set-prop -p hostmodel=strong ipv6",
            common::cmd::to_string(&cmd)
        );
    }

    #[test]
    fn test_construct_ipadm_prop_cmd_addrprop() {
        let input = AlignIpPropArg {
            ipadm_cmd: "set-addrprop",
            protocol: None,
            property: "transmit",
            current_value: Some("off"), // not relevant here
            desired_value: "on",
            protocol_requires_flag: false,
            ip_object: Some(&IpObjType::AddrObj(&AddrObj::new("e1000g0/v4").unwrap())),
        };

        let cmd = construct_ipadm_prop_cmd(&input);

        assert_eq!(
            "/usr/sbin/ipadm set-addrprop -p transmit=on e1000g0/v4",
            common::cmd::to_string(&cmd)
        );
    }

    #[test]
    fn test_parse_ipadm_props() {
        let expected_ipv4 = HashMap::from([("hostmodel".to_owned(), "weak".to_owned())]);

        let expected_icmp = HashMap::from([
            ("max_buf".to_owned(), "262144".to_owned()),
            ("recv_buf".to_owned(), "8192".to_owned()),
        ]);

        let expected_tcp = HashMap::from([("congestion_control".to_owned(), "sunreno".to_owned())]);

        let expected: IpProtocolMap = HashMap::from([
            ("ipv4".to_owned(), expected_ipv4),
            ("icmp".to_owned(), expected_icmp),
            ("tcp".to_owned(), expected_tcp),
        ]);

        let input = indoc! { "
                ipv4:hostmodel:weak
                icmp:max_buf:262144
                icmp:recv_buf:8192
                tcp:congestion_control:sunreno
        "
        };

        assert_eq!(expected, parse_ipadm_props(input));
    }
}
