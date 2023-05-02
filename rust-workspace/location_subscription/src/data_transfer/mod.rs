use entities::locations::LocationId;
use entities::power_interruptions::location::{LocationName, NairobiTZDateTime};
use entities::subscriptions::SubscriberId;
use shared_kernel::uuid_key;

uuid_key!(LineScheduleId);

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]

pub enum AffectedSubscriber {
    DirectlyAffected(SubscriberId),
    PotentiallyAffected(SubscriberId),
}

pub struct LocationMatchedAndLineSchedule {
    pub line_schedule: LineWithScheduledInterruptionTime,
    pub location_id: LocationId,
}

pub struct AffectedSubscriberWithLocationMatchedAndLineSchedule {
    pub affected_subscriber: AffectedSubscriber,
    pub location_matched: LocationMatchedAndLineSchedule,
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct LineWithScheduledInterruptionTime {
    pub line_name: String,
    pub from: NairobiTZDateTime,
    pub to: NairobiTZDateTime,
}

pub struct LocationDetails {
    id: LocationId,
    name: LocationName,
    address: String,
}
