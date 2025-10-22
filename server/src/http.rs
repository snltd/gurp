use crate::handlers;
use axum::Router;
use axum::routing::get;
use common::constants::SERVER_PORT;
use std::net::SocketAddr;

pub async fn start() -> anyhow::Result<()> {
    let app = Router::new()
        .route("/status", get(handlers::status))
        .route("/config/{host}", get(handlers::config));

    let addr = SocketAddr::from(([0, 0, 0, 0], SERVER_PORT));
    tracing::info!("Listening on {}", addr);

    axum::serve(
        tokio::net::TcpListener::bind(addr).await.unwrap(),
        app.into_make_service(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await?;

    Ok(())
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install Ctrl+C handler");
    eprintln!("Shutting down");
}
