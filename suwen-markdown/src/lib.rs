mod highlighter;
pub mod importer;

use anyhow::Result;
use std::{fs::create_dir_all, path::PathBuf, sync::LazyLock};

use pulldown_cmark::{Event, Options};

use crate::highlighter::Highlighter;

static HIGHLIGHTER: LazyLock<Highlighter> =
    LazyLock::new(|| Highlighter::new("base16-ocean.dark").expect("Failed to create highlighter"));

pub static UPLOAD_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    let env_path = std::env::var("UPLOAD_DIR").unwrap_or_else(|_| "uploads".into());
    let path = PathBuf::from(env_path);
    if !path.exists() {
        create_dir_all(&path).expect("Failed to create upload directory");
    }
    path
});

pub struct VecEvents<'a>(Vec<Event<'a>>);

impl<'a> Into<VecEvents<'a>> for Vec<Event<'a>> {
    fn into(self) -> VecEvents<'a> {
        VecEvents(self)
    }
}

pub fn format_markdown(input: &str) -> String {
    autocorrect::format_for(input, "markdown").out
}

pub fn parse_markdown(input: &str) -> Result<VecEvents<'_>> {
    let parser = pulldown_cmark::Parser::new_ext(
        &input,
        Options::ENABLE_GFM | Options::ENABLE_TABLES | Options::ENABLE_TASKLISTS,
    );
    HIGHLIGHTER.highlight(parser.into_iter()).map(Into::into)
}

impl Into<String> for VecEvents<'_> {
    fn into(self) -> String {
        let mut output = String::new();
        pulldown_cmark::html::push_html(&mut output, self.0.into_iter());
        output
    }
}
