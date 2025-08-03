use std::collections::HashMap;
use std::path::Path;

use crate::db::Lang;
use crate::db::schema::{Archive, ArticleByList, ArticleBySlug, Short, Site, TagWithCount};
use crate::db::utils::sha256_hash;
use anyhow::Result;
use chrono::Datelike;
use sea_orm::ActiveValue::Set;
use sea_orm::{ColumnTrait, ConnectionTrait, JoinType, TransactionTrait};
use sea_orm::{
    DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, QuerySelect, RelationTrait,
};
use suwen_entity::*;

pub async fn init(conn: &DatabaseConnection) -> Result<()> {
    let transaction = conn.begin().await?;
    let site = get_site(&transaction).await?;
    if site.is_none() {
        let result = user::Entity::insert(user::ActiveModel {
            email: Set("amtoaer@gmail.com".into()),
            username: Set("amtoaer".into()),
            display_name: Set("amtoaer".into()),
            avatar_url: Set("https://ipfs.crossbell.io/ipfs/QmWyMrr5hZTzuJ7eDmns8ZheyRf6xaQH4vh9ZpFYo2YFXL?img-quality=75&img-format=auto&img-onerror=redirect&img-width=384".into()),
            password_hash: Set(sha256_hash("password")),
            ..Default::default()
        })
        .exec(&transaction)
        .await?;
        site::Entity::insert(site::ActiveModel {
            site_name: Set("晓风残月".into()),
            intro: Set("叹息似的渺茫，你仍要保存着那真！".into()),
            keywords: Set(vec!["技术".to_owned(), "编程".to_owned()].into()),
            related_links: Set(vec![
                RelatedLink {
                    name: "GitHub".into(),
                    icon: "https://icons.ly/github/_/fff".into(),
                    url: "https://github.com/amtoaer".into(),
                },
                RelatedLink {
                    name: "Telegram".into(),
                    icon: "https://icons.ly/telegram".into(),
                    url: "https://t.me/amtoaer".into(),
                },
                RelatedLink {
                    name: "GMail".into(),
                    icon: "https://icons.ly/gmail/_/EA4335".into(),
                    url: "mailto:amtoaer@gmail.com".into(),
                },
                RelatedLink {
                    name: "X".into(),
                    icon: "https://icons.ly/x/_/fff".into(),
                    url: "https://x.com/amtoaer".into(),
                },
                RelatedLink {
                    name: "Bilibili".into(),
                    icon: "https://icons.ly/bilibili/_/00A1D6".into(),
                    url: "https://space.bilibili.com/9183758".into(),
                },
                RelatedLink {
                    name: "Steam".into(),
                    icon: "https://icons.ly/steam/_/fff".into(),
                    url: "https://steamcommunity.com/id/amtoaer".into(),
                },
            ]
            .into()),
            tabs: Set(vec![
                Tab {
                    name: "首页".into(),
                    url: "/".into(),
                },
                Tab {
                    name: "文章".into(),
                    url: "/articles".into(),
                },
                Tab {
                    name: "短篇".into(),
                    url: "/shorts".into(),
                },
                Tab {
                    name: "关于".into(),
                    url: "/about".into(),
                },
            ]
            .into()),
            owner_id: Set(result.last_insert_id),
            ..Default::default()
        })
        .exec(&transaction)
        .await?;
        let (articles, shorts) = suwen_markdown::importer::import_path(
            &Path::new("/Users/amtoaer/Downloads/Zen/amtoaer/notes-markdown"),
            &suwen_markdown::importer::xlog::import_file,
        )?;
        for article in articles {
            let result = content_metadata::Entity::insert(content_metadata::ActiveModel {
                slug: Set(article.slug),
                content_type: Set("article".into()),
                cover_images: Set(article.cover_images.into()),
                tags: Set(article.tags.into()),
                view_count: Set(0),
                comment_count: Set(0),
                published_at: Set(Some(article.published_at)),
                ..Default::default()
            })
            .exec(&transaction)
            .await?;
            content::Entity::insert(content::ActiveModel {
                title: Set(article.title),
                original_text: Set(article.content),
                rendered_html: Set(Some(article.rendered_html)),
                lang_code: Set(Lang::ZhCN.to_string()),
                content_metadata_id: Set(result.last_insert_id),
                ..Default::default()
            })
            .exec(&transaction)
            .await?;
        }
        for short in shorts {
            let result = content_metadata::Entity::insert(content_metadata::ActiveModel {
                slug: Set(short.slug),
                content_type: Set("gallery".into()),
                cover_images: Set(short.cover_images.into()),
                tags: Set(vec![].into()),
                view_count: Set(0),
                comment_count: Set(0),
                published_at: Set(Some(short.published_at)),
                ..Default::default()
            })
            .exec(&transaction)
            .await?;
            content::Entity::insert(content::ActiveModel {
                title: Set(short.title),
                original_text: Set(short.content),
                lang_code: Set(Lang::ZhCN.to_string()),
                content_metadata_id: Set(result.last_insert_id),
                ..Default::default()
            })
            .exec(&transaction)
            .await?;
        }
    }
    transaction.commit().await?;
    Ok(())
}

