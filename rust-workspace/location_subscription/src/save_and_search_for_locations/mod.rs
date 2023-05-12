use crate::data_transfer::LineWithScheduledInterruptionTime;
use crate::db_access::DbAccess;
use crate::use_cases::get_affected_subscribers::Region;
use anyhow::{anyhow, Context};
use entities::locations::{ExternalLocationId, LocationId};
use itertools::Itertools;
use lazy_static::lazy_static;
use regex::{Captures, Regex, RegexBuilder};
use sqlx::types::Json;
use std::collections::HashMap;

use entities::power_interruptions::location::{AreaName, NairobiTZDateTime, TimeFrame};
use serde::Deserialize;
use shared_kernel::uuid_key;
use sqlx::types::chrono::{DateTime, Utc};
use sqlx_postgres::affected_subscribers::SearcheableCandidate;
use std::fmt::{Display, Formatter};
use url::Url;
use uuid::Uuid;

uuid_key!(NearbyLocationId);

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

pub struct LocationWithCoordinates {
    pub location_id: LocationId,
    pub latitude: f64,
    pub longitude: f64,
}

impl Default for SaveAndSearchLocations {
    fn default() -> Self {
        Self::new()
    }
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
        let results = BareAffectedLine::lines_affected_in_the_future(&self.db_access).await?;
        for (area_name, affected_lines) in results.iter() {}

