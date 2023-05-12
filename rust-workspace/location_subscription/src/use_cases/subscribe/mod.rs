mod db_access;

use entities::locations::{ExternalLocationId, LocationId};

use crate::data_transfer::{
    AffectedSubscriber, AffectedSubscriberWithLocationMatchedAndLineSchedule, LineScheduleId,
    LineWithScheduledInterruptionTime, LocationMatchedAndLineSchedule,
};
use crate::save_and_search_for_locations::AffectedLocation;
use crate::use_cases::subscribe::db_access::{LocationWithCoordinates, SubscriptionDbAccess};
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
            None => {
                self.search_for_location_details_from_api_and_save(external_id)
                    .await?
            }
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
