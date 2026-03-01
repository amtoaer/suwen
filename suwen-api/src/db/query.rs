use std::collections::HashMap;

use anyhow::{Result, bail, ensure};
use chrono::Datelike;
use dashmap::DashMap;
use sea_orm::ActiveValue::Set;
use sea_orm::{
    ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, JoinType, QueryFilter, QueryOrder, QuerySelect,
    QueryTrait, RelationTrait, TransactionTrait,
};
use suwen_entity::*;
use suwen_llm::generate_article_summary;
use suwen_markdown::manager::Markdown;
use suwen_migration::{Expr, OnConflict};

use crate::db::schema::{Archive, ArticleByList, ArticleBySlug, Short, Site, SitemapUrl, TagWithCount};
use crate::db::utils::sha256_hash;
use crate::db::{ArticleForRSS, Comment, Lang, get_metadata_id_for_slug};
use crate::routes::IdentityInfo;

pub async fn init(conn: &DatabaseConnection) -> Result<()> {
    let txn = conn.begin().await?;
    let site = get_site(&txn).await?;
    if site.is_none() {
        let result = user::Entity::insert(user::ActiveModel {
            email: Set("amtoaer@gmail.com".into()),
            username: Set("amtoaer".into()),
            display_name: Set("amtoaer".into()),
            avatar_url: Set("https://obj.amto.cc/avatar.webp".into()),
            password_hash: Set(sha256_hash("password")),
            ..Default::default()
        })
        .exec(&txn)
        .await?;
        site::Entity::insert(site::ActiveModel {
            site_name: Set("晓风残月".into()),
            intro: Set("叹息似的渺茫，你仍要保存着那真！".into()),
            related_links: Set(vec![
                RelatedLink {
                    name: "GitHub".into(),
                    icon: "https://cdn.simpleicons.org/github/_/fff".into(),
                    url: "https://github.com/amtoaer".into(),
                },
                RelatedLink {
                    name: "Telegram".into(),
                    icon: "https://cdn.simpleicons.org/telegram".into(),
                    url: "https://t.me/amtoaer".into(),
                },
                RelatedLink {
                    name: "GMail".into(),
                    icon: "https://cdn.simpleicons.org/gmail/_/EA4335".into(),
                    url: "mailto:amtoaer@gmail.com".into(),
                },
                RelatedLink {
                    name: "X".into(),
                    icon: "https://cdn.simpleicons.org/x/_/fff".into(),
                    url: "https://x.com/amtoaer".into(),
                },
                RelatedLink {
                    name: "Bilibili".into(),
                    icon: "https://cdn.simpleicons.org/bilibili/_/00A1D6".into(),
                    url: "https://space.bilibili.com/9183758".into(),
                },
                RelatedLink {
                    name: "Steam".into(),
                    icon: "https://cdn.simpleicons.org/steam/_/fff".into(),
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
                    name: "图文".into(),
                    url: "/shorts".into(),
                },
                Tab {
                    name: "归档".into(),
                    url: "/archives".into(),
                },
            ]
            .into()),
            owner_id: Set(result.last_insert_id),
            ..Default::default()
        })
        .exec(&txn)
        .await?;
        txn.commit().await?;
    }
    Ok(())
}

