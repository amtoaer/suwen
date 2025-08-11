#[macro_use]
extern crate tracing;

use std::sync::LazyLock;

use suwen_markdown::UPLOAD_DIR;
use tower::ServiceExt;

use axum::extract::Path as AxumPath;
use axum::{Router, extract::Request, response::IntoResponse, routing::get};
use axum_reverse_proxy::ReverseProxy;
use tower_http::{compression::CompressionLayer, services::ServeFile};

pub mod db;
mod routes;
mod wrapper;

static FRONTEND_PORT: LazyLock<String> =
    LazyLock::new(|| std::env::var("FRONTEND_PORT").unwrap_or_else(|_| "4173".to_string()));

pub fn router() -> Router {
    Router::new()
        .nest("/api", routes::router())
        .route("/uploads/{file}", get(uploads_handler))
        .merge(ReverseProxy::new(
            "/",
            &format!("http://localhost:{}", FRONTEND_PORT.as_str()),
        ))
        .layer(CompressionLayer::new().gzip(true).br(true).zstd(true))
}

async fn uploads_handler(AxumPath(path): AxumPath<String>, request: Request) -> impl IntoResponse {
    ServeFile::new(UPLOAD_DIR.join(path)).oneshot(request).await
}
