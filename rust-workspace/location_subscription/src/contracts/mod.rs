use async_trait::async_trait;
use entities::locations::LocationId;
use entities::subscriptions::SubscriberId;
use use_cases::subscriber_locations::delete_locations_subscribed_to::DeleteSubscribedLocationOp;
pub mod get_affected_subscribers_from_import;
pub mod list_subscribed_locations;
pub mod subscribe;
pub mod unsubscribe;

#[derive(Clone)]
pub struct LocationSubscriptionSubSystem;

#[async_trait]
impl DeleteSubscribedLocationOp for LocationSubscriptionSubSystem {
    async fn delete_subscribed(
        &self,
        subscriber_id: SubscriberId,
        location_id: LocationId,
    ) -> anyhow::Result<()> {
        unsubscribe::UnsubscribeFromLocationInteractor::unsubscribe_from_location(
            subscriber_id,
            location_id,
        )
        .await
    }
}
