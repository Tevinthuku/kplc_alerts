use crate::import_affected_areas::NotifySubscribersOfAffectedAreas;
use async_trait::async_trait;
use entities::notifications::Notification;
use entities::power_interruptions::location::{
    AffectedLine, ImportInput, NairobiTZDateTime, Region,
};
use entities::subscriptions::{AffectedSubscriber, SubscriberId};
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use std::collections::HashMap;
use std::sync::Arc;
use url::Url;

pub struct Notifier {
    subscriber_repo: Arc<dyn SubscriberRepo>,
    subscriber_delivery_strategies: Arc<dyn GetPreferredDeliveryStrategies>,
}

#[derive(Clone)]
pub struct SubscriberWithAffectedLines {
    subscriber: AffectedSubscriber,
    lines: Vec<AffectedLine<NairobiTZDateTime>>,
}

#[async_trait]
pub trait SubscriberRepo: Send + Sync {
    async fn get_affected_subscribers(
        &self,
        regions: &[Region],
    ) -> anyhow::Result<HashMap<AffectedSubscriber, Vec<AffectedLine<NairobiTZDateTime>>>>;
}

pub struct StrategyWithSubscribers {
    pub subscribers: Vec<SubscriberId>,
}

#[async_trait]
pub trait GetPreferredDeliveryStrategies: Send + Sync {
    async fn get_strategies(
        &self,
        subscribers: Vec<SubscriberId>,
    ) -> anyhow::Result<Vec<StrategyWithSubscribers>>;
}

impl Notifier {
    pub fn new(
        subscriber_repo: Arc<dyn SubscriberRepo>,
        subscriber_delivery_strategies: Arc<dyn GetPreferredDeliveryStrategies>,
    ) -> Self {
        Self {
            subscriber_repo,
            subscriber_delivery_strategies,
        }
    }
}
