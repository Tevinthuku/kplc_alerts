use crate::repository::Repository;
use anyhow::Context;
use async_trait::async_trait;
use entities::subscriptions::SubscriberId;
use use_cases::subscriber_locations::data::LocationWithId;
use use_cases::subscriber_locations::list_subscribed_locations::LocationsSubscribedRepo;

#[async_trait]
impl LocationsSubscribedRepo for Repository {
    async fn list(&self, id: SubscriberId) -> anyhow::Result<Vec<LocationWithId>> {
        let id = id.inner();
        let results = sqlx::query!(
            "
            SELECT id, location_id FROM location.subscriber_locations WHERE subscriber_id = $1
            ",
            id
        )
        .fetch_all(self.pool())
        .await
        .context("primary locations")?;

        todo!()
    }
}
