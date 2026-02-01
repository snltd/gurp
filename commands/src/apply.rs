use camino::Utf8PathBuf;
use common::types::{ApplyOpts, ApplySummary};
use doers::types::Applicator;
use embed::compiler;
use std::process::ExitCode;
use std::time::{Duration, Instant};
use util::metrics;

pub fn run(host_file: Option<&Utf8PathBuf>, opts: &ApplyOpts) -> ExitCode {
    let start_time = Instant::now();

    let json_config = match compiler::compile_to_json(host_file, opts) {
        Ok(config) => config,
        Err(e) => {
            tracing::error!("error compiling config: {e}");
            return ExitCode::FAILURE;
        }
    };

    let applicator = Applicator::from(json_config);

    match applicator.run(opts) {
        Ok(apply_summary) => {
            let elapsed_time = start_time.elapsed();
            report_success(&apply_summary, elapsed_time, opts.metrics_to.as_deref());
            ExitCode::SUCCESS
        }
        Err(e) => {
            if let Some(host_file) = host_file {
                tracing::error!("apply error on {host_file}: {e}");
            } else {
                tracing::error!("apply error: {e}");
            }

            let elapsed_time = start_time.elapsed();
            report_failure(elapsed_time, opts.metrics_to.as_deref());
            ExitCode::FAILURE
        }
    }
}

fn report_success(summary_total: &ApplySummary, elapsed_time: Duration, metrics_to: Option<&str>) {
    tracing::info!("Run time: {:.3?}", elapsed_time);
    tracing::info!(
        "resources: {}  changes: {}",
        summary_total.resources,
        summary_total.changes,
    );

    if let Some(metrics_host) = metrics_to {
        match metrics::send_as_influx(Some(summary_total), elapsed_time, metrics_host) {
            Ok(_) => (),
            Err(e) => tracing::error!("error sending metrics: {}", e),
        }
    }
}

fn report_failure(elapsed_time: Duration, metrics_to: Option<&str>) {
    if let Some(metrics_host) = metrics_to {
        match metrics::send_as_influx(None, elapsed_time, metrics_host) {
            Ok(_) => (),
            Err(e) => tracing::error!("error sending metrics: {}", e),
        }
    }
}
