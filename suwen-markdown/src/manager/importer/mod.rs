mod xlog;

use std::path::PathBuf;

use anyhow::Result;
use futures::TryStreamExt;
use futures::stream::FuturesUnordered;
use tokio::fs::{self, create_dir_all};
use tokio::task::JoinSet;
pub use xlog::import_file as XlogImporter;

use crate::manager::Markdown;

pub async fn import_path<T, F>(source: PathBuf, output: PathBuf, obj_output: Option<PathBuf>, importer: T) -> Result<()>
where
    T: Fn(PathBuf, PathBuf, PathBuf) -> F,
    F: Future<Output = Result<Markdown>> + Send + 'static,
{
    create_dir_all(&output).await?;
    let obj_output = obj_output.unwrap_or_else(|| output.join("objects"));
    create_dir_all(&obj_output).await?;
    let mut join_set = JoinSet::new();
    for file in collect_files(source).await? {
        join_set.spawn(importer(file, output.clone(), obj_output.clone()));
    }
    let write_task = join_set
        .join_all()
        .await
        .into_iter()
        .filter_map(Result::ok)
        .filter_map(|result| {
            let target = output.join(format!("{}.md", result.slug()));
            if let Ok(str) = result.to_string() {
                Some(async move { fs::write(&target, str).await })
            } else {
                None
            }
        })
        .collect::<FuturesUnordered<_>>();
    Ok(write_task.try_collect().await?)
}

async fn collect_files(source: PathBuf) -> Result<Vec<PathBuf>> {
    let (mut dirs, mut files) = (vec![source], Vec::new());
    while let Some(dir) = dirs.pop() {
        let mut entries = fs::read_dir(&dir).await?;
        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if path.is_dir() {
                dirs.push(path);
            } else if path.is_file() {
                files.push(path);
            }
        }
    }
    Ok(files)
}

#[cfg(test)]
mod tests {
    use std::fs::read_to_string;
    use std::path::PathBuf;

    use crate::manager::importer::{XlogImporter, import_path};
    use crate::parse_markdown;

    #[ignore = "only for manual test"]
    #[tokio::test]
    async fn test_format_path() {
        let _ = import_path(
            PathBuf::from("/Users/amtoaer/Downloads/Zen/amtoaer/notes"),
            PathBuf::from("/Users/amtoaer/Downloads/Zen/amtoaer/notes-imported"),
            None,
            XlogImporter,
        )
        .await;
    }

    #[ignore = "only for manual test"]
    #[test]
    fn test_parse_markdown() {
        let content =
            read_to_string("/Users/amtoaer/Downloads/Zen/amtoaer/notes-imported/street-fighter-6-introduction-1.md")
                .unwrap();
        let _ = parse_markdown(&content);
    }
}
