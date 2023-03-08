use std::sync::Arc;

use anyhow::{Context, Ok};
use use_cases::search_for_locations::{LocationApiResponse, LocationSearchApi};

use async_trait::async_trait;
use secrecy::ExposeSecret;
use serde::Deserialize;
use serde::Serialize;
use shared_kernel::http_client::HttpClient;
use url::Url;

use crate::configuration::{LocationSearcherConfig, Settings};

#[derive(Deserialize, Serialize)]
pub struct MatchedSubstrings {
    length: usize,
    offset: usize,
}

#[derive(Deserialize, Serialize)]
pub struct LocationSearchApiResponsePrediction {
    description: String,
    matched_substrings: Vec<MatchedSubstrings>,
    place_id: String,
}

#[derive(Deserialize, Serialize)]
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
    fn is_cacheable(&self) -> bool {
        matches!(self, StatusCode::OK | StatusCode::ZERORESULTS)
    }
}

#[derive(Deserialize, Serialize)]
pub struct LocationSearchApiResponse {
    status: StatusCode,
    predictions: Vec<LocationSearchApiResponsePrediction>,
    error_message: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct StatusContainer {
    status: StatusCode,
}

impl LocationSearchApiResponse {
    fn is_cacheable(&self) -> bool {
        self.status.is_cacheable()
    }
}

#[async_trait]
pub trait LocationSearchApiResponseCache: Send + Sync {
    async fn get(&self, key: &Url) -> anyhow::Result<Option<LocationSearchApiResponse>>;
    async fn set(&self, key: &Url, response: &serde_json::Value) -> anyhow::Result<()>;
}

pub struct Searcher {
    cache: Arc<dyn LocationSearchApiResponseCache>,
    config: LocationSearcherConfig,
}

impl From<LocationSearchApiResponse> for Vec<LocationApiResponse> {
    fn from(api_response: LocationSearchApiResponse) -> Self {
        api_response
            .predictions
            .iter()
            .map(|prediction| LocationApiResponse {
                id: prediction.place_id.clone().into(),
                name: prediction.description.clone(),
            })
            .collect::<Vec<_>>()
    }
}

#[async_trait]
impl LocationSearchApi for Searcher {
    async fn search(&self, text: String) -> anyhow::Result<Vec<LocationApiResponse>> {
        let url = Url::parse_with_params(
            &self.config.host,
            &[
                ("key", self.config.api_key.expose_secret()),
                ("input", &text),
            ],
        )
        .context("Failed to parse url")?;

        let cached_response = self.cache.get(&url).await;

        if let Err(err) = &cached_response {
            // TODO: Log error
            println!("{err:?}")
        }

        let response = cached_response.ok().flatten();

        if let Some(response) = response {
            return Ok(response.into());
        }

        let api_response = HttpClient::get_json::<serde_json::Value>(url.clone()).await?;
        let response = serde_json::from_value::<LocationSearchApiResponse>(api_response.clone())
            .context("Failed to get valid api response")?;
        if response.is_cacheable() {
            let cached_result = self.cache.set(&url, &api_response).await;
            if let Err(err) = cached_result {
                // TODO: Log error
                println!("{err:?}")
            }
        }

        Ok(response.into())
    }
}

impl Searcher {
    pub fn new(cache: Arc<dyn LocationSearchApiResponseCache>) -> anyhow::Result<Self> {
        let settings = Settings::parse()?;

        Ok(Searcher {
            cache,
            config: settings.location_searcher,
        })
    }
}
