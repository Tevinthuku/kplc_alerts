use crate::db_access::DbAccess;
use entities::locations::LocationId;

pub(crate) struct SubscriptionDbAccess {
    db: DbAccess,
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
    pub(crate) async fn subscribe(&self) -> anyhow::Result<()> {
        todo!()
    }

    pub(crate) async fn find_location_by_external_id(
        &self,
    ) -> anyhow::Result<Option<LocationWithCoordinates>> {
        todo!()
    }
}
