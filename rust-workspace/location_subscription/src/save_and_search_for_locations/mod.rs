use crate::data_transfer::LineWithScheduledInterruptionTime;
use crate::db_access::DbAccess;
use crate::use_cases::get_affected_subscribers::Region;
use entities::locations::{ExternalLocationId, LocationId};

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

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct AffectedLocation {
    pub location_id: LocationId,
    pub line_matched: LineWithScheduledInterruptionTime,
    pub is_directly_affected: bool,
}

impl SaveAndSearchLocations {
    pub fn new() -> Self {
        Self {
            db_access: DbAccess,
        }
    }
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

    pub async fn get_affected_locations_from_regions(
        &self,
        regions: &[Region],
    ) -> anyhow::Result<Vec<AffectedLocation>> {
        todo!()
    }
}
