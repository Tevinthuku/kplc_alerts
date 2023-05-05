mod db_access;

use entities::locations::{ExternalLocationId, LocationId};
use entities::subscriptions::SubscriberId;

pub struct UnsubscribeFromLocationInteractor;

impl UnsubscribeFromLocationInteractor {
    pub async fn unsubscribe_from_location(
        subscriber_id: SubscriberId,
        location_id: LocationId,
    ) -> anyhow::Result<()> {
        todo!()
    }
}
