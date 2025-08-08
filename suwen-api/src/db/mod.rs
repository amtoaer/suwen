mod query;
mod schema;
mod utils;

use std::{
    fmt::{Display, Formatter},
    time::Duration,
};

pub use query::*;
pub use schema::*;

use anyhow::{Context, Result};
use dirs::config_dir;
pub use sea_orm::DatabaseConnection;
use sea_orm::{
    ConnectOptions, Database, SqlxSqliteConnector,
    sqlx::{
        ConnectOptions as SqlxConnectOptions, Sqlite,
        sqlite::{SqliteConnectOptions, SqliteJournalMode, SqliteSynchronous},
    },
};
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

pub enum Lang {
    ZhCN,
    EnUS,
    JaJP,
    KoKR,
}

impl Display for Lang {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Lang::ZhCN => write!(f, "zh-CN"),
            Lang::EnUS => write!(f, "en-US"),
            Lang::JaJP => write!(f, "ja-JP"),
            Lang::KoKR => write!(f, "ko-KR"),
        }
    }
}

impl TryFrom<&str> for Lang {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "zh-CN" => Ok(Lang::ZhCN),
            "en-US" => Ok(Lang::EnUS),
            "ja-JP" => Ok(Lang::JaJP),
            "ko-KR" => Ok(Lang::KoKR),
            _ => Err("Unsupported language code"),
        }
    }
}
