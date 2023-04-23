use crate::subscribe_to_location::db::{
    BareAffectedLine, NotificationGenerator, SearcheableCandidate, DB,
};
use celery::error::TaskError;
use celery::prelude::{TaskResult, TaskResultExt};
use entities::locations::LocationId;
use entities::notifications::Notification;
use entities::subscriptions::{AffectedSubscriber, SubscriberId};
use itertools::Itertools;
use shared_kernel::uuid_key;
use sqlx::types::Json;
use url::Url;
use uuid::Uuid;

uuid_key!(NearbyLocationId);

impl DB<'_> {
    pub(super) async fn is_nearby_locations_already_fetched(
        &self,
        source_url: Url,
    ) -> TaskResult<Option<NearbyLocationId>> {
        let pool = self.pool();
        let db_results = sqlx::query!(
            r#"
            SELECT id
            FROM location.nearby_locations WHERE source_url = $1
            "#,
            source_url.to_string()
        )
        .fetch_optional(pool)
        .await
        .with_unexpected_err(|| "Failed to fetch nearby_locations")?;

        Ok(db_results.map(|record| NearbyLocationId::from(record.id)))
    }

    pub(super) async fn is_potentially_affected(
        &self,
        subscriber: SubscriberId,
        nearby_location_id: NearbyLocationId,
    ) -> TaskResult<Option<Notification>> {
        let results = BareAffectedLine::lines_affected_in_the_future(self)
            .await
            .map_err(|err| TaskError::UnexpectedError(err.to_string()))?;
        let affected_lines = results.into_values().flatten().collect_vec();

        let searcheable_candidates = affected_lines
            .iter()
            .map(|line| SearcheableCandidate::from(line.line.as_ref()).to_string())
            .collect_vec();

        #[derive(sqlx::FromRow, Debug)]
        struct NearbySearchResult {
            candidate: String,
            location_id: Uuid,
        }

        let nearby_location = sqlx::query_as::<_, NearbySearchResult>(
            "
            SELECT * FROM location.search_nearby_locations_with_nearby_location_id($1::text[], $2::uuid)
            ",
        )
        .bind(searcheable_candidates.clone())
        .bind(nearby_location_id.inner())
        .fetch_optional(self.pool())
        .await.with_unexpected_err(|| "Failed to get results from search_nearby_locations_with_nearby_location_id")?;

        if let Some(nearby_location) = nearby_location {
            let notification = NotificationGenerator {
                subscriber: AffectedSubscriber::PotentiallyAffected(subscriber),
                affected_lines: &affected_lines,
            }
            .generate(
                nearby_location.candidate,
                nearby_location.location_id.into(),
            )
            .map_err(|err| TaskError::UnexpectedError(err.to_string()))?;
            return Ok(Some(notification));
        }
        Ok(None)
    }

    pub(super) async fn save_nearby_locations(
        &self,
        url: Url,
        primary_location: LocationId,
        api_response: serde_json::Value,
    ) -> TaskResult<NearbyLocationId> {
        let pool = self.pool();
        sqlx::query!(
            "
            INSERT INTO location.nearby_locations (source_url, location_id, response) 
            VALUES ($1, $2, $3) ON CONFLICT DO NOTHING
            ",
            url.to_string(),
            primary_location.inner(),
            Json(api_response) as _
        )
        .execute(pool)
        .await
        .with_unexpected_err(|| "Failed to insert nearby_locations")?;

        let record = sqlx::query!(
            r#"
            SELECT id FROM location.nearby_locations WHERE source_url = $1
            "#,
            url.to_string()
        )
        .fetch_one(pool)
        .await
        .with_unexpected_err(|| "Failed to fetch the nearby_location by id")?;

        Ok(record.id.into())
    }
}
