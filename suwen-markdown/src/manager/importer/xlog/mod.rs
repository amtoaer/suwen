mod schema;

use std::{
    borrow::Cow,
    path::{Path, PathBuf},
    sync::LazyLock,
};

use crate::{
    format_markdown,
    manager::importer::{Markdown, xlog::schema::Content},
    parse_markdown,
};
use anyhow::{Context, Result, bail};
use futures::TryStreamExt;
use mime2ext::mime2ext;
use pathdiff::diff_paths;
use pulldown_cmark::{Event, Tag};
use pulldown_cmark_to_cmark::cmark_resume;
use regex::{Captures, Regex};
use tokio::{
    fs::{File, read_to_string},
    io,
};
use tokio_util::io::StreamReader;
use yaml_rust2::YamlLoader;

pub async fn import_file(
    file: PathBuf,
    output: PathBuf,
    image_output: PathBuf,
) -> Result<super::Markdown> {
    let mut content = read_content(&file).await?;
    format_content(&mut content);
    let slug = extract_slug(&content)?;
    let cover_images = init_cover_images(&content, &slug, &output, &image_output).await;
    match extract_type(&content) {
        Some("short") => handle_short(content, slug, cover_images),
        Some("post") => handle_post(content, slug, cover_images, &output, &image_output).await,
        _ => bail!("Unsupported content type in file: {}", file.display()),
    }
}

async fn read_content(source: &Path) -> Result<Content> {
    if !source.extension().is_some_and(|ext| ext == "json") {
        bail!("Unsupported file type: {}", source.display());
    }
    let json = read_to_string(&source).await?;
    serde_json::from_str::<Content>(&json).context("Failed to parse JSON content")
}

fn extract_type(content: &Content) -> Option<&str> {
    content.metadata.content.tags.first().map(|s| s.as_str())
}

fn format_content(content: &mut Content) {
    content.metadata.content.title = format_markdown(&content.metadata.content.title);
    content.metadata.content.content = format_markdown(&content.metadata.content.content);
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
    image_output: &Path,
) -> Result<Markdown> {
    let parts: Vec<&str> = content.metadata.content.content.splitn(3, "---").collect();
    let content_text = if parts.len() == 3 && YamlLoader::load_from_str(parts[1]).is_ok() {
        Cow::Owned(String::from(parts[0]) + parts[2])
    } else {
        Cow::Borrowed(&content.metadata.content.content)
    };
    let mut events = parse_markdown(&content_text)?;
    let mut idx = 0;
    for mut event in &mut events {
        match &mut event {
            Event::Start(Tag::Image { dest_url, .. }) => {
                let image = process_ipfs(dest_url);
                if !image.is_empty() {
                    let new_url = match download_image(
                        &image,
                        &image_output.join(format!("{}-image-{}", slug, idx)),
                    )
                    .await
                    {
                        Ok(image_path) => diff_paths(&image_path, &output)
                            .unwrap()
                            .to_string_lossy()
                            .to_string(),
                        Err(err) => {
                            error!("Failed to download image {}: {}", image, err);
                            format!("$dead_link/{}", image)
                        }
                    };
                    *dest_url = new_url.clone().into();
                    cover_images.push(new_url.to_string());
                    idx += 1;
                }
            }
            _ => {}
        }
    }
    let mut buf = String::new();
    cmark_resume(events.into_iter(), &mut buf, None).context("Failed to resume cmark")?;
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
    image_output: &Path,
) -> Vec<String> {
    let mut cover_images = Vec::new();
    for (idx, attachment) in content.metadata.content.attachments.iter().enumerate() {
        let image = process_ipfs(&attachment.address).into_owned();
        if !image.is_empty() {
            match download_image(
                &image,
                &image_output.join(format!("{}-attachment-{}", slug, idx)),
            )
            .await
            {
                Ok(image_path) => {
                    cover_images.push(
                        diff_paths(&image_path, output)
                            .unwrap()
                            .to_string_lossy()
                            .to_string(),
                    );
                }
                Err(err) => {
                    error!("Failed to download image {}: {}", image, err);
                }
            }
        }
    }
    cover_images
}

fn process_ipfs(input: &str) -> Cow<'_, str> {
    static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"ipfs://([a-zA-Z0-9]+)").unwrap());
    RE.replace_all(input, |caps: &Captures| {
        format!(
            "https://ipfs.crossbell.io/ipfs/{}?img-format=webp",
            &caps[1]
        )
    })
}

async fn download_image(url: &str, image_output: &Path) -> Result<PathBuf> {
    static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
        reqwest::Client::builder()
        .user_agent(
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:141.0) Gecko/20100101 Firefox/141.0",
        )
        .build()
        .unwrap()
    });
    let resp = CLIENT
        .get(url)
        .send()
        .await
        .context("Failed to download image")?;
    let extension = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|h| h.to_str().ok())
        .and_then(|s| mime2ext(s))
        .context("Failed to parse image mime type")?;
    let image_file = image_output.with_extension(extension);
    let mut file = File::create(&image_file)
        .await
        .context("Failed to create image file")?;
    let mut stream_reader = StreamReader::new(resp.bytes_stream().map_err(std::io::Error::other));
    io::copy(&mut stream_reader, &mut file).await?;
    Ok(image_file)
}
