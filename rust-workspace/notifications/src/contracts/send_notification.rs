use entities::locations::LocationId;
use entities::power_interruptions::location::NairobiTZDateTime;
use entities::subscriptions::SubscriberId;
use url::Url;

pub enum AffectedSubscriber {
    DirectlyAffected(SubscriberId),
    PotentiallyAffected(SubscriberId),
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct LineWithScheduledInterruptionTime {
    pub line_name: String,
    pub from: NairobiTZDateTime,
    pub to: NairobiTZDateTime,
    pub source_url: Url,
}

pub struct LocationMatchedAndLineSchedule {
    pub line_schedule: LineWithScheduledInterruptionTime,
    pub location_id: LocationId,
}

pub struct SubscriberWithLocations {
    pub subscriber: AffectedSubscriber,
    pub locations: Vec<LocationMatchedAndLineSchedule>,
}

pub struct SendNotificationsInteractor;

impl SendNotificationsInteractor {
    pub async fn send_notifications(
        &self,
        subscribers_with_locations: Vec<SubscriberWithLocations>,
    ) -> anyhow::Result<()> {
        todo!()
    }
}
