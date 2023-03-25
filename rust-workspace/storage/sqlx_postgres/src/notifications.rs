use crate::repository::Repository;
use anyhow::Context;
use entities::{notifications::Notification, subscriptions::AffectedSubscriber};
use itertools::Itertools;
use url::Url;
use uuid::Uuid;

struct NotificationInsert {
    source: Uuid,
    directly_affected: bool,
    subscriber: Uuid,
    line: String,
    location_matched: Uuid,
    external_id: String,
    strategy_id: Uuid,
}

impl Repository {
    pub async fn save_email_notification_sent(
        &self,
        notification: &Notification,
        external_id: String,
    ) -> anyhow::Result<()> {
        let source_id = self.get_source_by_url(&notification.url).await?;
        let subscriber = notification.subscriber.id().inner();
        let email_strategy_id = self.get_email_id().await?;
        let is_directly_affected = matches!(
            notification.subscriber,
            AffectedSubscriber::DirectlyAffected(_)
        );

        let notification_inserts = notification
            .lines
            .iter()
            .map(|affected_line| NotificationInsert {
                source: source_id,
                directly_affected: is_directly_affected,
                subscriber,
                line: affected_line.line.clone(),
                location_matched: affected_line.location_matched.inner(),
                external_id: external_id.clone(),
                strategy_id: email_strategy_id,
            })
            .collect_vec();

        let source_ids = notification_inserts
            .iter()
            .map(|notification| notification.source)
            .collect_vec();
        let directly_affected = notification_inserts
            .iter()
            .map(|data| data.directly_affected)
            .collect_vec();
        let subscriber_id = notification_inserts
            .iter()
            .map(|data| data.subscriber)
            .collect_vec();
        let line = notification_inserts
            .iter()
            .map(|data| data.line.clone())
            .collect_vec();
        let strategy_id = notification_inserts
            .iter()
            .map(|data| data.strategy_id)
            .collect_vec();
        let location_id_matched = notification_inserts
            .iter()
            .map(|data| data.location_matched)
            .collect_vec();
        let external_ids = notification_inserts
            .iter()
            .map(|data| data.external_id.clone())
            .collect_vec();

        sqlx::query!(
                "
                INSERT INTO communication.notifications(source_id, directly_affected, subscriber_id, line, strategy_id, location_id_matched, external_id)
                SELECT * FROM UNNEST($1::uuid[], $2::bool[], $3::uuid[], $4::text[], $5::uuid[], $6::uuid[], $7::text[]) ON CONFLICT DO NOTHING
                ",
                &source_ids[..],
                &directly_affected[..],
                &subscriber_id[..],
                &line[..],
                &strategy_id[..],
                &location_id_matched[..],
                &external_ids[..]
            )
            .execute(self.pool())
            .await?;

        Ok(())
    }

    async fn get_source_by_url(&self, url: &Url) -> anyhow::Result<Uuid> {
        let source = sqlx::query!(
            "SELECT id FROM public.source WHERE url = $1",
            url.to_string()
        )
        .fetch_one(self.pool())
        .await
        .context("Failed to get source")?;

        Ok(source.id)
    }

    async fn get_email_id(&self) -> anyhow::Result<Uuid> {
        let strategy_name = "EMAIL";
        let record = sqlx::query!(
            "SELECT id FROM communication.strategies WHERE name = $1",
            strategy_name
        )
        .fetch_one(self.pool())
        .await
        .context("Failed to get strategy id")?;

        Ok(record.id)
    }
}
