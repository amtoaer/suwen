pub(crate) fn standardize_text(text: &str) -> String {
    auto_correct(&clear_text(text))
}

fn auto_correct(text: &str) -> String {
    autocorrect::format_for(text, "markdown").out
}

fn clear_text(text: &str) -> String {
    text.lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}
