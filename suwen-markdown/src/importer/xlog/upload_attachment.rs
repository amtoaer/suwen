use anyhow::{Context, Result};
use std::fs::copy;
use std::{collections::HashMap, path::Path};

use crate::UPLOAD_DIR;

pub(super) fn get_uploaded(
    copied_files: &HashMap<String, String>,
    file_name: &str,
) -> Option<String> {
    let base_name = file_name
        .trim()
        .strip_prefix("./attachments/")
        .and_then(|s| s.split('.').next())?;
    copied_files
        .get(base_name)
        .map(|s| "/uploads/".to_owned() + s)
}

/// 将 `path` 相关联的 attachment（即 file_path.parent().join("attachments") 目录下的文件）移动到 UPLOAD_DIR
pub(super) fn upload_attachment(path: &Path) -> Result<HashMap<String, String>> {
    let mut copied_files = HashMap::new();
    let attachments_dir = path
        .parent()
        .map(|p| p.join("attachments"))
        .context("Failed to get attachments directory")?;
    for entry in attachments_dir.read_dir()? {
        let entry = entry?;
        let file_path = entry.path();
        if file_path.is_file() {
            let file_name = file_path
                .file_name()
                .and_then(|n| n.to_str())
                .context("Failed to get file name")?;
            copy(&file_path, UPLOAD_DIR.join(&file_name))?;
            // safety: 即使是空字符串，split('.') 也至少会有一个 ''
            let base_name = file_name.split('.').next().unwrap();
            copied_files.insert(base_name.to_owned(), file_name.to_owned());
        }
    }
    Ok(copied_files)
}
