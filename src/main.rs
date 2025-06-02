mod doers;
mod test_utils;
mod utils;

use crate::doers::host;
use crate::doers::types::ApplySummary;
use crate::utils::janet_helpers;
use crate::utils::types::Opts;
use camino::Utf8PathBuf;
use clap::Parser;
use std::time::{Duration, Instant};

#[derive(Parser)]
#[clap(version, about = "Configures hosts, or might do one day", long_about = None)]
struct Cli {
    /// Be verbose
    #[arg(short, long, global = true)]
    verbose: bool,
    /// Be very verbose
    #[arg(short, long, global = true)]
    debug: bool,
    /// Say what would happen, without actually doing it
    #[arg(short, long, global = true)]
    noop: bool,
    /// Specify a gurp Janet library, in preference to the built-in
    #[arg(short = 'L', long = "gurp-lib", global = true)]
    gurp_lib_path: Option<Utf8PathBuf>,
    /// Host configuration file
    #[arg(required = true)]
    host_config_file: Utf8PathBuf,
} // might not need the global. Will there be subcommands?

fn main() -> anyhow::Result<()> {
    let mut exit_code = 0;
    let cli = Cli::parse();

    let opts = Opts {
        debug: cli.debug,
        noop: cli.noop,
        verbose: cli.verbose,
        gurp_lib_path: cli.gurp_lib_path,
    };

    let start_time = Instant::now();
    let apply_result = host::apply(&cli.host_config_file, &opts);
    let elapsed_time = start_time.elapsed();

    // Without this we can't unwrap the summary
    unsafe {
        janetrs::lowlevel::janet_init();
    }

    match apply_result {
        Ok(res) => match res.unwrap() {
            janetrs::TaggedJanet::Struct(_) => match janet_helpers::unwrap_summary(&res) {
                Ok(summary) => report_results(&summary, elapsed_time, &opts),
                Err(e) => eprintln!("Failed to unwrap host summary: {}: {}", e, res),
            },
            _ => {
                exit_code = 1;
                eprintln!("ERROR: execution error");
            }
        },
        Err(e) => {
            exit_code = 1;
            eprintln!("ERROR: {}", e);
        }
    }

    std::process::exit(exit_code);
}

// TODO this should be able to produce machine parseable output, and also send to Wavefront.
fn report_results(summary_total: &ApplySummary, elapsed_time: Duration, _opts: &Opts) {
    println!("Run time: {:.3?}", elapsed_time);
    println!(
        "resources: {}  changes: {}  errors: {}",
        summary_total.resources, summary_total.changes, summary_total.errors
    );
}
