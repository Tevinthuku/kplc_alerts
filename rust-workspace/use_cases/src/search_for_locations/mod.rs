use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::{
    actor::Actor, authentication::subscriber_authentication::SubscriberResolverInteractor,
};

pub struct LocationId(String);
pub struct LocationResponse {
    id: LocationId,
    name: String,
}

#[async_trait]
pub trait LocationSearchApi: Send + Sync {
    async fn search(&self, text: String) -> anyhow::Result<Vec<LocationResponse>>;
}

#[async_trait]
pub trait LocationSearchInteractor {
    async fn search(
        &self,
        actor: &dyn Actor,
        text: String,
    ) -> anyhow::Result<Vec<LocationResponse>>;
}

pub struct LocationSearchInteractorImpl {
    search_api: Arc<dyn LocationSearchApi>,
    subscriber_resolver: Arc<dyn SubscriberResolverInteractor>,
}

#[async_trait]
impl LocationSearchInteractor for LocationSearchInteractorImpl {
    async fn search(
        &self,
        actor: &dyn Actor,
        text: String,
    ) -> anyhow::Result<Vec<LocationResponse>> {
        // you just need to be authenticated in order to do the search
        let _ = self.subscriber_resolver.resolve_from_actor(actor).await?;
        self.search_api.search(text).await
    }
}
