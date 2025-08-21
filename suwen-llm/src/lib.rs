use anyhow::Result;
use llm::{
    builder::{LLMBackend, LLMBuilder},
    chat::ChatMessage,
};
use suwen_config::CONFIG;

mod utils;

pub async fn generate_article_summary(article: &str) -> Result<Option<String>> {
    let llm = LLMBuilder::new()
        .backend(LLMBackend::DeepSeek)
        .system("你是一个优秀的文章摘要生成器。接下来我会提供一篇 Markdown 格式的博客文章，你需要为它生成一条三百字以内的摘要，摘要格式应该为纯文本，允许但尽量少使用换行以保证信息密度。请仅返回摘要，不要包含其他内容。")
        .api_key(&CONFIG.openai_api_key)
        .model("deepseek-chat").timeout_seconds(60).temperature(1.2).stream(false).build()?;
    let msgs = vec![ChatMessage::user().content(article).build()];
    Ok(llm
        .chat(&msgs)
        .await?
        .text()
        .map(|s| utils::standardize_text(&s)))
}
