use crate::handlers;
use axum::Router;
use axum::extract::Extension;
use axum::routing::get;
use common::constants::SERVER_PORT;
use common::types::ServerOpts;
use std::net::SocketAddr;
use std::sync::Arc;

pub async fn start(opts: ServerOpts) -> anyhow::Result<()> {
    let conf_dir = opts.config_dir.clone();
    let server_opts = Arc::new(opts);

    let app = Router::new()
        .route("/status", get(handlers::status))
        .route("/file/{*path}", get(handlers::file))
        .route("/config/{host}", get(handlers::config))
        .layer(Extension(server_opts));

    let addr = SocketAddr::from(([0, 0, 0, 0], SERVER_PORT));
    tracing::info!("Listening on {addr}");
    tracing::info!("Config dir is {conf_dir}");

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
        .expect("failed to install ctrl-c handler");
    eprintln!("Shutting down");
}
