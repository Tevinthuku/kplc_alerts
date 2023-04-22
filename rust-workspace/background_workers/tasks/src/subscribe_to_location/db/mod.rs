mod affected_subscriber;

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

    fn pool(&self) -> &PgPool {
        self.0.pool()
    }

    pub async fn subscribe_to_primary_location(
        &self,
        subscriber: SubscriberId,
        location_id: LocationId,
    ) -> TaskResult<Uuid> {
        let subscriber = subscriber.inner();
        let location_id = location_id.inner();
        let _ = sqlx::query!(
            r#"
              INSERT INTO location.subscriber_locations (subscriber_id, location_id) 
              VALUES ($1, $2) ON CONFLICT DO NOTHING
            "#,
            subscriber,
            location_id
        )
        .execute(self.pool())
        .await
        .map_err(|err| TaskError::UnexpectedError(err.to_string()))?;

        let record = sqlx::query!(
            r#"
            SELECT id FROM location.subscriber_locations WHERE subscriber_id = $1 AND location_id = $2
            "#,
             subscriber,
              location_id
        ).fetch_one(self.pool()).await.map_err(|err| TaskError::UnexpectedError(err.to_string()))?;

        Ok(record.id)
    }

    pub async fn find_location_id(
        &self,
        location: ExternalLocationId,
    ) -> TaskResult<Option<LocationId>> {
        let pool = self.pool();
        let db_results = sqlx::query!(
            r#"
            SELECT id
            FROM location.locations WHERE external_id = $1
            "#,
            location.inner()
        )
        .fetch_optional(pool)
        .await
        .map_err(|err| TaskError::UnexpectedError(err.to_string()))?;
        Ok(db_results.map(|record| record.id.into()))
    }
}
