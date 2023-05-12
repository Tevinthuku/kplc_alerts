use crate::data_transfer::{
    AffectedSubscriberWithLocationMatchedAndLineSchedule, LocationMatchedAndLineSchedule,
};
use crate::db_access::DbAccess;
use crate::save_and_search_for_locations::{
    AffectedLocation, LocationWithCoordinates, SaveAndSearchLocations,
};
use anyhow::Context;
use entities::locations::{ExternalLocationId, LocationId};
use entities::subscriptions::SubscriberId;
use serde::Deserialize;
use sqlx::types::Json;
use uuid::Uuid;

pub(crate) struct SubscriptionDbAccess {
    db: DbAccess,
    save_and_search_for_locations: SaveAndSearchLocations,
}

impl SubscriptionDbAccess {
    pub fn new() -> Self {
        Self {
            db: DbAccess,
            save_and_search_for_locations: SaveAndSearchLocations::new(),
        }
    }
    pub(crate) async fn subscribe(
        &self,
        subscriber: SubscriberId,
        location_id: LocationId,
    ) -> anyhow::Result<()> {
        todo!()
    }

    pub(crate) async fn find_location_by_external_id(
        &self,
        location: ExternalLocationId,
    ) -> anyhow::Result<Option<LocationWithCoordinates>> {
        self.save_and_search_for_locations
            .find_location_coordinates_by_external_id(location)
            .await
    }

    pub(crate) async fn are_nearby_locations_already_saved(
        &self,
        location: LocationId,
    ) -> anyhow::Result<bool> {
        todo!()
    }

    pub(crate) async fn is_location_affected(
        &self,
        location: LocationId,
    ) -> anyhow::Result<Option<AffectedLocation>> {
        self.save_and_search_for_locations
            .affected_location(location)
            .await
    }
}
