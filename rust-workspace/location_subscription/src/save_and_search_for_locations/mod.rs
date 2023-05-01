use crate::data_transfer::{AffectedSubscriberWithLocationMatchedAndLineSchedule, LineScheduleId};
use crate::db_access::DbAccess;
use entities::locations::{ExternalLocationId, LocationId};
use entities::subscriptions::SubscriberId;

pub struct SaveAndSearchLocations {
    db_access: DbAccess,
}

#[derive(Clone)]
pub struct LocationInput {
    pub name: String,
    pub external_id: ExternalLocationId,
    pub address: String,
    pub api_response: serde_json::Value,
}

pub struct AffectedLocation {
    pub location_id: LocationId,
    pub line_matched: LineScheduleId,
    pub is_directly_affected: bool,
}

impl SaveAndSearchLocations {
    pub async fn save_main_location(&self, primary_input: LocationInput) -> anyhow::Result<()> {
        todo!()
    }

    pub async fn save_nearby_location(
        &self,
        primary_location_id: LocationId,
    ) -> anyhow::Result<()> {
        todo!()
    }

    async fn potentially_affected(
        &self,
        location_id: LocationId,
    ) -> anyhow::Result<Option<AffectedLocation>> {
        todo!()
    }

    async fn directly_affected(
        &self,
        location_id: LocationId,
    ) -> anyhow::Result<Option<AffectedLocation>> {
        todo!()
    }

    pub async fn affected_location(
        &self,
        location_id: LocationId,
    ) -> anyhow::Result<Option<AffectedLocation>> {
        let directly_affected = self.directly_affected(location_id).await?;

        if directly_affected.is_some() {
            return Ok(directly_affected);
        }
        self.potentially_affected(location_id).await
    }
}