pub async fn get_site(conn: &impl ConnectionTrait) -> Result<Option<Site>> {
    Ok(site::Entity::find()
        .select_only()
        .columns([
            site::Column::SiteName,
            site::Column::Intro,
            site::Column::Keywords,
            site::Column::RelatedLinks,
            site::Column::Tabs,
        ])
        .column_as(user::Column::DisplayName, "display_name")
        .column_as(user::Column::AvatarUrl, "avatar_url")
        .left_join(user::Entity)
        .limit(1)
        .into_model::<Site>()
        .one(conn)
        .await?)
}

pub async fn get_articles(
    conn: &DatabaseConnection,
    lang: Lang,
    sort_column: content_metadata::Column,
    published: Option<bool>,
    limit: u64,
) -> Result<Vec<ArticleByList>> {
    let query = content_metadata::Entity::find()
        .select_only()
        .columns([
            content_metadata::Column::Slug,
            content_metadata::Column::CoverImages,
            content_metadata::Column::Tags,
            content_metadata::Column::ViewCount,
            content_metadata::Column::CommentCount,
            content_metadata::Column::PublishedAt,
        ])
        .column_as(content::Column::Title, "title")
        .inner_join(content::Entity)
        .filter(content_metadata::Column::ContentType.eq("article"))
        .filter(content::Column::LangCode.eq(lang.to_string()))
        .order_by_desc(sort_column)
        .limit(limit);
    let query = if let Some(published) = published {
        query.filter(if published {
            content_metadata::Column::PublishedAt.is_not_null()
        } else {
            content_metadata::Column::PublishedAt.is_null()
        })
    } else {
        query
    };
    Ok(query.into_model::<ArticleByList>().all(conn).await?)
}

pub async fn get_shorts(
    conn: &DatabaseConnection,
    lang: Lang,
    sort_column: content_metadata::Column,
    published: Option<bool>,
    limit: u32,
) -> Result<Vec<Short>> {
    let query = content_metadata::Entity::find()
        .select_only()
        .columns([
            content_metadata::Column::Slug,
            content_metadata::Column::CoverImages,
        ])
        .column_as(content::Column::Title, "title")
        .column_as(content::Column::OriginalText, "content")
        .inner_join(content::Entity)
        .filter(content_metadata::Column::ContentType.eq("gallery"))
        .filter(content::Column::LangCode.eq(lang.to_string()))
        .order_by_desc(sort_column)
        .limit(limit as u64);
    let query = if let Some(published) = published {
        query.filter(if published {
            content_metadata::Column::PublishedAt.is_not_null()
        } else {
            content_metadata::Column::PublishedAt.is_null()
        })
    } else {
        query
    };
    Ok(query.into_model::<Short>().all(conn).await?)
}

