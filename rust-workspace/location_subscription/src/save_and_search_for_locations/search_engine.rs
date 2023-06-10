use entities::locations::{ExternalLocationId, LocationId};
use itertools::Itertools;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use std::fmt::Debug;

use self::algolia_search_engine::AlgoliaClient;

use super::NearbyLocationId;

#[derive(Debug, Deserialize, Serialize)]
struct LocationDTO {
    #[serde(rename = "objectID")]
    pub object_id: LocationId,
    pub name: String,
    pub external_id: ExternalLocationId,
    pub address: String,
    pub api_response: serde_json::Value,
}

#[derive(Debug, Deserialize, Serialize)]
struct NearbyLocationDTO {
    #[serde(rename = "objectID")]
    pub object_id: NearbyLocationId,
    pub location_id: LocationId,
    pub api_response: serde_json::Value,
}

pub mod directly_affected_area_locations {
    use std::collections::HashMap;

    use entities::{locations::LocationId, power_interruptions::location::AreaName};
    use futures::{stream::FuturesUnordered, StreamExt};
    use itertools::Itertools;

    use crate::save_and_search_for_locations::searcheable_candidate::SearcheableAreaName;

    use super::{LocationDTO, SearchEngine as SearchEngineInner};
    const PRIMARY_LOCATIONS_INDEX: &str = "primary_locations";

    pub struct DirectlyAffectedLocationsSearchEngine {
        search_engine: SearchEngineInner,
        area_name: AreaName,
    }

    impl DirectlyAffectedLocationsSearchEngine {
        pub fn new(area_name: AreaName) -> Self {
            Self {
                search_engine: SearchEngineInner::new(),
                area_name,
            }
        }

        #[tracing::instrument(err, skip(self), level = "info")]
        pub async fn search(
            &self,
            items: Vec<String>,
        ) -> anyhow::Result<HashMap<String, LocationId>> {
            let searcheable_area_names = SearcheableAreaName::new(&self.area_name);
            let searcheable_area_names = searcheable_area_names.into_inner();
            let mapping_of_searcheable_item_to_item = searcheable_area_names
                .into_iter()
                .flat_map(|area| {
                    items
                        .iter()
                        .map(move |item| (format!("{} {}", item, &area), item.to_owned()))
                })
                .collect::<HashMap<_, _>>();

            let searcheable_items = mapping_of_searcheable_item_to_item
                .keys()
                .cloned()
                .collect_vec();
            let mut results = vec![];
            let mut futures: FuturesUnordered<_> = searcheable_items
                .into_iter()
                .map(|query| {
                    self.search_engine
                        .search::<LocationDTO, String>(vec![PRIMARY_LOCATIONS_INDEX], query)
                })
                .collect();

            while let Some(result) = futures.next().await {
                results.push(result?);
            }
            let results = results
                .into_iter()
                .filter_map(|(query, data)| {
                    mapping_of_searcheable_item_to_item
                        .get(&query)
                        .map(|item| (item.to_owned(), data.object_id))
                })
                .collect::<HashMap<_, _>>();
            Ok(results)
        }
    }
}

pub mod potentially_affected_area_locations {
    use std::collections::HashMap;

    use entities::{locations::LocationId, power_interruptions::location::AreaName};
    use futures::{stream::FuturesUnordered, StreamExt};
    use itertools::Itertools;

    use crate::save_and_search_for_locations::searcheable_candidate::SearcheableAreaName;

    use super::{NearbyLocationDTO, SearchEngine as SearchEngineInner};

    const NEARBY_LOCATIONS_INDEX: &str = "primary_locations";

    pub struct NearbyLocationsSearchEngine {
        search_engine: SearchEngineInner,
        area_name: AreaName,
    }

    impl NearbyLocationsSearchEngine {
        pub fn new(area_name: AreaName) -> Self {
            Self {
                search_engine: SearchEngineInner::new(),
                area_name,
            }
        }

        #[tracing::instrument(err, skip(self), level = "info")]
        pub async fn search(
            &self,
            items: Vec<String>,
        ) -> anyhow::Result<HashMap<String, LocationId>> {
            let searcheable_area_names = SearcheableAreaName::new(&self.area_name);
            let searcheable_area_names = searcheable_area_names.into_inner();
            let mapping_of_searcheable_item_to_item = searcheable_area_names
                .into_iter()
                .flat_map(|area| {
                    items
                        .iter()
                        .map(move |item| (format!("{} {}", item, &area), item.to_owned()))
                })
                .collect::<HashMap<_, _>>();

            let searcheable_items = mapping_of_searcheable_item_to_item
                .keys()
                .cloned()
                .collect_vec();
            let mut results = vec![];
            let mut futures: FuturesUnordered<_> = searcheable_items
                .into_iter()
                .map(|query| {
                    self.search_engine
                        .search::<NearbyLocationDTO, String>(vec![NEARBY_LOCATIONS_INDEX], query)
                })
                .collect();

            while let Some(result) = futures.next().await {
                results.push(result?);
            }
            let results = results
                .into_iter()
                .filter_map(|(query, data)| {
                    mapping_of_searcheable_item_to_item
                        .get(&query)
                        .map(|item| (item.to_owned(), data.location_id))
                })
                .collect::<HashMap<_, _>>();
            Ok(results)
        }
    }
}

