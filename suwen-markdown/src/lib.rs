#[macro_use]
extern crate tracing;

mod highlighter;
pub mod manager;

use anyhow::Result;
use std::{fs::create_dir_all, path::PathBuf, sync::LazyLock};

use pulldown_cmark::{Event, Options, Tag, TagEnd};

pub static UPLOAD_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    let env_path = std::env::var("UPLOAD_DIR").unwrap_or_else(|_| "uploads".into());
    let path = PathBuf::from(env_path);
    if !path.exists() {
        create_dir_all(&path).expect("Failed to create upload directory");
    }
    path
});

pub fn format_markdown(input: &str) -> String {
    autocorrect::format_for(input, "markdown").out
}

pub fn parse_markdown(input: &str) -> Result<Vec<Event<'_>>> {
    let parser = pulldown_cmark::Parser::new_ext(
        &input,
        Options::ENABLE_GFM
            | Options::ENABLE_TABLES
            | Options::ENABLE_TASKLISTS
            | Options::ENABLE_STRIKETHROUGH,
    );
    let events = parser.into_iter().collect::<Vec<_>>();
    // 将相邻的 HTML、 Inline HTML 合并
    let mut merged_events = Vec::new();
    let (mut in_html, mut in_inline_html) = (false, false);
    let (mut html, mut inline_html) = (String::new(), String::new());
    for event in events {
        match event {
            Event::Start(Tag::HtmlBlock) => {
                in_html = true;
                merged_events.push(event);
            }
            Event::Html(html_content) if in_html => {
                html.push_str(&html_content);
            }
            Event::End(TagEnd::HtmlBlock) => {
                in_html = false;
                merged_events.push(Event::Html(html.clone().into()));
                merged_events.push(event);
                html.clear();
            }
            Event::InlineHtml(inline_html_content) => {
                if !in_inline_html {
                    in_inline_html = true;
                    inline_html.clear();
                }
                inline_html.push_str(&inline_html_content);
            }
            Event::SoftBreak if in_inline_html => {
                inline_html.push('\n');
            }
            _ => {
                if in_inline_html {
                    merged_events.push(Event::InlineHtml(inline_html.clone().into()));
                    inline_html.clear();
                    in_inline_html = false;
                }
                merged_events.push(event);
            }
        }
    }
    if in_inline_html && !inline_html.is_empty() {
        merged_events.push(Event::InlineHtml(inline_html.clone().into()));
    }
    Ok(merged_events)
}
