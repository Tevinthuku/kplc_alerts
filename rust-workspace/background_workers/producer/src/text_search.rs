use crate::producer::Producer;
use anyhow::Context;
use async_trait::async_trait;
use futures::FutureExt;
use redis_client::client::CLIENT;
use std::str::FromStr;
use std::string::ToString;
use strum_macros::Display;
use strum_macros::EnumString;
use tasks::utils::progress_tracking::{get_progress_status, set_progress_status, TaskStatus};
use tasks::{
    configuration::REPO,
    text_search::{generate_search_url, search_locations_by_text},
};
use use_cases::search_for_locations::{
    LocationApiResponse, LocationResponseWithStatus, LocationSearchApi, Status,
};

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

        let progress = get_progress_status::<String, _>(&text, |val| {
            val.map(|val| TaskStatus::from_str(&val).context("Invalid progress type"))
                .transpose()
        })
        .await?;

        if let Some(progress) = progress {
            // for Success or Pending, just return Pending,
            if matches!(progress, TaskStatus::Pending | TaskStatus::Success) {
                return Ok(LocationResponseWithStatus {
                    responses: Default::default(),
                    status: Status::Pending,
                });
            }

            return Ok(LocationResponseWithStatus {
                responses: Default::default(),
                status: progress.into(),
            });
        }

        let status = set_progress_status(&text, TaskStatus::Pending.to_string(), |val| {
            TaskStatus::from_str(&val).context("Invalid progress type")
        })
        .await?;

        self.app
            .send_task(search_locations_by_text::new(text))
            .await
            .context("Failed to send task")?;

        Ok(LocationResponseWithStatus {
            responses: Default::default(),
            status: status.into(),
        })
    }
}
