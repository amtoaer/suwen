use anyhow::Context;
use axum::{
    Extension,
    extract::{Path, Query},
    http::HeaderValue,
    response::IntoResponse,
    routing::{get, post},
};
use axum_extra::extract::cookie::{Cookie, SameSite};
pub(crate) use schema::IdentityInfo;
use sea_orm::{ActiveValue::Set as ActiveSet, QueryFilter};
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, Set, TransactionTrait,
};
use serde::Deserialize;
use suwen_entity::content_metadata;
use suwen_migration::Expr;

use crate::{
    auth::Identity,
    db::{self, Archive, Comment, get_metadata_id_for_slug},
    wrapper::{ApiError, ApiResponse},
};

mod middleware;
mod schema;

#[derive(Deserialize)]
struct LikeRequest {
    like: bool,
}

#[derive(Deserialize)]
struct CommentRequest {
    parent_id: Option<i32>,
    content: String,
}

#[derive(Deserialize)]
struct DeleteCommentRequest {
    id: i32,
}

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

async fn me(Extension(identity): Extension<Identity>) -> impl IntoResponse {
    if !matches!(identity, Identity::None) {
        ApiResponse::ok(Into::<IdentityInfo>::into(identity)).into_response()
    } else {
        let uuid = uuid::Uuid::new_v4();
        let mut resp = ApiResponse::ok(Into::<IdentityInfo>::into(Identity::Anonymous {
            uuid,
            identity: None,
        }))
        .into_response();
        let cookie = Cookie::build(("anonymous", uuid.simple().to_string()))
            .path("/")
            .http_only(true)
            .same_site(SameSite::Lax)
            .build();
        resp.headers_mut().insert(
            axum::http::header::SET_COOKIE,
            HeaderValue::from_str(&cookie.to_string()).unwrap(),
        );
        resp
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
    let article = db::get_article_by_slug(&conn, &slug, query.lang.unwrap_or(db::Lang::ZhCN))
        .await?
        .ok_or_else(|| ApiError::not_found("Article not found"))?;
    Ok(ApiResponse::ok(article))
}

async fn increase_view_count(
    Extension(conn): Extension<DatabaseConnection>,
    Path((slug,)): Path<(String,)>,
) -> Result<ApiResponse<i32>, ApiError> {
    Ok(ApiResponse::ok(
        db::increase_article_view_count(&conn, &slug).await?,
    ))
}

async fn get_likes(
    Extension(identity): Extension<Identity>,
    Extension(conn): Extension<DatabaseConnection>,
    Path((slug,)): Path<(String,)>,
) -> Result<ApiResponse<bool>, ApiError> {
    let Some(identity_model) = identity.identity() else {
        return Ok(ApiResponse::ok(false));
    };
    let metadata_id = get_metadata_id_for_slug(&slug, &conn).await?;
    let exists = suwen_entity::like::Entity::find()
        .filter(suwen_entity::like::Column::IdentityId.eq(identity_model.id))
        .filter(suwen_entity::like::Column::ContentMetadataId.eq(metadata_id))
        .count(&conn)
        .await?
        > 0;
    Ok(ApiResponse::ok(exists))
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

async fn like_content(
    Extension(conn): Extension<DatabaseConnection>,
    Extension(mut identity): Extension<Identity>,
    Path((slug,)): Path<(String,)>,
    axum::Json(request): axum::Json<LikeRequest>,
) -> Result<ApiResponse<u64>, ApiError> {
    let identity_model = identity.ensure_identity(&conn).await?;
    let metadata_id = get_metadata_id_for_slug(&slug, &conn).await?;
    let txn = conn.begin().await?;
    if request.like {
        suwen_entity::like::Entity::insert(suwen_entity::like::ActiveModel {
            identity_id: Set(identity_model.id),
            content_metadata_id: Set(metadata_id),
            ..Default::default()
        })
        .on_conflict_do_nothing()
        .exec(&txn)
        .await?;
    } else {
        suwen_entity::like::Entity::delete_many()
            .filter(suwen_entity::like::Column::IdentityId.eq(identity_model.id))
            .filter(suwen_entity::like::Column::ContentMetadataId.eq(metadata_id))
            .exec(&txn)
            .await?;
    }
    let like_count = suwen_entity::like::Entity::find()
        .filter(suwen_entity::like::Column::ContentMetadataId.eq(metadata_id))
        .count(&txn)
        .await?;
    content_metadata::Entity::update_many()
        .filter(content_metadata::Column::Id.eq(metadata_id))
        .col_expr(content_metadata::Column::LikeCount, Expr::value(like_count))
        .exec(&txn)
        .await?;
    txn.commit().await?;
    Ok(ApiResponse::ok(like_count))
}

async fn add_comment(
    Extension(conn): Extension<DatabaseConnection>,
    Extension(mut identity): Extension<Identity>,
    Path((slug,)): Path<(String,)>,
    axum::Json(request): axum::Json<CommentRequest>,
) -> Result<ApiResponse<u64>, ApiError> {
    let identity_model = identity.ensure_identity(&conn).await?;
    let metadata_id = get_metadata_id_for_slug(&slug, &conn).await?;
    let comment_model = suwen_entity::comment::ActiveModel {
        identity_id: ActiveSet(identity_model.id),
        content_metadata_id: ActiveSet(metadata_id),
        parent_id: ActiveSet(request.parent_id),
        content: ActiveSet(request.content),
        ..Default::default()
    };
    let txn = conn.begin().await?;
    suwen_entity::comment::Entity::insert(comment_model)
        .exec(&txn)
        .await?;
    let comment_count = suwen_entity::comment::Entity::find()
        .filter(
            suwen_entity::comment::Column::ContentMetadataId
                .eq(metadata_id)
                .and(suwen_entity::comment::Column::ParentId.is_null()),
        )
        .count(&txn)
        .await?;
    content_metadata::Entity::update_many()
        .filter(content_metadata::Column::Id.eq(metadata_id))
        .col_expr(
            content_metadata::Column::CommentCount,
            Expr::value(comment_count),
        )
        .exec(&txn)
        .await?;
    txn.commit().await?;
    Ok(ApiResponse::ok(comment_count))
}

async fn delete_comment(
    Extension(conn): Extension<DatabaseConnection>,
    Extension(identity): Extension<Identity>,
    Path((slug,)): Path<(String,)>,
    axum::Json(request): axum::Json<DeleteCommentRequest>,
) -> Result<ApiResponse<()>, ApiError> {
    let identity_model = identity
        .identity()
        .ok_or_else(|| ApiError::unauthorized("No identity available"))?;
    let comment = suwen_entity::comment::Entity::find_by_id(request.id)
        .left_join(suwen_entity::content_metadata::Entity)
        .filter(content_metadata::Column::Slug.eq(slug))
        .one(&conn)
        .await?
        .ok_or_else(|| ApiError::not_found("Comment not found"))?;
    if comment.identity_id != identity_model.id && !matches!(identity, Identity::Admin { .. }) {
        return Err(ApiError::forbidden("Not allowed to delete this comment"));
    }
    suwen_entity::comment::Entity::update_many()
        .filter(suwen_entity::comment::Column::Id.eq(comment.id))
        .col_expr(suwen_entity::comment::Column::IsDeleted, Expr::value(true))
        .col_expr(suwen_entity::comment::Column::Content, Expr::value(""))
        .exec(&conn)
        .await?;
    Ok(ApiResponse::ok(()))
}

async fn get_comments_by_slug(
    Extension(conn): Extension<DatabaseConnection>,
    Path((slug,)): Path<(String,)>,
) -> Result<ApiResponse<Vec<Comment>>, ApiError> {
    Ok(ApiResponse::ok(
        db::get_comments_by_slug(&conn, &slug).await?,
    ))
}

pub fn router() -> axum::Router {
    axum::Router::new()
        .route("/me", get(me))
        .route("/site", get(get_site))
        .route("/articles", get(get_articles))
        .route("/shorts", get(get_shorts))
        .route("/shorts/{slug}", get(get_short_by_slug))
        .route("/articles/{slug}", get(get_article_by_slug))
        .route("/articles/{slug}/views", post(increase_view_count))
        .route(
            "/articles/{slug}/comments",
            get(get_comments_by_slug)
                .post(add_comment)
                .delete(delete_comment),
        )
        .route("/articles/{slug}/likes", get(get_likes).post(like_content))
        .route("/tags", get(get_tags_with_count))
        .route("/archives", get(get_archives_group_by_year))
        .route("/tags/{tag_name}/articles", get(get_articles_by_tag))
        .layer(axum::middleware::from_fn(middleware::auth))
}
