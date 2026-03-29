use crate::metrics::names;
use crate::runtime_stats;
use opentelemetry::global;
use opentelemetry::metrics::{Counter, Histogram, ObservableGauge};

#[derive(Clone)]
pub struct ServerMetrics {
    pub rss: ObservableGauge<u64>,
    pub http_requests_total: Counter<u64>,
    pub http_request_duration: Histogram<f64>,
}

#[allow(clippy::new_without_default)]
impl ServerMetrics {
    pub fn new() -> Self {
        let meter = global::meter("gurp-server");
        Self {
            http_requests_total: meter
                .u64_counter(names::SERVER_REQUESTS_TOTAL)
                .with_description("Number of server requests handled")
                .build(),
            http_request_duration: meter
                .f64_histogram(names::SERVER_REQUEST_DURATION_MS)
                .with_boundaries(vec![
                    1.0, 2.0, 5.0, 10.0, 15.0, 20.0, 25.0, 30.0, 40.0, 50.0, 75.0, 100.0,
                ])
                .with_description("Server request duration in milliseconds")
                .with_unit("ms")
                .build(),
            rss: meter
                .u64_observable_gauge(names::SERVER_RSS_BYTES)
                .with_description("RSS of current process")
                .with_unit("bytes")
                .with_callback(|o| {
                    if let Some(rss) = runtime_stats::rss_bytes() {
                        o.observe(rss as u64, &[]);
                    }
                })
                .build(),
        }
    }
}
