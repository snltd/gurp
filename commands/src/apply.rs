use anyhow::Context;
use camino::Utf8PathBuf;
use common::types::{ApplyOpts, ApplySummary, ExitCode};
use doers::host;
use nix::unistd;
use std::time::{Duration, Instant};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn run(host_config_file: &Utf8PathBuf, metrics_to: Option<&str>, opts: &ApplyOpts) -> ExitCode {
    let start_time = Instant::now();
    let apply_summary = match host::apply(host_config_file, opts) {
        Ok(result) => result,
        Err(e) => {
            tracing::error!("apply error on {}: {}", host_config_file, e);
            return 1;
        }
    };

    let elapsed_time = start_time.elapsed();
    report_results(&apply_summary, elapsed_time, metrics_to)
}

fn send_metrics(
    summary: &ApplySummary,
    elapsed_time: Duration,
    metrics_host: &str,
) -> anyhow::Result<()> {
    let url = format!("http://{metrics_host}:8428/write");

    tracing::debug!("Sending metrics to {}", url);

    let ns_timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    let hostname = unistd::gethostname()
        .context("Failed getting hostname")?
        .to_string_lossy()
        .into_owned();

    // myMeasurement,tag1=val1,tag2=val2 field1="v1",field2=1i 0000000000000000000

    let payload = format!(
        "gurp.summary,host={},resources={},changes={},errors={},ms_time={} {}",
        hostname,
        summary.resources,
        summary.changes,
        summary.errors,
        elapsed_time.as_millis(),
        ns_timestamp,
    );

    let resp = ureq::post(url).content_type("text/plain").send(payload)?;

    if resp.status().is_success() {
        tracing::debug!("Metrics pushed successfully");
    } else {
        tracing::warn!("Failed to push metrics: {}", resp.status());
    }

    Ok(())
}

fn report_results(
    summary_total: &ApplySummary,
    elapsed_time: Duration,
    metrics_to: Option<&str>,
) -> ExitCode {
    tracing::info!("Run time: {:.3?}", elapsed_time);
    tracing::info!(
        "resources: {}  changes: {}  errors: {}",
        summary_total.resources,
        summary_total.changes,
        summary_total.errors
    );

    if let Some(metrics_host) = metrics_to {
        match send_metrics(summary_total, elapsed_time, metrics_host) {
            Ok(_) => (),
            Err(e) => tracing::error!("error sending metrics: {}", e),
        }
    }

    if summary_total.errors > 0 { 1 } else { 0 }
}
