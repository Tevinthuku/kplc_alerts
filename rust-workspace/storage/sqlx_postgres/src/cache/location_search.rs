use crate::repository::Repository;
use anyhow::Context;
use async_trait::async_trait;
use location_searcher::text_search::{LocationSearchApiResponse, LocationSearchApiResponseCache};
use sqlx::types::Json;
use url::Url;

impl Repository {
    pub async fn get_cached_text_search_response(
        &self,
        key: &Url,
    ) -> anyhow::Result<Option<LocationSearchApiResponse>> {
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
        .fetch_optional(self.pool())
        .await
        .context("Failed to fetch from cache")?;

        if let Some(data) = result {
            return Ok(Some(data.value.0));
        }

        Ok(None)
    }

    pub async fn set_cached_text_search_response(
        &self,
        key: &Url,
        response: &serde_json::Value,
    ) -> anyhow::Result<()> {
        let _ = sqlx::query!(
            r#"
            INSERT INTO location.location_search_cache ( key, value )
            VALUES ( $1, $2 )
            "#,
            key.as_str(),
            Json(response) as _
        )
        .execute(self.pool())
        .await
        .context("Failed to save response in cache")?;

        Ok(())
    }
}

#[async_trait]
impl LocationSearchApiResponseCache for Repository {
    async fn get(&self, key: &Url) -> anyhow::Result<Option<LocationSearchApiResponse>> {
        self.get_cached_text_search_response(key).await
    }

    async fn set(&self, key: &Url, response: &serde_json::Value) -> anyhow::Result<()> {
        self.set_cached_text_search_response(key, response).await
    }
}
