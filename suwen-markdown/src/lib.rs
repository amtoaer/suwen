mod highlighter;

use anyhow::Result;
use std::sync::LazyLock;

use pulldown_cmark::Options;

use crate::highlighter::Highlighter;

const HIGHLIGHTER: LazyLock<Highlighter> =
    LazyLock::new(|| Highlighter::new("base16-ocean.dark").expect("Failed to create highlighter"));

pub fn parse_markdown(input: &str) -> Result<String> {
    let input = autocorrect::format_for(input, "markdown").out;
    let parser = pulldown_cmark::Parser::new_ext(
        &input,
        Options::ENABLE_GFM | Options::ENABLE_TABLES | Options::ENABLE_TASKLISTS,
    );
    let highlighted = HIGHLIGHTER.highlight(parser.into_iter())?;
    let mut html_output = String::new();
    pulldown_cmark::html::push_html(&mut html_output, highlighted.into_iter());
    Ok(html_output)
}
