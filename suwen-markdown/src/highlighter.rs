use anyhow::{Context, Result, anyhow};
use pulldown_cmark::{CodeBlockKind, CowStr, Event, Tag, TagEnd};

pub struct Highlighter {
    highlighter: arborium::Highlighter,
}

impl Highlighter {
    pub fn new() -> Highlighter {
        Highlighter {
            highlighter: arborium::Highlighter::new(),
        }
    }

    pub fn highlight<'a, It>(&mut self, events: It) -> Result<Vec<Event<'a>>>
    where
        It: Iterator<Item = Event<'a>>,
    {
        let mut syntax = None;
        let mut in_code_block = false;
        let mut to_hightlight = String::new();
        events
            .filter_map(|e| match e {
                Event::Start(Tag::CodeBlock(kind)) => {
                    if let CodeBlockKind::Fenced(lang) = kind {
                        if self.highlighter.store().get(lang.as_ref()).is_some() {
                            syntax = Some(lang.into_string());
                        } else {
                            syntax = None;
                        }
                    }
                    in_code_block = true;
                    None
                }
                Event::End(TagEnd::CodeBlock) => {
                    if !in_code_block {
                        return Some(Err(anyhow!("Unmatched code block end event")));
                    }
                    let html = self
                        .highlighter
                        .highlight(syntax.as_deref().unwrap_or("markdown"), &to_hightlight)
                        .map(|v| Event::Html(CowStr::from(format!("<pre>{}</pre>", v))))
                        .context("Failed to highlight code block");
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
