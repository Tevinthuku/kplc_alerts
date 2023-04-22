pub mod db;

use anyhow::Context;
use celery::export::async_trait;
use celery::prelude::*;
use entities::locations::ExternalLocationId;

use entities::subscriptions::SubscriberId;
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use secrecy::ExposeSecret;
use shared_kernel::http_client::HttpClient;
use sqlx_postgres::locations::insert_location::LocationInput;
use url::Url;

use crate::{
    send_notifications::email::send_email_notification,
    utils::{get_token::get_location_token, progress_tracking::generate_key},
};
use entities::locations::LocationId;
use redis_client::client::CLIENT;
use serde::Deserialize;

use sqlx_postgres::cache::location_search::StatusCode;
use use_cases::subscriber_locations::subscribe_to_location::TaskId;
use uuid::Uuid;

use crate::utils::progress_tracking::set_progress_status;
use crate::{
    configuration::{REPO, SETTINGS_CONFIG},
    utils::callbacks::failure_callback,
};

use self::db::DB;

#[celery::task(max_retries = 200, bind=true, retry_for_unexpected = false, on_failure = failure_callback)]
pub async fn get_and_subscribe_to_nearby_location(
    task: &Self,
    external_id: ExternalLocationId,
    subscriber_primary_location_id: Uuid,
    subscriber_id: SubscriberId,
    subscriber_directly_affected: bool,
    task_id: TaskId,
) -> TaskResult<()> {
    let db = DB::new().await;
    let id = db.find_location_id(external_id.clone()).await?;
    let location_id = match id {
        None => {
            let token_count = get_location_token().await?;

            if token_count < 0 {
                return Task::retry_with_countdown(task, 1);
            }

            get_location_from_api(external_id.clone()).await?
        }
        Some(id) => id,
    };
    let repo = REPO.get().await;

    repo.subscribe_to_adjuscent_location(subscriber_primary_location_id, location_id)
        .await
        .map_err(|err| TaskError::UnexpectedError(err.to_string()))?;

    if subscriber_directly_affected {
        return Ok(());
    }
    let notification = repo
        .subscriber_potentially_affected(subscriber_id, location_id)
        .await
        .map_err(|err| TaskError::UnexpectedError(err.to_string()))?;

    decr_count_by_one(task_id).await?;

    if let Some(notification) = notification {
        let _ = task
            .request
            .app
            .send_task(send_email_notification::new(notification))
            .await
            .with_expected_err(|| "Failed to send task")?;
    }

    Ok(())
}

async fn decr_count_by_one(task_id: TaskId) -> TaskResult<()> {
    let key = generate_key(task_id.as_ref());
    let client = CLIENT.get().await;

    client
        .decr_count(&key, 1)
        .await
        .map_err(|err| TaskError::UnexpectedError(err.to_string()))
}

#[celery::task(max_retries = 200, bind = true, retry_for_unexpected = false, on_failure = failure_callback)]
pub async fn fetch_and_subscribe_to_locations(
    task: &Self,
    primary_location: ExternalLocationId,
    nearby_locations: Vec<ExternalLocationId>,
    subscriber: SubscriberId,
    task_id: TaskId,
) -> TaskResult<()> {
    let total_count_locations = nearby_locations.len() + 1;
    set_progress_status(task_id.as_ref(), total_count_locations, |_| Ok(()))
        .await
        .map_err(|err| TaskError::UnexpectedError(err.to_string()))?;
    let db = DB::new().await;
    let id = db.find_location_id(primary_location.clone()).await?;
    let location_id = match id {
        None => {
            let token_count = get_location_token().await?;

            if token_count < 0 {
                return Task::retry_with_countdown(task, 1);
            }

            get_location_from_api(primary_location).await?
        }
        Some(id) => id,
    };

    let id = db
        .subscribe_to_primary_location(subscriber, location_id)
        .await?;

    let direct_notification = db
        .subscriber_directly_affected(subscriber, location_id)
        .await?;

    decr_count_by_one(task_id.clone()).await?;

    let subscriber_directly_affected = direct_notification.is_some();
    if let Some(notification) = direct_notification {
        let _ = task
            .request
            .app
            .send_task(send_email_notification::new(notification))
            .await
            .with_expected_err(|| "Failed to send task")?;
    }

    let mut futures: FuturesUnordered<_> = nearby_locations
        .into_iter()
        .map(|nearby_location| {
            task.request
                .app
                .send_task(get_and_subscribe_to_nearby_location::new(
                    nearby_location,
                    id,
                    subscriber,
                    subscriber_directly_affected,
                    task_id.clone(),
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

async fn save_location_returning_id(location: LocationInput) -> TaskResult<LocationId> {
    let repo = REPO.get().await;
    let external_id = location.external_id.clone();
    let _ = repo
        .insert_location(location)
        .await
        .map_err(|err| TaskError::UnexpectedError(err.to_string()))?;
    let location_id = repo
        .find_location_id(external_id)
        .await
        .map_err(|err| TaskError::UnexpectedError(err.to_string()))?;
    location_id
        .ok_or("Location not found")
        .map_err(|err| TaskError::UnexpectedError(err.to_string()))
}

fn generate_url(id: ExternalLocationId) -> anyhow::Result<Url> {
    let place_details_path = "/place/details/json";

    let host = &SETTINGS_CONFIG.location.host;
    Url::parse_with_params(
        &format!("{}{}", host, place_details_path),
        &[
            ("key", SETTINGS_CONFIG.location.api_key.expose_secret()),
            ("place_id", &id.inner()),
        ],
    )
    .context("Failed to parse Url")
}

async fn get_location_from_api(external_id: ExternalLocationId) -> TaskResult<LocationId> {
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
        .with_expected_err(|| format!("Invalid response {result:?}"))?;

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
