#[macro_use]
extern crate tracing;

use std::sync::LazyLock;

use axum::Extension;
use reqwest::{StatusCode, header};
use sea_orm::DatabaseConnection;
use suwen_config::CONFIG;
use suwen_markdown::UPLOAD_DIR;
use tower::ServiceExt;

use axum::extract::{Path as AxumPath, Query};
use axum::{Router, extract::Request, response::IntoResponse, routing::get};
use axum_reverse_proxy::ReverseProxy;
use tower_http::services::ServeFile;

use crate::routes::UrlQuery;

mod auth;
pub mod db;
mod routes;
mod rss;
mod sitemap;
mod wrapper;

static FRONTEND_ORIGIN: LazyLock<String> = LazyLock::new(|| {
    std::env::var("FRONTEND_ORIGIN").unwrap_or_else(|_| "http://localhost:5545".to_string())
});

pub fn router() -> Router {
    Router::new()
        .nest("/api", routes::router())
        .route("/uploads/{file}", get(uploads_handler))
        .route("/feed", get(rss_handler))
        .route("/sitemap.xml", get(sitemap_handler))
        .merge(ReverseProxy::new("/", FRONTEND_ORIGIN.as_str()))
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

async fn sitemap_handler(
    Query(query): Query<UrlQuery>,
    Extension(conn): Extension<DatabaseConnection>,
) -> impl IntoResponse {
    let base_url = CONFIG
        .host_url
        .clone()
        .unwrap_or_else(|| "https://amto.cc".to_owned());
    let lang = query.lang.unwrap_or(db::Lang::ZhCN);
    let Ok(articles) = db::get_sitemap_articles(&conn, lang).await else {
        return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch data").into_response();
    };
    let sitemap = sitemap::generate_sitemap(&base_url, articles);
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/xml")],
        sitemap,
    )
        .into_response()
}
