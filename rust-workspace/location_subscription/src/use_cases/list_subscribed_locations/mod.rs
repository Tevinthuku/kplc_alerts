use crate::data_transfer::LocationDetails;
use entities::subscriptions::SubscriberId;

pub struct ListSubscribedLocationsInteractor;

impl ListSubscribedLocationsInteractor {
    pub async fn list_subscribed_locations(
        subscriber_id: SubscriberId,
    ) -> anyhow::Result<Vec<LocationDetails>> {
        todo!()
    }
}