pub async fn get_short_by_slug(
    conn: &DatabaseConnection,
    slug: &str,
    lang: Lang,
) -> Result<Option<Short>> {
    Ok(content_metadata::Entity::find()
        .select_only()
        .columns([
            content_metadata::Column::Slug,
            content_metadata::Column::CoverImages,
        ])
        .column_as(content::Column::Title, "title")
        .column_as(content::Column::OriginalText, "content")
        .inner_join(content::Entity)
        .filter(content_metadata::Column::ContentType.eq("gallery"))
        .filter(content::Column::LangCode.eq(lang.to_string()))
        .filter(content_metadata::Column::Slug.eq(slug))
        .into_model::<Short>()
        .one(conn)
        .await?)
}

pub async fn get_article_by_slug(
    conn: &DatabaseConnection,
    slug: &str,
    lang: Lang,
) -> Result<Option<ArticleBySlug>> {
    Ok(content_metadata::Entity::find()
        .select_only()
        .columns([
            content_metadata::Column::Slug,
            content_metadata::Column::Tags,
            content_metadata::Column::ViewCount,
            content_metadata::Column::CommentCount,
            content_metadata::Column::PublishedAt,
        ])
        .column_as(content::Column::Title, "title")
        .column_as(content::Column::RenderedHtml, "rendered_html")
        .inner_join(content::Entity)
        .filter(content_metadata::Column::ContentType.eq("article"))
        .filter(content::Column::LangCode.eq(lang.to_string()))
        .filter(content_metadata::Column::Slug.eq(slug))
        .into_model::<ArticleBySlug>()
        .one(conn)
        .await?)
}

pub async fn get_tags_with_count(conn: &DatabaseConnection) -> Result<Vec<TagWithCount>> {
    Ok(tag::Entity::find()
        .select_only()
        .column(tag::Column::TagName)
        .column_as(content_metadata::Column::Id.count(), "count")
        .left_join(content_metadata::Entity)
        .into_model::<TagWithCount>()
        .all(conn)
        .await?)
}

pub async fn get_archives_grouped_by_year(
    conn: &DatabaseConnection,
    lang: Lang,
) -> Result<HashMap<i32, Vec<Archive>>> {
    let archives = content_metadata::Entity::find()
        .select_only()
        .columns([
            content_metadata::Column::Slug,
            content_metadata::Column::PublishedAt,
        ])
        .column_as(content::Column::Title, "title")
        .inner_join(content::Entity)
        .filter(content::Column::LangCode.eq(lang.to_string()))
        .order_by_desc(content_metadata::Column::PublishedAt)
        .into_model::<Archive>()
        .all(conn)
        .await?;
    let mut grouped: HashMap<i32, Vec<Archive>> = HashMap::new();
    for archive in archives {
        let year = archive.published_at.year();
        grouped.entry(year).or_default().push(archive);
    }
    Ok(grouped)
}

pub async fn get_articles_by_tag(
    conn: &DatabaseConnection,
    tag_name: &str,
    lang: Lang,
    sort_column: content_metadata::Column,
    limit: u64,
) -> Result<Vec<ArticleByList>> {
    Ok(tag::Entity::find()
        .select_only()
        .column(tag::Column::TagName)
        .columns([
            content_metadata::Column::Slug,
            content_metadata::Column::CoverImages,
            content_metadata::Column::Tags,
            content_metadata::Column::ViewCount,
            content_metadata::Column::CommentCount,
            content_metadata::Column::PublishedAt,
        ])
        .inner_join(content_metadata::Entity)
        .join(
            JoinType::InnerJoin,
            content_metadata::Relation::Content.def(),
        )
        .filter(tag::Column::TagName.eq(tag_name))
        .filter(content_metadata::Column::ContentType.eq("article"))
        .filter(content::Column::LangCode.eq(lang.to_string()))
        .order_by_desc(sort_column)
        .limit(limit)
        .into_model::<ArticleByList>()
        .all(conn)
        .await?)
}
