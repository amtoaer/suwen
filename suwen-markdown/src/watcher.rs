use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock};
use std::time::Duration;

use anyhow::{Context, Result};
use aws_credential_types::Credentials;
use aws_sdk_s3::Client as S3Client;
use aws_sdk_s3::config::Region;
use futures::StreamExt;
use futures::stream::FuturesUnordered;
use lol_html::{HtmlRewriter, Settings, element};
use mime2ext::mime2ext;
use notify::event::{ModifyKind, RenameMode};
use notify::{EventKind, RecursiveMode};
use notify_debouncer_full::{DebouncedEvent, new_debouncer};
use pulldown_cmark::{Event as MdEvent, Tag};
use pulldown_cmark_to_cmark::cmark_resume;
use sha2::{Digest, Sha256};
use suwen_config::CONFIG;
use tokio::sync::{Semaphore, mpsc};
use tracing::{debug, error, info, warn};

use crate::{Markdown, parse_markdown};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MediaType {
    Image,
    Video,
}

#[derive(Debug, Clone)]
pub struct MediaResource {
    pub url: String,
    pub media_type: MediaType,
}

#[derive(Debug, Clone)]
pub struct UploadedMedia {
    pub original_url: String,
    pub new_url: String,
}

pub struct MarkdownWatcher {
    watch_path: PathBuf,
    object_output: PathBuf,
    db_sender: mpsc::UnboundedSender<MarkdownChange>,
}

#[derive(Debug)]
pub enum MarkdownChange {
    Upsert(Markdown),
    Deleted(String),
    Renamed(String, String),
    SyncExisting(Vec<String>),
}

impl MarkdownWatcher {
    pub fn new(
        watch_path: PathBuf,
        object_output: Option<PathBuf>,
        db_sender: mpsc::UnboundedSender<MarkdownChange>,
    ) -> Self {
        let object_output = object_output.unwrap_or_else(|| watch_path.join("objects"));
        Self {
            watch_path,
            object_output,
            db_sender,
        }
    }

    pub async fn start_watching(self) -> Result<()> {
        let (tx, mut rx) = mpsc::unbounded_channel();

        info!("Initial scan of markdown files in {:?}", self.watch_path);
        self.scan_existing_files().await?;

        let mut debouncer = new_debouncer(Duration::from_secs(5), None, move |result| {
            if let Ok(events) = result {
                for event in events {
                    let _ = tx.send(event);
                }
            }
        })?;
        debouncer.watch(&self.watch_path, RecursiveMode::NonRecursive)?;
        info!("Started watching markdown files in {:?}", self.watch_path);

        while let Some(event) = rx.recv().await {
            info!("Received file system event: {:?}", event);
            self.handle_event(event).await?;
        }
        Ok(())
    }

