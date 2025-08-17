use anyhow::Context;
use axum::{
    Extension,
    extract::{Path, Query},
    routing::get,
};
use sea_orm::DatabaseConnection;
use suwen_entity::content_metadata;

use crate::{
    db::{self, Archive},
    wrapper::{ApiError, ApiResponse},
};

pub(super) struct UrlQuery {
    pub lang: Option<db::Lang>,
    pub sort: Option<content_metadata::Column>,
    pub published: Option<bool>,
    pub limit: Option<u64>,
}

impl<'de> serde::Deserialize<'de> for UrlQuery {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut lang = None;
        let mut sort = None;
        let mut published = None;
        let mut limit = None;

        let map: serde_json::Map<String, serde_json::Value> =
            serde_json::Map::deserialize(deserializer)?;

        if let Some(value) = map.get("lang") {
            lang = value.as_str().and_then(|s| db::Lang::try_from(s).ok());
        }
        if let Some(value) = map.get("sort") {
            match value.as_str() {
                Some("trending") => sort = Some(content_metadata::Column::ViewCount),
                Some("top-comments") => sort = Some(content_metadata::Column::CommentCount),
                _ => {}
            }
        }
        if let Some(value) = map.get("published") {
            published = value.as_bool();
        }
        if let Some(value) = map.get("limit") {
            limit = value.as_u64();
        }
        Ok(Self {
            lang,
            sort,
            published,
            limit,
        })
    }
}

async fn get_site(
    Extension(conn): Extension<DatabaseConnection>,
) -> Result<ApiResponse<db::Site>, ApiError> {
    Ok(ApiResponse::ok(
        db::get_site(&conn).await?.context("Site not found")?,
    ))
}

async fn get_articles(
    Extension(conn): Extension<DatabaseConnection>,
    Query(query): Query<UrlQuery>,
) -> Result<ApiResponse<Vec<db::ArticleByList>>, ApiError> {
    Ok(ApiResponse::ok(
        db::get_articles(
            &conn,
            query.lang.unwrap_or(db::Lang::ZhCN),
            query.sort.unwrap_or(content_metadata::Column::PublishedAt),
            query.published,
            query.limit.unwrap_or(100),
        )
        .await?,
    ))
}

async fn get_shorts(
    Extension(conn): Extension<DatabaseConnection>,
    Query(query): Query<UrlQuery>,
) -> Result<ApiResponse<Vec<db::Short>>, ApiError> {
    Ok(ApiResponse::ok(
        db::get_shorts(
            &conn,
            query.lang.unwrap_or(db::Lang::ZhCN),
            query.sort.unwrap_or(content_metadata::Column::PublishedAt),
            query.published,
            query.limit.unwrap_or(100) as u32,
        )
        .await?,
    ))
}

async fn get_short_by_slug(
    Extension(conn): Extension<DatabaseConnection>,
    Query(query): Query<UrlQuery>,
    Path((slug,)): Path<(String,)>,
) -> Result<ApiResponse<db::Short>, ApiError> {
    Ok(ApiResponse::ok(
        db::get_short_by_slug(&conn, &slug, query.lang.unwrap_or(db::Lang::ZhCN))
            .await?
            .ok_or_else(|| ApiError::not_found("Short not found"))?,
    ))
}

async fn get_article_by_slug(
    Extension(conn): Extension<DatabaseConnection>,
    Query(query): Query<UrlQuery>,
    Path((slug,)): Path<(String,)>,
) -> Result<ApiResponse<db::ArticleBySlug>, ApiError> {
    Ok(ApiResponse::ok(
        db::get_article_by_slug(&conn, &slug, query.lang.unwrap_or(db::Lang::ZhCN))
            .await?
            .ok_or_else(|| ApiError::not_found("Article not found"))?,
    ))
}

async fn get_tags_with_count(
    Extension(conn): Extension<DatabaseConnection>,
) -> Result<ApiResponse<Vec<db::TagWithCount>>, ApiError> {
    Ok(ApiResponse::ok(db::get_tags_with_count(&conn).await?))
}

async fn get_archives_group_by_year(
    Extension(conn): Extension<DatabaseConnection>,
    Query(query): Query<UrlQuery>,
) -> Result<ApiResponse<Vec<(i32, Vec<Archive>)>>, ApiError> {
    Ok(ApiResponse::ok(
        db::get_archives_grouped_by_year(&conn, query.lang.unwrap_or(db::Lang::ZhCN)).await?,
    ))
}

async fn get_articles_by_tag(
    Extension(conn): Extension<DatabaseConnection>,
    Query(query): Query<UrlQuery>,
    Path((tag_name,)): Path<(String,)>,
) -> Result<ApiResponse<Vec<db::ArticleByList>>, ApiError> {
    Ok(ApiResponse::ok(
        db::get_articles_by_tag(
            &conn,
            &tag_name,
            query.lang.unwrap_or(db::Lang::ZhCN),
            query.sort.unwrap_or(content_metadata::Column::PublishedAt),
            query.limit.unwrap_or(100),
        )
        .await?,
    ))
}

pub fn router() -> axum::Router {
    axum::Router::new()
        .route("/site", get(get_site))
        .route("/articles", get(get_articles))
        .route("/shorts", get(get_shorts))
        .route("/shorts/{slug}", get(get_short_by_slug))
        .route("/articles/{slug}", get(get_article_by_slug))
        .route("/tags", get(get_tags_with_count))
        .route("/archives", get(get_archives_group_by_year))
        .route("/tags/{tag_name}/articles", get(get_articles_by_tag))
}
