use std::path::PathBuf;
use std::time::Duration;

use anyhow::Result;
use notify::event::{ModifyKind, RenameMode};
use notify::{EventKind, RecursiveMode};
use notify_debouncer_full::{DebouncedEvent, new_debouncer};
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

use crate::{Markdown, MarkdownProcessor};

pub struct MarkdownWatcher {
    watch_path: PathBuf,
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
    pub fn new(watch_path: PathBuf, db_sender: mpsc::UnboundedSender<MarkdownChange>) -> Self {
        Self { watch_path, db_sender }
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
                match MarkdownProcessor::get().await.process_file(path).await {
                    Ok(markdown) => {
                        let _ = self.db_sender.send(MarkdownChange::Upsert(markdown));
                    }
                    Err(e) => {
                        warn!("Failed to process markdown file {:?}: {}", path, e);
                    }
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
                match MarkdownProcessor::get().await.process_file(&path).await {
                    Ok(markdown) => {
                        let _ = self.db_sender.send(MarkdownChange::Upsert(markdown));
                    }
                    Err(e) => {
                        warn!("Failed to process existing markdown file {:?}: {}", path, e);
                    }
                }
            }
        }
        let _ = self.db_sender.send(MarkdownChange::SyncExisting(existing_slugs));
        Ok(())
    }
}
