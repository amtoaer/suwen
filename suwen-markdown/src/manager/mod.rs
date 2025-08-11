use std::{
    collections::HashMap,
    fs::{self, read_dir, read_to_string},
    path::{Path, PathBuf},
};

use crate::manager::importer::Markdown;
use anyhow::{Context, Error, Result, anyhow, bail, ensure};
use image::ImageReader;
use pathdiff::diff_paths;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use webp::Encoder;
pub mod importer;

pub struct MarkdownManager {
    output: PathBuf,
    image_output: PathBuf,
}

impl MarkdownManager {
    pub fn new(output: PathBuf, image_output: Option<PathBuf>) -> Self {
        let image_output = image_output.unwrap_or_else(|| output.join("images"));
        Self {
            output,
            image_output,
        }
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
        for image in read_dir(&self.image_output)? {
            let image = image?.path();
            let old_file_name = image
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or_default();
            if Self::extract_image_slug_from_file_name(&old_file_name)
                .is_some_and(|s| s == old_slug)
            {
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

    /// 将其它类型的图片压缩为 webp 以节省空间，包含的步骤：
    /// 1. 找到所有需要转换的图片并转换为 webp，并记录这些转换
    /// 2. 打开内容并根据转换将旧的图片引用替换成新的，原地写入新内容
    /// 3. 将旧的图片文件删除
    pub fn convert_images(&self, quality: Option<f32>) -> Result<()> {
        let quality = quality.unwrap_or(80.0);
        ensure!(
            quality >= 0.0 && quality <= 100.0,
            "Quality must be between 0.0 and 100.0"
        );
        let mut files_to_convert = vec![];
        for image in read_dir(&self.image_output)? {
            let image = image?.path();
            if image.is_dir() {
                continue;
            }
            if image
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.to_ascii_lowercase())
                .is_some_and(|ext| {
                    ext == "jpg" || ext == "png" || ext == "jpeg" || ext == "gif" || ext == "webp"
                })
            {
                files_to_convert.push(image);
            }
        }
        let image_rename_pairs = files_to_convert
            .par_iter()
            .map(|file| {
                let image = ImageReader::open(&file)?.with_guessed_format()?.decode()?;
                let image = Encoder::from_image(&image)
                    .map_err(|e| anyhow!("Failed to encode image {:?}: {}", file, e))?
                    .encode(quality);
                let target_path = file.with_extension("webp");
                fs::write(&target_path, &*image)?;
                Result::<_, Error>::Ok((file, target_path))
            })
            .filter_map(Result::ok)
            .collect::<Vec<_>>();
        let mut files_to_delete = Vec::new();
        let mut image_rename_map: HashMap<String, HashMap<String, String>> = HashMap::new();
        for (old, new) in image_rename_pairs {
            // webp 自身的压缩，不改变文件名，不需要处理
            if old == &new {
                continue;
            }
            files_to_delete.push(old);
            image_rename_map
                .entry(
                    Self::extract_image_slug(&old)
                        .map(|s| s.to_string())
                        .context(format!("Image file {:?} does not have a valid slug", old))?,
                )
                .or_default()
                .insert(
                    diff_paths(&old, &self.output)
                        .unwrap()
                        .to_string_lossy()
                        .to_string(),
                    diff_paths(&new, &self.output)
                        .unwrap()
                        .to_string_lossy()
                        .to_string(),
                );
        }
        for (slug, rename_map) in image_rename_map {
            let article = self.output.join(slug).with_extension("md");
            if !article.exists() {
                warn!("Article {} does not exist", article.display());
                continue;
            }
            let mut content = Markdown::from_string(&read_to_string(&article)?)?;
            content.replace_images(&rename_map)?;
            fs::write(&article, content.to_string()?)?;
        }
        for file in files_to_convert {
            fs::remove_file(file)?;
        }
        Ok(())
    }

    fn extract_image_slug(path: &Path) -> Option<&str> {
        path.file_name()
            .and_then(|s| s.to_str())
            .and_then(|s| Self::extract_image_slug_from_file_name(s))
    }

    fn extract_image_slug_from_file_name(file_name: &str) -> Option<&str> {
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
            (
                "ji-yi-ci-dui-Rust-Embed-ya-suo-de-tan-suo",
                "rust-embed-compression",
            ),
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
            (
                "uploadImageToDogedogeViaPicgo",
                "upload-image-to-dogedoge-via-picgo",
            ),
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
    #[test]
    fn test_convert_images() {
        let markdown_manager = super::MarkdownManager::new(
            PathBuf::from("/Users/amtoaer/Downloads/Zen/amtoaer/notes-imported"),
            None,
        );
        markdown_manager.convert_images(None).unwrap();
    }
}
