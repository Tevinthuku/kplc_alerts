use anyhow::Context;
use celery::{prelude::Task, task::TaskResultExt};
use celery::{prelude::TaskError, task::TaskResult};
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};
use shared_kernel::http_client::HttpClient;
use sqlx_postgres::cache::location_search::StatusCode;
use url::Url;

use crate::utils::get_token::get_location_token;
use crate::{
    configuration::{REPO, SETTINGS_CONFIG},
    utils::callbacks::failure_callback,
};

#[derive(Deserialize, Serialize, Debug)]
pub struct LocationSearchApiResponsePrediction {
    description: String,
    place_id: Option<String>,
    /// At this point, we don't need to fully define the structure of the values
    matched_substrings: serde_json::Value,
    structured_formatting: serde_json::Value,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct LocationSearchApiResponse {
    status: StatusCode,
    pub predictions: Vec<LocationSearchApiResponsePrediction>,
    error_message: Option<String>,
}

impl LocationSearchApiResponse {
    fn is_cacheable(&self) -> bool {
        self.status.is_cacheable()
    }

    fn remove_invalid_predictions(self) -> Self {
        let predictions = self
            .predictions
            .into_iter()
            .filter(|prediction| prediction.place_id.is_some())
            .collect::<Vec<_>>();
        Self {
            predictions,
            ..self
        }
    }
}

pub fn generate_search_url(text: String) -> anyhow::Result<Url> {
    let path_details = "/place/autocomplete/json";
    let host_with_path = &format!("{}{}", SETTINGS_CONFIG.location.host, path_details);
    Url::parse_with_params(
        host_with_path,
        &[
            ("key", SETTINGS_CONFIG.location.api_key.expose_secret()),
            ("input", &text),
            ("components", &"country:ke".to_string()),
        ],
    )
    .context("Failed to parse url")
}

#[celery::task(bind=true, max_retries = 200, retry_for_unexpected = false, on_failure = failure_callback)]
pub async fn search_locations_by_text(task: &Self, text: String) -> TaskResult<()> {
    let url =
        generate_search_url(text).map_err(|err| TaskError::UnexpectedError(format!("{err}")))?;

    let repo = REPO.get().await;
    let cached_response = repo
        .get_cached_text_search_response(&url)
        .await
        .map_err(|err| TaskError::UnexpectedError(format!("{err}")))?;
    if cached_response.is_some() {
        // Don't return anything, once the client makes a second request
        // the response will be in the cache ready for them
        return Ok(());
    }

    let token_count = get_location_token().await?;

    if token_count < 0 {
        return Task::retry_with_countdown(task, 1);
    }

    let api_response = HttpClient::get_json::<LocationSearchApiResponse>(url.clone())
        .await
        .map_err(|err| TaskError::UnexpectedError(err.to_string()))?;

    let response = api_response.remove_invalid_predictions();

    let api_response = serde_json::to_string(&response)
        .with_unexpected_err(|| "Failed to convert api_response to string")?;

    let api_response = serde_json::from_str(&api_response)
        .with_unexpected_err(|| "Failed to convert api_response to JSON value")?;

    if response.is_cacheable() {
        return repo
            .set_cached_text_search_response(&url, &api_response)
            .await
            .map_err(|err| TaskError::UnexpectedError(err.to_string()));
    }

    Err(TaskError::UnexpectedError(format!(
        "Invalid status code in the response {response:?}"
    )))
}
