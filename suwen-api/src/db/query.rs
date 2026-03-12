use std::collections::HashMap;

use anyhow::{Context, Result, bail, ensure};
use chrono::Datelike;
use sea_orm::ActiveValue::{NotSet, Set};
use sea_orm::{
    ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, JoinType, QueryFilter, QueryOrder, QuerySelect,
    RelationTrait, TransactionTrait,
};
use suwen_entity::*;
use suwen_llm::generate_article_summary;
use suwen_markdown::{Markdown, MarkdownChange};
use suwen_migration::Expr;

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
                    icon: "https://obj.amto.cc/icon/github.svg".into(),
                    url: "https://github.com/amtoaer".into(),
                },
                RelatedLink {
                    name: "Telegram".into(),
                    icon: "https://obj.amto.cc/icon/telegram.svg".into(),
                    url: "https://t.me/amtoaer".into(),
                },
                RelatedLink {
                    name: "GMail".into(),
                    icon: "https://obj.amto.cc/icon/gmail.svg".into(),
                    url: "mailto:amtoaer@gmail.com".into(),
                },
                RelatedLink {
                    name: "X".into(),
                    icon: "https://obj.amto.cc/icon/x.svg".into(),
                    url: "https://x.com/amtoaer".into(),
                },
                RelatedLink {
                    name: "Bilibili".into(),
                    icon: "https://obj.amto.cc/icon/bilibili.svg".into(),
                    url: "https://space.bilibili.com/9183758".into(),
                },
                RelatedLink {
                    name: "Steam".into(),
                    icon: "https://obj.amto.cc/icon/steam.svg".into(),
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
        .column_as(content::Column::RenderedHtml, "rendered_html")
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
        .column_as(content::Column::RenderedHtml, "rendered_html")
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
    Ok(content_metadata_tag::Entity::find()
        .select_only()
        .column(content_metadata_tag::Column::TagName)
        .column_as(content_metadata_tag::Column::ContentMetadataId.count(), "count")
        .group_by(content_metadata_tag::Column::TagName)
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
    Ok(content_metadata_tag::Entity::find()
        .select_only()
        .column(content_metadata_tag::Column::TagName)
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
            content_metadata_tag::Column::TagName.eq(tag_name).and(
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

pub async fn handle_markdown_change(conn: &DatabaseConnection, change: MarkdownChange) -> Result<()> {
    match change {
        MarkdownChange::Upsert(mut markdown) => {
            let slug = markdown.slug().to_owned();
            let cover_images = markdown.extract_images()?;
            markdown.strip_images()?;
            markdown.auto_format()?;
            let content_hash = markdown.hash();
            let existing = content_metadata::Entity::find()
                .filter(content_metadata::Column::Slug.eq(&slug))
                .one(conn)
                .await?;
            if let Some(metadata) = &existing
                && content_hash == metadata.content_hash
            {
                info!("Content hash unchanged, skipping update: {}", &slug);
                return Ok(());
            }
            let summary = generate_article_summary(&markdown).await?;
            let (toc, rendered_html) = markdown.render_to_html()?;
            let txn = conn.begin().await?;
            match existing {
                Some(metadata) => {
                    info!("Article already exists, updating: {}", &slug);
                    update_article_internal(
                        markdown,
                        metadata,
                        cover_images,
                        summary,
                        toc,
                        rendered_html,
                        content_hash,
                        &txn,
                    )
                    .await?;
                }
                None => {
                    info!("Article does not exist, creating: {}", &slug);
                    create_article_internal(markdown, cover_images, summary, toc, rendered_html, content_hash, &txn)
                        .await?;
                }
            }
            txn.commit().await?;
            info!("Article upserted: {}", &slug);
        }
        MarkdownChange::Deleted(slug) => {
            info!("Deleting article: {}", slug);
            content_metadata::Entity::delete_many()
                .filter(content_metadata::Column::Slug.eq(&slug))
                .exec(conn)
                .await?;
        }
        MarkdownChange::SyncExisting(existing_slugs) => {
            info!("Syncing existing articles, found {} files", existing_slugs.len());
            content_metadata::Entity::delete_many()
                .filter(content_metadata::Column::Slug.is_not_in(existing_slugs))
                .exec(conn)
                .await?;
        }
        MarkdownChange::Renamed(old_slug, new_slug) => {
            info!("Renaming article from {} to {}", old_slug, new_slug);
            content_metadata::Entity::update_many()
                .filter(content_metadata::Column::Slug.eq(&old_slug))
                .col_expr(content_metadata::Column::Slug, Expr::value(new_slug))
                .exec(conn)
                .await?;
        }
    }
    Ok(())
}

async fn create_article_internal(
    markdown: Markdown,
    cover_images: Vec<String>,
    summary: Option<String>,
    toc: Option<suwen_entity::Toc>,
    rendered_html: Option<String>,
    content_hash: String,
    conn: &impl ConnectionTrait,
) -> Result<()> {
    let metadata = content_metadata::ActiveModel {
        slug: Set(markdown.slug().to_owned()),
        content_hash: Set(content_hash),
        content_type: Set(markdown.content_type().to_owned()),
        cover_images: Set(cover_images.into()),
        tags: Set(markdown.tags().into()),
        view_count: Set(0),
        comment_count: Set(0),
        created_at: markdown.created_at().map(Set).unwrap_or(NotSet),
        updated_at: markdown.updated_at().map(Set).unwrap_or(NotSet),
        published_at: markdown.published_at().map(|dt| Set(Some(dt))).unwrap_or(NotSet),
        original_lang: Set(markdown.lang().to_string()),
        ..Default::default()
    };
    let metadata_id = content_metadata::Entity::insert(metadata)
        .exec(conn)
        .await?
        .last_insert_id;

    let content = content::ActiveModel {
        title: Set(markdown.title().to_owned()),
        original_text: Set(markdown.content().to_owned()),
        rendered_html: Set(rendered_html),
        toc: Set(toc),
        lang_code: Set(markdown.lang().to_string()),
        summary: Set(summary),
        content_metadata_id: Set(metadata_id),
        ..Default::default()
    };
    content::Entity::insert(content).exec(conn).await?;

    let content_tags = markdown
        .tags()
        .into_iter()
        .map(|tag| content_metadata_tag::ActiveModel {
            content_metadata_id: Set(metadata_id),
            tag_name: Set(tag),
        })
        .collect::<Vec<_>>();
    if !content_tags.is_empty() {
        content_metadata_tag::Entity::insert_many(content_tags)
            .exec(conn)
            .await?;
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn update_article_internal(
    markdown: Markdown,
    metadata: content_metadata::Model,
    cover_images: Vec<String>,
    summary: Option<String>,
    toc: Option<suwen_entity::Toc>,
    rendered_html: Option<String>,
    content_hash: String,
    conn: &impl ConnectionTrait,
) -> Result<()> {
    let metadata_id = metadata.id;
    let metadata = content_metadata::ActiveModel {
        content_hash: Set(content_hash),
        content_type: Set(markdown.content_type().to_owned()),
        cover_images: Set(cover_images.into()),
        tags: Set(markdown.tags().into()),
        updated_at: Set(chrono::Local::now()),
        original_lang: Set(markdown.lang().to_string()),
        ..metadata.into()
    };
    content_metadata::Entity::update(metadata).exec(conn).await?;

    let content = content::Entity::find()
        .filter(
            content::Column::ContentMetadataId
                .eq(metadata_id)
                .and(content::Column::LangCode.eq(markdown.lang().to_string())),
        )
        .one(conn)
        .await?
        .context("no article found")?;
    let content = content::ActiveModel {
        title: Set(markdown.title().to_owned()),
        original_text: Set(markdown.content().to_owned()),
        rendered_html: Set(rendered_html),
        toc: Set(toc),
        summary: Set(summary),
        ..content.into()
    };
    content::Entity::update(content).exec(conn).await?;

    content_metadata_tag::Entity::delete_many()
        .filter(content_metadata_tag::Column::ContentMetadataId.eq(metadata_id))
        .exec(conn)
        .await?;
    let content_tags = markdown
        .tags()
        .into_iter()
        .map(|tag| content_metadata_tag::ActiveModel {
            content_metadata_id: Set(metadata_id),
            tag_name: Set(tag),
        })
        .collect::<Vec<_>>();
    if !content_tags.is_empty() {
        content_metadata_tag::Entity::insert_many(content_tags)
            .exec(conn)
            .await?;
    }
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
