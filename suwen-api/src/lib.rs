use std::sync::LazyLock;

use axum::body::Body;
use reqwest::header;
use suwen_markdown::UPLOAD_DIR;
use tower::ServiceExt;

use axum::extract::{Path as AxumPath, Query};
use axum::{Router, extract::Request, response::IntoResponse, routing::get};
use axum_reverse_proxy::ReverseProxy;
use tower_http::{compression::CompressionLayer, services::ServeFile};

pub mod db;
mod routes;
mod wrapper;

static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .user_agent(
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:141.0) Gecko/20100101 Firefox/141.0",
        )
        .build()
        .unwrap()
});

static FRONTEND_PORT: LazyLock<String> =
    LazyLock::new(|| std::env::var("FRONTEND_PORT").unwrap_or_else(|_| "4173".to_string()));

pub fn router() -> Router {
    Router::new()
        .nest("/api", routes::router())
        .route("/proxy", get(proxy_handler))
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

async fn proxy_handler(Query(query): Query<serde_json::Value>) -> impl IntoResponse {
    let url = match query.get("url").and_then(|v| v.as_str()) {
        Some(url) => url,
        None => return axum::http::StatusCode::BAD_REQUEST.into_response(),
    };
    let response = match CLIENT.get(url).send().await {
        Ok(resp) => resp,
        Err(_) => return axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };
    let mut headers = header::HeaderMap::new();
    for (key, value) in response.headers() {
        if key != header::CONTENT_DISPOSITION {
            headers.insert(key, value.clone());
        }
    }
    (headers, Body::from_stream(response.bytes_stream())).into_response()
}
