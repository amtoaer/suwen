use std::hash::Hasher;
use std::path::Path;
use std::sync::LazyLock;

use anyhow::{Context, Result, bail};
use chrono::{DateTime, Local};
use pulldown_cmark::{Event, HeadingLevel, Tag, TagEnd, html};
use pulldown_cmark_to_cmark::cmark_resume;
use serde::{Deserialize, Serialize};
use suwen_config::Lang;
use suwen_entity::{Toc, TocItem};
use twox_hash::XxHash3_64;

use crate::highlighter::Highlighter;
use crate::parse_markdown;

static HIGHLIGHTER: LazyLock<parking_lot::Mutex<Highlighter>> =
    LazyLock::new(|| parking_lot::Mutex::new(Highlighter::new()));

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum Markdown {
    Article {
        #[serde(default)]
        slug: String,
        title: String,
        tags: Vec<String>,
        #[serde(skip)]
        content: String,
        created_at: DateTime<Local>,
        updated_at: DateTime<Local>,
        published_at: DateTime<Local>,
        #[serde(skip)]
        lang: Lang,
    },
    Short {
        #[serde(default)]
        slug: String,
        title: String,
        #[serde(skip)]
        content: String,
        created_at: DateTime<Local>,
        updated_at: DateTime<Local>,
        published_at: DateTime<Local>,
        #[serde(skip)]
        lang: Lang,
    },
}
impl Markdown {
    pub fn slug(&self) -> &str {
        match self {
            Markdown::Article { slug, .. } | Markdown::Short { slug, .. } => slug,
        }
    }

    pub fn content(&self) -> &str {
        match self {
            Markdown::Article { content, .. } | Markdown::Short { content, .. } => content,
        }
    }

    pub fn title(&self) -> &str {
        match self {
            Markdown::Article { title, .. } | Markdown::Short { title, .. } => title,
        }
    }

    pub fn lang(&self) -> &Lang {
        match self {
            Markdown::Article { lang, .. } | Markdown::Short { lang, .. } => lang,
        }
    }

    pub fn tags(&self) -> Vec<String> {
        match self {
            Markdown::Article { tags, .. } => tags.clone(),
            Markdown::Short { .. } => Vec::new(),
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

    pub(super) fn from_string(input: &str, lang: Lang) -> Result<Self> {
        let parts = input.splitn(3, "---\n").collect::<Vec<_>>();
        if parts.len() != 3 {
            bail!("Invalid markdown format: missing metadata or content");
        }
        let mut metadata: Markdown = serde_json::from_str(parts[1])?;
        let article = parts[2].to_string();
        match &mut metadata {
            Markdown::Article {
                content, lang: m_lang, ..
            }
            | Markdown::Short {
                content, lang: m_lang, ..
            } => {
                *content = article;
                *m_lang = lang;
            }
        }
        Ok(metadata)
    }

    pub async fn from_file(path: impl AsRef<Path>, lang: Lang) -> Result<Self> {
        let path = path.as_ref();
        if path.extension().is_none_or(|ext| ext != "md") {
            bail!("File {:?} does not have .md extension", path);
        }
        let content = tokio::fs::read_to_string(path).await?;
        let mut markdown = Self::from_string(&content, lang)?;
        if let Some(new_slug) = path.file_stem().and_then(|s| s.to_str()) {
            match &mut markdown {
                Markdown::Article { slug, .. } | Markdown::Short { slug, .. } => {
                    *slug = new_slug.to_string();
                }
            }
        }
        Ok(markdown)
    }

    pub fn extract_images(&self) -> Result<Vec<String>> {
        let mut images = Vec::new();
        let events = parse_markdown(self.content())?;
        for event in events {
            if let Event::Start(Tag::Image { dest_url, .. }) = event {
                images.push(dest_url.to_string());
            }
        }
        Ok(images)
    }

    pub fn strip_images(&mut self) -> Result<()> {
        if let Markdown::Short { content, .. } = self {
            let events = parse_markdown(content)?;
            let mut filtered_events = Vec::new();
            let mut skip_next_end = false;
            for event in events {
                match event {
                    Event::Start(Tag::Image { .. }) => {
                        skip_next_end = true;
                        continue;
                    }
                    Event::End(TagEnd::Image) if skip_next_end => {
                        skip_next_end = false;
                        continue;
                    }
                    _ => {
                        filtered_events.push(event);
                    }
                }
            }
            let mut buf = String::new();
            cmark_resume(filtered_events.into_iter(), &mut buf, None)
                .context("Failed to resume cmark after stripping images")?;
            *content = buf;
        }
        Ok(())
    }

    pub fn auto_format(&mut self) -> Result<()> {
        match self {
            Markdown::Article { title, content, .. } | Markdown::Short { title, content, .. } => {
                *title = autocorrect::format_for(title, "markdown").out;
                *content = autocorrect::format_for(content, "markdown").out;
            }
        }
        Ok(())
    }

    pub fn render_to_html(&self) -> Result<(Option<Toc>, Option<String>)> {
        if matches!(self, Markdown::Short { .. }) {
            return Ok((None, None));
        }
        let mut events = parse_markdown(self.content())?;
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
        });
        let highlighted_events = HIGHLIGHTER.lock().highlight(events.into_iter())?;
        let mut buf = String::new();
        html::push_html(&mut buf, highlighted_events.into_iter());
        Ok((Some(toc.into()), Some(buf)))
    }

    pub fn hash(&self) -> String {
        let mut hasher = XxHash3_64::default();
        hasher.write(self.title().as_bytes());
        hasher.write(self.content().as_bytes());
        let hash = hasher.finish();

        format!("v1:{:x}/{}", hash, self.lang())
    }
}
