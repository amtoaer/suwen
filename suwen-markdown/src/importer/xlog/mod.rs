mod content;
mod front_matter;
mod upload_attachment;

use std::path::Path;

use crate::importer::{
    ImportResult,
    xlog::{
        content::{parse_article, parse_short},
        front_matter::parse_front_matter,
    },
};
use anyhow::{Context, Result};
use yaml_rust2::YamlLoader;

pub fn import_file(path: &Path) -> Result<ImportResult> {
    if !path.extension().is_some_and(|ext| ext == "md") {
        return Ok(ImportResult::None);
    }
    let content = std::fs::read_to_string(path)?;
    // front matter 在开头的两个 `---` 之间，这样拆分后
    // parts[0] 是空字符串，parts[1] 是 front matter，parts[2] 是正文内容
    let parts: Vec<&str> = content.splitn(3, "---").collect();
    if parts.len() != 3 {
        return Ok(ImportResult::None);
    }
    let front_matter = YamlLoader::load_from_str(parts[1])?
        .into_iter()
        .next()
        .context("Failed to parse front matter")?;
    let Ok(front_matter) = parse_front_matter(front_matter) else {
        // 解析评论的 front matter 时会出现这个错误，直接忽略就好
        return Ok(ImportResult::None);
    };
    match front_matter
        .tags
        .first()
        .context("article type not found")?
        .trim()
    {
        "post" => Ok(ImportResult::Article(parse_article(
            front_matter,
            path,
            parts[2],
        )?)),
        "short" => Ok(ImportResult::Short(parse_short(
            front_matter,
            path,
            parts[2],
        )?)),
        _ => Ok(ImportResult::None),
    }
}
