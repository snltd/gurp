use anyhow::{Context, bail};
use camino::Utf8PathBuf;
use nix::unistd;
use std::env;
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

pub fn gurp_path() -> anyhow::Result<Utf8PathBuf> {
    let gurp_path = env::current_exe().context("cannot get current Gurp path")?;
    match Utf8PathBuf::from_path_buf(gurp_path) {
        Ok(path) => Ok(path),
        Err(_) => bail!("cannot make utf8 path for gurp binary"),
    }
}
