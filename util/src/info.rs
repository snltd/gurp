use anyhow::Context;
use nix::unistd;
use std::sync::LazyLock;

pub static BUILD_HASH: LazyLock<&'static str> = LazyLock::new(|| {
    let sha = env!("VERGEN_GIT_SHA");
    &sha[..sha.len().min(7)]
});

pub fn build_hash() -> &'static str {
    *BUILD_HASH
}

pub fn my_hostname() -> anyhow::Result<String> {
    let hostname = unistd::gethostname()
        .context("failed to get hostname")?
        .to_string_lossy()
        .into_owned();

    Ok(hostname)
}
