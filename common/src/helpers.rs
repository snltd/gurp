use crate::types::Opts;
use serde_json::Value;
use std::process::Command;
use xml::reader::{EventReader, XmlEvent};

const TW: usize = 80;

pub fn command_to_string(cmd: &Command) -> String {
    let program = cmd.get_program().to_string_lossy();
    let args = cmd
        .get_args()
        .map(|a| a.to_string_lossy())
        .collect::<Vec<_>>()
        .join(" ");

    format!("{program} {args}")
}

pub fn parse_xml(content: &str) -> Result<Vec<XmlEvent>, xml::reader::Error> {
    EventReader::from_str(content).into_iter().collect()
}

pub fn pretty_json(json_str: &str) -> anyhow::Result<String> {
    let value: Value = serde_json::from_str(json_str)?;
    Ok(serde_json::to_string_pretty(&value)?)
}

pub fn dump_config(code: &str, description: &str, opts: &Opts) -> String {
    let mut banner_begin = format!("--- BEGIN {description} ");
    let mut banner_end = format!("--- END {description} ");
    banner_begin.push_str("-".repeat(TW - banner_begin.len()).as_str());
    banner_end.push_str("-".repeat(TW - banner_end.len()).as_str());

    let mut ret = banner_begin;
    ret.push('\n');

    if opts.line_no {
        code.lines()
            .enumerate()
            .for_each(|(i, l)| ret.push_str(&format!("{:>5} | {}\n", i + 1, l)));
    } else {
        code.lines().for_each(|l| ret.push_str(&format!("{l}\n")));
    }

    ret.push_str(&banner_end);
    ret.push('\n');
    ret
}
