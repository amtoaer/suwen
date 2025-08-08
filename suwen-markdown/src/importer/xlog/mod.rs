mod schema;

use std::{borrow::Cow, fs::read_to_string, path::Path, sync::LazyLock};

use crate::{
    format_markdown,
    importer::{Article, ImportResult, Short, xlog::schema::Content},
    parse_markdown,
};
use anyhow::{Context, Result};
use pulldown_cmark::{Event, HeadingLevel, Tag, TagEnd};
use regex::{Captures, Regex};
use suwen_entity::TocItem;
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
                let mut toc_item = TocItem {
                    id: String::new(),
                    text: String::new(),
                    level: 0,
                };
                let (mut start_handled, mut text_handled, mut in_heading) = (false, false, false);
                let (mut head_count, mut head_level) = (0, HeadingLevel::H1);
                let (mut toc, mut stack) = (Vec::new(), Vec::new());
                // 对事件进行处理，包括生成目录、替换图片链接、为 video 加入 control 等
                events.0.iter_mut().for_each(|mut event| {
                    match &mut event {
                        // 在上一步全局替换中已经将 ipfs 资源替换成可访问的 URL，这里的替换是将其它资源替换成使用本地 proxy
                        Event::Start(Tag::Image { dest_url, .. }) => {
                            if !dest_url.contains("ipfs") {
                                *dest_url = ("/proxy?url=".to_string() + dest_url).into();
                            }
                            cover_images.push(dest_url.to_string());
                        }
                        // xlog 的视频只有 <video> 标签而没有 controls 属性，手动尝试添加
                        Event::Html(content) => {
                            *content = content.replace("<video>", "<video controls>").into();
                        }
                        // 标题开头，设置标题 id，记录 heading 层级
                        Event::Start(Tag::Heading { level, id, .. }) => {
                            head_count += 1;
                            let generated_id = format!("heading-{}", head_count);
                            (*id, toc_item.id) = (Some(generated_id.clone().into()), generated_id);
                            head_level = *level;
                            start_handled = true;
                            in_heading = true;
                        }
                        // 标题文本，设置文本内容
                        Event::Text(text) | Event::Code(text) if in_heading => {
                            toc_item.text += text;
                            text_handled = true;
                        }
                        Event::End(TagEnd::Heading(level)) => {
                            if in_heading && start_handled && text_handled && head_level == *level {
                                while let Some(last) = stack.last()
                                    && *last >= head_level
                                {
                                    stack.pop();
                                }
                                toc_item.level = stack.len();
                                stack.push(head_level);
                                toc.push(toc_item.clone());
                            }
                            (start_handled, text_handled, in_heading) = (false, false, false);
                            toc_item.text.clear();
                        }
                        _ => {}
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
                    toc: toc.into(),
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
