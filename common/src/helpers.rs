use crate::types::ApplyOpts;
use anyhow::Context;
use colored::Colorize;
use nix::unistd;
use serde_json::Value;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
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

fn banner(marker: &str, description: &str) -> String {
    let mut ret = format!("--- {marker} {description} ");
    ret.push_str("-".repeat(TW - ret.len()).as_str());
    ret.push('\n');
    ret
}

pub fn dump_config(code: &str, description: &str, opts: &ApplyOpts) -> String {
    let mut ret = banner("BEGIN", description);

    if opts.line_no {
        code.lines()
            .enumerate()
            .for_each(|(i, l)| ret.push_str(&format!("{:>5} | {}\n", i + 1, l)));
    } else {
        code.lines().for_each(|l| ret.push_str(&format!("{l}\n")));
    }

    ret.push_str(&banner("END", description));
    ret
}

pub fn dump_diff(existing: &str, desired: &str, description: &str, colour: bool) -> String {
    let mut ret = banner("BEGIN", &format!("{description} diff"));

    for diff in diff::lines(existing, desired) {
        match diff {
            diff::Result::Left(l) if colour => ret.push_str(&format!("-{}\n", l.red())),
            diff::Result::Left(l) => ret.push_str(&format!("-{l}\n")),
            diff::Result::Both(_, _) => (),
            diff::Result::Right(r) if colour => ret.push_str(&format!("+{}\n", r.green())),
            diff::Result::Right(r) => ret.push_str(&format!("+{r}\n")),
        }
    }

    ret.push_str(&banner("END", &format!("{description} diff")));
    ret
}

pub fn epoch_time_as_string() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .to_string()
}

pub fn my_hostname() -> anyhow::Result<String> {
    let hostname = unistd::gethostname()
        .context("Failed getting hostname")?
        .to_string_lossy()
        .into_owned();

    Ok(hostname)
}

pub fn split_unescaped_colon(s: &str) -> Vec<String> {
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
