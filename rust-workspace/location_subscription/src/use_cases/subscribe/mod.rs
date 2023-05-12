mod db_access;

use anyhow::Context;
use entities::locations::{ExternalLocationId, LocationId};
use secrecy::ExposeSecret;

use crate::config::SETTINGS_CONFIG;
use crate::data_transfer::{
    AffectedSubscriber, AffectedSubscriberWithLocationMatchedAndLineSchedule, LineScheduleId,
    LineWithScheduledInterruptionTime, LocationMatchedAndLineSchedule,
};
use crate::save_and_search_for_locations::{AffectedLocation, LocationWithCoordinates};
use crate::use_cases::subscribe::db_access::SubscriptionDbAccess;
use entities::power_interruptions::location::{NairobiTZDateTime, TimeFrame};
use entities::subscriptions::SubscriberId;
use shared_kernel::uuid_key;
use thiserror::Error;
use url::Url;

#[derive(Error, Debug)]
pub enum SubscribeToLocationError {
    #[error("Internal error")]
    InternalError(#[from] anyhow::Error),
    #[error("Expected error: {0}")]
    ExpectedError(String),
}

pub struct SubscribeInteractor {
    db: SubscriptionDbAccess,
}

impl Default for SubscribeInteractor {
    fn default() -> Self {
        Self::new()
    }
}

impl SubscribeInteractor {
    pub fn new() -> Self {
        Self {
            db: SubscriptionDbAccess::new(),
        }
    }
    pub async fn subscribe_to_location(
        &self,
        subscriber_id: SubscriberId,
        external_id: ExternalLocationId,
    ) -> Result<
        Option<AffectedSubscriberWithLocationMatchedAndLineSchedule>,
        SubscribeToLocationError,
    > {
        let existing_location = self
            .db
            .find_location_by_external_id(external_id.clone())
            .await
            .map_err(SubscribeToLocationError::InternalError)?;

        let location = match existing_location {
            None => main_location_search_and_save::execute(external_id, &self.db).await?,
            Some(location) => location,
        };

        let location_id = location.location_id;

        self.db
            .subscribe(subscriber_id, location_id)
            .await
            .map_err(SubscribeToLocationError::InternalError)?;

        let already_saved = self
            .db
            .are_nearby_locations_already_saved(location_id)
            .await
            .map_err(SubscribeToLocationError::InternalError)?;

        if !already_saved {
            self.fetch_nearby_locations_from_api_and_save(location)
                .await?;
        }

        let affected_location = self
            .db
            .is_location_affected(location_id)
            .await
            .map_err(SubscribeToLocationError::InternalError)?;

        let result = affected_location.map(|location| {
            let affected_subscriber = match location.is_directly_affected {
                true => AffectedSubscriber::DirectlyAffected(subscriber_id),
                false => AffectedSubscriber::PotentiallyAffected(subscriber_id),
            };
            AffectedSubscriberWithLocationMatchedAndLineSchedule {
                affected_subscriber,
                location_matched: LocationMatchedAndLineSchedule {
                    line_schedule: location.line_matched,
                    location_id,
                },
            }
        });

        Ok(result)
    }

    async fn search_for_location_details_from_api_and_save(
        &self,
        external_id: ExternalLocationId,
    ) -> Result<LocationWithCoordinates, SubscribeToLocationError> {
        todo!()
    }

    async fn fetch_nearby_locations_from_api_and_save(
        &self,
        location: LocationWithCoordinates,
    ) -> Result<(), SubscribeToLocationError> {
        todo!()
    }
}

mod main_location_search_and_save {
    use crate::config::SETTINGS_CONFIG;
    use crate::save_and_search_for_locations::{LocationInput, LocationWithCoordinates};
    use crate::use_cases::subscribe::db_access::SubscriptionDbAccess;
    use anyhow::{anyhow, bail, Context};
    use entities::locations::ExternalLocationId;
    use secrecy::ExposeSecret;
    use serde::Deserialize;
    use shared_kernel::http_client::HttpClient;
    use sqlx_postgres::cache::location_search::StatusCode;
    use url::Url;

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

    async fn get_place_details(url: Url) -> anyhow::Result<LocationInput> {
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
        let result = HttpClient::get_json::<serde_json::Value>(url).await?;
        let response: Response = serde_json::from_value(result.clone())
            .with_context(|| format!("Invalid response {result:?}"))?;

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
        bail!("Failed to get valid response {result:?}")
    }

    async fn save_location_returning_id_and_coordinates(
        location: LocationInput,
        db: &SubscriptionDbAccess,
    ) -> anyhow::Result<LocationWithCoordinates> {
        let external_id = location.external_id.clone();
        let _ = db.save_main_location(location).await?;
        let location_id = db.find_location_by_external_id(external_id).await?;
        location_id
            .ok_or("Location not found")
            .map_err(|err| anyhow!(err))
    }

    pub(super) async fn execute(
        id: ExternalLocationId,
        db: &SubscriptionDbAccess,
    ) -> anyhow::Result<LocationWithCoordinates> {
        let url = generate_url(id)?;
        let location = get_place_details(url).await?;
        save_location_returning_id_and_coordinates(location, db).await
    }
}