        todo!()
    }

    pub async fn affected_location(
        &self,
        location_id: LocationId,
    ) -> anyhow::Result<Option<AffectedLocation>> {
        let directly_affected =
            directly_affected_location::execute(&self.db_access, location_id).await?;

        if directly_affected.is_some() {
            return Ok(directly_affected);
        }
        potentially_affected_location::execute(&self.db_access, location_id).await
    }

    pub async fn find_location_coordinates_by_external_id(
        &self,
        location: ExternalLocationId,
    ) -> anyhow::Result<Option<LocationWithCoordinates>> {
        #[derive(Deserialize)]

        struct LatitudeAndLongitude {
            lat: f64,
            lng: f64,
        }

        #[derive(Deserialize)]

        struct Geometry {
            location: LatitudeAndLongitude,
        }

        #[derive(Deserialize)]
        struct DataResult {
            geometry: Geometry,
        }

        #[derive(Deserialize)]
        struct ResultWrapper {
            result: DataResult,
        }

        #[derive(Deserialize)]
        struct Row {
            id: Uuid,
            value: Json<ResultWrapper>,
        }
        let pool = self.db_access.pool().await;
        let result = sqlx::query_as!(
            Row,
            r#"
            SELECT id, external_api_response as "value: Json<ResultWrapper>" FROM location.locations WHERE external_id = $1
            "#,
            location.inner()
        )
        .fetch_optional(pool.as_ref())
        .await.with_context(|| {
            format!("Failed to get response from FROM location.locations WHERE external_id = {}", location.inner())
        })?;

        let result = result.map(|data| LocationWithCoordinates {
            location_id: data.id.into(),
            latitude: data.value.result.geometry.location.lat,
            longitude: data.value.result.geometry.location.lng,
        });

        Ok(result)
    }

    pub async fn get_affected_locations_from_regions(
        &self,
        regions: &[Region],
    ) -> anyhow::Result<Vec<AffectedLocation>> {
        todo!()
    }

    pub async fn was_nearby_location_already_saved(
        &self,
        location_id: LocationId,
    ) -> anyhow::Result<Option<NearbyLocationId>> {
        let pool = self.db_access.pool().await;
        let location_id = location_id.inner();
        // TODO: Add an index for the location_id column
        let db_results = sqlx::query!(
            r#"
            SELECT id
            FROM location.nearby_locations WHERE location_id = $1
            "#,
            location_id
        )
        .fetch_optional(pool.as_ref())
        .await
        .context("Failed to fetch nearby_locations")?;

        Ok(db_results.map(|record| record.id.into()))
    }

    pub(super) async fn save_nearby_locations(
        &self,
        url: Url,
        primary_location: LocationId,
        api_response: serde_json::Value,
    ) -> anyhow::Result<NearbyLocationId> {
        let pool = self.db_access.pool().await;
        sqlx::query!(
            "
            INSERT INTO location.nearby_locations (source_url, location_id, response) 
            VALUES ($1, $2, $3) ON CONFLICT DO NOTHING
            ",
            url.to_string(),
            primary_location.inner(),
            Json(api_response) as _
        )
        .execute(pool.as_ref())
        .await
        .context("Failed to insert nearby_locations")?;

        let record = sqlx::query!(
            r#"
            SELECT id FROM location.nearby_locations WHERE source_url = $1
            "#,
            url.to_string()
        )
        .fetch_one(pool.as_ref())
        .await
        .context("Failed to fetch the nearby_location by id")?;

        Ok(record.id.into())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BareAffectedLine {
    pub line: String,
    pub url: Url,
    pub time_frame: TimeFrame<NairobiTZDateTime>,
}

impl BareAffectedLine {
    pub(crate) async fn lines_affected_in_the_future(
        db: &DbAccess,
    ) -> anyhow::Result<HashMap<AreaName, Vec<Self>>> {
        #[derive(sqlx::FromRow, Debug)]
        struct DbAreaLine {
            line_name: String,
            area_name: String,
            start_time: DateTime<Utc>,
            end_time: DateTime<Utc>,
            url: String,
        }
        let pool = db.pool().await;
        let results = sqlx::query_as::<_, DbAreaLine>(
            "
                WITH upcoming_scheduled_blackouts AS (
                  SELECT schedule.id, url, start_time, end_time FROM location.blackout_schedule schedule INNER JOIN  source ON  schedule.source_id = source.id WHERE start_time > now()
                ), blackout_schedule_with_lines_and_areas AS (
                  SELECT line_id, url, start_time, end_time, name, area_id FROM location.line_schedule INNER JOIN location.line ON line_schedule.line_id = location.line.id INNER JOIN upcoming_scheduled_blackouts ON line_schedule.schedule_id = upcoming_scheduled_blackouts.id
                ),line_area_source_url AS (
                  SELECT blackout_schedule_with_lines_and_areas.name as line_name, location.area.name as area_name, start_time, end_time , url FROM blackout_schedule_with_lines_and_areas INNER JOIN location.area ON blackout_schedule_with_lines_and_areas.area_id = location.area.id
                )
                SELECT * FROM line_area_source_url
                "
        )
        .fetch_all(pool.as_ref())
        .await
        .context("Failed to get lines that will be affected")?;

        let results = results
            .into_iter()
            .map(|data| {
                let url_result = Url::parse(&data.url);
                url_result.map(|url| {
                    (
                        data.area_name.into(),
                        BareAffectedLine {
                            line: data.line_name,
                            url,
                            time_frame: TimeFrame {
                                from: NairobiTZDateTime::from(data.start_time),
                                to: NairobiTZDateTime::from(data.end_time),
                            },
                        },
                    )
                })
            })
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to map urls")?;

        Ok(results.into_iter().into_group_map())
    }
}

struct BareAffectedLinesMapping<'a> {
    mapping_of_line_to_time_frame: HashMap<&'a String, &'a TimeFrame<NairobiTZDateTime>>,
    mapping_of_searcheble_candidate_to_original_line_candidate: HashMap<String, &'a String>,
    mapping_of_line_to_url: HashMap<&'a String, &'a Url>,
}

