use axum::extract::{Extension, Path};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use common::types::{ApplyOpts, ServerOpts};
use janet_int::helpers;
use std::sync::Arc;

pub async fn status() -> &'static str {
    tracing::debug!("received status request");
    "ok"
}

pub async fn config(
    Path(host): Path<String>,
    Extension(opts): Extension<Arc<ServerOpts>>,
) -> impl IntoResponse {
    tracing::info!("request for {}", host);

    let host_filename = format!("{host}.janet");
    let host_file = opts.config_dir.join(&host_filename);

    if host_file.exists() {
        match helpers::compile_config(&host_file, &ApplyOpts::default()) {
            Ok(body) => (StatusCode::OK, body).into_response(),
            Err(e) => {
                tracing::error!("{e}");
                (StatusCode::INTERNAL_SERVER_ERROR, format!("error: {e}")).into_response()
            }
        }
    } else {
        tracing::error!("No config at {host_file}");
        (StatusCode::NOT_FOUND, "host config file not found").into_response()
    }
}
