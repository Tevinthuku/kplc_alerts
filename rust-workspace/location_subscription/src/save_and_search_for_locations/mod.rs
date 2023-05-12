use crate::data_transfer::LineWithScheduledInterruptionTime;
use crate::db_access::DbAccess;
use crate::use_cases::get_affected_subscribers::Region;
use anyhow::Context;
use entities::locations::{ExternalLocationId, LocationId};
use itertools::Itertools;
use lazy_static::lazy_static;
use regex::{Captures, Regex, RegexBuilder};
use sqlx::types::Json;
use std::collections::HashMap;

use std::fmt::{Display, Formatter};

lazy_static! {
    static ref ACRONYM_MAP: HashMap<String, &'static str> = HashMap::from([
        ("pri".to_string(), "Primary"),
        ("rd".to_string(), "Road"),
        ("est".to_string(), "Estate"),
        ("sch".to_string(), "School"),
        ("sec".to_string(), "Secondary"),
        ("stn".to_string(), "Station"),
        ("apts".to_string(), "Apartments"),
        ("hqtrs".to_string(), "Headquaters"),
        ("mkt".to_string(), "Market"),
        ("fact".to_string(), "Factory"),
        ("t/fact".to_string(), "Tea Factory"),
        ("c/fact".to_string(), "Coffee Factory")
    ]);
    static ref REGEX_STR: String = {
        let keys = ACRONYM_MAP.keys().join("|");
        format!(r"\b(?:{})\b", keys)
    };
    static ref ACRONYMS_MATCHER: Regex = RegexBuilder::new(&REGEX_STR)
        .case_insensitive(true)
        .build()
        .expect("ACRONYMS_MATCHER to have been built successfully");
}

#[derive(Debug)]
pub struct NonAcronymString(String);

impl From<String> for NonAcronymString {
    fn from(value: String) -> Self {
        let result = ACRONYMS_MATCHER
            .replace_all(&value, |cap: &Captures| {
                let cap_as_lower_case = cap[0].to_lowercase();
                ACRONYM_MAP
                    .get(&cap_as_lower_case)
                    .cloned()
                    .unwrap_or_default()
                    .to_string()
            })
            .trim()
            .to_string();

        NonAcronymString(result)
    }
}

impl Display for NonAcronymString {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for NonAcronymString {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

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
    pub async fn save_main_location(&self, location: LocationInput) -> anyhow::Result<LocationId> {
        let pool = self.db_access.pool().await;
        let external_id = location.external_id.as_ref();
        let address = location.address.clone();
        let sanitized_address = NonAcronymString::from(location.address);

        sqlx::query!(
            "
            INSERT INTO location.locations (name, external_id, address, sanitized_address, external_api_response) 
            VALUES ($1, $2, $3, $4, $5) ON CONFLICT DO NOTHING
            ",
            location.name,
            external_id,
            address,
            sanitized_address.as_ref(),
            Json(location.api_response) as _
        )
        .execute(pool.as_ref())
        .await
        .context("Failed to insert location")?;

        let record = sqlx::query!(
            r#"
            SELECT id FROM location.locations WHERE external_id = $1
            "#,
            external_id
        )
        .fetch_one(pool.as_ref())
        .await
        .context("Failed to get inserted location")?;

        Ok(record.id.into())
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