    async fn handle_event(&self, event: DebouncedEvent) -> Result<()> {
        let event = event.event;
        if event.paths.iter().any(|p| p.extension().is_none_or(|ext| ext != "md")) {
            return Ok(());
        }
        match event.kind {
            EventKind::Modify(ModifyKind::Name(RenameMode::Both)) => {
                let (old_path, new_path) = (&event.paths[0], &event.paths[1]);
                if let (Some(old_stem), Some(new_stem)) = (
                    old_path.file_stem().and_then(|s| s.to_str()),
                    new_path.file_stem().and_then(|s| s.to_str()),
                ) {
                    info!("Markdown file renamed from {:?} to {:?}", old_path, new_path);
                    let _ = self
                        .db_sender
                        .send(MarkdownChange::Renamed(old_stem.to_string(), new_stem.to_string()));
                }
            }
            EventKind::Create(_) | EventKind::Modify(_) => {
                let path = &event.paths[0];
                debug!("Markdown file changed: {:?}", path);
                if let Err(e) = self.process_markdown_file(&path).await {
                    error!("Failed to process markdown file {:?}: {}", path, e);
                }
            }
            EventKind::Remove(_) => {
                let path = &event.paths[0];
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    info!("Markdown file deleted: {:?}", path);
                    let _ = self.db_sender.send(MarkdownChange::Deleted(stem.to_string()));
                }
            }
            _ => {
                info!("Unhandled file system event: {:?}", event.kind);
            }
        }
        Ok(())
    }

    async fn scan_existing_files(&self) -> Result<()> {
        let mut existing_slugs = Vec::new();
        let mut entries = tokio::fs::read_dir(&self.watch_path).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "md") {
                if let Some(slug) = path.file_stem().and_then(|s| s.to_str()) {
                    existing_slugs.push(slug.to_string());
                }
                if let Err(e) = self.process_markdown_file(&path).await {
                    error!("Failed to process markdown file {:?} during initial scan: {}", path, e);
                }
            }
        }
        let _ = self.db_sender.send(MarkdownChange::SyncExisting(existing_slugs));
        Ok(())
    }

    async fn process_markdown_file(&self, path: &Path) -> Result<()> {
        info!("Processing markdown file: {:?}", path);
        let mut markdown = Markdown::from_file(path, CONFIG.source_lang).await?;

        let Some(r2_config) = &suwen_config::CONFIG.r2 else {
            warn!("R2 config not found, skipping media upload");
            let _ = self.db_sender.send(MarkdownChange::Upsert(markdown));
            return Ok(());
        };

        let media_resources = extract_media_from_markdown(&markdown);
        if media_resources.is_empty() {
            debug!("No media resources found in markdown");
            let _ = self.db_sender.send(MarkdownChange::Upsert(markdown));
            return Ok(());
        }

        let uploader = MediaUploader::new(r2_config.clone(), &self.object_output);
        let uploaded_media = uploader.process_media(&media_resources, markdown.slug()).await?;

        update_media_links(&mut markdown, &uploaded_media)?;

        let _ = self.db_sender.send(MarkdownChange::Upsert(markdown));

        Ok(())
    }
}

pub fn extract_media_from_markdown(markdown: &Markdown) -> Vec<MediaResource> {
    let content = markdown.content();

    let mut resources = Vec::new();

    let events = match parse_markdown(content) {
        Ok(events) => events,
        Err(e) => {
            error!("Failed to parse markdown: {}", e);
            return resources;
        }
    };

    for event in events {
        match event {
            MdEvent::Start(Tag::Image { dest_url, .. }) => {
                resources.push(MediaResource {
                    url: dest_url.to_string(),
                    media_type: MediaType::Image,
                });
            }
            MdEvent::Html(html) | MdEvent::InlineHtml(html) => {
                resources.extend(extract_media_from_html(&html));
            }
            _ => {}
        }
    }

    resources
}

fn extract_media_from_html(html: &str) -> Vec<MediaResource> {
    let mut resources = Vec::new();

    if let Ok(videos) = collect_videos_from_html(html) {
        for url in videos {
            resources.push(MediaResource {
                url,
                media_type: MediaType::Video,
            });
        }
    }

    if let Ok(images) = collect_images_from_html(html) {
        for url in images {
            resources.push(MediaResource {
                url,
                media_type: MediaType::Image,
            });
        }
    }

    resources
}

