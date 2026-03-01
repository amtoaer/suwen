use std::collections::HashMap;
use std::fs::{self, read_dir, read_to_string};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;

use anyhow::{Context, Result, anyhow, bail, ensure};
use futures::future::ready;
use futures::stream::FuturesUnordered;
use futures::{StreamExt, TryStreamExt};
pub use markdown::Markdown;
use pathdiff::diff_paths;
use tokio::process::Command;
use tokio::sync::Semaphore;

pub mod importer;
mod markdown;
pub mod watcher;

pub struct MarkdownManager {
    output: PathBuf,
    obj_output: PathBuf,
}

impl MarkdownManager {
    pub fn new(output: PathBuf, obj_output: Option<PathBuf>) -> Self {
        let obj_output = obj_output.unwrap_or_else(|| output.join("objects"));
        Self { output, obj_output }
    }

    pub async fn all_markdown_files(&self) -> Result<Vec<Markdown>> {
        let mut files = vec![];
        let mut entries = tokio::fs::read_dir(&self.output)
            .await
            .context("Failed to read markdown directory")?;
        while let Some(entry) = entries.next_entry().await? {
            if entry.path().extension().is_some_and(|ext| ext == "md") {
                files.push(Markdown::from_file(entry.path()).await?);
            }
        }
        Ok(files)
    }

    /// 重命名 slug，包含的步骤：
    /// 1. 读取旧 slug 的内容，替换 slug 相关内容后写入到新的
    /// 2. 复制旧 slug 相关的图片到新的
    /// 3. 删除旧 slug 内容和图片
    pub fn rename_slug(&self, old_slug: &str, new_slug: &str) -> Result<()> {
        let origin_path = self.output.join(old_slug).with_extension("md");
        let target_path = self.output.join(new_slug).with_extension("md");
        if origin_path == target_path {
            return Ok(());
        }
        if target_path.exists() || !origin_path.exists() {
            bail!(
                "Cannot rename slug: {} to {}. Target already exists or origin does not exist.",
                old_slug,
                new_slug
            );
        }
        let mut content = Markdown::from_string(&read_to_string(&origin_path)?)?;
        content.rename_slug(new_slug)?;
        fs::write(&target_path, content.to_string()?)?;
        let mut path_to_remove = vec![origin_path];
        for image in read_dir(&self.obj_output)? {
            let image = image?.path();
            let old_file_name = image.file_name().and_then(|s| s.to_str()).unwrap_or_default();
            if Self::extract_object_slug_from_file_name(old_file_name).is_some_and(|s| s == old_slug) {
                fs::copy(
                    &image,
                    image.with_file_name(old_file_name.replacen(old_slug, new_slug, 1)),
                )?;
                path_to_remove.push(image);
            }
        }
        for path in path_to_remove {
            fs::remove_file(path)?;
        }
        Ok(())
    }

    fn extract_object_slug(path: &Path) -> Option<&str> {
        path.file_name()
            .and_then(|s| s.to_str())
            .and_then(|s| Self::extract_object_slug_from_file_name(s))
    }

    fn extract_object_slug_from_file_name(file_name: &str) -> Option<&str> {
        file_name.rsplitn(3, '-').nth(2)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    #[ignore = "only for manual test"]
    #[test]
    fn test_rename_slug() {
        let markdown_manager = super::MarkdownManager::new(
            PathBuf::from("/Users/amtoaer/Downloads/Zen/amtoaer/notes-imported"),
            None,
        );
        for (old_slug, new_slug) in [
            ("_BoLjwA9X2sg7ev1bwrKw", "dragon-loop"),
            ("1v9CwL3tqsA4FFQ-iC6B7", "fusion-2025"),
            ("6ppkLsuV_F646SrbD_CFY", "sf6-aki"),
            ("androidSkipCheck", "android-skip-check"),
            ("archlinuxAria2", "archlinux-aria2"),
            ("bytedance2", "bytedance-2"),
            ("changeTheme", "change-theme"),
            ("cleanUpFiles", "clean-up-files"),
            ("configOnedrive", "config-onedrive"),
            ("configureVscode", "configure-vscode"),
            ("disableKeyboard", "disable-keyboard"),
            ("fishScriptingManual", "fish-scripting-manual"),
            ("formSubmit", "form-submit"),
            ("freeWildcardCertificates", "free-wildcard-certificates"),
            ("goGenerics", "go-generics"),
            ("huawei2021", "huawei-2021"),
            ("jellyfinBasicTutorial", "jellyfin-basic-tutorial"),
            ("ji-yi-ci-dui-Rust-Embed-ya-suo-de-tan-suo", "rust-embed-compression"),
            (
                "jie-ba-6-xiang-jie-di-yi-tan--quan-jiao-de-zheng-ti-jie-shao",
                "street-fighter-6-introduction-1",
            ),
            ("kO3cbR8eQjH8_GitvzO6-", "sf6-mai"),
            ("learningJava", "learning-java"),
            ("leetcodeBinary", "leetcode-binary"),
            ("leetcodeInGoLand", "leetcode-in-goland"),
            ("makeJarExecutable", "make-jar-executable"),
            ("myRssBot", "my-rss-bot"),
            ("nas_1", "nas-1"),
            ("nas_2", "nas-2"),
            ("nas_3", "nas-3"),
            ("newTerm", "new-term"),
            ("questionTest", "question-test"),
            ("refreshGoogleSitemap", "refresh-google-sitemap"),
            ("sameString", "same-string"),
            ("securityBootForArchLinux", "security-boot-for-archlinux"),
            ("singleFlight", "single-flight"),
            ("sortBasedOnGoGenerics", "sort-based-on-go-generics"),
            ("stringCenter", "string-center"),
            ("sync-cond", "sync-cond"),
            ("tencent2021Internship", "tencent-2021-internship"),
            (
                "tokiofs-zhong-flush-fang-fa-yu-biao-zhun-ku-tong-ming-fang-fa-de-cha-yi",
                "tokio-fs-flush-difference",
            ),
            ("triliumIntroduction", "trilium-introduction"),
            ("typora", "typora"),
            ("uploadImage", "upload-image"),
            ("uploadImageToDogedoge", "upload-image-to-dogedoge"),
            ("uploadImageToDogedogeViaPicgo", "upload-image-to-dogedoge-via-picgo"),
            ("useArtitalk", "use-artitalk"),
            ("useFcitx5", "use-fcitx5"),
            ("vscodeLeetcode", "vscode-leetcode"),
            ("websiteMigration", "website-migration"),
            ("windowsFontForArchlinux", "windows-font-for-archlinux"),
            ("wpsChangeLanguage", "wps-change-language"),
            ("wsl2-automatic-hosts", "wsl2-automatic-hosts"),
            ("wslHighCpuUsage", "wsl-high-cpu-usage"),
            ("wslSystemd", "wsl-systemd"),
            ("yourRssBot", "your-rss-bot"),
            (
                "zai-Mac-shang-shi-yong-Ghostty-dai-ti-Alacritty",
                "use-ghostty-instead-of-alacritty-on-mac",
            ),
            ("zshToFish", "zsh-to-fish"),
        ] {
            markdown_manager.rename_slug(old_slug, new_slug).unwrap();
        }
    }

    #[ignore = "only for manual test"]
    #[tokio::test]
    async fn test_convert_images() {
        let markdown_manager = super::MarkdownManager::new(
            PathBuf::from("/Users/amtoaer/Downloads/Zen/amtoaer/notes-imported"),
            None,
        );
        markdown_manager.convert_images(None).await.unwrap();
    }
}
