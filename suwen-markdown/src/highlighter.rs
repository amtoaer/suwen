use anyhow::{Result, anyhow};
use pulldown_cmark::{CodeBlockKind, CowStr, Event, Tag, TagEnd};
use syntect::highlighting::{Theme, ThemeSet};
use syntect::html::highlighted_html_for_string;
use syntect::parsing::SyntaxSet;

pub struct Highlighter {
    syntax_set: SyntaxSet,
    theme: Theme,
}

impl Highlighter {
    pub fn new(theme: &str) -> Result<Highlighter> {
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let mut theme_set = ThemeSet::load_defaults();
        let theme = theme_set
            .themes
            .remove(theme)
            .ok_or_else(|| anyhow!("Theme '{}' not found in the theme set", theme))?;

        Ok(Highlighter { syntax_set, theme })
    }

    pub fn highlight<'a, It>(&self, events: It) -> Result<Vec<Event<'a>>>
    where
        It: Iterator<Item = Event<'a>>,
    {
        let fallback_syntax = self.syntax_set.find_syntax_plain_text();
        let mut in_code_block = false;
        let mut syntax = fallback_syntax;
        let mut to_hightlight = String::new();
        events
            .filter_map(|e| match e {
                Event::Start(Tag::CodeBlock(kind)) => {
                    match kind {
                        CodeBlockKind::Fenced(lang) => {
                            syntax = self
                                .syntax_set
                                .find_syntax_by_token(&lang)
                                .unwrap_or(fallback_syntax);
                        }
                        _ => {}
                    }
                    in_code_block = true;
                    None
                }
                Event::End(TagEnd::CodeBlock) => {
                    if !in_code_block {
                        return Some(Err(anyhow!("Unmatched code block end event")));
                    }
                    let html = highlighted_html_for_string(
                        &to_hightlight,
                        &self.syntax_set,
                        syntax,
                        &self.theme,
                    )
                    .map(|v| Event::Html(CowStr::from(v)))
                    .map_err(|e| anyhow!("Highlighting error: {}", e));
                    to_hightlight.clear();
                    in_code_block = false;
                    Some(html)
                }
                Event::Text(text) if in_code_block => {
                    to_hightlight.push_str(&text);
                    None
                }
                event => Some(Ok(event)),
            })
            .collect::<Result<Vec<_>>>()
    }
}
