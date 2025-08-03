mod query;
mod schema;
mod utils;

use std::fmt::{Display, Formatter};

pub use query::*;
pub use schema::*;

use anyhow::Result;
pub use sea_orm::DatabaseConnection;
use sea_orm::{ConnectOptions, Database};
use suwen_migration::{Migrator, MigratorTrait};

pub async fn database_connection() -> Result<DatabaseConnection> {
    let mut option = ConnectOptions::new("sqlite://suwen.db?mode=rwc");
    option
        .max_connections(30)
        .acquire_timeout(std::time::Duration::from_secs(30));
    let conn = Database::connect(option).await?;
    Migrator::up(&conn, None).await?;
    Ok(conn)
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
