use std::hash::Hasher;
use std::path::Path;
use std::sync::LazyLock;

use anyhow::{Context, Result, bail};
use aws_config::BehaviorVersion;
use aws_sdk_s3::error::SdkError;
use futures::TryStreamExt;
use futures::stream::FuturesUnordered;
use suwen_config::CONFIG;
use tokio::io::AsyncWriteExt;
use tokio::sync::{OnceCell, Semaphore};
use twox_hash::XxHash3_64;

use crate::Markdown;
use crate::markdown::MediaResource;

pub struct MarkdownProcessor {
    s3_client: aws_sdk_s3::Client,
    bucket_name: &'static str,
    prefix: &'static str,
    s3_domain: &'static str,
}

#[derive(Debug, Clone)]
pub struct UploadedMedia {
    pub original_url: String,
    pub new_url: String,
}

impl MarkdownProcessor {
    pub async fn get() -> &'static Self {
        static INSTANCE: OnceCell<MarkdownProcessor> = OnceCell::const_new();
        INSTANCE
            .get_or_init(|| async {
                let config = aws_config::defaults(BehaviorVersion::latest())
                    .endpoint_url(format!("https://{}.r2.cloudflarestorage.com", CONFIG.r2.account_id))
                    .credentials_provider(aws_sdk_s3::config::Credentials::new(
                        CONFIG.r2.access_key_id.to_owned(),
                        CONFIG.r2.access_key_secret.to_owned(),
                        None,
                        None,
                        "R2",
                    ))
                    .region("auto")
                    .load()
                    .await;
                let s3_client = aws_sdk_s3::Client::new(&config);
                Self {
                    s3_client,
                    bucket_name: CONFIG.r2.bucket_name.as_str(),
                    prefix: CONFIG.r2.prefix.as_str(),
                    s3_domain: CONFIG.r2.s3_domain.as_str(),
                }
            })
            .await
    }

    pub async fn process_file(&self, path: &Path) -> Result<Markdown> {
        info!("Processing markdown file: {:?}", path);
        let mut markdown = Markdown::from_file(path, CONFIG.source_lang).await?;
        let media_resources = markdown.extract_resources()?;
        if media_resources.is_empty() {
            debug!("No media resources found in markdown");
            return Ok(markdown);
        }
        let markdown_dir = path.parent().context("Failed to get markdown file directory")?;
        let uploaded_media = self
            .upload_medias(markdown.slug(), media_resources, markdown_dir)
            .await?;
        markdown.update_by_uploaded_resource(uploaded_media)?;
        Ok(markdown)
    }

    pub async fn upload_medias(
        &self,
        slug: &str,
        medias: Vec<MediaResource>,
        base_dir: &Path,
    ) -> Result<Vec<UploadedMedia>> {
        let semaphore = Semaphore::new(8);
        let tasks = medias
            .into_iter()
            .map(|media| async {
                let _permit = semaphore.acquire().await?;
                self.upload_media(slug, media, base_dir).await
            })
            .collect::<FuturesUnordered<_>>();
        tasks
            .try_collect::<Vec<Option<UploadedMedia>>>()
            .await
            .map(|results| results.into_iter().flatten().collect())
    }

    async fn upload_media(&self, slug: &str, media: MediaResource, base_dir: &Path) -> Result<Option<UploadedMedia>> {
        let media_url = media.url();
        if media_url.starts_with(self.s3_domain) {
            dbg!("Media already uploaded to S3, skipping: {}", media_url);
            return Ok(None);
        }
        if media_url.starts_with("$dead_link") {
            dbg!("Media is a dead link, skipping: {}", media_url);
            return Ok(None);
        }
        let (data, ext) = if media_url.starts_with("http://") || media_url.starts_with("https://") {
            static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
                reqwest::Client::builder()
                    .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:141.0) Gecko/20100101 Firefox/141.0")
                    .build()
                    .unwrap()
            });
            let resp = CLIENT.get(media_url).send().await.context("Failed to download media")?;
            let ext = resp
                .headers()
                .get(reqwest::header::CONTENT_TYPE)
                .and_then(|h| h.to_str().ok())
                .and_then(mime_guess::get_mime_extensions_str)
                .and_then(|exts| exts.first())
                .copied()
                .unwrap_or("bin");
            let data = resp.bytes().await?.to_vec();
            Result::<_, anyhow::Error>::Ok((data, ext.to_lowercase()))
        } else {
            let media_path = Path::new(&media_url);
            let local_path = if media_path.is_absolute() {
                media_path.to_path_buf()
            } else {
                base_dir.join(media_path)
            };
            let data = tokio::fs::read(&local_path)
                .await
                .context("Failed to read media file")?;
            let ext = local_path
                .extension()
                .and_then(|s| s.to_str())
                .map(|s| s.to_lowercase())
                .unwrap_or("bin".to_owned());
            Ok((data, ext))
        }?;
        let hash = Self::hash_binary_data(&data);
        let (data, ext) = if ["jpg", "jpeg", "png", "webp"].contains(&ext.as_str()) {
            if let Ok(webp_data) = Self::convert_to_webp(&data).await {
                (webp_data, "webp".to_owned())
            } else {
                warn!("Failed to convert image to webp, using original data");
                (data, ext)
            }
        } else {
            (data, ext)
        };
        let key = format!("{}/{}/{}.{}", self.prefix, slug, hash, ext);
        if self.file_exists(&key).await? {
            dbg!("File already exists in S3, skipping upload: {}", &key);
            Ok(Some(UploadedMedia {
                original_url: media_url.to_owned(),
                new_url: format!("{}/{}", self.s3_domain, key),
            }))
        } else {
            self.upload_file(&key, data).await?;
            Ok(Some(UploadedMedia {
                original_url: media_url.to_owned(),
                new_url: format!("{}/{}", self.s3_domain, key),
            }))
        }
    }

    fn hash_binary_data(data: &[u8]) -> String {
        let mut hasher = XxHash3_64::default();
        hasher.write(data);
        hasher.finish().to_string()
    }

    async fn convert_to_webp(data: &[u8]) -> Result<Vec<u8>> {
        let mut command = tokio::process::Command::new("cwebp")
            .args([
                "-sharp_yuv",
                "-mt",
                "-q",
                "80",
                "-metadata",
                "all",
                "-o",
                "-",
                "--",
                "-",
            ])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .spawn()?;
        if let Some(mut stdin) = command.stdin.take() {
            stdin.write_all(data).await?;
        }
        let output = command.wait_with_output().await?;
        if !output.status.success() {
            bail!("cwebp failed with status: {}", output.status);
        }
        Ok(output.stdout)
    }

    async fn file_exists(&self, key: &str) -> Result<bool> {
        match self
            .s3_client
            .head_object()
            .bucket(self.bucket_name)
            .key(key)
            .send()
            .await
        {
            Ok(_) => Ok(true),
            Err(SdkError::ServiceError(err)) if err.err().is_not_found() => Ok(false),
            Err(e) => Err(e.into()),
        }
    }

    async fn upload_file(&self, key: &str, content: Vec<u8>) -> Result<()> {
        let content_type = match key.rsplit_once('.').map(|(_, ext)| ext.to_lowercase()).as_deref() {
            Some("jpg") | Some("jpeg") => "image/jpeg",
            Some("png") => "image/png",
            Some("gif") => "image/gif",
            Some("webp") => "image/webp",
            Some("mp4") => "video/mp4",
            Some("webm") => "video/webm",
            Some("mov") => "video/quicktime",
            _ => "application/octet-stream",
        };
        self.s3_client
            .put_object()
            .bucket(self.bucket_name)
            .key(key)
            .body(content.into())
            .content_type(content_type)
            .send()
            .await?;
        Ok(())
    }
}
