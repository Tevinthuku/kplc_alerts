use crate::{searcher::Searcher, status_code::StatusCode};
use anyhow::{anyhow, Context};
use async_trait::async_trait;
use entities::locations::ExternalLocationId;
use futures::{stream::FuturesUnordered, StreamExt};
use secrecy::ExposeSecret;
use serde::Deserialize;
use shared_kernel::http_client::HttpClient;
use std::collections::{HashMap, HashSet};
use url::Url;
use use_cases::subscriber_locations::data::{LocationId, LocationInput};
use use_cases::subscriber_locations::subscribe_to_location::LocationDetailsFinder;

pub struct LocationDetailsInput {
    pub id: ExternalLocationId,
    pub name: String,
    pub address: String,
    pub api_response: serde_json::Value, // TODO: Clean this up
}

#[async_trait]
pub trait LocationDetailsCache: Send + Sync {
    async fn find_many(
        &self,
        locations: Vec<ExternalLocationId>,
    ) -> anyhow::Result<HashMap<ExternalLocationId, LocationId>>;

    async fn save_many(&self, locations: Vec<LocationDetailsInput>) -> anyhow::Result<()>;
}

struct LocationInputAndCacheResultsWrapper(
    LocationInput<ExternalLocationId>,
    HashMap<ExternalLocationId, LocationId>,
);

impl TryFrom<LocationInputAndCacheResultsWrapper> for LocationInput<LocationId> {
    type Error = anyhow::Error;

    fn try_from(value: LocationInputAndCacheResultsWrapper) -> Result<Self, Self::Error> {
        let primary_id = value.0.primary_id();
        let primary_id = value.1.get(primary_id).ok_or(anyhow!(
            "Failed to find id for location with external identifier {primary_id:?}"
        ))?;
        let nearby_location_ids = value
            .0
            .nearby_locations
            .iter()
            .map(|location| {
                value.1.get(location).cloned().ok_or(anyhow!(
                    "Failed to find id for location with external identifier {primary_id:?}"
                ))
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(LocationInput {
            id: *primary_id,
            nearby_locations: nearby_location_ids,
        })
    }
}

async fn get_place_details(url: Url) -> anyhow::Result<LocationDetailsInput> {
    #[derive(Deserialize, Debug, Clone)]
    struct ResponseResult {
        name: String,
        formatted_address: String,
        place_id: String,
    }
    #[derive(Deserialize, Debug, Clone)]

    struct Response {
        result: ResponseResult,
        status: StatusCode,
    }
    let result = HttpClient::get_json::<serde_json::Value>(url).await?;
    let response: Response =
        serde_json::from_value(result.clone()).context("Failed to get valid response")?;
    if response.status.is_cacheable() {
        return Ok(LocationDetailsInput {
            id: ExternalLocationId::new(response.result.place_id),
            name: response.result.name,
            address: response.result.formatted_address,
            api_response: result,
        });
    }
    Err(anyhow!("Failed to get valid response {result:?}"))
}
