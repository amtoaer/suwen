#[macro_use]
extern crate tracing;

use std::sync::LazyLock;

use axum::Extension;
use reqwest::{StatusCode, header};
use sea_orm::DatabaseConnection;
use suwen_markdown::UPLOAD_DIR;
use tower::ServiceExt;

use axum::extract::{Path as AxumPath, Query};
use axum::{Router, extract::Request, response::IntoResponse, routing::get};
use axum_reverse_proxy::ReverseProxy;
use tower_http::{compression::CompressionLayer, services::ServeFile};

use crate::routes::UrlQuery;

mod auth;
pub mod db;
mod routes;
mod rss;
mod wrapper;

static FRONTEND_PORT: LazyLock<String> =
    LazyLock::new(|| std::env::var("FRONTEND_PORT").unwrap_or_else(|_| "4173".to_string()));

pub fn router() -> Router {
    Router::new()
        .nest("/api", routes::router())
        .route("/uploads/{file}", get(uploads_handler))
        .route("/feed", get(rss_handler))
        .merge(ReverseProxy::new(
            "/",
            &format!("http://localhost:{}", FRONTEND_PORT.as_str()),
        ))
        .layer(CompressionLayer::new().gzip(true).br(true).zstd(true))
}

async fn uploads_handler(AxumPath(path): AxumPath<String>, request: Request) -> impl IntoResponse {
    ServeFile::new(UPLOAD_DIR.join(path)).oneshot(request).await
}

async fn rss_handler(
    Query(query): Query<UrlQuery>,
    Extension(conn): Extension<DatabaseConnection>,
) -> impl IntoResponse {
    let Ok((site, articles)) = tokio::try_join!(
        db::get_site(&conn),
        db::get_rss_articles(&conn, query.lang.unwrap_or(db::Lang::ZhCN), 25),
    ) else {
        return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch data").into_response();
    };
    let Some(site) = site else {
        return (StatusCode::INTERNAL_SERVER_ERROR, "Site not initialized").into_response();
    };
    let rss = rss::generate_rss(site, articles);
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/xml")],
        rss,
    )
        .into_response()
}