impl<'a> BareAffectedLinesMapping<'a> {
    fn generate(affected_lines: &'a [BareAffectedLine]) -> Self {
        let (
            mapping_of_line_to_time_frame,
            mapping_of_line_to_url,
            mapping_of_searcheble_candidate_to_original_line_candidate,
        ): (HashMap<_, _>, HashMap<_, _>, HashMap<_, _>) = affected_lines
            .iter()
            .map(|line| {
                (
                    (&line.line, &line.time_frame),
                    (&line.line, &line.url),
                    (
                        SearcheableCandidate::from(line.line.as_ref()).to_string(),
                        &line.line,
                    ),
                )
            })
            .multiunzip();
        Self {
            mapping_of_line_to_time_frame,
            mapping_of_searcheble_candidate_to_original_line_candidate,
            mapping_of_line_to_url,
        }
    }
}

struct AffectedLocationGenerator<'a> {
    affected_lines: &'a [BareAffectedLine],
}

impl<'a> AffectedLocationGenerator<'a> {
    fn generate(
        &self,
        search_query: String,
        location_id: LocationId,
        is_directly_affected: bool,
    ) -> anyhow::Result<AffectedLocation> {
        let BareAffectedLinesMapping {
            mapping_of_line_to_time_frame,
            mapping_of_searcheble_candidate_to_original_line_candidate,
            mapping_of_line_to_url,
        } = BareAffectedLinesMapping::generate(self.affected_lines);
        let original_line_candidate = mapping_of_searcheble_candidate_to_original_line_candidate
            .get(&search_query)
            .ok_or(anyhow!(
                "Failed to get original_line_candidate from search_query {}",
                search_query
            ))?;

        let time_frame = *mapping_of_line_to_time_frame
            .get(original_line_candidate)
            .ok_or(anyhow!(
            "Failed to get time_frame when we should have for candidate {original_line_candidate}"
        ))?;

        let url = *mapping_of_line_to_url
            .get(original_line_candidate)
            .ok_or(anyhow!(
                "Failed to get url from mapping_of_line_to_url for line {original_line_candidate}"
            ))?;

        Ok(AffectedLocation {
            location_id,
            line_matched: LineWithScheduledInterruptionTime {
                line_name: original_line_candidate.to_string(),
                from: time_frame.from.clone(),
                to: time_frame.to.clone(),
                source_url: url.clone(),
            },
            is_directly_affected,
        })
    }
}

mod directly_affected_location {
    use crate::db_access::DbAccess;
    use crate::save_and_search_for_locations::{
        AffectedLocation, AffectedLocationGenerator, BareAffectedLine,
    };
    use anyhow::Context;
    use entities::locations::LocationId;
    use entities::power_interruptions::location::AreaName;
    use itertools::Itertools;
    use sqlx_postgres::affected_subscribers::SearcheableCandidate;
    use uuid::Uuid;
    #[derive(sqlx::FromRow, Debug)]
    struct DbLocationSearchResults {
        pub search_query: String,
        pub location: String,
        pub id: Uuid,
    }
    pub(super) async fn execute(
        db: &DbAccess,
        location_id: LocationId,
    ) -> anyhow::Result<Option<AffectedLocation>> {
        let results = BareAffectedLine::lines_affected_in_the_future(db).await?;
        for (area_name, affected_lines) in results.iter() {
            let affected_location =
                directly_affected_location(db, location_id, area_name, &affected_lines).await?;
            if let Some(affected_location) = affected_location {
                return Ok(Some(affected_location));
            }
        }

        Ok(None)
    }

