use crate::common::types::{ApplySummary, ExitCode, Opts};
use crate::doers::host;
use crate::error;
use crate::utils::{janet_helpers, parser, reader};
use camino::Utf8PathBuf;
use colored::Colorize;
use janetrs::{Janet, TaggedJanet, env::CFunOptions};
use std::time::{Duration, Instant};

pub fn run(
    host_file: &Utf8PathBuf,
    gurp_lib_path: &Option<Utf8PathBuf>,
    global_opts: &Opts,
) -> ExitCode {
    let host_config =
        match reader::read_and_enrich_host_config(host_file, gurp_lib_path, global_opts, true) {
            Ok(config) => config,
            Err(e) => {
                error!(opts, "compile/run", "Failed to compile: {}", e);
                return 1;
            }
        };

    let client = janet_helpers::janet_client(global_opts);
    match client.run(host_config) {
        Ok(_) => 0,
        Err(e) => {
            error!(opts, "compile/run", "Failed to compile: {}", e);
            1
        }
    }
}
