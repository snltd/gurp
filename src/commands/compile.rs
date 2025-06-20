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
    let mut client = janet_helpers::janet_client(global_opts);
    let host_config =
        match reader::read_and_enrich_host_config(host_file, gurp_lib_path, global_opts, true) {
            Ok(config) => config,
            Err(e) => {
                error!(opts, "compile/run", "Failed to compile: {}", e);
                return 1;
            }
        };

    client.add_c_fn(CFunOptions::new(
        c"output-machine-configuration",
        output_config_handler_c,
    ));

    match client.run(host_config) {
        Ok(_) => 0,
        Err(e) => {
            error!(opts, "compile/run", "Failed to compile: {}", e);
            1
        }
    }
}

#[janetrs::janet_fn(arity(fix(1)))]
fn output_config_handler(janet_config: &mut [Janet]) -> Janet {
    let config_elements = janet_config.len() as i32;

    if config_elements != 1 {
        error!(
            opts,
            "handler", "expected single host configuration element, got {}", config_elements
        );
        return Janet::from(false);
    }

    println!("{}", janet_helpers::pretty_janet(&janet_config[0], 4));
    Janet::from(true)
}
