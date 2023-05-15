use crate::db_access::DbAccess;
use anyhow::Context;
use serde::Deserialize;
use serde::Serialize;
use sqlx::types::Json;
use url::Url;

struct TextSearchDbAccess {
    db: DbAccess,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum StatusCode {
    OK,
    #[serde(rename = "ZERO_RESULTS")]
    ZERORESULTS,
    #[serde(rename = "INVALID_REQUEST")]
    INVALIDREQUEST,
    #[serde(rename = "OVER_QUERY_LIMIT")]
    OVERQUERYLIMIT,
    #[serde(rename = "REQUEST_DENIED")]
    REQUESTDENIED,
    #[serde(rename = "UNKNOWN_ERROR")]
    UNKNOWNERROR,
}

impl StatusCode {
    pub fn is_cacheable(&self) -> bool {
        matches!(self, StatusCode::OK | StatusCode::ZERORESULTS)
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct StructuredFormatting {
    pub main_text: String,
    pub secondary_text: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct LocationSearchApiResponsePrediction {
    pub description: String,
    pub place_id: String,
    pub structured_formatting: StructuredFormatting,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct LocationSearchApiResponse {
    status: StatusCode,
    pub predictions: Vec<LocationSearchApiResponsePrediction>,
    error_message: Option<String>,
}

impl LocationSearchApiResponse {
    pub fn is_cacheable(&self) -> bool {
        self.status.is_cacheable()
    }
}

impl TextSearchDbAccess {
    pub fn new() -> Self {
        TextSearchDbAccess { db: DbAccess }
    }

    pub async fn get_cached_text_search_response(
        &self,
        key: &Url,
    ) -> anyhow::Result<Option<LocationSearchApiResponse>> {
        let pool = self.db.pool().await;
        struct Row {
            value: Json<LocationSearchApiResponse>,
        }
        let result = sqlx::query_as!(
            Row,
            r#"
            SELECT value as "value: Json<LocationSearchApiResponse>" FROM location.location_search_cache WHERE key = $1
            "#,
            key.as_str()
        )
        .fetch_optional(pool.as_ref())
        .await
        .context("Failed to fetch from cache")?;

        if let Some(data) = result {
            return Ok(Some(data.value.0));
        }

        Ok(None)
    }
}
