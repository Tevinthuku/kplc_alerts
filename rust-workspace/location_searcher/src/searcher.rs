use std::sync::Arc;

use anyhow::{Context, Ok};
use use_cases::search_for_locations::{LocationApiResponse, LocationSearchApi};

use async_trait::async_trait;
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;
use serde::Serialize;
use shared_kernel::http_client::HttpClient;
use url::Url;

use crate::configuration::{LocationSearcherConfig, Settings};
use crate::location_details_finder::LocationDetailsCache;
use crate::text_search::LocationSearchApiResponseCache;

pub struct Searcher {
    pub(crate) text_search_cache: Arc<dyn LocationSearchApiResponseCache>,
    pub(crate) location_details_cache: Arc<dyn LocationDetailsCache>,
    config: LocationSearcherConfig,
}

impl Searcher {
    pub fn new(
        text_search_cache: Arc<dyn LocationSearchApiResponseCache>,
        location_details_cache: Arc<dyn LocationDetailsCache>,
    ) -> anyhow::Result<Self> {
        let settings = Settings::parse()?;

        Ok(Searcher {
            text_search_cache,
            location_details_cache,
            config: settings.location_searcher,
        })
    }

    pub(crate) fn host(&self) -> &str {
        &self.config.host
    }

    pub(crate) fn api_key(&self) -> &Secret<String> {
        &self.config.api_key
    }
}
