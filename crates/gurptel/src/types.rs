use opentelemetry_sdk::logs::SdkLoggerProvider;
use opentelemetry_sdk::metrics::SdkMeterProvider;

#[derive(Default)]
pub struct TelemetryProviders {
    pub metrics: Option<SdkMeterProvider>,
    pub logging: Option<SdkLoggerProvider>,
}

impl TelemetryProviders {
    pub fn are_active(&self) -> bool {
        self.metrics.is_some() || self.logging.is_some()
    }
}
