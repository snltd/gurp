use opentelemetry_sdk::logs::SdkLoggerProvider;
use opentelemetry_sdk::metrics::SdkMeterProvider;

#[derive(Default)]
pub struct TelemetryProviders {
    pub metrics: Option<SdkMeterProvider>,
    pub logging: Option<SdkLoggerProvider>,
}
