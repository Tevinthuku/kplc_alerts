use anyhow::Context;
use async_trait::async_trait;
use redis_client::client::CLIENT;
use tasks::{
    configuration::REPO,
    text_search::{generate_search_url, search_locations_by_text},
};
use use_cases::search_for_locations::{
    LocationApiResponse, LocationResponseWithStatus, LocationSearchApi, Status,
};

use crate::producer::Producer;

#[async_trait]
impl LocationSearchApi for Producer {
    async fn search(&self, text: String) -> anyhow::Result<LocationResponseWithStatus> {
        let repository = REPO.get().await;
        let url = generate_search_url(text.clone())?;
        let cached_response = repository.get_cached_text_search_response(&url).await?;
        if let Some(response) = cached_response {
            let responses = response
                .predictions
                .into_iter()
                .map(|prediction| LocationApiResponse {
                    id: prediction.place_id.into(),
                    name: prediction.structured_formatting.main_text,
                    address: prediction.structured_formatting.secondary_text,
                })
                .collect();
            return Ok(LocationResponseWithStatus {
                responses,
                status: Status::Success,
            });
        }
        let progress_tracker = CLIENT.get().await;

        let progress = progress_tracker.get_status::<_, Status>(&text).await?;

        if let Some(progress) = progress {
            let progress = if matches!(progress, Status::Success) {
                /// return pending here, so that the client can make the call again to the backend
                /// which will then be handled by the cached result
                Status::Pending
            } else {
                progress
            };
            return Ok(LocationResponseWithStatus {
                responses: Default::default(),
                status: progress,
            });
        }

        let status = progress_tracker.set_status(&text, Status::Pending).await?;

        self.app
            .send_task(search_locations_by_text::new(text))
            .await
            .context("Failed to send task")?;

        Ok(LocationResponseWithStatus {
            responses: Default::default(),
            status,
        })
    }
}
