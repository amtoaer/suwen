use std::{borrow::Cow, path::Path, sync::LazyLock};

use crate::{
    format_markdown,
    importer::{
        Article, Short,
        xlog::{
            front_matter::FrontMatter,
            upload_attachment::{get_uploaded, upload_attachment},
        },
    },
    parse_markdown,
};
use anyhow::{Result, bail};
use pulldown_cmark::{Event, Tag};
use regex::Regex;
use yaml_rust2::YamlLoader;

static IMAGE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"!\[.*?\]\((.*?)\)").expect("Failed to compile image regex"));

pub(super) fn parse_article(
    front_matter: FrontMatter,
    path: &Path,
    content: &str,
) -> Result<Article> {
    let mut cover_images = Vec::new();
    // 如果文章本身是从其它平台导入到 xlog 的，很可能还会在标题后面跟着一个额外的 front matter
    let parts: Vec<&str> = content.splitn(3, "---").collect();
    let content = if parts.len() == 3 && YamlLoader::load_from_str(parts[1]).is_ok() {
        // 如果确定中间仍然是一个合法的 yaml，视作多余的 front matter 直接跳过
        Cow::Owned(String::from(parts[0]) + parts[2])
    } else {
        content.into()
    };
    let copied_file = upload_attachment(path)?;
    let lines = content.lines().collect::<Vec<_>>();
    let (mut content_start, mut content_end) = (0, lines.len());
    for (idx, line) in lines.iter().rev().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        // 文章的结尾可能会有两种情况：
        // 1. 有封面图片，格式为 ![](./attachments/xxx)
        // 2. 无封面照片，格式为 []()
        if let Some(image) = IMAGE_REGEX.captures(line).and_then(|c| c.get(1)) {
            if let Some(uploaded) = get_uploaded(&copied_file, image.as_str()) {
                cover_images.push(uploaded);
            }
        } else if line.trim() != "[]()" {
            bail!("Unexpected content in article: {}", line);
        }
        // 无论如何，封面照片都应该跳过，不被包含在正文中
        content_end -= idx + 1;
        break;
    }
    for (idx, line) in lines.iter().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        // 找到第一行标题，这行标题和 front matter 上面的 title 是重复的，需要去掉吧避免重复
        if !line.starts_with("# ") {
            bail!("Expected title line to start with '# ', found: {}", line);
        }
        content_start = idx + 1;
        break;
    }
    let content = if content_start >= content_end {
        // 正文是空的
        String::new()
    } else {
        // 正文是非空的，去掉首尾空格后跑一下格式化
        format_markdown(lines[content_start..content_end].join("\n").trim())
    };
    let mut events = parse_markdown(&content)?;
    events.0 = events
        .0
        .into_iter()
        .map(|mut event| {
            if let Event::Start(Tag::Image { dest_url, .. }) = &mut event {
                if let Some(uploaded) = get_uploaded(&copied_file, &dest_url) {
                    *dest_url = uploaded.clone().into();
                    cover_images.push(uploaded);
                }
            }
            event
        })
        .collect();
    Ok(Article {
        slug: front_matter.slug,
        title: front_matter.title,
        cover_images,
        tags: front_matter.tags.into_iter().skip(1).collect(),
        rendered_html: events.into(),
        content,
        published_at: front_matter.published_at,
    })
}

pub(super) fn parse_short(front_matter: FrontMatter, path: &Path, content: &str) -> Result<Short> {
    let mut cover_images = Vec::new();
    let copied_file = upload_attachment(path)?;
    let lines = content.lines().collect::<Vec<_>>();
    let (mut content_start, mut content_end) = (0, lines.len());
    for (idx, line) in lines.iter().rev().enumerate() {
        // 从后向前跳过空行和封面，将匹配到的封面添加到 cover_images 中
        if line.trim().is_empty() {
            continue;
        }
        if let Some(image) = IMAGE_REGEX.captures(line).and_then(|c| c.get(1)) {
            if let Some(uploaded) = get_uploaded(&copied_file, image.as_str()) {
                cover_images.push(uploaded);
            }
        }
        content_end -= idx;
        break;
    }
    for (idx, line) in lines.iter().enumerate() {
        // 正向部分同理，移除第一个与 front matter 重合的标题行
        if line.trim().is_empty() {
            continue;
        }
        if !line.starts_with("# ") {
            bail!("Expected title line to start with '# ', found: {}", line);
        }
        content_start = idx + 1;
        break;
    }
    let content = if content_start >= content_end {
        String::new()
    } else {
        format_markdown(lines[content_start..content_end].join("\n").trim())
    };
    // 封面是逆向添加的，需要反转一下
    cover_images.reverse();
    Ok(Short {
        slug: front_matter.slug,
        title: front_matter.title,
        cover_images,
        content,
        published_at: front_matter.published_at,
    })
}
