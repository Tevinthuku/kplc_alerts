mod db_access;

use entities::locations::{ExternalLocationId, LocationId};

use crate::data_transfer::{AffectedSubscriberWithLocationMatchedAndLineSchedule, LineScheduleId};
use crate::use_cases::subscribe::db_access::{LocationWithCoordinates, SubscriptionDbAccess};
use entities::power_interruptions::location::{NairobiTZDateTime, TimeFrame};
use entities::subscriptions::{AffectedSubscriber, SubscriberId};
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
            None => {
                self.search_for_location_details_from_api_and_save(external_id)
                    .await?
            }
            Some(location) => location,
        };

        self.db
            .subscribe(subscriber_id, location.location_id)
            .await
            .map_err(SubscribeToLocationError::InternalError)?;

        let already_saved = self
            .db
            .are_nearby_locations_already_saved(location.location_id)
            .await
            .map_err(SubscribeToLocationError::InternalError)?;

        if !already_saved {
            self.fetch_nearby_locations_from_api_and_save(location)
                .await?;
        }

        todo!()
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
