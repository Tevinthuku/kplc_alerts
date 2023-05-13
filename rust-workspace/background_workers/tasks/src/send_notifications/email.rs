use crate::{
    configuration::{REPO, SETTINGS_CONFIG},
    utils::callbacks::failure_callback,
};
use celery::error::TaskError;
use celery::prelude::{Task, TaskResultExt};
use celery::task::TaskResult;

use crate::utils::get_token::get_email_token;
use entities::{
    locations::LocationName,
    notifications::Notification,
    power_interruptions::location::{AffectedLine, NairobiTZDateTime},
};
use itertools::Itertools;
use location_subscription::data_transfer::{
    AffectedSubscriber, AffectedSubscriberWithLocationMatchedAndLineSchedule,
    LocationMatchedAndLineSchedule,
};
use notifications::contracts::send_notification::AffectedSubscriberWithLocations;
use notifications::contracts::send_notification::LocationMatchedAndLineSchedule as NotificationLocationMatchedAndLineSchedule;
use notifications::contracts::send_notification::{
    AffectedSubscriber as NotificationAffectedSubscriber, LineWithScheduledInterruptionTime,
};
use secrecy::ExposeSecret;
use serde::Deserialize;
use serde::Serialize;
use shared_kernel::http_client::HttpClient;
use std::collections::{HashMap, HashSet};
use url::Url;

#[celery::task(max_retries = 200, bind=true, retry_for_unexpected = false, on_failure = failure_callback)]
pub async fn send_email_notification(
    task: &Self,
    data: AffectedSubscriberWithLocations,
) -> TaskResult<()> {
    let token_count = get_email_token().await?;

    if token_count < 0 {
        return Task::retry_with_countdown(task, 1);
    }
    let interactor =
        notifications::contracts::send_notification::email::EmailNotificationInteractor;

    interactor.send(data).await.map_err(|err| {
        TaskError::UnexpectedError(format!("Failed to send email notification: {}", err))
    })
}
