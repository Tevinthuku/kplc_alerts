use entities::locations::LocationId;
use entities::power_interruptions::location::LocationName;
use entities::subscriptions::AffectedSubscriber;
use shared_kernel::uuid_key;

uuid_key!(LineScheduleId);

pub struct AffectedSubscriberWithLocationMatchedAndLineSchedule {
    affected_subscriber: AffectedSubscriber,
    line_schedule_id: LineScheduleId,
    location_id: LocationId,
}

pub struct LocationDetails {
    id: LocationId,
    name: LocationName,
    address: String,
}
