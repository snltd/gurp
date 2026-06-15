use crate::types::TelemetryProviders;
use anyhow::Context;
use common::constants::SERVER_METRICS_INTERVAL;
use common::types::GlobalOpts;
use opentelemetry::KeyValue;
use opentelemetry::global;
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::{LogExporter, Protocol, WithExportConfig};
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::logs::SdkLoggerProvider;
use opentelemetry_sdk::metrics::{PeriodicReader, SdkMeterProvider};
use std::io::IsTerminal;
use std::time::Duration;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};
use util::info;

pub fn init_telemetry(label: &str, gopts: &GlobalOpts) -> anyhow::Result<TelemetryProviders> {
    let use_colour =
        std::io::stdout().is_terminal() && std::env::var_os("GURP_NO_COLOUR").is_none();

    let metrics_provider = init_metrics(gopts.metrics_to.as_deref(), label)?;
    let logger_provider = init_logs(gopts.logs_to.as_deref(), label)?;

    let subscriber = tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_ansi(use_colour))
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")));

    if let Some(ref provider) = logger_provider {
        subscriber
            .with(OpenTelemetryTracingBridge::new(provider))
            .init();
    } else {
        subscriber.init();
    }

    Ok(TelemetryProviders {
        metrics: metrics_provider,
        logging: logger_provider,
    })
}

fn init_metrics(
    endpoint: Option<&str>,
    service_name: &str,
) -> anyhow::Result<Option<SdkMeterProvider>> {
    let Some(endpoint) = endpoint else {
        return Ok(None);
    };

    let otel_endpoint = if endpoint.starts_with("http") {
        endpoint.to_owned()
    } else {
        format!("http://{endpoint}:8428/opentelemetry/v1/metrics")
    };

    let service_name = service_name.to_owned();

    // exporter is common across client and server
    let exporter = opentelemetry_otlp::MetricExporter::builder()
        .with_http()
        .with_endpoint(otel_endpoint)
        .with_protocol(Protocol::HttpBinary)
        .with_timeout(Duration::from_secs(3))
        .build()
        .context("failed to build OTEL metric exporter")?;

    let resource = Resource::builder()
        .with_service_name(service_name)
        .with_attribute(KeyValue::new("service.version", env!("CARGO_PKG_VERSION")))
        .with_attribute(KeyValue::new(
            "host.name",
            info::my_hostname().unwrap_or("unknown".to_owned()),
        ))
        .build();

    let reader = PeriodicReader::builder(exporter)
        .with_interval(SERVER_METRICS_INTERVAL)
        .build();

    let provider = SdkMeterProvider::builder()
        .with_reader(reader)
        .with_resource(resource)
        .build();

    tracing::debug!("set up metrics provider for {endpoint}");

    global::set_meter_provider(provider.clone());

    Ok(Some(provider))
}

fn init_logs(
    endpoint: Option<&str>,
    service_name: &str,
) -> anyhow::Result<Option<SdkLoggerProvider>> {
    let Some(endpoint) = endpoint else {
        return Ok(None);
    };

    let otel_endpoint = if endpoint.starts_with("http") {
        endpoint.to_owned()
    } else {
        format!("http://{endpoint}:9428/insert/opentelemetry/v1/logs")
    };

    let service_name = service_name.to_owned();

    let exporter = LogExporter::builder()
        .with_http()
        .with_endpoint(otel_endpoint)
        .with_protocol(Protocol::HttpBinary)
        .with_timeout(Duration::from_secs(3))
        .build()
        .context("failed to build OTEL log exporter")?;

    let resource = Resource::builder()
        .with_service_name(service_name)
        .with_attribute(KeyValue::new("service.version", env!("CARGO_PKG_VERSION")))
        .with_attribute(KeyValue::new(
            "host.name",
            info::my_hostname().unwrap_or("unknown".to_owned()),
        ))
        .build();

    let provider = SdkLoggerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(resource)
        .build();

    Ok(Some(provider))
}

pub fn init_stdout_logging() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_ansi(use_colour())
        .init();
}

fn use_colour() -> bool {
    std::io::stdout().is_terminal() && std::env::var_os("GURP_NO_COLOUR").is_none()
}
