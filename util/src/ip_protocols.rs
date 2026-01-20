use common::prelude::*;
use std::collections::HashMap;

pub type IpProtocol = String;
pub type IpadmPropertyMap = HashMap<String, String>;
pub type IpProtocolMap = HashMap<IpProtocol, IpadmPropertyMap>;

pub struct AlignIpPropArg<'a> {
    pub ipadm_cmd: &'a str,
    pub protocol: Option<&'a str>,
    pub property: &'a str,
    pub current_value: Option<&'a str>,
    pub desired_value: &'a str,
    pub pass_protocol_to_ipadm: bool,
    pub ipadm_final_arg: Option<&'a str>,
    pub opts: &'a ApplyOpts,
}

pub fn align_property(args: AlignIpPropArg) -> anyhow::Result<bool> {
    let AlignIpPropArg {
        ipadm_cmd,
        protocol,
        property,
        current_value,
        desired_value,
        pass_protocol_to_ipadm,
        opts,
        ipadm_final_arg,
    } = args;

    let mut resource = if let Some(protocol) = protocol {
        format!("{protocol}/{property}")
    } else {
        property.to_owned()
    };

    if let Some(final_arg) = ipadm_final_arg {
        resource = format!("{final_arg}:{resource}");
    }

    if let Some(current_value) = current_value {
        if current_value == desired_value {
            tracing::debug!("{resource} already {current_value}");
            return Ok(false);
        }

        tracing::info!("{resource} changing {current_value} -> {desired_value}");
    } else {
        tracing::info!("{resource}/{property} setting to {desired_value}");
    }

    let mut cmd = cmd!(
        IPADM_BIN,
        ipadm_cmd,
        "-p",
        format!("{property}={desired_value}"),
    );

    if let Some(protocol) = protocol {
        cmd.arg(protocol);

        if pass_protocol_to_ipadm {
            cmd.args(["-m", protocol]);
        }
    }

    if let Some(ipadm_arg) = ipadm_final_arg {
        cmd.arg(ipadm_arg);
    }

    if !opts.noop {
        let _stdout = run_cmd!(cmd);
    }

    Ok(true)
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
