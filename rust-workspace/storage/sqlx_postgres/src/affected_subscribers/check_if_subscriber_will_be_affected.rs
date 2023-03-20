use crate::repository::Repository;
use entities::locations::LocationId;
use entities::power_interruptions::location::AffectedLine;
use entities::subscriptions::{AffectedSubscriber, SubscriberId};

impl Repository {
    async fn is_subscriber_affected(
        &self,
        subscriber_id: SubscriberId,
        location_id: LocationId,
    ) -> anyhow::Result<Option<(AffectedSubscriber, AffectedLine)>> {
        // check blackout schedule
        // select all lines in the affected blackout schedules
        // grab area_names from lines and use both to form searcheable_candidates
        // check if any candidate matches location provided via both primary or secondary search
        todo!()
    }
}