pub struct SearchEngine {
    client: AlgoliaClient,
}

impl SearchEngine {
    pub fn new() -> Self {
        Self {
            client: AlgoliaClient::new(),
        }
    }

    #[tracing::instrument(err, skip(self), level = "info")]
    pub async fn save_object<DTO: DeserializeOwned>(
        &self,
        index: impl ToString + Debug,
        body: Value,
    ) -> anyhow::Result<DTO> {
        self.client.post::<DTO>(index.to_string(), body).await
    }

    #[tracing::instrument(err, skip(self), level = "info")]
    pub async fn search<DTO: DeserializeOwned, Q: ToString + Debug>(
        &self,
        index: Vec<impl ToString + Debug>,
        query: Q,
    ) -> anyhow::Result<(Q, DTO)> {
        let indexes = index.into_iter().map(|data| data.to_string()).collect_vec();
        let response = self.client.get::<DTO>(indexes, query.to_string()).await?;

        Ok((query, response))
    }

    #[tracing::instrument(err, skip(self), level = "info")]
    pub async fn import(
        &self,
        index: impl ToString + Debug,
        data: Vec<Value>,
    ) -> anyhow::Result<()> {
        self.client.import(index.to_string(), data).await
    }
}

mod algolia_search_engine {
    use anyhow::{anyhow, bail, Context};

    use itertools::Itertools;
    use serde::{de::DeserializeOwned, Deserialize, Serialize};
    use serde_json::Value;
    use shared_kernel::non_empty_string;
    use shared_kernel::{http_client::HttpClient, string_key};
    use std::{collections::HashMap, fmt::Debug, iter};
    use tracing::{error, warn};
    use url::Url;

    use crate::config::SETTINGS_CONFIG;
    string_key!(APIKey);
    string_key!(ApplicationId);
    non_empty_string!(IndexName);
    non_empty_string!(Query);

    const NUMBER_OF_ITEMS_PER_REQUEST: usize = 100;

    //The primary hosts are {Application-ID}.algolia.net for write operations and {Application-ID}-dsn.algolia.net for read operations.
    // The *-dsn host guarantees high availability through automatic load balancing and also leverages the Distributed Search Network (if you subscribed that option).
    // To guarantee a high availability, you should implement a retry strategy for all API calls on the following fallback hosts:
    // {Application-ID}-1.algolianet.com, {Application-ID}-2.algolianet.com, {Application-ID}-3.algolianet.com.
    // (Note that the domain is different because it’s hosted on another DNS provider, to increase reliability).
    // It’s best to shuffle (randomize) the list of fallback hosts to ensure load balancing across clients. All Algolia API clients implement this retry strategy.
    struct Hosts {
        read_hosts: Vec<String>,
        write_hosts: Vec<String>,
    }

    impl Hosts {
        fn new(application_id: ApplicationId) -> Self {
            let fallback_hosts = (1..=3)
                .into_iter()
                .map(|item| format!("{}-{item}.algolianet.com", &application_id))
                .collect_vec();
            let primary_read_host = format!("{}-dsn.algolia.net", &application_id);
            let primary_write_host = format!("{}.algolia.net", &application_id);
            Self {
                read_hosts: iter::once(primary_read_host)
                    .chain(fallback_hosts.iter().cloned())
                    .collect_vec(),
                write_hosts: iter::once(primary_write_host)
                    .chain(fallback_hosts.into_iter())
                    .collect_vec(),
            }
        }
    }

    struct AlgoliaHeaders(HashMap<&'static str, String>);

    impl AlgoliaHeaders {
        fn new(api_key: APIKey, application_id: ApplicationId) -> Self {
            let headers = HashMap::from([
                ("X-Algolia-API-Key", api_key.to_string()),
                ("X-Algolia-Application-Id", application_id.to_string()),
            ]);
            Self(headers)
        }

        fn inner(&self) -> HashMap<&'static str, String> {
            self.0.clone()
        }
    }

    pub struct AlgoliaClient {
        client: HttpClient,
        hosts: Hosts,
        headers: AlgoliaHeaders,
    }

    impl AlgoliaClient {
        pub fn new() -> Self {
            let api_key = SETTINGS_CONFIG.search_engine.api_key.clone().into();
            let application_id: ApplicationId =
                SETTINGS_CONFIG.search_engine.application_key.clone().into();
            let headers = AlgoliaHeaders::new(api_key, application_id.clone());
            Self {
                client: HttpClient,
                hosts: Hosts::new(application_id),
                headers,
            }
        }

