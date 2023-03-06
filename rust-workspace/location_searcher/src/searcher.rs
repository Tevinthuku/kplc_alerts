use std::sync::Arc;

use anyhow::Ok;
use use_cases::search_for_locations::{LocationResponse, LocationSearchApi};

use async_trait::async_trait;
use serde::Deserialize;
use serde::Serialize;

use crate::configuration::Settings;

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

#[derive(Deserialize, Serialize)]
pub struct LocationSearchApiResponse {
    status: StatusCode,
    predictions: Vec<LocationSearchApiResponsePrediction>,
    error_message: Option<String>,
}

#[async_trait]
pub trait LocationSearchApiResponseCache: Send + Sync {
    async fn get(&self, key: String) -> anyhow::Result<Option<LocationSearchApiResponse>>;
    async fn set(&self, key: String, response: LocationSearchApiResponse) -> anyhow::Result<()>;
}

pub struct Searcher {
    cache: Arc<dyn LocationSearchApiResponseCache>,
}

#[async_trait]
impl LocationSearchApi for Searcher {
    async fn search(&self, text: String) -> anyhow::Result<Vec<LocationResponse>> {
        todo!()
    }
}

impl Searcher {
    pub fn new(cache: Arc<dyn LocationSearchApiResponseCache>) -> anyhow::Result<Self> {
        let _settings = Settings::parse()?;

        Ok(Searcher { cache })
    }
}
