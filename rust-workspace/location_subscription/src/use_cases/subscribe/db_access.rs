use crate::data_transfer::{
    AffectedSubscriberWithLocationMatchedAndLineSchedule, LocationMatchedAndLineSchedule,
};
use crate::db_access::DbAccess;
use crate::save_and_search_for_locations::{AffectedLocation, SaveAndSearchLocations};
use entities::locations::{ExternalLocationId, LocationId};
use entities::subscriptions::SubscriberId;

pub(crate) struct SubscriptionDbAccess {
    db: DbAccess,
    save_and_search_for_locations: SaveAndSearchLocations,
}

pub struct LocationWithCoordinates {
    pub location_id: LocationId,
    pub latitude: f64,
    pub longitude: f64,
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
        external_id: ExternalLocationId,
    ) -> anyhow::Result<Option<LocationWithCoordinates>> {
        todo!()
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
