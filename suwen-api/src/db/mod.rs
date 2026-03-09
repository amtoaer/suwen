mod kv;
mod query;
mod schema;
mod utils;
use std::time::Duration;

use anyhow::{Context, Result};
use dirs::config_dir;
pub use kv::get_metadata_id_for_slug;
pub use query::*;
pub use schema::*;
pub use sea_orm::DatabaseConnection;
use sea_orm::sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqliteSynchronous};
use sea_orm::sqlx::{ConnectOptions as SqlxConnectOptions, Sqlite};
use sea_orm::{ConnectOptions, Database, SqlxSqliteConnector};
pub use suwen_config::Lang;
use suwen_migration::{Migrator, MigratorTrait};
use tokio::fs::create_dir_all;

async fn migrate(url: &str) -> Result<()> {
    let db = Database::connect(ConnectOptions::new(url)).await?;
    Migrator::up(&db, None).await?;
    db.close().await?;
    Ok(())
}

pub async fn database_connection() -> Result<DatabaseConnection> {
    let db_path = config_dir()
        .map(|path| path.join("suwen").join("suwen.db"))
        .unwrap_or_else(|| "suwen.db".into());
    info!("数据库存储于： {}", db_path.display());
    if let Some(parent) = db_path.parent() {
        create_dir_all(parent).await.context("创建数据库目录失败")?;
    }
    let url = format!("sqlite://{}?mode=rwc", db_path.display());
    migrate(&url).await?;
    let mut option = ConnectOptions::new(&url);
    option
        .max_connections(50)
        .min_connections(5)
        .acquire_timeout(Duration::from_secs(90));
    let connect_option = option
        .get_url()
        .parse::<SqliteConnectOptions>()
        .context("Failed to parse database URL")?
        .disable_statement_logging()
        .busy_timeout(Duration::from_secs(90))
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal)
        .optimize_on_close(true, None);
    Ok(SqlxSqliteConnector::from_sqlx_sqlite_pool(
        option
            .sqlx_pool_options::<Sqlite>()
            .connect_with(connect_option)
            .await?,
    ))
}
