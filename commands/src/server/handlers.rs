use axum::Json;
use axum::body::Body;
use axum::extract::{Extension, Path, Query};
use axum::http::{Response, StatusCode};
use axum::response::IntoResponse;
use common::constants::GURP_VERSION;
use common::types::{ApplyOpts, ServerOpts};
use embed::compiler;
use mime_guess::from_path;
use std::sync::Arc;
use tokio::fs::File;
use tokio_util::io::ReaderStream;

#[derive(serde::Deserialize)]
pub struct ConfigQuery {
    server_name: String,
    format: String,
}

pub async fn status() -> &'static str {
    tracing::debug!("received status request");
    "ok"
}

pub async fn version() -> &'static str {
    tracing::debug!("received version request");
    GURP_VERSION
}

pub async fn config(
    Path(remote_host_name): Path<String>,
    Query(params): Query<ConfigQuery>,
    Extension(opts): Extension<Arc<ServerOpts>>,
) -> impl IntoResponse {
    tracing::info!("request for {remote_host_name} config");

    let host_filename = format!("{remote_host_name}.janet");
    let host_file = opts.config_dir.join(&host_filename);

    if host_file.exists() {
        let opts = ApplyOpts {
            server_name: Some(params.server_name),
            client_name: Some(remote_host_name.clone()),
            ..Default::default()
        };

        tracing::info!(
            "received request for {} as {}",
            remote_host_name,
            params.format
        );

        match params.format.as_str() {
            "jimage" => match compiler::local_janet_to_jimage(&host_file, &opts) {
                Ok(body) => {
                    // jimage is a vec<u8> so it's automatically application/octet-stream
                    let bytes = body.len();
                    let ret = (StatusCode::OK, body).into_response();
                    tracing::info!("sent {bytes}b image config generated from {host_file}");
                    ret
                }
                Err(e) => {
                    tracing::error!(
                        remote_host = remote_host_name.to_string(),
                        message = e.to_string()
                    );
                    (StatusCode::INTERNAL_SERVER_ERROR, format!("error: {e}")).into_response()
                }
            },
            "json" => match compiler::local_janet_to_json(&host_file, &opts) {
                Ok(body) => {
                    let ret = (StatusCode::OK, Json(body)).into_response();
                    tracing::info!("sent JSON config from {host_file}");
                    ret
                }
                Err(e) => {
                    tracing::error!(
                        remote_host = remote_host_name.to_string(),
                        message = e.to_string()
                    );
                    (StatusCode::INTERNAL_SERVER_ERROR, format!("error: {e}")).into_response()
                }
            },
            other => {
                tracing::error!("unsupported format: {other}");
                (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    format!("unsupported format: {other}"),
                )
                    .into_response()
            }
        }
    } else {
        tracing::warn!("No config at {host_file}");
        (StatusCode::NOT_FOUND, "host config file not found").into_response()
    }
}

pub async fn file(
    Path(path): Path<String>,
    Extension(opts): Extension<Arc<ServerOpts>>,
) -> impl IntoResponse {
    tracing::info!("request for file {}", path);

    let path = opts.config_dir.join("files").join(&path);

    let fh = match File::open(&path).await {
        Ok(fh) => fh,
        Err(e) => {
            tracing::warn!("could not read {path}: {e}");
            return (StatusCode::NOT_FOUND).into_response();
        }
    };

    let stream = ReaderStream::new(fh);
    let mime = from_path(&path).first_or_octet_stream();

    match Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", mime.as_ref())
        .body(Body::from_stream(stream))
    {
        Ok(resp) => {
            let ret = resp.into_response();
            tracing::info!("sent file {path}");
            ret
        }
        Err(e) => {
            tracing::warn!("failed to build response for {path}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to build response: {e}"),
            )
                .into_response()
        }
    }
}
