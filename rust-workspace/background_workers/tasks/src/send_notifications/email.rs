use crate::{
    configuration::{REPO, SETTINGS_CONFIG},
    utils::callbacks::failure_callback,
};
use celery::error::TaskError;
use celery::prelude::TaskResultExt;
use celery::task::TaskResult;
use entities::notifications::Notification;
use secrecy::ExposeSecret;
use serde::Deserialize;
use serde::Serialize;
use shared_kernel::http_client::HttpClient;
use std::collections::HashMap;
use url::Url;

const TEMPLATE: &str = "565V4THQRHM19SMJNC6WFKZRVWGR";

#[derive(Serialize, Deserialize)]
enum AffectedState {
    #[serde(rename = "directly affected")]
    DirectlyAffected,
    #[serde(rename = "potentially affected")]
    PotentiallyAffected,
}

#[derive(Serialize, Deserialize)]
struct AffectedLocation {
    pub location: String,
    pub date: String,
    pub start_time: String,
    pub end_time: String,
}

#[derive(Serialize, Deserialize)]
struct MessageData {
    pub recipient_name: String,
    pub affected_state: String,
    pub link: String,
    pub affected_locations: Vec<AffectedLocation>,
}

#[derive(Serialize, Deserialize)]
struct To {
    pub email: String,
}

#[derive(Serialize, Deserialize)]
struct Message {
    pub to: To,
    pub template: String,
    pub data: MessageData,
}

#[derive(Serialize, Deserialize)]
struct Data {
    pub message: Message,
}

impl Data {
    fn as_json(&self) -> serde_json::Result<serde_json::Value> {
        let as_str = serde_json::to_string(self)?;
        serde_json::from_str(&as_str)
    }
}

#[derive(Deserialize)]
struct Response {
    #[serde(rename = "requestId")]
    request_id: String,
}

#[celery::task(max_retries = 200, bind=true, retry_for_unexpected = false, on_failure = failure_callback)]
pub async fn send_email_notification(task: &Self, notification: Notification) -> TaskResult<()> {
    let body = generate_email_body(notification).await?;
    let settings = &SETTINGS_CONFIG.email;
    let url = Url::parse(&settings.host)
        .with_unexpected_err(|| format!("Invalid url {}", &settings.host))?;

    let auth_token = settings.auth_token.expose_secret();
    let bearer_token = format!("Bearer {auth_token}");

    let headers = HashMap::from([("Authorization", bearer_token)]);

    let body = body
        .as_json()
        .with_unexpected_err(|| "Failed to convert the body to a valid json")?;

    // TODO: Save response.request_id in the notifications table as an external_id
    let response = HttpClient::post_json::<Response>(url, headers, body)
        .await
        .map_err(|err| TaskError::ExpectedError(format!("{err}")))?;

    Ok(())
}

async fn generate_email_body(notification: Notification) -> TaskResult<Data> {
    todo!()
}
