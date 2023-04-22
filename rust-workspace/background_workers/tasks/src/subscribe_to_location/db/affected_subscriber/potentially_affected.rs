use std::{collections::HashMap, iter::once_with};

use crate::subscribe_to_location::db::affected_subscriber::{
    BareAffectedLine, BareAffectedLinesMapping, DbLocationSearchResults, NotificationGenerator,
    SearcheableCandidate,
};
use crate::subscribe_to_location::db::DB;
use anyhow::anyhow;
use anyhow::Context;
use celery::{prelude::TaskError, task::TaskResult};
use chrono::Utc;
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
    pub async fn subscriber_potentially_affected(
        &self,
        subscriber_id: SubscriberId,
        location_id: LocationId,
    ) -> anyhow::Result<Option<Notification>> {
        let mapping_of_areas_with_affected_lines =
            BareAffectedLine::lines_affected_in_the_future(self).await?;

        for (area_name, affected_lines) in mapping_of_areas_with_affected_lines.into_iter() {
            let (time_frame, url) = affected_lines
                .first()
                .map(|line| (line.time_frame.clone(), line.url.clone()))
                .ok_or(anyhow!("Failed to get time_frame and url"))?;

            let affected_lines = affected_lines
                .into_iter()
                .chain(once_with(|| BareAffectedLine {
                    line: area_name.to_string(),
                    time_frame,
                    url,
                }))
                .collect_vec();

            let maybe_notification = self
                .potentially_affected_notification(subscriber_id, location_id, &affected_lines)
                .await?;
            if maybe_notification.is_some() {
                return Ok(maybe_notification);
            }
        }

        Ok(None)
    }

    async fn potentially_affected_notification(
        &self,
        subscriber_id: SubscriberId,
        location_id: LocationId,
        affected_lines: &[BareAffectedLine],
    ) -> anyhow::Result<Option<Notification>> {
        let searcheable_candidates = affected_lines
            .iter()
            .map(|line| SearcheableCandidate::from(line.line.as_ref()).to_string())
            .collect_vec();
        let nearby_location = self
            .get_potentially_affected_nearby_location(location_id, searcheable_candidates)
            .await?;

        if let Some(location) = nearby_location {
            let notification = NotificationGenerator {
                affected_lines,
                subscriber: AffectedSubscriber::PotentiallyAffected(subscriber_id),
            }
            .generate(location)?;
            return Ok(Some(notification));
        }

        Ok(None)
    }

    async fn get_potentially_affected_nearby_location(
        &self,
        location_id: LocationId,
        searcheable_candidates: Vec<String>,
    ) -> anyhow::Result<Option<DbLocationSearchResults>> {
        let nearby_location = sqlx::query_as::<_, DbLocationSearchResults>(
            "
            SELECT * FROM location.search_specific_location_secondary_text($1::text[], $2::uuid)
            ",
        )
        .bind(searcheable_candidates.clone())
        .bind(location_id.to_string())
        .fetch_optional(self.pool())
        .await
        .context("Failed to fetch results from search_specific_location_secondary_text")?;
        Ok(nearby_location)
    }
}
