use crate::info;
use anyhow::Context;
use common::constants::SERVER_METRICS_INTERVAL;
use opentelemetry::KeyValue;
use opentelemetry::global;
use opentelemetry_otlp::{Protocol, WithExportConfig};
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::metrics::{PeriodicReader, SdkMeterProvider};
use std::time::Duration;

pub fn init_metrics(
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
