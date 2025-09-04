use anyhow::Context;
use common::constants::GURP_VERSION;
use common::types::ApplySummary;
use nix::unistd;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// If there's a summary, we're reporting success. If not, we're reporting an error.:
pub fn send_as_influx(
    summary: Option<&ApplySummary>,
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

    let common_tags = format!("host={},gurp-version={}", hostname, GURP_VERSION);

    let points = if let Some(summary) = summary {
        vec![
            format!(
                "gurp,{} ms_time={} {}",
                common_tags,
                elapsed_time.as_millis(),
                ns_timestamp
            ),
            format!(
                "gurp,{} resources={} {}",
                common_tags, summary.resources, ns_timestamp
            ),
            format!(
                "gurp,{} changes={} {}",
                common_tags, summary.changes, ns_timestamp
            ),
        ]
    } else {
        vec![format!(
            "gurp_error,{} ms_time={} {}",
            common_tags,
            elapsed_time.as_millis(),
            ns_timestamp
        )]
    };

    let payload = points.join("\n");

    tracing::debug!("metrics payload: {}", payload);

    let resp = ureq::post(url).content_type("text/plain").send(payload)?;

    if resp.status().is_success() {
        tracing::debug!("Metrics pushed successfully");
    } else {
        tracing::warn!("Failed to push metrics: {}", resp.status());
    }

    Ok(())
}
