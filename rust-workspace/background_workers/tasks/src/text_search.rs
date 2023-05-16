use anyhow::Context;
use celery::{prelude::Task, task::TaskResultExt};
use celery::{prelude::TaskError, task::TaskResult};
use location_search::contracts::text_search::TextSearcher;
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};
use shared_kernel::http_client::HttpClient;
use sqlx_postgres::cache::location_search::StatusCode;
use url::Url;

use crate::utils::get_token::get_location_token;
use crate::{
    configuration::{REPO, SETTINGS_CONFIG},
    utils::callbacks::failure_callback,
};

#[celery::task(bind=true, max_retries = 200, retry_for_unexpected = false, on_failure = failure_callback)]
pub async fn search_locations_by_text(task: &Self, text: String) -> TaskResult<()> {
    let token_count = get_location_token().await?;

    if token_count < 0 {
        return Task::retry_with_countdown(task, 1);
    }

    let text_searcher = TextSearcher::new();
    // the client / Producer will query the response from the cache
    // no need to return the response here
    text_searcher
        .api_search(text)
        .await
        .map_err(|err| TaskError::UnexpectedError(err.to_string()))?;

    Ok(())
}
