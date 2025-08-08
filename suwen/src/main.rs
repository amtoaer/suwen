#[macro_use]
extern crate tracing;

use anyhow::Result;
use axum::Extension;
use suwen_api::db;
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
async fn main() -> Result<()> {
    let connection = init().await?;
    let router = suwen_api::router().layer(Extension(connection.clone()));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;

    let (tx, rx) = tokio::sync::oneshot::channel();
    tokio::spawn(async move {
        let _ = tx.send(axum::serve(listener, router).await);
        info!("服务器在 127.0.0.1:3000 上运行");
    });
    tokio::select! {
        res = rx => {
            error!("服务器意外终止，结果为: {:?}", res);
        }
        _ = tokio::signal::ctrl_c() => {
            info!("正在关闭服务器..");
        }
    };
    let _ = connection.close().await;
    info!("服务器正常关闭");
    Ok(())
}

async fn init() -> Result<db::DatabaseConnection> {
    tracing_subscriber::fmt::Subscriber::builder()
        .compact()
        .with_target(false)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_timer(tracing_subscriber::fmt::time::ChronoLocal::new(
            "%b %d %H:%M:%S".to_owned(),
        ))
        .finish()
        .try_init()
        .expect("日志初始化失败");
    let connection = db::database_connection().await?;
    db::init(&connection).await?;
    Ok(connection)
}