    async fn directly_affected_location(
        db: &DbAccess,
        location_id: LocationId,
        area_name: &AreaName,
        affected_lines: &[BareAffectedLine],
    ) -> anyhow::Result<Option<AffectedLocation>> {
        let searcheable_candidates = affected_lines
            .iter()
            .map(|line| SearcheableCandidate::from(line.line.as_ref()).to_string())
            .collect_vec();
        let primary_location =
            get_primary_location_search_result(location_id, area_name, db, searcheable_candidates)
                .await?;

        if let Some(location) = primary_location {
            let affected_location = AffectedLocationGenerator { affected_lines }.generate(
                location.search_query,
                location_id,
                true,
            )?;
            return Ok(Some(affected_location));
        }

        Ok(None)
    }
    async fn get_primary_location_search_result(
        location_id: LocationId,
        area_name: &AreaName,
        db: &DbAccess,
        searcheable_candidates: Vec<String>,
    ) -> anyhow::Result<Option<DbLocationSearchResults>> {
        let mut primary_location: Option<DbLocationSearchResults> = None;
        let pool = db.pool().await;
        for (searcheable_candidates, location_id, searcheable_area) in
            SearcheableCandidate::from_area_name(area_name)
                .into_iter()
                .map(|area_candidate| {
                    (
                        searcheable_candidates.clone(),
                        location_id.inner(),
                        area_candidate,
                    )
                })
        {
            let location = sqlx::query_as::<_, DbLocationSearchResults>(
                "
                    SELECT * FROM location.search_specific_location_primary_text($1::text[], $2::uuid, $3::text)
                    ",
            )
                .bind(searcheable_candidates)
                .bind(location_id)
                .bind(searcheable_area.to_string())
                .fetch_optional(pool.as_ref())
                .await
                .context("Failed to fetch results from search_specific_location_primary_text")?;
            if let Some(location) = location {
                primary_location = Some(location);
                break;
            }
        }
        Ok(primary_location)
    }
}

mod potentially_affected_location {
    use crate::db_access::DbAccess;
    use crate::save_and_search_for_locations::{
        AffectedLocation, AffectedLocationGenerator, BareAffectedLine, NearbyLocationId,
    };
    use anyhow::Context;
    use entities::locations::LocationId;
    use itertools::Itertools;
    use sqlx_postgres::affected_subscribers::SearcheableCandidate;
    use std::iter;
    use uuid::Uuid;

    pub async fn execute(
        db: &DbAccess,
        location_id: LocationId,
    ) -> anyhow::Result<Option<AffectedLocation>> {
        let results = BareAffectedLine::lines_affected_in_the_future(db).await?;

        let bare_affected_lines = results
            .into_iter()
            .filter_map(|(area, affected_lines)| {
                affected_lines.first().cloned().map(|line| {
                    iter::once(BareAffectedLine {
                        line: area.to_string(),
                        url: line.url.clone(),
                        time_frame: line.time_frame.clone(),
                    })
                    .chain(affected_lines.into_iter())
                })
            })
            .flatten()
            .collect_vec();

        let searcheable_candidates = bare_affected_lines
            .iter()
            .map(|line| SearcheableCandidate::from(line.line.as_ref()))
            .map(|candidate| candidate.to_string())
            .collect_vec();

        #[derive(sqlx::FromRow, Debug)]
        struct NearbySearchResult {
            candidate: String,
            location_id: Uuid,
        }

        let pool = db.pool().await;

        let location_id = location_id.inner();

        let nearby_location_id = sqlx::query!(
            "
               SELECT id FROM location.nearby_locations WHERE id = $1::uuid
            ",
            location_id
        )
        .fetch_one(pool.as_ref())
        .await
        .context("Failed to fetch nearby_location_id")?;

        let nearby_location_id = NearbyLocationId::from(nearby_location_id.id);

        let nearby_location = sqlx::query_as::<_, NearbySearchResult>(
            "
            SELECT * FROM location.search_nearby_locations_with_nearby_location_id($1::text[], $2::uuid)
            ",
        )
        .bind(searcheable_candidates.clone())
        .bind(nearby_location_id.inner())
        .fetch_optional(pool.as_ref())
        .await.context("Failed to get results from search_nearby_locations_with_nearby_location_id")?;

        if let Some(nearby_location) = nearby_location {
            let affected_location = AffectedLocationGenerator {
                affected_lines: &bare_affected_lines,
            }
            .generate(
                nearby_location.candidate,
                nearby_location.location_id.into(),
                false,
            )?;
            return Ok(Some(affected_location));
        }
        Ok(None)
    }
}
