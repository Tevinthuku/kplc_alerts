use anyhow::{anyhow, Context};
use async_once::AsyncOnce;
use celery::export::async_trait;
use celery::prelude::*;
use entities::locations::ExternalLocationId;
use entities::locations::LocationInput;
use entities::subscriptions::{AffectedSubscriber, SubscriberId};
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use lazy_static::lazy_static;
use redis::{AsyncCommands, Commands};
use shared_kernel::http_client::HttpClient;
use sqlx_postgres::repository::Repository;
use std::collections::VecDeque;
use std::sync::Arc;
use url::Url;

lazy_static! {
    static ref REPO: AsyncOnce<Repository> = AsyncOnce::new(async {
        Repository::new()
            .await
            .expect("Repository to be initialzed")
    });
}

use serde::Deserialize;
use serde::Serialize;
use use_cases::subscriber_locations::data::LocationId;
use uuid::Uuid;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum StatusCode {
    OK,
    #[serde(rename = "ZERO_RESULTS")]
    ZERORESULTS,
    #[serde(rename = "INVALID_REQUEST")]
    INVALIDREQUEST,
    #[serde(rename = "OVER_QUERY_LIMIT")]
    OVERQUERYLIMIT,
    #[serde(rename = "REQUEST_DENIED")]
    REQUESTDENIED,
    #[serde(rename = "UNKNOWN_ERROR")]
    UNKNOWNERROR,
}

impl StatusCode {
    pub fn is_cacheable(&self) -> bool {
        matches!(self, StatusCode::OK | StatusCode::ZERORESULTS)
    }
}

pub async fn subscribe_to_primary_location(
    location_id: LocationId,
    subscriber: SubscriberId,
) -> TaskResult<Uuid> {
    let repo = REPO.get().await;
    repo.subscribe_to_location(subscriber, location_id)
        .await
        .map_err(|err| TaskError::ExpectedError(format!("{err}")))
}

#[celery::task(max_retries = 200)]
pub async fn get_and_subscribe_to_nearby_location(
    external_id: ExternalLocationId,
    subscriber_primary_location_id: Uuid,
) -> TaskResult<()> {
    let repo = REPO.get().await;
    let id = get_location_from_cache_or_api(external_id).await?;
    repo.subscribe_to_adjuscent_location(subscriber_primary_location_id, id)
        .await
        .map_err(|err| TaskError::ExpectedError(format!("{err}")))
}
async fn save_location_returning_id(location: LocationInput) -> TaskResult<LocationId> {
    let repo = REPO.get().await;
    let external_id = location.external_id.clone();
    repo.insert_location(location)
        .await
        .map_err(|err| TaskError::ExpectedError(format!("{err}")))?;
    let location_id = repo
        .find_location_id(external_id)
        .await
        .map_err(|err| TaskError::ExpectedError(format!("{err}")))?;
    location_id
        .ok_or("Location not found")
        .map_err(|err| TaskError::ExpectedError(err.to_string()))
}

const PLACE_DETAILS_PATH: &str = "/place/details/json";

fn generate_url(id: ExternalLocationId) -> anyhow::Result<Url> {
    Url::parse_with_params(
        &format!(
            "{}{}",
            "https://maps.googleapis.com/maps/api", PLACE_DETAILS_PATH
        ),
        &[("key", ""), ("place_id", &id.inner())],
    )
    .context("Failed to parse Url")
}

// TODO: Instead of just passing the URL, maybe pass a struct that has both URL and ExternalId
#[celery::task(max_retries = 200, bind = true)]
pub async fn fetch_and_subscribe_to_locations(
    task: &Self,
    primary_location: ExternalLocationId,
    nearby_locations: Vec<ExternalLocationId>,
    subscriber: SubscriberId,
) -> TaskResult<()> {
    let location_id = get_location_from_cache_or_api(primary_location).await?;
    let id = subscribe_to_primary_location(location_id, subscriber).await?;

    let mut futures: FuturesUnordered<_> = nearby_locations
        .into_iter()
        .map(|nearby_location| {
            task.request
                .app
                .send_task(get_and_subscribe_to_nearby_location::new(
                    nearby_location,
                    id,
                ))
        })
        .collect();

    while let Some(result) = futures.next().await {
        if let Err(e) = result {
            // TODO: Setup logging
            println!("Error creating nearby location search: {e:?}")
        }
    }

    Ok(())
}

async fn get_location_from_cache_or_api(external_id: ExternalLocationId) -> TaskResult<LocationId> {
    let repo = REPO.get().await;
    let item = repo
        .find_location_id(external_id.clone())
        .await
        .map_err(|err| TaskError::ExpectedError(format!("{err}")))?;

    if let Some(item) = item {
        return Ok(item);
    }

    let url =
        generate_url(external_id).map_err(|err| TaskError::UnexpectedError(format!("{err}")))?;

    let location = get_place_details(url).await?;

    save_location_returning_id(location.clone()).await
}

async fn get_place_details(url: Url) -> TaskResult<LocationInput> {
    #[derive(Deserialize, Debug, Clone)]
    struct ResponseResult {
        name: String,
        formatted_address: String,
        place_id: String,
    }
    #[derive(Deserialize, Debug, Clone)]
    struct Response {
        result: Option<ResponseResult>,
        status: StatusCode,
    }
    let result = HttpClient::get_json::<serde_json::Value>(url)
        .await
        .map_err(|err| TaskError::ExpectedError(format!("{err}")))?;
    let response: Response = serde_json::from_value(result.clone())
        .with_unexpected_err(|| format!("Invalid response {result:?}"))?;

    if response.status.is_cacheable() {
        if let Some(response_result) = response.result {
            return Ok(LocationInput {
                external_id: ExternalLocationId::new(response_result.place_id),
                name: response_result.name,
                address: response_result.formatted_address,
                api_response: result,
            });
        }
    }
    Err(TaskError::UnexpectedError(format!(
        "Failed to get valid response {result:?}"
    )))
}
