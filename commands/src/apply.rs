use camino::Utf8PathBuf;
use common::types::{ApplyOpts, ApplySummary, ExitCode};
use doers::host;
use std::time::{Duration, Instant};

pub fn run(host_config_file: &Utf8PathBuf, opts: &ApplyOpts) -> ExitCode {
    let start_time = Instant::now();
    let apply_summary = match host::apply(host_config_file, opts) {
        Ok(result) => result,
        Err(e) => {
            tracing::error!("apply error on {}: {}", host_config_file, e);
            return 1;
        }
    };

    let elapsed_time = start_time.elapsed();
    report_results(&apply_summary, elapsed_time)
}

// TODO this should be able to produce machine parseable output, and also send to Wavefront.
fn report_results(summary_total: &ApplySummary, elapsed_time: Duration) -> ExitCode {
    tracing::info!("Run time: {:.3?}", elapsed_time);
    tracing::info!(
        "resources: {}  changes: {}  errors: {}",
        summary_total.resources,
        summary_total.changes,
        summary_total.errors
    );

    if summary_total.errors > 0 { 1 } else { 0 }
}