fn collect_videos_from_html(html: &str) -> Result<Vec<String>> {
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

fn collect_images_from_html(html: &str) -> Result<Vec<String>> {
    let mut images = Vec::new();
    let mut scanner = HtmlRewriter::new(
        Settings {
            element_content_handlers: vec![element!("img[src]", |el| {
                let img_url = el.get_attribute("src").unwrap_or_default();
                images.push(img_url.clone());
                Ok(())
            })],
            ..Settings::new()
        },
        |_: &[u8]| {},
    );
    scanner.write(html.as_bytes())?;
    scanner.end()?;
    Ok(images)
}

pub struct MediaUploader {
    r2_config: suwen_config::R2Config,
    object_output: PathBuf,
}

impl MediaUploader {
    pub fn new(r2_config: suwen_config::R2Config, object_output: &Path) -> Self {
        Self {
            r2_config,
            object_output: object_output.to_path_buf(),
        }
    }

    pub async fn process_media(&self, media: &[MediaResource], slug: &str) -> Result<Vec<UploadedMedia>> {
        let semaphore = Arc::new(Semaphore::new(8));
        let mut tasks = FuturesUnordered::new();

        for resource in media {
            let semaphore = semaphore.clone();
            let slug = slug.to_string();
            let resource = resource.clone();
            let object_output = self.object_output.clone();
            let r2_config = self.r2_config.clone();
            let object_domain = suwen_config::CONFIG.object_storage_domain.clone();

            tasks.push(async move {
                let _permit = semaphore.acquire().await?;
                process_single_media(&resource, &slug, &object_output, &r2_config, &object_domain).await
            });
        }

        let mut results = Vec::new();
        while let Some(result) = tasks.next().await {
            match result {
                Ok(Some(uploaded)) => results.push(uploaded),
                Ok(None) => {}
                Err(e) => error!("Failed to process media: {}", e),
            }
        }

        Ok(results)
    }
}

fn is_convertible_image(ext: &str) -> bool {
    matches!(ext.to_lowercase().as_str(), "jpg" | "jpeg" | "png")
}

async fn convert_to_webp(data: &[u8], ext: &str) -> Result<Vec<u8>> {
    let temp_dir = tempfile::tempdir()?;
    let input_path = temp_dir.path().join(format!("input.{}", ext));
    let output_path = temp_dir.path().join("output.webp");

    tokio::fs::write(&input_path, data).await?;

    let status = tokio::process::Command::new("cwebp")
        .args([
            "-sharp_yuv",
            "-mt",
            "-q",
            "80",
            "-metadata",
            "all",
            input_path.to_str().unwrap(),
            "-o",
            output_path.to_str().unwrap(),
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .await?;

    if !status.success() {
        return Err(anyhow::anyhow!("Failed to convert image to webp"));
    }

    let webp_data = tokio::fs::read(&output_path).await?;
    Ok(webp_data)
}

async fn process_single_media(
    resource: &MediaResource,
    slug: &str,
    object_output: &Path,
    r2_config: &suwen_config::R2Config,
    object_domain: &str,
) -> Result<Option<UploadedMedia>> {
    if resource.url.starts_with(object_domain) {
        debug!("Skipping already uploaded media: {}", resource.url);
        return Ok(None);
    }

    if resource.url.starts_with("$dead_link/") {
        debug!("Skipping dead link: {}", resource.url);
        return Ok(None);
    }

    if resource.media_type != MediaType::Image {
        debug!("Skipping non-image media: {}", resource.url);
        return Ok(None);
    }

    let (original_data, original_ext) = fetch_media_content(&resource.url, object_output).await?;

    let hash = compute_hash(&original_data);

    let (upload_data, upload_ext) = if is_convertible_image(&original_ext) {
        match convert_to_webp(&original_data, &original_ext).await {
            Ok(webp_data) => (webp_data, "webp".to_string()),
            Err(e) => {
                warn!("Failed to convert image to webp, using original: {}", e);
                (original_data, original_ext)
            }
        }
    } else {
        (original_data, original_ext)
    };

    let filename = format!("{}_{}.{}", slug, hash, upload_ext);

    if !check_file_exists(r2_config, &filename).await? {
        upload_to_r2(r2_config, &filename, &upload_data, &resource.url).await?;
    }

    let new_url = format!("{}/{}", object_domain.trim_end_matches('/'), filename);

    Ok(Some(UploadedMedia {
        original_url: resource.url.clone(),
        new_url,
    }))
}

async fn fetch_media_content(url: &str, object_output: &Path) -> Result<(Vec<u8>, String)> {
    if url.starts_with("http://") || url.starts_with("https://") {
        fetch_remote_media(url).await
    } else {
        fetch_local_media(url, object_output).await
    }
}

async fn fetch_remote_media(url: &str) -> Result<(Vec<u8>, String)> {
    static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
        reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:141.0) Gecko/20100101 Firefox/141.0")
            .build()
            .unwrap()
    });

    let resp = CLIENT.get(url).send().await.context("Failed to download media")?;

    let ext = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|h| h.to_str().ok())
        .and_then(|s| mime2ext(s))
        .unwrap_or("bin");

    let data = resp.bytes().await?.to_vec();
    Ok((data, ext.to_string()))
}

async fn fetch_local_media(path: &str, object_output: &Path) -> Result<(Vec<u8>, String)> {
    let local_path = if Path::new(path).is_absolute() {
        PathBuf::from(path)
    } else {
        let try1 = object_output.join(path);
        if try1.exists() { try1 } else { PathBuf::from(path) }
    };

    let data = tokio::fs::read(&local_path)
        .await
        .context(format!("Failed to read local file: {:?}", local_path))?;

    let ext = local_path.extension().and_then(|e| e.to_str()).unwrap_or("bin");

    Ok((data, ext.to_string()))
}

