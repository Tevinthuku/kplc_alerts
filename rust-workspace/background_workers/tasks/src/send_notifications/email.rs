use crate::{
    configuration::{REPO, SETTINGS_CONFIG},
    utils::callbacks::failure_callback,
};
use celery::task::TaskResult;
use entities::notifications::Notification;

#[celery::task(max_retries = 200, bind=true, retry_for_unexpected = false, on_failure = failure_callback)]
pub async fn send_email_notification(task: &Self, notification: Notification) -> TaskResult<()> {
    Ok(())
}
