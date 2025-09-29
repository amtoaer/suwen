use chrono::{DateTime, Local};
use sea_orm::FromQueryResult;
use serde::{Deserialize, Serialize};

use suwen_entity::{RelatedLinks, Tabs, Toc, VecString};

use crate::routes::IdentityInfo;

#[derive(Debug, Clone, Serialize, Deserialize, FromQueryResult)]
#[serde(rename_all = "camelCase")]
pub struct Site {
    pub site_name: String,
    pub intro: String,
    pub display_name: String,
    pub avatar_url: String,
    pub related_links: RelatedLinks,
    pub tabs: Tabs,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromQueryResult)]
#[serde(rename_all = "camelCase")]
pub struct ArticleByList {
    pub slug: String,
    pub title: String,
    pub intro: Option<String>,
    pub summary: Option<String>,
    pub cover_images: VecString,
    pub tags: VecString,
    pub view_count: i32,
    pub comment_count: i32,
    pub published_at: DateTime<Local>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromQueryResult)]
#[serde(rename_all = "camelCase")]
pub struct ArticleBySlug {
    pub title: String,
    pub rendered_html: String,
    pub summary: Option<String>,
    pub intro: Option<String>,
    pub tags: VecString,
    pub toc: Toc,
    pub view_count: i32,
    pub comment_count: i32,
    pub like_count: i32,
    pub published_at: DateTime<Local>,
}

#[derive(Debug, Clone, FromQueryResult)]
pub struct ArticleForRSS {
    pub slug: String,
    pub title: String,
    pub intro: Option<String>,
    pub summary: Option<String>,
    pub tags: VecString,
    pub rendered_html: String,
    pub updated_at: DateTime<Local>,
    pub published_at: DateTime<Local>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromQueryResult)]
#[serde(rename_all = "camelCase")]
pub struct Short {
    pub slug: String,
    pub title: String,
    pub cover_images: VecString,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromQueryResult)]
#[serde(rename_all = "camelCase")]
pub struct TagWithCount {
    pub tag_name: String,
    pub count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromQueryResult)]
#[serde(rename_all = "camelCase")]
pub struct Archive {
    pub slug: String,
    pub title: String,
    pub published_at: DateTime<Local>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Comment {
    pub id: i32,
    pub content: String,
    pub commenter: IdentityInfo,
    pub replies: Vec<Comment>,
    pub is_deleted: bool,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

impl From<(IdentityInfo, suwen_entity::comment::Model)> for Comment {
    fn from((commenter, comment): (IdentityInfo, suwen_entity::comment::Model)) -> Self {
        Self {
            id: comment.id,
            content: comment.content,
            commenter,
            replies: vec![],
            is_deleted: comment.is_deleted,
            created_at: comment.created_at,
            updated_at: comment.updated_at,
        }
    }
}
