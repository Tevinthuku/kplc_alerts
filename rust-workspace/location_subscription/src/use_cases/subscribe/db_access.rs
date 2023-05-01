use crate::db_access::DbAccess;
use crate::save_locations_and_search_affected_subscribers::SaveLocationsAndSearchAffectedSubscribers;
use entities::locations::{ExternalLocationId, LocationId};
use entities::subscriptions::SubscriberId;

pub(crate) struct SubscriptionDbAccess {
    db: DbAccess,
    search_affected_subscribers: SaveLocationsAndSearchAffectedSubscribers,
}

pub struct LocationWithCoordinates {
    pub location_id: LocationId,
    pub latitude: f64,
    pub longitude: f64,
}

impl SubscriptionDbAccess {
    pub fn new() -> Self {
        Self { db: DbAccess }
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
}
