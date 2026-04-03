use crate::server::handlers;
use axum::Router;
use axum::extract::{Extension, Request, State};
use axum::middleware::Next;
use axum::response::Response;
use axum::routing::get;
use common::constants::{GURP_VERSION, SERVER_PORT};
use common::types::ServerOpts;
use opentelemetry::KeyValue;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use util::info;
use util::metrics::server::ServerMetrics;

pub async fn start(opts: ServerOpts) -> anyhow::Result<()> {
    let metrics = ServerMetrics::new();

    let conf_dir = opts.config_dir.clone();
    let server_opts = Arc::new(opts);

    let app = Router::new()
        .route("/v1/version", get(handlers::version))
        .route("/v1/status", get(handlers::status))
        .route("/v1/file/{*path}", get(handlers::file))
        .route("/v1/file-hash/{*path}", get(handlers::file_hash))
        .route(
            "/v1/file-hash-filtered/{*path}",
            get(handlers::file_hash_filtered),
        )
        .route("/v1/config/{host}", get(handlers::config))
        .route_layer(axum::middleware::from_fn_with_state(
            metrics.clone(),
            metrics_middleware,
        ))
        .with_state(metrics)
        .layer(Extension(server_opts));

    let addr = SocketAddr::from(([0, 0, 0, 0], SERVER_PORT));

    tracing::info!(
        "Gurp version {GURP_VERSION} [{}] listening on {addr}",
        info::build_hash()
    );

    tracing::info!("Config dir is {conf_dir}");

    axum::serve(
        tokio::net::TcpListener::bind(addr).await.unwrap(),
        app.into_make_service(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await?;

    Ok(())
}

pub async fn metrics_middleware(
    State(metrics): State<ServerMetrics>,
    req: Request,
    next: Next,
) -> Response {
    let start = Instant::now();
    let path = req.uri().path().to_owned();
    let method = req.method().to_string();
    let response = next.run(req).await;

    metrics.http_requests_total.add(
        1,
        &[
            KeyValue::new("method", method.clone()),
            KeyValue::new("path", path.clone()),
            KeyValue::new("status", response.status().as_u16().to_string()),
        ],
    );

    metrics.http_request_duration.record(
        start.elapsed().as_secs_f64() * 1000.0,
        &[KeyValue::new("method", method), KeyValue::new("path", path)],
    );

    response
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install ctrl-c handler");
    eprintln!("Shutting down");
}
