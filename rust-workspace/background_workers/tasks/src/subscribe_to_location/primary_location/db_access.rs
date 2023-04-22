use crate::subscribe_to_location::db::affected_subscriber::{
    BareAffectedLine, DbLocationSearchResults, NotificationGenerator, SearcheableCandidate,
};
use crate::subscribe_to_location::db::DB;
use anyhow::anyhow;
use anyhow::Context;
use celery::{prelude::TaskError, task::TaskResult};
use chrono::Utc;
use entities::locations::ExternalLocationId;
use entities::notifications::Notification;
use entities::power_interruptions::location::{AffectedLine, NairobiTZDateTime, TimeFrame};
use entities::subscriptions::{AffectedSubscriber, SubscriberId};
use entities::{locations::LocationId, power_interruptions::location::AreaName};
use itertools::Itertools;
use sqlx::types::chrono::DateTime;
use sqlx::PgPool;
use url::Url;
use uuid::Uuid;

impl DB<'_> {
    pub async fn subscribe_to_primary_location(
        &self,
        subscriber: SubscriberId,
        location_id: LocationId,
    ) -> TaskResult<Uuid> {
        let subscriber = subscriber.inner();
        let location_id = location_id.inner();
        let _ = sqlx::query!(
            r#"
              INSERT INTO location.subscriber_locations (subscriber_id, location_id) 
              VALUES ($1, $2) ON CONFLICT DO NOTHING
            "#,
            subscriber,
            location_id
        )
        .execute(self.pool())
        .await
        .map_err(|err| TaskError::UnexpectedError(err.to_string()))?;

        let record = sqlx::query!(
            r#"
            SELECT id FROM location.subscriber_locations WHERE subscriber_id = $1 AND location_id = $2
            "#,
             subscriber,
              location_id
        ).fetch_one(self.pool()).await.map_err(|err| TaskError::UnexpectedError(err.to_string()))?;

        Ok(record.id)
    }

    pub async fn find_location_id(
        &self,
        location: ExternalLocationId,
    ) -> TaskResult<Option<LocationId>> {
        let pool = self.pool();
        let db_results = sqlx::query!(
            r#"
            SELECT id
            FROM location.locations WHERE external_id = $1
            "#,
            location.inner()
        )
        .fetch_optional(pool)
        .await
        .map_err(|err| TaskError::UnexpectedError(err.to_string()))?;
        Ok(db_results.map(|record| record.id.into()))
    }

    pub async fn subscriber_directly_affected(
        &self,
        subscriber_id: SubscriberId,
        location_id: LocationId,
    ) -> TaskResult<Option<Notification>> {
        let results = BareAffectedLine::lines_affected_in_the_future(self)
            .await
            .map_err(|err| TaskError::UnexpectedError(err.to_string()))?;

        for (area_name, affected_lines) in results.iter() {
            let notification = self
                .directly_affected_subscriber_notification(
                    subscriber_id,
                    location_id,
                    area_name,
                    affected_lines,
                )
                .await?;
            if let Some(notification) = notification {
                return Ok(Some(notification));
            }
        }

        Ok(None)
    }

    async fn directly_affected_subscriber_notification(
        &self,
        subscriber_id: SubscriberId,
        location_id: LocationId,
        area_name: &AreaName,
        affected_lines: &[BareAffectedLine],
    ) -> TaskResult<Option<Notification>> {
        let pool = self.pool();

        let searcheable_candidates = affected_lines
            .iter()
            .map(|line| SearcheableCandidate::from(line.line.as_ref()).to_string())
            .collect_vec();

        let primary_location = Self::get_primary_location_search_result(
            location_id,
            area_name,
            pool,
            searcheable_candidates,
        )
        .await
        .map_err(|err| TaskError::UnexpectedError(err.to_string()))?;

        if let Some(location) = primary_location {
            let notification = NotificationGenerator {
                subscriber: AffectedSubscriber::DirectlyAffected(subscriber_id),
                affected_lines,
            }
            .generate(location)
            .map_err(|err| TaskError::UnexpectedError(err.to_string()))?;

            Ok(Some(notification))
        } else {
            Ok(None)
        }
    }

    async fn get_primary_location_search_result(
        location_id: LocationId,
        area_name: &AreaName,
        pool: &PgPool,
        searcheable_candidates: Vec<String>,
    ) -> anyhow::Result<Option<DbLocationSearchResults>> {
        let mut primary_location: Option<DbLocationSearchResults> = None;

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
                .fetch_optional(pool)
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
