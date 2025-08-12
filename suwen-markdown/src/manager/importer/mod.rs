use anyhow::{Context, Result, bail};
use chrono::{DateTime, Local};
use futures::{TryStreamExt, stream::FuturesUnordered};
use lol_html::{HtmlRewriter, Settings, element};
use pulldown_cmark::{Event, HeadingLevel, Tag, TagEnd, html};
use pulldown_cmark_to_cmark::cmark_resume;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf, sync::LazyLock};
use suwen_entity::{Toc, TocItem};
use tokio::{
    fs::{self, create_dir_all},
    task::JoinSet,
};
use two_face::theme::EmbeddedThemeName;

mod xlog;

pub use xlog::import_file as XlogImporter;

use crate::{highlighter::Highlighter, parse_markdown};

static HIGHLIGHTER: LazyLock<Highlighter> = LazyLock::new(Highlighter::new);

pub async fn import_path<T, F>(
    source: PathBuf,
    output: PathBuf,
    obj_output: Option<PathBuf>,
    importer: T,
) -> Result<()>
where
    T: Fn(PathBuf, PathBuf, PathBuf) -> F,
    F: Future<Output = Result<Markdown>> + Send + 'static,
{
    create_dir_all(&output).await?;
    let obj_output = obj_output.unwrap_or_else(|| output.join("objects"));
    create_dir_all(&obj_output).await?;
    let mut join_set = JoinSet::new();
    for file in collect_files(source).await? {
        join_set.spawn(importer(file, output.clone(), obj_output.clone()));
    }
    let write_task = join_set
        .join_all()
        .await
        .into_iter()
        .filter_map(Result::ok)
        .filter_map(|result| {
            let target = output.join(format!("{}.md", result.slug()));
            if let Ok(str) = result.to_string() {
                Some(async move { fs::write(&target, str).await })
            } else {
                None
            }
        })
        .collect::<FuturesUnordered<_>>();
    Ok(write_task.try_collect().await?)
}

async fn collect_files(source: PathBuf) -> Result<Vec<PathBuf>> {
    let (mut dirs, mut files) = (vec![source], Vec::new());
    while let Some(dir) = dirs.pop() {
        let mut entries = fs::read_dir(&dir).await?;
        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if path.is_dir() {
                dirs.push(path);
            } else if path.is_file() {
                files.push(path);
            }
        }
    }
    Ok(files)
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum Markdown {
    Article {
        slug: String,
        title: String,
        cover_images: Vec<String>,
        tags: Vec<String>,
        #[serde(skip)]
        content: String,
        created_at: DateTime<Local>,
        updated_at: DateTime<Local>,
        published_at: DateTime<Local>,
    },
    Short {
        slug: String,
        title: String,
        cover_images: Vec<String>,
        #[serde(skip)]
        content: String,
        created_at: DateTime<Local>,
        updated_at: DateTime<Local>,
        published_at: DateTime<Local>,
    },
}

impl Markdown {
    pub(super) fn slug(&self) -> &str {
        match self {
            Markdown::Article { slug, .. } | Markdown::Short { slug, .. } => slug,
        }
    }

    fn content(&self) -> &str {
        match self {
            Markdown::Article { content, .. } | Markdown::Short { content, .. } => content,
        }
    }

    pub(super) fn to_string(&self) -> Result<String> {
        let metadata = serde_json::to_string_pretty(self)?;
        match self {
            Markdown::Article { content, .. } | Markdown::Short { content, .. } => {
                Ok(format!("---\n{}\n---\n{}", metadata, content))
            }
        }
    }

    pub(super) fn from_string(input: &str) -> Result<Self> {
        let parts = input.splitn(3, "---\n").collect::<Vec<_>>();
        if parts.len() != 3 {
            bail!("Invalid markdown format: missing metadata or content");
        }
        let mut metadata: Markdown = serde_json::from_str(parts[1])?;
        let article = parts[2].to_string();
        match &mut metadata {
            Markdown::Article { content, .. } | Markdown::Short { content, .. } => {
                *content = article;
            }
        }
        Ok(metadata)
    }

