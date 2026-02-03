use common::types::ApplySummary;
use std::time::Duration;
use util::metrics;

pub fn success(summary_total: &ApplySummary, elapsed_time: Duration, metrics_to: Option<&str>) {
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

pub fn failure(elapsed_time: Duration, metrics_to: Option<&str>) {
    if let Some(metrics_host) = metrics_to {
        match metrics::send_as_influx(None, elapsed_time, metrics_host) {
            Ok(_) => (),
            Err(e) => tracing::error!("error sending metrics: {}", e),
        }
    }
}
