use anyhow::{Result, bail};
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::path::Path;

pub mod xlog;

pub fn import_path(
    path: &Path,
    importer: &impl Fn(&Path) -> Result<ImportResult>,
) -> Result<(Vec<Article>, Vec<Short>)> {
    let (mut articles, mut shorts) = (Vec::new(), Vec::new());
    for entry in path.read_dir()? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let (sub_articles, sub_shorts) = import_path(&path, importer)?;
            articles.extend(sub_articles);
            shorts.extend(sub_shorts);
        } else {
            match importer(&path) {
                Err(e) => {
                    bail!("Failed to import {}: {}", path.display(), e);
                }
                Ok(ImportResult::Article(article)) => articles.push(article),
                Ok(ImportResult::Short(short)) => shorts.push(short),
                Ok(ImportResult::None) => {}
            }
        }
    }
    articles.sort_by_key(|a| a.published_at);
    shorts.sort_by_key(|s| s.published_at);
    Ok((articles, shorts))
}

pub enum ImportResult {
    Article(Article),
    Short(Short),
    None,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Article {
    pub slug: String,
    pub title: String,
    pub cover_images: Vec<String>,
    pub tags: Vec<String>,
    pub content: String,
    pub rendered_html: String,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
    pub published_at: DateTime<Local>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Short {
    pub slug: String,
    pub title: String,
    pub cover_images: Vec<String>,
    pub content: String,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
    pub published_at: DateTime<Local>,
}
