use crate::names;
use opentelemetry::KeyValue;
use opentelemetry::global;
use opentelemetry::metrics::Gauge;

pub struct ClientMetrics {
    apply_duration: Gauge<u64>,
    apply_changes: Gauge<u64>,
    apply_resources: Gauge<u64>,
    apply_rss: Gauge<u64>,
}

#[allow(clippy::new_without_default)]
impl ClientMetrics {
    pub fn new() -> Self {
        let meter = global::meter("gurp-client");
        Self {
            apply_duration: meter
                .u64_gauge(names::APPLY_DURATION_MS)
                .with_description("apply duration in milliseconds")
                .build(),
            apply_resources: meter
                .u64_gauge(names::APPLY_RESOURCES)
                .with_description("number of resources examined in apply")
                .build(),

            apply_changes: meter
                .u64_gauge(names::APPLY_CHANGES)
                .with_description("number of resources changed in apply")
                .build(),

            apply_rss: meter
                .u64_gauge(names::APPLY_RSS_BYTES)
                .with_description("apply RSS in bytes")
                .with_unit("bytes")
                .build(),
        }
    }

    pub fn record_apply_duration(&self, result: &str, elapsed_ms: u64, phase: Option<&str>) {
        let mut tags = vec![KeyValue::new("status", result.to_owned())];

        if let Some(phase) = phase {
            tags.push(KeyValue::new("phase", phase.to_owned()));
        }

        self.apply_duration.record(elapsed_ms, &tags);
    }

    pub fn record_apply_resources(&self, n: u64) {
        self.apply_resources.record(n, &[]);
    }

    pub fn record_apply_changes(&self, n: u64) {
        self.apply_changes.record(n, &[]);
    }

    pub fn record_apply_rss(&self, state: &str, rss: u64) {
        self.apply_rss
            .record(rss, &[KeyValue::new("status", state.to_owned())]);
    }
}
