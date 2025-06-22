use crate::common::types::{ExitCode, Opts};
use crate::utils::{janet_helpers, reader};
use camino::Utf8PathBuf;

pub fn run(
    host_file: &Utf8PathBuf,
    gurp_lib_path: &Option<Utf8PathBuf>,
    global_opts: &Opts,
) -> ExitCode {
    let host_config =
        match reader::read_and_enrich_host_config(host_file, gurp_lib_path, global_opts, true) {
            Ok(config) => config,
            Err(e) => {
                tracing::error!("reader error: {}", e);
                return 1;
            }
        };

    let client = janet_helpers::janet_client();

    match client.run(host_config) {
        Ok(_) => 0,
        Err(e) => {
            tracing::error!("Janet execution error: {}", e);
            1
        }
    }
}
