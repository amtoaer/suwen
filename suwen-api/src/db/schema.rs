use chrono::{DateTime, Local};
use sea_orm::FromQueryResult;
use serde::{Deserialize, Serialize};

use suwen_entity::{RelatedLinks, Tabs, Toc, VecString};

#[derive(Debug, Clone, Serialize, Deserialize, FromQueryResult)]
#[serde(rename_all = "camelCase")]
pub struct Site {
    pub site_name: String,
    pub intro: String,
    pub display_name: String,
    pub avatar_url: String,
    pub related_links: RelatedLinks,
    pub tabs: Tabs,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromQueryResult)]
#[serde(rename_all = "camelCase")]
pub struct ArticleByList {
    pub slug: String,
    pub title: String,
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
    pub tags: VecString,
    pub toc: Toc,
    pub view_count: i32,
    pub comment_count: i32,
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
    pub name: String,
    pub count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromQueryResult)]
#[serde(rename_all = "camelCase")]
pub struct Archive {
    pub slug: String,
    pub title: String,
    pub published_at: DateTime<Local>,
}
