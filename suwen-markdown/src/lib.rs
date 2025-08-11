#[macro_use]
extern crate tracing;

mod highlighter;
pub mod manager;

use anyhow::Result;
use std::{fs::create_dir_all, path::PathBuf, sync::LazyLock};
use two_face::theme::EmbeddedThemeName;

use pulldown_cmark::{Event, Options};

use crate::highlighter::Highlighter;

static HIGHLIGHTER: LazyLock<Highlighter> = LazyLock::new(Highlighter::new);

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
        Options::ENABLE_GFM | Options::ENABLE_TABLES | Options::ENABLE_TASKLISTS,
    );
    Ok(parser.into_iter().collect::<Vec<_>>())
}
