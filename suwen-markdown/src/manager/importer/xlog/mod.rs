mod schema;

use anyhow::{Error, ensure};
use std::{
    borrow::Cow,
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, LazyLock},
};

use crate::{
    manager::importer::{Markdown, xlog::schema::Content},
    parse_markdown,
};
use anyhow::{Context, Result, bail};
use futures::{StreamExt, TryStreamExt, stream::FuturesUnordered};
use lol_html::{HtmlRewriter, Settings, element};
use mime2ext::mime2ext;
use pathdiff::diff_paths;
use pulldown_cmark::{Event, Tag};
use pulldown_cmark_to_cmark::cmark_resume;
use regex::{Captures, Regex};
use tokio::{
    fs::{File, read_to_string},
    io,
    sync::Semaphore,
};
use tokio_util::io::StreamReader;
use yaml_rust2::YamlLoader;

pub async fn import_file(
    file: PathBuf,
    output: PathBuf,
    obj_output: PathBuf,
) -> Result<super::Markdown> {
    let mut content = read_content(&file).await?;
    format_content(&mut content);
    let content_type = extract_type(&content);
    ensure!(
        content_type.is_some_and(|s| s == "post" || s == "short"),
        "Unsupported content type in file: {}",
        file.display()
    );
    let slug = extract_slug(&content)?;
    let cover_images = init_cover_images(&content, &slug, &output, &obj_output).await;
    match content_type {
        Some("short") => handle_short(content, slug, cover_images),
        Some("post") => handle_post(content, slug, cover_images, &output, &obj_output).await,
        _ => unreachable!(),
    }
}

async fn read_content(source: &Path) -> Result<Content> {
    if source.extension().is_none_or(|ext| ext != "json") {
        bail!("Unsupported file type: {}", source.display());
    }
    let json = read_to_string(&source).await?;
    serde_json::from_str::<Content>(&json).context("Failed to parse JSON content")
}

fn extract_type(content: &Content) -> Option<&str> {
    content.metadata.content.tags.first().map(|s| s.as_str())
}

fn format_content(content: &mut Content) {
    content.metadata.content.title =
        autocorrect::format_for(&content.metadata.content.title, "markdown").out;
    content.metadata.content.content =
        autocorrect::format_for(&content.metadata.content.content, "markdown").out;
}

fn extract_slug(content: &Content) -> Result<String> {
    content
        .metadata
        .content
        .attributes
        .iter()
        .find_map(|attr| {
            if attr.trait_type == "xlog_slug" {
                attr.value.as_str().map(String::from)
            } else {
                None
            }
        })
        .context("Slug not found")
}

fn handle_short(content: Content, slug: String, cover_images: Vec<String>) -> Result<Markdown> {
    Ok(Markdown::Short {
        slug,
        title: content.metadata.content.title,
        cover_images,
        content: content.metadata.content.content,
        created_at: content.created_at,
        updated_at: content.updated_at,
        published_at: content.published_at,
    })
}

async fn handle_post(
    content: Content,
    slug: String,
    mut cover_images: Vec<String>,
    output: &Path,
    obj_output: &Path,
) -> Result<Markdown> {
    let parts: Vec<&str> = content.metadata.content.content.splitn(3, "---").collect();
    let content_text = if parts.len() == 3 && YamlLoader::load_from_str(parts[1]).is_ok() {
        Cow::Owned(String::from(parts[0]) + parts[2])
    } else {
        Cow::Borrowed(&content.metadata.content.content)
    };
    let events = parse_markdown(&content_text)?;
    let mut images = Vec::new();
    events.iter().for_each(|event| {
        if let Event::Start(Tag::Image { dest_url, .. }) = event {
            images.push(dest_url.clone());
        }
    });
    let results = batch_download_replace(
        images
            .into_iter()
            .enumerate()
            .map(|(idx, url)| (url, obj_output.join(format!("{}-image-{}", slug, idx)))),
        output,
    )
    .await;
    let url_map = results
        .into_iter()
        .filter_map(Result::ok)
        .collect::<HashMap<_, _>>();
    let mut video_idx = 0;
    let mut filtered_events = Vec::new();
    for mut event in events {
        match &mut event {
            Event::Start(Tag::Image { dest_url, .. }) => {
                if let Some(new_url) = url_map.get(dest_url.as_ref()) {
                    *dest_url = new_url.clone().into();
                    cover_images.push(new_url.to_string());
                    filtered_events.push(event);
                }
            }
            Event::InlineHtml(inline_html_content) => {
                filtered_events.push(Event::InlineHtml(
                    rewrite_html(
                        inline_html_content,
                        output.to_path_buf(),
                        obj_output.to_path_buf(),
                        slug.clone(),
                        &mut video_idx,
                    )
                    .await?
                    .into(),
                ));
            }
            Event::Html(html_content) => {
                filtered_events.push(Event::Html(
                    rewrite_html(
                        html_content,
                        output.to_path_buf(),
                        obj_output.to_path_buf(),
                        slug.clone(),
                        &mut video_idx,
                    )
                    .await?
                    .into(),
                ));
            }
            _ => {
                filtered_events.push(event);
            }
        }
    }
    let mut buf = String::new();
    cmark_resume(filtered_events.into_iter(), &mut buf, None).context("Failed to resume cmark")?;
    Ok(Markdown::Article {
        slug,
        title: content.metadata.content.title,
        cover_images,
        content: buf,
        tags: content.metadata.content.tags.into_iter().skip(1).collect(),
        created_at: content.created_at,
        updated_at: content.updated_at,
        published_at: content.published_at,
    })
}

