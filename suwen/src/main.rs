#[macro_use]
extern crate tracing;

use std::path::PathBuf;
use std::sync::LazyLock;

use anyhow::{Result, bail};
use axum::Extension;
use clap::{Parser, Subcommand};
use dashmap::DashMap;
use suwen_api::db;
use suwen_config::CONFIG;
use suwen_markdown::manager::importer::XlogImporter;
use suwen_markdown::manager::{MarkdownManager, watcher};
use tokio::signal;
use tokio::sync::mpsc;
use tracing_subscriber::util::SubscriberInitExt;

static BACKEND_PORT: LazyLock<String> =
    LazyLock::new(|| std::env::var("BACKEND_PORT").unwrap_or_else(|_| "4545".to_string()));

#[derive(Parser)]
#[command(name = "suwen", about = "Suwen - A modern blogging platform", version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the server
    Serve,
    /// Import content from xlog platform
    ImportXlog {
        /// Source path
        #[arg(short = 's', long)]
        source: PathBuf,
        /// Output path
        #[arg(short, long)]
        output: PathBuf,
        /// Image output path (optional, defaults to output/objects)
        #[arg(short = 'i', long)]
        obj_output: Option<PathBuf>,
    },
    /// Rename slug
    RenameSlug {
        /// Output path
        #[arg(short, long)]
        output: PathBuf,
        /// Image output path (optional, defaults to output/objects)
        #[arg(short = 'i', long)]
        obj_output: Option<PathBuf>,
        /// Old slug
        #[arg(long)]
        old_slug: String,
        /// New slug
        #[arg(long)]
        new_slug: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Serve) | None => serve().await,
        Some(Commands::ImportXlog {
            source,
            output,
            obj_output,
        }) => import_xlog_content(source, output, obj_output).await,
        Some(Commands::RenameSlug {
            output,
            obj_output,
            old_slug,
            new_slug,
        }) => rename_slug(output, obj_output, old_slug, new_slug),
    }
}

async fn serve() -> Result<()> {
    let (redis_connection, sqlite_connection) = init().await?;
    let router = suwen_api::router()
        .layer(Extension(sqlite_connection.clone()))
        .layer(Extension(redis_connection.clone()));
    let bind_address = format!("0.0.0.0:{}", BACKEND_PORT.as_str());
    let listener = tokio::net::TcpListener::bind(&bind_address).await?;

    let (db_sender, mut db_receiver) = mpsc::unbounded_channel();
    let summary_cache: DashMap<String, Option<String>> = DashMap::new();

    if let Some(markdown_path) = &CONFIG.markdown_path {
        let watch_path = PathBuf::from(markdown_path);
        if watch_path.exists() {
            info!("Starting markdown watcher at {:?}", watch_path);
            let watcher = watcher::MarkdownWatcher::new(watch_path, None, db_sender);
            tokio::spawn(async move {
                if let Err(e) = watcher.start_watching().await {
                    error!("Markdown watcher error: {}", e);
                }
            });
        } else {
            bail!("Markdown path {:?} does not exist", watch_path);
        }
    }

    let db_conn = sqlite_connection.clone();
    let cache_clone = summary_cache.clone();
    tokio::spawn(async move {
        while let Some(change) = db_receiver.recv().await {
            if let Err(e) = db::handle_markdown_change(&db_conn, change, db::Lang::ZhCN, &cache_clone).await {
                error!("Failed to handle markdown change: {}", e);
            }
        }
    });

    let (tx, rx) = tokio::sync::oneshot::channel();
    tokio::spawn(async move {
        let _ = tx.send(axum::serve(listener, router).await);
    });
    info!("Server running on {}", bind_address);
    let mut term = signal::unix::signal(signal::unix::SignalKind::terminate())?;
    let mut int = signal::unix::signal(signal::unix::SignalKind::interrupt())?;
    tokio::select! {
        res = rx => {
            error!("Server terminated unexpectedly with result: {:?}", res);
        }
        _ = term.recv() => {
            info!("Shutting down server...");
        }
        _ = int.recv() => {
            info!("Shutting down server...");
        }
    };
    let _ = sqlite_connection.close().await;
    info!("Server shutdown completed");
    Ok(())
}

async fn init() -> Result<(redis::aio::ConnectionManager, db::DatabaseConnection)> {
    tracing_subscriber::fmt::Subscriber::builder()
        .compact()
        .with_target(false)
        .with_env_filter(
            "NONE,suwen=INFO,suwen-api=INFO,suwen-config=INFO,\
            suwen-entity=INFO,suwen-llm=INFO,\
            suwen-markdown=INFO,suwen-migration=INFO",
        )
        .with_timer(tracing_subscriber::fmt::time::ChronoLocal::new(
            "%b %d %H:%M:%S".to_owned(),
        ))
        .finish()
        .try_init()
        .expect("Failed to initialize logging");
    let redis_client = redis::Client::open(CONFIG.redis_url.as_str())?;
    let redis_connection = redis_client.get_connection_manager().await?;
    let sqlite_connection = db::database_connection().await?;
    db::init(&sqlite_connection).await?;
    Ok((redis_connection, sqlite_connection))
}

async fn import_xlog_content(source: PathBuf, output: PathBuf, obj_output: Option<PathBuf>) -> Result<()> {
    info!("Starting to import xlog content from {:?} to {:?}", source, output);

    suwen_markdown::manager::importer::import_path(source, output, obj_output, XlogImporter).await?;

    info!("Content import completed");
    Ok(())
}

fn rename_slug(output: PathBuf, obj_output: Option<PathBuf>, old_slug: String, new_slug: String) -> Result<()> {
    info!("Renaming slug: {} -> {}", old_slug, new_slug);

    let manager = MarkdownManager::new(output, obj_output);
    manager.rename_slug(&old_slug, &new_slug)?;

    info!("Slug rename completed");
    Ok(())
}
