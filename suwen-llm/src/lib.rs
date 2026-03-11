use anyhow::Result;
use llm::builder::{LLMBackend, LLMBuilder};
use llm::chat::ChatMessage;
use suwen_config::CONFIG;
use suwen_markdown::Markdown;
mod utils;

static PROMPT: &str = "
你是一个专业的博客文章摘要生成器，你的任务是提炼文章的核心观点和主要论据。生成的摘要应语气专业、流畅自然，如同人类撰写的导读，避免生硬的堆砌或句式重复。
接下来我会提供一篇 Markdown 格式的博客文章，请你为它生成一条精炼的摘要，长度严格控制在 300 字以内。
摘要中推荐使用“本文”指代文章，使用“作者”指代撰文人，并将这些称谓自然地融入语句中。摘要必须为纯文本格式，仅在段落衔接处允许使用一次换行，以最大程度保证信息的连续性和密度。
请务必遵守所有格式和语气要求，仅输出摘要内容，不得包含任何前言、后记或解释性文字。
";

pub async fn generate_article_summary(article: &Markdown) -> Result<Option<String>> {
    if matches!(article, Markdown::Short { .. }) {
        return Ok(None);
    }
    let mut llm = LLMBuilder::new()
        .backend(LLMBackend::OpenAI)
        .system(PROMPT)
        .api_key(&CONFIG.openai_api_key)
        .model(&CONFIG.openai_model)
        .timeout_seconds(60)
        .temperature(1.2);
    if let Some(base_url) = &CONFIG.openai_base_url {
        llm = llm.base_url(base_url);
    }
    let llm = llm.build()?;
    let msgs = vec![
        ChatMessage::user()
            .content(format!(
                "标题：{}\n\n标签：{}\n\n内容：\n{}",
                article.title(),
                article.tags().join(", "),
                article.content()
            ))
            .build(),
    ];
    Ok(llm.chat(&msgs).await?.text().map(|s| utils::standardize_text(&s)))
}
