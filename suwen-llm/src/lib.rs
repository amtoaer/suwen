use anyhow::Result;
use llm::{
    builder::{LLMBackend, LLMBuilder},
    chat::ChatMessage,
};
use suwen_config::CONFIG;

mod utils;

static PROMPT: &str = "
你是一个专业的博客文章摘要生成器，你的任务是提炼文章的核心观点和主要论据。生成的摘要应语气专业、流畅自然，如同人类撰写的导读，避免生硬的堆砌或句式重复。
接下来我会提供一篇 Markdown 格式的博客文章，请你为它生成一条精炼的摘要，长度严格控制在 300 字以内。
摘要中推荐使用“本文”指代文章，使用“作者”指代撰文人，并将这些称谓自然地融入语句中。摘要必须为纯文本格式，仅在段落衔接处允许使用一次换行，以最大程度保证信息的连续性和密度。
请务必遵守所有格式和语气要求，仅输出摘要内容，不得包含任何前言、后记或解释性文字。
";

pub async fn generate_article_summary(article: &str) -> Result<Option<String>> {
    let llm = LLMBuilder::new()
        .backend(LLMBackend::DeepSeek)
        .system(PROMPT)
        .api_key(&CONFIG.openai_api_key)
        .model("deepseek-chat")
        .timeout_seconds(60)
        .temperature(1.2)
        .stream(false)
        .build()?;
    let msgs = vec![ChatMessage::user().content(article).build()];
    Ok(llm
        .chat(&msgs)
        .await?
        .text()
        .map(|s| utils::standardize_text(&s)))
}
