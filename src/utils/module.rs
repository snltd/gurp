/*
use crate::utils::types::Opts;
use crate::{debug, verbose};
use camino::Utf8PathBuf;

pub fn process(module_path: &Utf8PathBuf, opts: &Opts) -> anyhow::Result<()> {
    verbose!(opts, "Running module {}", module_path);
    // WHAT DO WE DO?
    todo!()
}

pub fn find(module: &str, module_path: &str, opts: &Opts) -> Option<Utf8PathBuf> {
    module_path.split(":").map(Utf8PathBuf::from).find(|d| {
        let candidate = d.join(format!("{}.janet", module));
        debug!(opts, "looking for {}", candidate);
        candidate.exists()
    })
}
*/
