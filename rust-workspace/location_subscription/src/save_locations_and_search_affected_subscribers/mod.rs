use crate::db_access::DbAccess;
use entities::locations::{ExternalLocationId, LocationId};
use entities::subscriptions::SubscriberId;

pub struct SaveLocationsAndSearchAffectedSubscribers {
    db_access: DbAccess,
}

#[derive(Clone)]
pub struct LocationInput {
    pub name: String,
    pub external_id: ExternalLocationId,
    pub address: String,
    pub api_response: serde_json::Value,
}

impl SaveLocationsAndSearchAffectedSubscribers {
    pub async fn save_main_location(&self, primary_input: LocationInput) -> anyhow::Result<()> {
        todo!()
    }

    pub async fn search_for_directly_affected_subscribers(&self) -> anyhow::Result<()> {
        todo!()
    }

    pub async fn save_nearby_location(
        &self,
        primary_location_id: LocationId,
    ) -> anyhow::Result<()> {
        todo!()
    }

    pub async fn search_for_potentially_affected_subscribers(&self) -> anyhow::Result<()> {
        todo!()
    }
}
