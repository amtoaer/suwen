use anyhow::Result;
use axum::Extension;
use suwen_api::db;
#[tokio::main]
async fn main() -> Result<()> {
    let connection = init().await?;
    let router = suwen_api::router().layer(Extension(connection));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;

    let (tx, rx) = tokio::sync::oneshot::channel();
    tokio::spawn(async move {
        let _ = tx.send(axum::serve(listener, router).await);
    });
    tokio::select! {
        res = rx => {
            println!("Server aborted with result: {:?}", res);
        }
        _ = tokio::signal::ctrl_c() => {
            println!("Shutting down server...");
        }
    };
    println!("Server shutdown gracefully.");
    Ok(())
}

async fn init() -> Result<db::DatabaseConnection> {
    let connection = db::database_connection().await?;
    db::init(&connection).await?;
    Ok(connection)
}
