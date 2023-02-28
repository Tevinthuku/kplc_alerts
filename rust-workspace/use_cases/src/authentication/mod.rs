pub mod subscriber_authentication;

use crate::actor::{Actor, ExternalId};
use anyhow::anyhow;
use async_trait::async_trait;
use std::sync::Arc;
use subscriber::subscriber::details::{SubscriberDetails, SubscriberExternalId};

#[derive(Debug)]
pub struct SubscriberDetailsInput {
    pub name: String,
    pub email: String,
}

#[async_trait]
pub trait SubscriberAuthenticationRepo: Send + Sync {
    async fn create_or_update_subscriber(
        &self,
        subscriber: SubscriberDetails,
    ) -> anyhow::Result<()>;
}

#[async_trait]
pub trait AuthenticationInteractor: Send + Sync {
    async fn authenticate(
        &self,
        actor: &dyn Actor,
        user: SubscriberDetailsInput,
    ) -> anyhow::Result<()>;
}

pub struct AuthenticationInteractorImpl {
    repo: Arc<dyn SubscriberAuthenticationRepo>,
}

#[async_trait]
impl AuthenticationInteractor for AuthenticationInteractorImpl {
    async fn authenticate(
        &self,
        actor: &dyn Actor,
        user: SubscriberDetailsInput,
    ) -> anyhow::Result<()> {
        let external_id = actor.external_id().map_err(|err| anyhow!("{err}"))?;

        let details = SubscriberDetails {
            name: user.name.try_into().map_err(|err| anyhow!("{err}"))?,
            email: user.email.try_into().map_err(|err| anyhow!("{err}"))?,
            external_id,
        };

        self.repo.create_or_update_subscriber(details).await
    }
}

impl AuthenticationInteractorImpl {
    pub fn new(repo: Arc<dyn SubscriberAuthenticationRepo>) -> Self {
        Self { repo }
    }
}
