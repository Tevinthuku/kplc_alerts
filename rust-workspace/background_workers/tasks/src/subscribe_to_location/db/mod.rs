pub mod affected_subscriber;

use anyhow::Context;
use celery::{prelude::TaskError, task::TaskResult};
use entities::locations::{ExternalLocationId, LocationId};
use entities::subscriptions::SubscriberId;
use sqlx::PgPool;
use sqlx_postgres::repository::Repository;
use uuid::Uuid;

use crate::configuration::REPO;

pub struct DB<'a>(&'a Repository);

impl<'a> DB<'a> {
    pub async fn new() -> DB<'a> {
        let repo = REPO.get().await;
        DB(repo)
    }

    pub fn pool(&self) -> &PgPool {
        self.0.pool()
    }
}