async fn init_cover_images(
    content: &Content,
    slug: &str,
    output: &Path,
    obj_output: &Path,
) -> Vec<String> {
    let mut cover_images = Vec::new();
    let results = batch_download_replace(
        content
            .metadata
            .content
            .attachments
            .iter()
            .enumerate()
            .map(|(idx, attachment)| {
                (
                    &attachment.address,
                    obj_output.join(format!("{}-attachment-{}", slug, idx)),
                )
            }),
        output,
    )
    .await;
    let results = results
        .into_iter()
        .filter_map(Result::ok)
        .collect::<HashMap<_, _>>();
    for attachment in &content.metadata.content.attachments {
        if let Some(new_url) = results.get(&attachment.address) {
            cover_images.push(new_url.clone());
        }
    }
    cover_images
}

// 批量下载并替换，生成 Vec<(url, new_url)>
async fn batch_download_replace<T>(
    old_new_pair: impl IntoIterator<Item = (T, PathBuf)>,
    output: &Path,
) -> Vec<Result<(T, String)>>
where
    T: AsRef<str>,
{
    let semaphore = Arc::new(Semaphore::new(8));
    let tasks = old_new_pair
        .into_iter()
        .map(|(url, target)| {
            let semaphore = semaphore.clone();
            async move {
                let _permit = semaphore.acquire().await;
                let res = download_replace(url.as_ref(), output, &target).await?;
                Ok::<_, Error>((url, res))
            }
        })
        .collect::<FuturesUnordered<_>>();
    tasks.collect::<Vec<_>>().await
}

/// 从 URL 下载文件，并返回替换后的相对路径，仅当 url 为空时返回 Error
async fn download_replace(url: &str, output: &Path, target: &Path) -> Result<String> {
    // 替换 ipfs:// 格式的链接为普通 URL
    static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"ipfs://([a-zA-Z0-9]+)").unwrap());
    let url = RE.replace_all(url, |caps: &Captures| {
        format!("https://ipfs.crossbell.io/ipfs/{}", &caps[1])
    });
    if url.is_empty() {
        bail!("Invalid URL");
    }
    Ok(match download(&url, target).await {
        // 成功，替换成下载的文件相对文章的相对路径
        Ok(file) => diff_paths(&file, output)
            .unwrap()
            .to_string_lossy()
            .to_string(),
        // 失败，添加死链标记
        Err(err) => {
            error!("Failed to download {}: {}", url, err);
            format!("$dead_link/{}", url)
        }
    })
}

// 下载某个 URL 到指定路径，返回下载成功的路径（在指定路径上附加拓展名）
async fn download(url: &str, target: &Path) -> Result<PathBuf> {
    static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
        reqwest::Client::builder()
        .user_agent(
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:141.0) Gecko/20100101 Firefox/141.0",
        )
        .build()
        .unwrap()
    });
    let resp = CLIENT.get(url).send().await.context("Failed to download")?;
    let extension = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|h| h.to_str().ok())
        .and_then(|s| mime2ext(s))
        .context("Failed to parse mime type")?;
    let download_file = target.with_extension(extension);
    let mut file = File::create(&download_file)
        .await
        .context("Failed to create file")?;
    let mut stream_reader = StreamReader::new(resp.bytes_stream().map_err(std::io::Error::other));
    io::copy(&mut stream_reader, &mut file).await?;
    Ok(download_file)
}

// 扫描 html，取出所有的视频链接
async fn collect_videos(html: &str) -> Result<Vec<String>> {
    let mut videos = Vec::new();
    let mut scanner = HtmlRewriter::new(
        Settings {
            element_content_handlers: vec![element!("source[src]", |el| {
                let video_url = el.get_attribute("src").unwrap_or_default();
                videos.push(video_url.clone());
                Ok(())
            })],
            ..Settings::new()
        },
        |_: &[u8]| {},
    );
    scanner.write(html.as_bytes())?;
    scanner.end()?;
    Ok(videos)
}

async fn rewrite_html(
    html: &str,
    output: PathBuf,
    obj_output: PathBuf,
    slug: String,
    video_idx: &mut usize,
) -> Result<String> {
    let videos = collect_videos(html).await?;
    let results = batch_download_replace(
        videos.into_iter().enumerate().map(|(idx, v)| {
            (
                v,
                obj_output.join(format!("{}-video-{}", slug, *video_idx + idx)),
            )
        }),
        &output,
    )
    .await;
    *video_idx += results.len();
    let url_map = results
        .into_iter()
        .filter_map(Result::ok)
        .collect::<HashMap<_, _>>();
    let mut buf = Vec::new();
    let mut rewriter = HtmlRewriter::new(
        Settings {
            element_content_handlers: vec![
                element!("video", |el| {
                    el.set_attribute("controls", "true")?;
                    Ok(())
                }),
                element!("source[src]", move |el| {
                    let video_url = el.get_attribute("src").unwrap_or_default();
                    if let Some(new_url) = url_map.get(&video_url) {
                        el.set_attribute("src", new_url)?;
                    }
                    Ok(())
                }),
            ],
            ..Settings::new()
        },
        |c: &[u8]| buf.extend_from_slice(c),
    );
    rewriter.write(html.as_bytes())?;
    rewriter.end()?;
    Ok(String::from_utf8_lossy(&buf).to_string())
}
