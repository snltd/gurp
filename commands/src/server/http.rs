use crate::server::handlers;
use axum::Router;
use axum::extract::{Extension, Request};
use axum::middleware::Next;
use axum::response::Response;
use axum::routing::get;
use common::constants::{GURP_VERSION, SERVER_PORT};
use common::types::ServerOpts;
use opentelemetry::{KeyValue, global};
use opentelemetry_otlp::{MetricExporter, WithExportConfig};
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::metrics::{PeriodicReader, SdkMeterProvider};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

pub async fn start(opts: ServerOpts) -> anyhow::Result<()> {
    let victoriametrics_url = opts
        .metrics_to
        .clone()
        .unwrap_or("http://localhost:8428".to_string());

    let _meter_provider = init_metrics(&victoriametrics_url)?;
    tracing::info!("{victoriametrics_url}");

    let conf_dir = opts.config_dir.clone();
    let server_opts = Arc::new(opts);

    let app = Router::new()
        .route("/version", get(handlers::version))
        .route("/status", get(handlers::status))
        .route("/file/{*path}", get(handlers::file))
        .route("/config/{host}", get(handlers::config))
        .layer(axum::middleware::from_fn(metrics_middleware))
        .layer(Extension(server_opts));

    let addr = SocketAddr::from(([0, 0, 0, 0], SERVER_PORT));
    tracing::info!("Gurp version {GURP_VERSION} listening on {addr}");
    tracing::info!("Config dir is {conf_dir}");

    axum::serve(
        tokio::net::TcpListener::bind(addr).await.unwrap(),
        app.into_make_service(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await?;

    Ok(())
}

pub async fn metrics_middleware(req: Request, next: Next) -> Response {
    let start = Instant::now();
    let path = req.uri().path().to_string();
    let method = req.method().to_string();
    let response = next.run(req).await;
    let duration = start.elapsed().as_secs_f64();
    let status = response.status().as_u16();
    let status_class = format!("{}xx", status / 100);
    let meter = global::meter("http_server");

    let request_counter = meter
        .u64_counter("http_requests_total")
        .with_description("Total HTTP requests")
        .build();

    request_counter.add(
        1,
        &[
            KeyValue::new("method", method.clone()),
            KeyValue::new("path", path.clone()),
            KeyValue::new("status", status_class),
        ],
    );

    let duration_histogram = meter
        .f64_histogram("http_request_duration_seconds")
        .with_description("HTTP request duration in seconds")
        .build();

    duration_histogram.record(
        duration,
        &[KeyValue::new("method", method), KeyValue::new("path", path)],
    );

    response
}

pub fn init_metrics(victoriametrics_url: &str) -> anyhow::Result<SdkMeterProvider> {
    let export_config = opentelemetry_otlp::ExportConfig {
        endpoint: Some(format!("{}/opentelemetry/api/v1/push", victoriametrics_url)),
        timeout: Some(Duration::from_secs(10)),
        ..Default::default()
    };

    let exporter = MetricExporter::builder()
        .with_http()
        .with_export_config(export_config)
        .build()?;

    let reader = PeriodicReader::builder(exporter)
        .with_interval(Duration::from_secs(10))
        .build();

    let provider = SdkMeterProvider::builder()
        .with_reader(reader)
        .with_resource(
            Resource::builder()
                .with_service_name("gurp-server")
                .with_attribute(KeyValue::new("service.version", env!("CARGO_PKG_VERSION")))
                .build(),
        )
        .build();

    global::set_meter_provider(provider.clone());

    Ok(provider)
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install ctrl-c handler");
    eprintln!("Shutting down");
}
