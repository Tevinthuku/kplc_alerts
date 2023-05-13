pub(crate) mod db_access;
pub mod email;

use entities::locations::LocationId;
use entities::power_interruptions::location::NairobiTZDateTime;
use entities::subscriptions::SubscriberId;
use url::Url;

#[derive(Clone)]
pub enum AffectedSubscriber {
    DirectlyAffected(SubscriberId),
    PotentiallyAffected(SubscriberId),
}

impl AffectedSubscriber {
    pub fn id(&self) -> SubscriberId {
        match self {
            Self::DirectlyAffected(subscriber_id) => *subscriber_id,
            Self::PotentiallyAffected(subscriber_id) => *subscriber_id,
        }
    }
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct LineWithScheduledInterruptionTime {
    pub line_name: String,
    pub from: NairobiTZDateTime,
    pub to: NairobiTZDateTime,
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct LocationMatchedAndLineSchedule {
    pub line_schedule: LineWithScheduledInterruptionTime,
    pub location_id: LocationId,
}

#[derive(Clone)]
pub struct AffectedSubscriberWithLocations {
    pub source_url: Url,
    pub subscriber: AffectedSubscriber,
    pub locations: Vec<LocationMatchedAndLineSchedule>,
}
