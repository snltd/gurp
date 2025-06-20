use crate::common::types::{ApplySummary, ExitCode, Opts};
use crate::doers::host;
use crate::error;
use crate::utils::janet_helpers;
use camino::Utf8PathBuf;
use colored::Colorize;
use std::time::{Duration, Instant};

pub fn run(
    host_config_file: &Utf8PathBuf,
    gurp_lib_path: &Option<Utf8PathBuf>,
    global_opts: &Opts,
) -> ExitCode {
    let start_time = Instant::now();
    let apply_result = host::apply(host_config_file, gurp_lib_path, global_opts);
    let elapsed_time = start_time.elapsed();

    // Without this we can't unwrap the summary
    unsafe {
        janetrs::lowlevel::janet_init();
    }

    match apply_result {
        Ok(res) => match res.unwrap() {
            janetrs::TaggedJanet::Struct(_) => match janet_helpers::unwrap_summary(&res) {
                Ok(summary) => report_results(&summary, elapsed_time, global_opts),
                Err(e) => {
                    error!(
                        opts,
                        "apply/run", "Failed to unwrap host summary: {}: {}", e, res
                    );
                    1
                }
            },
            _ => {
                error!(opts, "apply/run", "Janet execution error");
                1
            }
        },
        Err(e) => {
            error!(opts, "apply/run", "{}", e);
            1
        }
    }
}

// TODO this should be able to produce machine parseable output, and also send to Wavefront.
fn report_results(summary_total: &ApplySummary, elapsed_time: Duration, _opts: &Opts) -> ExitCode {
    println!("Run time: {:.3?}", elapsed_time);
    println!(
        "resources: {}  changes: {}  errors: {}",
        summary_total.resources, summary_total.changes, summary_total.errors
    );

    if summary_total.errors > 0 { 1 } else { 0 }
}
