use async_once::AsyncOnce;
use lazy_static::lazy_static;
use serde::Deserialize;
use shared_kernel::configuration::config;
use sqlx::postgres::PgPool;
use sqlx_postgres::pool_manager::{PoolManager, PoolWrapper};

#[derive(Deserialize)]
struct PoolSettings {
    location_connections: u32,
}

#[derive(Deserialize)]
struct Settings {
    database: PoolSettings,
}

lazy_static! {
    static ref SETTINGS_CONFIG: Settings = config::<Settings>().unwrap();
    static ref POOL_MANAGER: AsyncOnce<PoolManager> = AsyncOnce::new(async {
        PoolManager::new(SETTINGS_CONFIG.database.location_connections)
            .await
            .expect("PoolManager not initialized")
    });
}

#[derive(Copy, Clone)]
pub struct DbAccess;

impl DbAccess {
    pub async fn pool() -> PoolWrapper<'static> {
        POOL_MANAGER.get().await.pool()
    }
}
