use crate::constants::DEFAULT_TERM_WIDTH;
use crate::types::ApplyOutputOpts;
use owo_colors::OwoColorize;

pub fn dump_config(code: &str, description: Option<&str>, opts: &ApplyOutputOpts) -> String {
    let mut ret = String::new();

    if let Some(description) = description {
        ret.push_str(&banner("BEGIN", description));
    }

    if opts.line_no {
        code.lines()
            .enumerate()
            .for_each(|(i, l)| ret.push_str(&format!("{:>5} | {}\n", i + 1, l)));
    } else {
        code.lines().for_each(|l| ret.push_str(&format!("{l}\n")));
    }

    if let Some(description) = description {
        ret.push_str(&banner("END", description));
    }
    ret
}

pub fn dump_diff(
    existing: &str,
    desired: &str,
    description: Option<&str>,
    opts: &ApplyOutputOpts,
) -> String {
    let mut ret = String::new();

    if let Some(description) = description {
        ret.push_str(&banner("BEGIN", &format!("{description} diff")));
    }

    for diff in diff::lines(existing, desired) {
        match diff {
            diff::Result::Left(l) if opts.colour => ret.push_str(&format!("-{}\n", l.red())),
            diff::Result::Left(l) => ret.push_str(&format!("-{l}\n")),
            diff::Result::Both(_, _) => (),
            diff::Result::Right(r) if opts.colour => ret.push_str(&format!("+{}\n", r.green())),
            diff::Result::Right(r) => ret.push_str(&format!("+{r}\n")),
        }
    }

    if let Some(description) = description {
        ret.push_str(&banner("END", &format!("{description} diff")));
    }
    ret
}

fn banner(marker: &str, description: &str) -> String {
    let mut ret = format!("--- {marker} {description} ");
    ret.push_str("-".repeat(DEFAULT_TERM_WIDTH - ret.len()).as_str());
    ret.push('\n');
    ret
}
