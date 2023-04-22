use anyhow::Context;
use entities::locations::LocationId;
use entities::subscriptions::SubscriberId;
use sqlx::PgPool;
use sqlx_postgres::repository::Repository;
use uuid::Uuid;

use crate::configuration::REPO;

pub struct DataAccess<'a>(&'a Repository);

impl<'a> DataAccess<'a> {
    pub async fn new() -> DataAccess<'a> {
        let repo = REPO.get().await;
        DataAccess(repo)
    }

    fn pool(&self) -> &PgPool {
        self.0.pool()
    }

    pub async fn subscribe_to_location(
        &self,
        subscriber: SubscriberId,
        location_id: LocationId,
    ) -> anyhow::Result<Uuid> {
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
        .context("Failed to subscribe to location")?;

        let record = sqlx::query!(
            r#"
            SELECT id FROM location.subscriber_locations WHERE subscriber_id = $1 AND location_id = $2
            "#,
             subscriber,
              location_id
        ).fetch_one(self.pool()).await.context("Failed to get location")?;

        Ok(record.id)
    }
}
