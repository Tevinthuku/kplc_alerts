use crate::configuration::Settings;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::sync::Arc;

#[derive(Clone)]
pub struct PoolManager {
    pub(crate) pg_pool: Arc<PgPool>,
}

impl PoolManager {
    pub async fn new(max_connections: u32) -> anyhow::Result<Self> {
        let pg_connection = Settings::with_db()?;

        let pool = PgPoolOptions::new()
            .max_connections(max_connections)
            .connect_with(pg_connection)
            .await?;
        Ok(Self {
            pg_pool: Arc::new(pool),
        })
    }

    pub fn pool(&self) -> &PgPool {
        self.pg_pool.as_ref()
    }
}
