use anyhow::{Context, Result};
use chrono::{DateTime, Local};
use yaml_rust2::Yaml;

pub(super) struct FrontMatter {
    pub slug: String,
    pub title: String,
    pub tags: Vec<String>,
    pub published_at: DateTime<Local>,
}

pub(super) fn parse_front_matter(front_matter: Yaml) -> Result<FrontMatter> {
    let slug = front_matter["attributes"]
        .as_vec()
        .and_then(|v| {
            v.iter()
                .find(|x| x["trait_type"].as_str().is_some_and(|v| v == "xlog_slug"))
                .and_then(|x| x["value"].as_str().map(String::from))
        })
        .context("slug not found")?;
    let title = front_matter["title"]
        .as_str()
        .context("title not found")?
        .to_string();
    let tags = front_matter["tags"]
        .as_vec()
        .context("tags not found")?
        .iter()
        .filter_map(|x| x.as_str().map(String::from))
        .collect();
    let published_at = front_matter["date_published"]
        .as_str()
        .context("published_at not found")?
        .parse::<DateTime<Local>>()
        .context("failed to parse published_at")?;
    Ok(FrontMatter {
        slug,
        title,
        tags,
        published_at,
    })
}