fn compute_hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    hex::encode(&result[..16]) // 使用前16字节
}

async fn check_file_exists(r2_config: &suwen_config::R2Config, filename: &str) -> Result<bool> {
    let creds = Credentials::new(&r2_config.access_key_id, &r2_config.secret_access_key, None, None, "r2");

    let config = aws_sdk_s3::config::Builder::new()
        .credentials_provider(creds)
        .region(Region::new("auto"))
        .endpoint_url(format!("https://{}.r2.cloudflarestorage.com", r2_config.account_id))
        .force_path_style(true)
        .build();

    let client = S3Client::from_conf(config);

    let result = client
        .head_object()
        .bucket(&r2_config.bucket_name)
        .key(filename)
        .send()
        .await;

    match result {
        Ok(_) => Ok(true),
        Err(aws_sdk_s3::error::SdkError::ServiceError(err)) if err.err().is_not_found() => Ok(false),
        Err(e) => {
            warn!("Error checking file existence: {}", e);
            Ok(false)
        }
    }
}

async fn upload_to_r2(
    r2_config: &suwen_config::R2Config,
    filename: &str,
    data: &[u8],
    original_url: &str,
) -> Result<()> {
    info!("Uploading {} to R2: {}", original_url, filename);

    let creds = Credentials::new(&r2_config.access_key_id, &r2_config.secret_access_key, None, None, "r2");

    let config = aws_sdk_s3::config::Builder::new()
        .credentials_provider(creds)
        .region(Region::new("auto"))
        .endpoint_url(format!("https://{}.r2.cloudflarestorage.com", r2_config.account_id))
        .force_path_style(true)
        .build();

    let client = S3Client::from_conf(config);

    let content_type = match filename.rsplit_once('.').map(|(_, ext)| ext.to_lowercase()).as_deref() {
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("png") => "image/png",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("mp4") => "video/mp4",
        Some("webm") => "video/webm",
        Some("mov") => "video/quicktime",
        _ => "application/octet-stream",
    };

    client
        .put_object()
        .bucket(&r2_config.bucket_name)
        .key(filename)
        .body(data.to_vec().into())
        .content_type(content_type)
        .send()
        .await
        .context("Failed to upload to R2")?;

    Ok(())
}

pub fn update_media_links(markdown: &mut Markdown, uploaded_media: &[UploadedMedia]) -> Result<()> {
    let url_map: HashMap<_, _> = uploaded_media
        .iter()
        .map(|m| (m.original_url.clone(), m.new_url.clone()))
        .collect();

    match markdown {
        Markdown::Article { content, .. } | Markdown::Short { content, .. } => {
            let mut events = parse_markdown(content)?;
            let mut needs_update = false;

            for event in events.iter_mut() {
                match event {
                    MdEvent::Start(Tag::Image { dest_url, .. }) => {
                        if let Some(new_url) = url_map.get(dest_url.as_ref()) {
                            *dest_url = new_url.clone().into();
                            needs_update = true;
                        }
                    }
                    MdEvent::Html(html_content) | MdEvent::InlineHtml(html_content) => {
                        // 更新 HTML 中的媒体链接
                        if let Ok(new_html) = update_html_media_links(html_content, &url_map) {
                            *html_content = new_html.into();
                            needs_update = true;
                        }
                    }
                    _ => {}
                }
            }

            if needs_update {
                let mut buf = String::new();
                cmark_resume(events.into_iter(), &mut buf, None).context("Failed to resume cmark")?;
                *content = buf;
            }
        }
    }

    Ok(())
}

fn update_html_media_links(html: &str, url_map: &HashMap<String, String>) -> Result<String> {
    let mut buf = Vec::new();
    let mut rewriter = HtmlRewriter::new(
        Settings {
            element_content_handlers: vec![
                element!("img[src]", |el| {
                    if let Some(src) = el.get_attribute("src")
                        && let Some(new_url) = url_map.get(&src)
                    {
                        el.set_attribute("src", new_url)?;
                    }
                    Ok(())
                }),
                element!("source[src]", |el| {
                    if let Some(src) = el.get_attribute("src")
                        && let Some(new_url) = url_map.get(&src)
                    {
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