    /// 替换 slug，包括修改 metadata 中的 slug，更新 metadata、正文中的图片引用地址
    pub(super) fn rename_slug(&mut self, new_slug: &str) -> Result<()> {
        let old_slug = self.slug().to_string();
        match self {
            Markdown::Article {
                slug, cover_images, ..
            }
            | Markdown::Short {
                slug, cover_images, ..
            } => {
                *slug = new_slug.to_string();
                cover_images.iter_mut().for_each(|image| {
                    *image = image.replacen(&old_slug, new_slug, 1);
                });
            }
        }
        if let Markdown::Article { content, .. } = self {
            let mut events = parse_markdown(content)?;
            let old_slug = old_slug.as_str();
            for event in events.iter_mut() {
                match event {
                    Event::Start(Tag::Image { dest_url, .. }) => {
                        *dest_url = dest_url.replacen(old_slug, new_slug, 1).into();
                    }
                    Event::Html(html_content) | Event::InlineHtml(html_content) => {
                        let mut buf = Vec::new();
                        let mut rewriter = HtmlRewriter::new(
                            Settings {
                                element_content_handlers: vec![element!(
                                    "source[src]",
                                    move |el| {
                                        let video_url = el.get_attribute("src").unwrap_or_default();
                                        el.set_attribute(
                                            "src",
                                            &video_url.replacen(old_slug, new_slug, 1),
                                        )?;
                                        Ok(())
                                    }
                                )],
                                ..Settings::new()
                            },
                            |c: &[u8]| buf.extend_from_slice(c),
                        );
                        rewriter.write(html_content.as_bytes())?;
                        rewriter.end()?;
                        *html_content = String::from_utf8(buf)
                            .context("Failed to convert HTML to string")?
                            .into();
                    }
                    _ => {}
                }
            }
            let mut buf = String::new();
            cmark_resume(events.into_iter(), &mut buf, None).context("Failed to resume cmark")?;
            *content = buf;
        }
        Ok(())
    }

    pub(super) fn replace_images(&mut self, replace_pair: &HashMap<String, String>) -> Result<()> {
        match self {
            Markdown::Article { cover_images, .. } | Markdown::Short { cover_images, .. } => {
                cover_images.iter_mut().for_each(|image| {
                    if let Some(new_image) = replace_pair.get(image) {
                        *image = new_image.clone();
                    }
                });
            }
        }
        if let Markdown::Article { content, .. } = self {
            let mut events = parse_markdown(content)?;
            events.iter_mut().for_each(|event| {
                if let Event::Start(Tag::Image { dest_url, .. }) = event {
                    if let Some(new_url) = replace_pair.get(dest_url.as_ref()) {
                        *dest_url = new_url.clone().into();
                    }
                }
            });
            let mut buf = String::new();
            cmark_resume(events.into_iter(), &mut buf, None).context("Failed to resume cmark")?;
            *content = buf;
        }
        Ok(())
    }

    pub fn render_to_html(&self) -> Result<(Option<Toc>, Option<String>)> {
        if matches!(self, Markdown::Short { .. }) {
            return Ok((None, None));
        }
        let mut events = parse_markdown(&self.content())?;
        let mut toc_item = TocItem {
            id: String::new(),
            text: String::new(),
            level: 0,
        };
        let (mut start_handled, mut text_handled, mut in_heading) = (false, false, false);
        let (mut head_count, mut head_level) = (0, HeadingLevel::H1);
        let (mut toc, mut stack) = (Vec::new(), Vec::new());
        events.iter_mut().for_each(|mut event| match &mut event {
            Event::Start(Tag::Heading { level, id, .. }) => {
                head_count += 1;
                let generated_id = format!("heading-{}", head_count);
                (*id, toc_item.id) = (Some(generated_id.clone().into()), generated_id);
                head_level = *level;
                start_handled = true;
                in_heading = true;
            }
            Event::Text(text) | Event::Code(text) if in_heading => {
                toc_item.text += text;
                text_handled = true;
            }
            Event::End(TagEnd::Heading(level)) => {
                if in_heading && start_handled && text_handled && head_level == *level {
                    while let Some(last) = stack.last()
                        && *last <= head_level
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
        });
        let highlighted_events =
            HIGHLIGHTER.highlight(EmbeddedThemeName::Leet, events.into_iter())?;
        let mut buf = String::new();
        html::push_html(&mut buf, highlighted_events.into_iter());
        Ok((Some(toc.into()), Some(buf)))
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::read_to_string, path::PathBuf};

    use crate::{
        manager::importer::{XlogImporter, import_path},
        parse_markdown,
    };

    #[ignore = "only for manual test"]
    #[tokio::test]
    async fn test_format_path() {
        let _ = import_path(
            PathBuf::from("/Users/amtoaer/Downloads/Zen/amtoaer/notes"),
            PathBuf::from("/Users/amtoaer/Downloads/Zen/amtoaer/notes-imported"),
            None,
            XlogImporter,
        )
        .await;
    }

    #[ignore = "only for manual test"]
    #[test]
    fn test_parse_markdown() {
        let content = read_to_string("/Users/amtoaer/Downloads/Zen/amtoaer/notes-imported_副本/jie-ba-6-xiang-jie-di-yi-tan--quan-jiao-de-zheng-ti-jie-shao.md").unwrap();
        let events = parse_markdown(&content);
        dbg!(events);
    }
}
