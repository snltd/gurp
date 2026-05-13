use crate::apply::types::ApplyStatus;
use std::time::Duration;
use util::metrics::client::ClientMetrics;
use util::runtime_stats;

pub(crate) fn send(status: ApplyStatus, elapsed_time: &Duration) {
    let metrics_handle = ClientMetrics::new();
    let elapsed_ms = elapsed_time.as_millis();

    match status {
        ApplyStatus::Ok(summary) => {
            metrics_handle.record_apply_duration("ok", elapsed_ms as u64, None);
            metrics_handle.record_apply_changes(summary.changes as u64);
            metrics_handle.record_apply_resources(summary.resources as u64);
            #[cfg(test)]
            tracing::info!(
                "sending success metrics: {}/{}",
                summary.changes,
                summary.resources
            );
        }
        ApplyStatus::Fail(phase) => {
            metrics_handle.record_apply_duration(
                "fail",
                elapsed_ms as u64,
                Some(&phase.to_string()),
            );
            #[cfg(test)]
            tracing::info!("sending fail metrics: {phase}",);
        }
    }

    if let Some(rss) = runtime_stats::rss_bytes() {
        metrics_handle.record_apply_rss("ok", rss as u64);
    }
}
