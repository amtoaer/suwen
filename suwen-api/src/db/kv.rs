use anyhow::Result;
use sea_orm::{QuerySelect, prelude::*};
use std::sync::{Arc, LazyLock};

use dashmap::DashMap;

use crate::db;

static KEY_LOCK: LazyLock<DashMap<String, Arc<tokio::sync::Mutex<()>>>> =
    LazyLock::new(DashMap::new);

static SLUG_TO_ID: LazyLock<DashMap<String, i32>> = LazyLock::new(DashMap::new);

fn template_key<T: ToString>(scope: &str, key: T) -> String {
    format!("suwen_{}_{}", scope, key.to_string())
}

fn local_key_lock(key: String) -> Arc<tokio::sync::Mutex<()>> {
    KEY_LOCK
        .entry(key)
        .or_insert_with(|| Arc::new(tokio::sync::Mutex::new(())))
        .value()
        .clone()
}

async fn with_key_lock<R, W, K, V>(scope: &str, key: K, reader: R, writer: W) -> Result<V>
where
    R: AsyncFn(&str) -> Result<Option<V>> + Send,
    W: AsyncFnOnce(&str) -> Result<V> + Send,
    K: ToString,
{
    let key = template_key(scope, key);
    let res = reader(&key).await?;
    if let Some(res) = res {
        return Ok(res);
    }
    let lock = local_key_lock(key.clone());
    let _guard = lock.lock().await;
    let res = reader(&key).await?;
    if let Some(res) = res {
        drop(_guard);
        if Arc::strong_count(&lock) == 2 {
            KEY_LOCK.remove(&key);
        }
        return Ok(res);
    }
    let res = writer(&key).await?;
    drop(_guard);
    if Arc::strong_count(&lock) == 2 {
        KEY_LOCK.remove(&key);
    }
    Ok(res)
}

pub async fn get_metadata_id_for_slug(slug: &str, conn: &db::DatabaseConnection) -> Result<i32> {
    with_key_lock(
        "metadata_id_for_slug",
        slug,
        async |key: &str| {
            let cached: Option<i32> = SLUG_TO_ID.get(key).map(|v| *v.value());
            Ok(cached)
        },
        async move |key: &str| {
            let metadata_id = suwen_entity::content_metadata::Entity::find()
                .select_only()
                .column(suwen_entity::content_metadata::Column::Id)
                .filter(suwen_entity::content_metadata::Column::Slug.eq(slug))
                .into_tuple::<i32>()
                .one(conn)
                .await?
                .ok_or_else(|| anyhow::anyhow!("Metadata not found for slug: {}", slug))?;
            SLUG_TO_ID.insert(key.to_owned(), metadata_id);
            Ok(metadata_id)
        },
    )
    .await
}
