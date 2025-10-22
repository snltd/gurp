use axum::response::IntoResponse;

pub async fn status() -> &'static str {
    "ok"
}

pub async fn config(host: String) -> impl IntoResponse {
    format!("compiling {}", host)
}
