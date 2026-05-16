use anyhow::Context;
use common::constants::PKG_BIN;

pub(crate) fn publisher_exists(name: &str) -> anyhow::Result<bool> {
    let pattern = format!("{name} ");

    Ok(list_publishers()
        .context("cannot list publishers")?
        .lines()
        .filter(|l| l.contains("origin") || l.contains("mirror"))
        .any(|l| l.starts_with(&pattern)))
}

fn list_publishers() -> anyhow::Result<String> {
    cmd_output!(PKG_BIN, "publisher", "-H")
}
