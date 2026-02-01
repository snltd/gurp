use anyhow::Context;
use nix::unistd;

pub fn my_hostname() -> anyhow::Result<String> {
    let hostname = unistd::gethostname()
        .context("Failed getting hostname")?
        .to_string_lossy()
        .into_owned();

    Ok(hostname)
}