        #[tracing::instrument(err, skip(self), level = "info")]
        pub async fn post<DTO: DeserializeOwned>(
            &self,
            index: impl TryInto<IndexName, Error = String> + Debug,
            body: Value,
        ) -> Result<DTO, anyhow::Error> {
            let index = index.try_into().map_err(|err| anyhow!(err))?;
            let urls = self
                .hosts
                .write_hosts
                .iter()
                .map(|host| {
                    Url::parse(&format!("https://{}/1/indexes/{}", host, index))
                        .context("Failed to parse url")
                })
                .collect::<Result<Vec<_>, anyhow::Error>>()?;
            let mut errors = vec![];
            for url in urls {
                let result =
                    HttpClient::post_json::<DTO>(url, self.headers.inner(), body.clone()).await;
                match result {
                    Ok(res) => return Ok(res),
                    Err(err) => {
                        warn!("failed to get response from POST request {:?}", err);
                        errors.push(err)
                    }
                }
            }
            error!(
                "Errors from inserting data from index {} are {:?}",
                &index, errors
            );
            bail!("Failed to return response from algolia")
        }

        #[tracing::instrument(err, skip(self), level = "info")]
        pub async fn get<DTO: DeserializeOwned>(
            &self,
            indexes: Vec<String>,
            query: impl TryInto<Query, Error = String> + Debug,
        ) -> Result<DTO, anyhow::Error> {
            let indexes = indexes
                .into_iter()
                .map(|data| IndexName::try_from(data).map_err(|err| anyhow!(err)))
                .collect::<Result<Vec<_>, _>>()?;
            let query = query.try_into().map_err(|err| anyhow!(err))?;

            let urls = self
                .hosts
                .read_hosts
                .iter()
                .map(|host| {
                    Url::parse_with_params(
                        &format!("https://{}/1/indexes/*/queries", host),
                        &[("query", query.clone())],
                    )
                    .context("Failed to parse url")
                })
                .collect::<Result<Vec<_>, anyhow::Error>>()?;

            #[derive(Serialize, Deserialize, Debug)]

            struct RequestIndexWithParam {
                #[serde(rename = "indexName")]
                index_name: String,
                params: String,
            }
            #[derive(Serialize, Deserialize, Debug)]
            struct RequestData {
                requests: Vec<RequestIndexWithParam>,
            }

            let body = RequestData {
                requests: indexes
                    .iter()
                    .map(|index_name| RequestIndexWithParam {
                        index_name: index_name.to_string(),
                        params: format!("query={}", query),
                    })
                    .collect_vec(),
            };
            let body = serde_json::to_value(body).context("Failed to turn to valid json")?;
            let mut errors = vec![];
            for url in urls {
                let result =
                    HttpClient::post_json::<DTO>(url, self.headers.inner(), body.clone()).await;
                match result {
                    Ok(res) => return Ok(res),
                    Err(err) => {
                        warn!("failed to get response {err:?}");
                        errors.push(err);
                    }
                }
            }
            error!(
                "Errors from indexes {:?} & query {} are {:?}",
                &indexes, &query, errors
            );
            bail!("Failed to return response from algolia")
        }

        // https://www.algolia.com/doc/rest-api/search/#batch-write-operations
        #[tracing::instrument(err, skip(self), level = "info")]
        pub async fn import(
            &self,
            index: impl TryInto<IndexName, Error = String> + Debug,
            data: Vec<Value>,
        ) -> anyhow::Result<()> {
            let index = index.try_into().map_err(|err| anyhow!(err))?;
            let urls = self
                .hosts
                .write_hosts
                .iter()
                .map(|host| {
                    Url::parse(&format!("https://{}/1/indexes/{}/batch", host, index))
                        .context("Failed to parse url")
                })
                .collect::<Result<Vec<_>, anyhow::Error>>()?;

            #[derive(Serialize, Debug)]
            enum RequestAction {
                #[serde(rename = "addObject")]
                AddObject,
            }

            #[derive(Serialize, Debug)]
            struct Request {
                action: RequestAction,
                body: Value,
            }

            #[derive(Serialize, Debug)]
            struct RequestBody {
                requests: Vec<Request>,
            }

            let mut errors = vec![];

            for chunk in data.chunks(NUMBER_OF_ITEMS_PER_REQUEST) {
                let request = RequestBody {
                    requests: chunk
                        .iter()
                        .map(|val| Request {
                            action: RequestAction::AddObject,
                            body: val.clone(),
                        })
                        .collect_vec(),
                };
                let request = serde_json::to_value(request).context("Failed to convert to json")?;
                for url in urls.iter() {
                    let result = HttpClient::post_json::<Value>(
                        url.clone(),
                        self.headers.inner(),
                        request.clone(),
                    )
                    .await;
                    if let Err(err) = result {
                        warn!("failed to get response {err:?}");
                        errors.push(err);
                    }
                }
            }

            if !errors.is_empty() {
                error!(
                    "Errors from index {:?} during import are {:?}",
                    &index, errors
                );

                bail!("Failed to import")
            }

            Ok(())
        }
    }
}
