#[macro_use]
extern crate tracing;

use anyhow::Result;
use axum::Extension;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use suwen_api::db;
use suwen_markdown::manager::{MarkdownManager, importer::XlogImporter};
use tracing_subscriber::util::SubscriberInitExt;

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
    /// Convert images to WebP format
    ConvertImages {
        /// Output path
        #[arg(short, long)]
        output: PathBuf,
        /// Image output path (optional, defaults to output/objects)
        #[arg(short = 'i', long)]
        obj_output: Option<PathBuf>,
        /// Quality (0-100) for image conversion (optional, defaults to 80)
        #[arg(short = 'q', long)]
        quality: Option<f32>,
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
        Some(Commands::ConvertImages {
            output,
            obj_output,
            quality,
        }) => convert_images(output, obj_output, quality).await,
    }
}

async fn serve() -> Result<()> {
    let connection = init().await?;
    let router = suwen_api::router().layer(Extension(connection.clone()));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;

    let (tx, rx) = tokio::sync::oneshot::channel();
    tokio::spawn(async move {
        let _ = tx.send(axum::serve(listener, router).await);
        info!("Server running on 0.0.0.0:3000");
    });
    tokio::select! {
        res = rx => {
            error!("Server terminated unexpectedly with result: {:?}", res);
        }
        _ = tokio::signal::ctrl_c() => {
            info!("Shutting down server...");
        }
    };
    let _ = connection.close().await;
    info!("Server shutdown completed");
    Ok(())
}

async fn init() -> Result<db::DatabaseConnection> {
    tracing_subscriber::fmt::Subscriber::builder()
        .compact()
        .with_target(false)
        .with_env_filter("None,suwen=INFO,suwen-api=INFO,suwen-markdown=INFO")
        .with_timer(tracing_subscriber::fmt::time::ChronoLocal::new(
            "%b %d %H:%M:%S".to_owned(),
        ))
        .finish()
        .try_init()
        .expect("Failed to initialize logging");
    let connection = db::database_connection().await?;
    db::init(&connection).await?;
    Ok(connection)
}

async fn import_xlog_content(
    source: PathBuf,
    output: PathBuf,
    obj_output: Option<PathBuf>,
) -> Result<()> {
    info!(
        "Starting to import xlog content from {:?} to {:?}",
        source, output
    );

    suwen_markdown::manager::importer::import_path(source, output, obj_output, XlogImporter)
        .await?;

    info!("Content import completed");
    Ok(())
}

fn rename_slug(
    output: PathBuf,
    obj_output: Option<PathBuf>,
    old_slug: String,
    new_slug: String,
) -> Result<()> {
    info!("Renaming slug: {} -> {}", old_slug, new_slug);

    let manager = MarkdownManager::new(output, obj_output);
    manager.rename_slug(&old_slug, &new_slug)?;

    info!("Slug rename completed");
    Ok(())
}

async fn convert_images(
    output: PathBuf,
    obj_output: Option<PathBuf>,
    quality: Option<f32>,
) -> Result<()> {
    info!("Starting to convert images to WebP format");
    let manager = MarkdownManager::new(output, obj_output);
    manager.convert_images(quality).await?;
    info!("Image conversion completed");
    Ok(())
}
