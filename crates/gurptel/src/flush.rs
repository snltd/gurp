use crate::types::TelemetryProviders;

pub fn flush(providers: TelemetryProviders) {
    if let Some(p) = providers.metrics {
        if let Err(e) = p.force_flush() {
            tracing::warn!("failed to flush metrics: {e:#}");
        }

        if let Err(e) = p.shutdown() {
            tracing::warn!("failed to shut down OTEL provider: {e:#}");
        }
    } else {
        tracing::debug!("no metrics provider, so not sending metrics");
    }

    if let Some(p) = providers.logging {
        if let Err(e) = p.force_flush() {
            tracing::warn!("failed to flush logs: {e:#}");
        }

        if let Err(e) = p.shutdown() {
            tracing::warn!("failed to shut down log provider: {e:#}");
        }
    }
}
