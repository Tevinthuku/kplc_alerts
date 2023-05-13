pub mod db;

use anyhow::Context;
use celery::export::async_trait;
use celery::prelude::*;
use entities::locations::ExternalLocationId;
use itertools::Itertools;

use entities::subscriptions::SubscriberId;
use secrecy::ExposeSecret;
use shared_kernel::http_client::HttpClient;
use url::Url;

use crate::{
    send_notifications::email::send_email_notification, utils::get_token::get_location_token,
};

use notifications::contracts::send_notification::LocationMatchedAndLineSchedule as NotificationLocationMatchedAndLineSchedule;
use notifications::contracts::send_notification::{
    AffectedSubscriber as NotificationAffectedSubscriber, LineWithScheduledInterruptionTime,
};

use location_subscription::contracts::subscribe::SubscribeToLocationError;
use location_subscription::data_transfer::AffectedSubscriber;
use notifications::contracts::send_notification::AffectedSubscriberWithLocations;
use serde::Deserialize;

use sqlx_postgres::cache::location_search::StatusCode;
use use_cases::subscriber_locations::subscribe_to_location::TaskId;

use crate::utils::progress_tracking::{set_progress_status, TaskStatus};
use crate::{configuration::SETTINGS_CONFIG, utils::callbacks::failure_callback};

use self::db::DB;

#[celery::task(max_retries = 200, bind = true, retry_for_unexpected = false, on_failure = failure_callback)]
pub async fn fetch_and_subscribe_to_location(
    task: &Self,
    primary_location: ExternalLocationId,
    subscriber: SubscriberId,
    task_id: TaskId,
) -> TaskResult<()> {
    set_progress_status(
        task_id.as_ref(),
        TaskStatus::Pending.to_string(),
        |_| Ok(()),
    )
    .await
    .map_err(|err| TaskError::UnexpectedError(err.to_string()))?;

    let subscription_interactor =
        location_subscription::contracts::subscribe::SubscribeInteractor::new();
    let affected_subscriber = subscription_interactor
        .subscribe_to_location(subscriber, primary_location)
        .await
        .map_err(|err| match err {
            SubscribeToLocationError::InternalError(err) => {
                TaskError::UnexpectedError(err.to_string())
            }
            SubscribeToLocationError::ExpectedError(err) => {
                TaskError::ExpectedError(err.to_string())
            }
        })?;

    if let Some(affected_subscriber) = affected_subscriber {
        let data = AffectedSubscriberWithLocations {
            source_url: affected_subscriber
                .location_matched
                .line_schedule
                .source_url,
            subscriber: match affected_subscriber.affected_subscriber {
                AffectedSubscriber::DirectlyAffected(subscriber) => {
                    NotificationAffectedSubscriber::DirectlyAffected(subscriber)
                }
                AffectedSubscriber::PotentiallyAffected(subscriber) => {
                    NotificationAffectedSubscriber::PotentiallyAffected(subscriber)
                }
            },
            locations: vec![NotificationLocationMatchedAndLineSchedule {
                line_schedule: LineWithScheduledInterruptionTime {
                    line_name: affected_subscriber.location_matched.line_schedule.line_name,
                    from: affected_subscriber.location_matched.line_schedule.from,
                    to: affected_subscriber.location_matched.line_schedule.to,
                },
                location_id: affected_subscriber.location_matched.location_id,
            }],
        };
        let _ = task
            .request
            .app
            .send_task(send_email_notification::new(data))
            .await
            .with_expected_err(|| "Failed to send task")?;
    }

    Ok(())
}