pub async fn get_site(conn: &impl ConnectionTrait) -> Result<Option<Site>> {
    Ok(site::Entity::find()
        .select_only()
        .columns([
            site::Column::SiteName,
            site::Column::Intro,
            site::Column::RelatedLinks,
            site::Column::Tabs,
            site::Column::CreatedAt,
            site::Column::UpdatedAt,
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
        .column_as(content::Column::Intro, "intro")
        .column_as(content::Column::Summary, "summary")
        .inner_join(content::Entity)
        .filter(
            content_metadata::Column::ContentType
                .eq("article")
                .and(content::Column::LangCode.eq(lang.to_string())),
        )
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

pub async fn get_rss_articles(conn: &DatabaseConnection, lang: Lang, limit: u64) -> Result<Vec<ArticleForRSS>> {
    Ok(content_metadata::Entity::find()
        .select_only()
        .columns([
            content_metadata::Column::Slug,
            content_metadata::Column::Tags,
            content_metadata::Column::UpdatedAt,
            content_metadata::Column::PublishedAt,
        ])
        .column_as(content::Column::Title, "title")
        .column_as(content::Column::Intro, "intro")
        .column_as(content::Column::Summary, "summary")
        .column_as(content::Column::RenderedHtml, "rendered_html")
        .inner_join(content::Entity)
        .filter(
            content_metadata::Column::ContentType
                .eq("article")
                .and(content::Column::LangCode.eq(lang.to_string()))
                .and(content_metadata::Column::PublishedAt.is_not_null()),
        )
        .order_by_desc(content_metadata::Column::PublishedAt)
        .limit(limit)
        .into_model::<ArticleForRSS>()
        .all(conn)
        .await?)
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
        .columns([content_metadata::Column::Slug, content_metadata::Column::CoverImages])
        .column_as(content::Column::Title, "title")
        .column_as(content::Column::OriginalText, "content")
        .inner_join(content::Entity)
        .filter(
            content_metadata::Column::ContentType
                .eq("gallery")
                .and(content::Column::LangCode.eq(lang.to_string())),
        )
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

pub async fn get_short_by_slug(conn: &DatabaseConnection, slug: &str, lang: Lang) -> Result<Option<Short>> {
    Ok(content_metadata::Entity::find()
        .select_only()
        .columns([content_metadata::Column::Slug, content_metadata::Column::CoverImages])
        .column_as(content::Column::Title, "title")
        .column_as(content::Column::OriginalText, "content")
        .inner_join(content::Entity)
        .filter(
            content_metadata::Column::ContentType
                .eq("gallery")
                .and(content::Column::LangCode.eq(lang.to_string()))
                .and(content_metadata::Column::Slug.eq(slug))
                .and(content_metadata::Column::PublishedAt.is_not_null()),
        )
        .into_model::<Short>()
        .one(conn)
        .await?)
}

pub async fn get_article_by_slug(conn: &DatabaseConnection, slug: &str, lang: Lang) -> Result<Option<ArticleBySlug>> {
    Ok(content_metadata::Entity::find()
        .select_only()
        .columns([
            content_metadata::Column::Id,
            content_metadata::Column::Slug,
            content_metadata::Column::Tags,
            content_metadata::Column::ViewCount,
            content_metadata::Column::CommentCount,
            content_metadata::Column::LikeCount,
            content_metadata::Column::PublishedAt,
        ])
        .column_as(content::Column::Title, "title")
        .column_as(content::Column::RenderedHtml, "rendered_html")
        .column_as(content::Column::Toc, "toc")
        .column_as(content::Column::Summary, "summary")
        .column_as(content::Column::Intro, "intro")
        .inner_join(content::Entity)
        .filter(
            content_metadata::Column::ContentType
                .eq("article")
                .and(content::Column::LangCode.eq(lang.to_string()))
                .and(content_metadata::Column::Slug.eq(slug))
                .and(content_metadata::Column::PublishedAt.is_not_null()),
        )
        .into_model::<ArticleBySlug>()
        .one(conn)
        .await?)
}

pub async fn increase_article_view_count(conn: &DatabaseConnection, slug: &str) -> Result<i32> {
    let metadata = content_metadata::Entity::update_many()
        .filter(content_metadata::Column::Slug.eq(slug))
        .col_expr(
            content_metadata::Column::ViewCount,
            Expr::col(content_metadata::Column::ViewCount).add(1),
        )
        .exec_with_returning(conn)
        .await?;
    ensure!(metadata.len() == 1, "operation failed");
    Ok(metadata[0].view_count)
}

pub async fn get_tags_with_count(conn: &DatabaseConnection) -> Result<Vec<TagWithCount>> {
    Ok(tag::Entity::find()
        .select_only()
        .column(tag::Column::TagName)
        .join(JoinType::LeftJoin, tag::Relation::ContentMetadataTag.def())
        .join(
            JoinType::LeftJoin,
            content_metadata_tag::Relation::ContentMetadata.def(),
        )
        .column_as(content_metadata::Column::Id.count(), "count")
        .group_by(tag::Column::TagName)
        .into_model::<TagWithCount>()
        .all(conn)
        .await?)
}

pub async fn get_archives_grouped_by_year(conn: &DatabaseConnection, lang: Lang) -> Result<Vec<(i32, Vec<Archive>)>> {
    let archives = content_metadata::Entity::find()
        .select_only()
        .columns([content_metadata::Column::Slug, content_metadata::Column::PublishedAt])
        .column_as(content::Column::Title, "title")
        .inner_join(content::Entity)
        .filter(
            content::Column::LangCode
                .eq(lang.to_string())
                .and(content_metadata::Column::ContentType.eq("article")),
        )
        .order_by_desc(content_metadata::Column::PublishedAt)
        .into_model::<Archive>()
        .all(conn)
        .await?;
    let mut grouped: HashMap<i32, Vec<Archive>> = HashMap::new();
    for archive in archives {
        let year = archive.published_at.year();
        grouped.entry(year).or_default().push(archive);
    }
    let mut grouped: Vec<(i32, Vec<Archive>)> = grouped.into_iter().collect();
    grouped.sort_by_key(|(year, _)| -*year);
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
        .column(content::Column::Title)
        .column_as(content::Column::Intro, "intro")
        .column_as(content::Column::Summary, "summary")
        .inner_join(content_metadata::Entity)
        .join(JoinType::InnerJoin, content_metadata::Relation::Content.def())
        .filter(
            tag::Column::TagName.eq(tag_name).and(
                content_metadata::Column::ContentType
                    .eq("article")
                    .and(content::Column::LangCode.eq(lang.to_string()))
                    .and(content_metadata::Column::PublishedAt.is_not_null()),
            ),
        )
        .order_by_desc(sort_column)
        .limit(limit)
        .into_model::<ArticleByList>()
        .all(conn)
        .await?)
}

pub async fn create_article(
    conn: &impl ConnectionTrait,
    mut markdown: Markdown,
    lang: Lang,
    summary_cache: &DashMap<String, Option<String>>,
) -> Result<()> {
    let (toc, rendered_html) = markdown.render_to_html()?;
    let cover_images = markdown.extract_images()?;
    markdown.strip_images()?;
    markdown.auto_format()?;
    match markdown {
        Markdown::Article {
            slug,
            title,
            tags,
            content,
            created_at,
            updated_at,
            published_at,
        } => {
            let summary = if let Some(cache) = summary_cache.get(&format!("{}-{}", slug, lang)) {
                cache.value().clone()
            } else {
                let summary = generate_article_summary(&content).await?;
                summary_cache.insert(format!("{}-{}", slug, lang), summary.clone());
                summary
            };
            let result = content_metadata::Entity::insert(content_metadata::ActiveModel {
                slug: Set(slug),
                content_type: Set("article".into()),
                cover_images: Set(cover_images.into()),
                tags: Set(tags.clone().into()),
                view_count: Set(0),
                comment_count: Set(0),
                created_at: Set(created_at),
                updated_at: Set(updated_at),
                published_at: Set(Some(published_at)),
                original_lang: Set(lang.to_string()),
                ..Default::default()
            })
            .exec(conn)
            .await?;
            content::Entity::insert(content::ActiveModel {
                title: Set(title),
                original_text: Set(content),
                rendered_html: Set(rendered_html),
                toc: Set(toc),
                lang_code: Set(lang.to_string()),
                summary: Set(summary),
                content_metadata_id: Set(result.last_insert_id),
                ..Default::default()
            })
            .exec(conn)
            .await?;
            if !tags.is_empty() {
                let tag_models = tags
                    .iter()
                    .map(|tag| tag::ActiveModel {
                        tag_name: Set(tag.clone()),
                        ..Default::default()
                    })
                    .collect::<Vec<_>>();
                tag::Entity::insert_many(tag_models)
                    .on_conflict(OnConflict::column(tag::Column::TagName).do_nothing().to_owned())
                    .do_nothing()
                    .exec(conn)
                    .await?;
                let tag_records = tag::Entity::find()
                    .filter(tag::Column::TagName.is_in(tags))
                    .all(conn)
                    .await?;
                let tag_associations = tag_records
                    .into_iter()
                    .map(|tag| content_metadata_tag::ActiveModel {
                        content_metadata_id: Set(result.last_insert_id),
                        tag_id: Set(tag.id),
                    })
                    .collect::<Vec<_>>();
                content_metadata_tag::Entity::insert_many(tag_associations)
                    .exec(conn)
                    .await?;
            }
        }
        Markdown::Short {
            slug,
            title,
            content,
            created_at,
            updated_at,
            published_at,
        } => {
            let result = content_metadata::Entity::insert(content_metadata::ActiveModel {
                slug: Set(slug),
                content_type: Set("gallery".into()),
                cover_images: Set(cover_images.into()),
                tags: Set(vec![].into()),
                view_count: Set(0),
                comment_count: Set(0),
                original_lang: Set(lang.to_string()),
                created_at: Set(created_at),
                updated_at: Set(updated_at),
                published_at: Set(Some(published_at)),
                ..Default::default()
            })
            .exec(conn)
            .await?;
            content::Entity::insert(content::ActiveModel {
                title: Set(title),
                original_text: Set(content),
                lang_code: Set(lang.to_string()),
                content_metadata_id: Set(result.last_insert_id),
                ..Default::default()
            })
            .exec(conn)
            .await?;
        }
    };
    Ok(())
}

/// 处理 markdown 变更事件（创建/更新）
pub async fn handle_markdown_change(
    conn: &DatabaseConnection,
    change: suwen_markdown::manager::watcher::MarkdownChange,
    lang: Lang,
    summary_cache: &DashMap<String, Option<String>>,
) -> Result<()> {
    match change {
        suwen_markdown::manager::watcher::MarkdownChange::Upsert(markdown) => {
            let slug = markdown.slug().to_string();
            // 检查是否已存在
            let existing = content_metadata::Entity::find()
                .filter(content_metadata::Column::Slug.eq(&slug))
                .one(conn)
                .await?;

            if existing.is_some() {
                info!("Updating existing article: {}", slug);
                update_article(conn, markdown, lang).await?;
            } else {
                info!("Creating new article: {}", slug);
                create_article(conn, markdown, lang, summary_cache).await?;
            }
        }
        suwen_markdown::manager::watcher::MarkdownChange::Deleted(slug) => {
            info!("Deleting article: {}", slug);
            if let Some(metadata) = content_metadata::Entity::find()
                .filter(content_metadata::Column::Slug.eq(&slug))
                .one(conn)
                .await?
            {
                // 删除关联的 content 记录
                content::Entity::delete_many()
                    .filter(content::Column::ContentMetadataId.eq(metadata.id))
                    .exec(conn)
                    .await?;
                // 删除关联的标签关联
                content_metadata_tag::Entity::delete_many()
                    .filter(content_metadata_tag::Column::ContentMetadataId.eq(metadata.id))
                    .exec(conn)
                    .await?;
                // 删除 metadata
                content_metadata::Entity::delete_by_id(metadata.id).exec(conn).await?;
            }
        }
        suwen_markdown::manager::watcher::MarkdownChange::SyncExisting(existing_slugs) => {
            info!("Syncing existing articles, found {} files", existing_slugs.len());
            // 获取数据库中所有的 slug
            let all_db_slugs = content_metadata::Entity::find()
                .select_only()
                .column(content_metadata::Column::Slug)
                .into_tuple::<String>()
                .all(conn)
                .await?;
            
            // 找出数据库中存在但文件系统中不存在的 slug
            for db_slug in all_db_slugs {
                if !existing_slugs.contains(&db_slug) {
                    info!("Deleting orphaned article: {}", db_slug);
                    if let Some(metadata) = content_metadata::Entity::find()
                        .filter(content_metadata::Column::Slug.eq(&db_slug))
                        .one(conn)
                        .await?
                    {
                        // 删除关联的 content 记录
                        content::Entity::delete_many()
                            .filter(content::Column::ContentMetadataId.eq(metadata.id))
                            .exec(conn)
                            .await?;
                        // 删除关联的标签关联
                        content_metadata_tag::Entity::delete_many()
                            .filter(content_metadata_tag::Column::ContentMetadataId.eq(metadata.id))
                            .exec(conn)
                            .await?;
                        // 删除 metadata
                        content_metadata::Entity::delete_by_id(metadata.id).exec(conn).await?;
                    }
                }
            }
        }
    }
    Ok(())
}

pub async fn update_article(conn: &impl ConnectionTrait, mut markdown: Markdown, lang: Lang) -> Result<()> {
    let (toc, rendered_html) = markdown.render_to_html()?;
    let cover_images = markdown.extract_images()?;
    markdown.strip_images()?;
    markdown.auto_format()?;

    // 更新 cover_images 在 content_metadata 中
    let slug = markdown.slug().to_string();
    if let Some(metadata) = content_metadata::Entity::find()
        .filter(content_metadata::Column::Slug.eq(&slug))
        .one(conn)
        .await?
    {
        let mut metadata_model: content_metadata::ActiveModel = metadata.into();
        metadata_model.cover_images = Set(cover_images.into());
        metadata_model.updated_at = Set(chrono::Local::now());
        content_metadata::Entity::update(metadata_model).exec(conn).await?;
    }

    match markdown {
        Markdown::Article {
            slug, title, content, ..
        } => {
            content::Entity::update_many()
                .filter(
                    content::Column::LangCode.eq(lang.to_string()).and(
                        content::Column::ContentMetadataId.in_subquery(
                            content_metadata::Entity::find()
                                .select_only()
                                .column(content_metadata::Column::Id)
                                .filter(content_metadata::Column::Slug.eq(&slug))
                                .into_query(),
                        ),
                    ),
                )
                .col_expr(content::Column::Title, Expr::value(title))
                .col_expr(content::Column::OriginalText, Expr::value(content))
                .col_expr(content::Column::RenderedHtml, Expr::value(rendered_html))
                .col_expr(content::Column::Toc, Expr::value(toc))
                .exec(conn)
                .await?;
        }
        Markdown::Short {
            slug, title, content, ..
        } => {
            content::Entity::update_many()
                .filter(
                    content::Column::LangCode.eq(lang.to_string()).and(
                        content::Column::ContentMetadataId.in_subquery(
                            content_metadata::Entity::find()
                                .select_only()
                                .column(content_metadata::Column::Id)
                                .filter(content_metadata::Column::Slug.eq(&slug))
                                .into_query(),
                        ),
                    ),
                )
                .col_expr(content::Column::Title, Expr::value(title))
                .col_expr(content::Column::OriginalText, Expr::value(content))
                .exec(conn)
                .await?;
        }
    };
    Ok(())
}

pub async fn get_comments_by_slug(conn: &DatabaseConnection, slug: &str) -> Result<Vec<Comment>> {
    let metadata_id = get_metadata_id_for_slug(slug, conn).await?;
    let parent_comments = comment::Entity::find()
        .filter(
            comment::Column::ContentMetadataId
                .eq(metadata_id)
                .and(comment::Column::ParentId.is_null()),
        )
        .order_by_desc(comment::Column::CreatedAt)
        .all(conn)
        .await?;
    let child_comments = comment::Entity::find()
        .filter(
            comment::Column::ContentMetadataId
                .eq(metadata_id)
                .and(comment::Column::ParentId.is_not_null()),
        )
        .order_by_asc(comment::Column::CreatedAt)
        .all(conn)
        .await?;
    let identity_ids = parent_comments
        .iter()
        .map(|c| c.identity_id)
        .chain(child_comments.iter().map(|c| c.identity_id))
        .collect::<Vec<_>>();
    let identities = suwen_entity::identity::Entity::find()
        .filter(suwen_entity::identity::Column::Id.is_in(identity_ids))
        .find_also_related(suwen_entity::user::Entity)
        .all(conn)
        .await?;
    let identity_map: HashMap<i32, IdentityInfo> =
        identities.into_iter().map(|item| (item.0.id, item.into())).collect();
    let comments: Result<Vec<Comment>> = parent_comments
        .into_iter()
        .map(|c| match identity_map.get(&c.identity_id) {
            Some(identity) => Ok((identity.clone(), c).into()),
            None => Err(anyhow::anyhow!("Identity not found for comment {}", c.id)),
        })
        .collect();
    let mut comments = comments?;
    let mut comment_map: HashMap<i32, &mut Comment> = comments.iter_mut().map(|c| (c.id, c)).collect();
    for reply in child_comments {
        // safety: parent_id must exist in reply_comments
        let parent_id = reply.parent_id.unwrap();
        let Some(parent_comment) = comment_map.get_mut(&parent_id) else {
            bail!("Parent {} comment not found for reply {}", parent_id, reply.id);
        };
        match identity_map.get(&reply.identity_id) {
            Some(identity) => parent_comment.replies.push((identity.clone(), reply).into()),
            None => {
                bail!("Identity not found for comment {}", reply.id);
            }
        }
    }
    Ok(comments)
}

pub async fn get_sitemap_articles(conn: &DatabaseConnection, lang: Lang) -> Result<Vec<SitemapUrl>> {
    Ok(content_metadata::Entity::find()
        .select_only()
        .columns([content_metadata::Column::Slug, content_metadata::Column::UpdatedAt])
        .inner_join(content::Entity)
        .filter(
            content::Column::LangCode
                .eq(lang.to_string())
                .and(content_metadata::Column::ContentType.eq("article"))
                .and(content_metadata::Column::PublishedAt.is_not_null()),
        )
        .order_by_desc(content_metadata::Column::UpdatedAt)
        .into_model::<SitemapUrl>()
        .all(conn)
        .await?)
}
