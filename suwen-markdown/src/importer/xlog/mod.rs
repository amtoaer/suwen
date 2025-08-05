mod schema;

use std::{borrow::Cow, fs::read_to_string, path::Path, sync::LazyLock};

use crate::{
    format_markdown,
    importer::{Article, ImportResult, Short, xlog::schema::Content},
    parse_markdown,
};
use anyhow::{Context, Result};
use pulldown_cmark::{Event, Tag};
use regex::{Captures, Regex};
use yaml_rust2::YamlLoader;

pub fn import_file(path: &Path) -> Result<ImportResult> {
    if !path.extension().is_some_and(|ext| ext == "json") {
        return Ok(ImportResult::None);
    }
    let json = read_to_string(path)?;
    let Ok(mut json) = serde_json::from_str::<Content>(&json) else {
        return Ok(ImportResult::None);
    };
    // 提取 slug
    let slug = json
        .metadata
        .content
        .attributes
        .iter()
        .find_map(|attr| {
            if attr.trait_type == "xlog_slug" {
                attr.value.as_str().map(String::from)
            } else {
                None
            }
        })
        .context("Slug not found")?;
    // 格式化标题和内容，并将 ipfs 替换成可访问的 URL
    json.metadata.content.content = format_markdown(&process_ipfs(&json.metadata.content.content));
    let mut cover_images = Vec::new();
    json.metadata.content.attachments.into_iter().for_each(|a| {
        let image = process_ipfs(&a.address).into_owned();
        if !image.is_empty() {
            cover_images.push(image);
        }
    });
    json.metadata.content.title = format_markdown(&json.metadata.content.title);
    Ok(
        match json.metadata.content.tags.first().map(|s| s.as_str()) {
            Some("short") => ImportResult::Short(Short {
                slug,
                title: json.metadata.content.title,
                cover_images,
                content: json.metadata.content.content,
                created_at: json.created_at,
                updated_at: json.updated_at,
                published_at: json.published_at,
            }),
            Some("post") => {
                // 尝试检测 front matter，如果有直接跳过
                let parts: Vec<&str> = json.metadata.content.content.splitn(3, "---").collect();
                let content = if parts.len() == 3 && YamlLoader::load_from_str(parts[1]).is_ok() {
                    Cow::Owned(String::from(parts[0]) + parts[2])
                } else {
                    Cow::Borrowed(&json.metadata.content.content)
                };
                let mut events = parse_markdown(&content)?;
                events.0.iter_mut().for_each(|mut event| {
                    if let Event::Start(Tag::Image { dest_url, .. }) = &mut event {
                        if !dest_url.contains("ipfs") {
                            *dest_url = ("/proxy?url=".to_string() + dest_url).into();
                        }
                        cover_images.push(dest_url.to_string());
                    }
                });
                let rendered_html = events.into();
                ImportResult::Article(Article {
                    slug,
                    title: json.metadata.content.title,
                    cover_images,
                    tags: json.metadata.content.tags.into_iter().skip(1).collect(),
                    content: content.into_owned(),
                    rendered_html,
                    created_at: json.created_at,
                    updated_at: json.updated_at,
                    published_at: json.published_at,
                })
            }
            _ => ImportResult::None,
        },
    )
}

fn process_ipfs(input: &str) -> Cow<'_, str> {
    static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"ipfs://([a-zA-Z0-9]+)").unwrap());
    RE.replace_all(input, |caps: &Captures| {
        format!(
            "https://ipfs.crossbell.io/ipfs/{}?img-quality=75&img-format=webp&img-onerror=redirect",
            &caps[1]
        )
    })
}
