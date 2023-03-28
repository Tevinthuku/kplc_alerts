use crate::producer::Producer;
use anyhow::{anyhow, Context};
use async_trait::async_trait;
use entities::locations::ExternalLocationId;
use entities::subscriptions::SubscriberId;
use std::collections::HashMap;
use tasks::subscribe_to_location::fetch_and_subscribe_to_locations;
use tasks::utils::progress_tracking::generate_key;
use use_cases::subscriber_locations::data::LocationInput;
use use_cases::subscriber_locations::subscribe_to_location::{LocationSubscriber, TaskId};
use uuid::Uuid;

fn generate_task_id() -> TaskId {
    let key = Uuid::new_v4();
    let key = generate_key(&key.to_string());
    key.into()
}

#[async_trait]
impl LocationSubscriber for Producer {
    async fn subscribe_to_location(
        &self,
        location: LocationInput<ExternalLocationId>,
        subscriber_id: SubscriberId,
    ) -> anyhow::Result<TaskId> {
        let task_id = generate_task_id();
        self.app
            .send_task(fetch_and_subscribe_to_locations::new(
                location.primary_id().to_owned(),
                location.nearby_locations,
                subscriber_id,
                task_id.clone(),
            ))
            .await
            .context("Failed to send task")?;

        Ok(task_id)
    }
}
