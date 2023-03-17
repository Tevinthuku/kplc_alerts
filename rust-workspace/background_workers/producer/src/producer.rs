use celery::Celery;
use std::sync::Arc;
use tasks::app;

pub struct Producer {
    pub(crate) app: Arc<Celery>,
}

impl Producer {
    async fn new() -> anyhow::Result<Self> {
        let app = app().await?;

        Ok(Self { app })
    }
}
