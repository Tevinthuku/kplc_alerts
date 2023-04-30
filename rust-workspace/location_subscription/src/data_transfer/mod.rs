use entities::locations::LocationId;
use entities::subscriptions::AffectedSubscriber;
use shared_kernel::uuid_key;

uuid_key!(LineScheduleId);

pub struct AffectedSubscriberWithLocationMatchedAndLineSchedule {
    affected_subscriber: AffectedSubscriber,
    line_schedule_id: LineScheduleId,
    location_id: LocationId,
}
