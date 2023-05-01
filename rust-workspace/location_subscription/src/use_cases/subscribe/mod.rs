mod db_access;

use entities::locations::{ExternalLocationId, LocationId};

use crate::data_transfer::{AffectedSubscriberWithLocationMatchedAndLineSchedule, LineScheduleId};
use crate::use_cases::subscribe::db_access::SubscriptionDbAccess;
use entities::power_interruptions::location::{NairobiTZDateTime, TimeFrame};
use entities::subscriptions::{AffectedSubscriber, SubscriberId};
use shared_kernel::uuid_key;
use thiserror::Error;
use url::Url;

#[derive(Error, Debug)]
pub enum SubscribeToLocationError {
    #[error("Internal error")]
    InternalError(#[from] anyhow::Error),
    #[error("Expected error: {0}")]
    ExpectedError(String),
}

pub struct SubscribeInteractor;

impl SubscribeInteractor {
    pub async fn subscribe_to_location(
        subscriber_id: SubscriberId,
        external_id: ExternalLocationId,
    ) -> Result<
        Option<AffectedSubscriberWithLocationMatchedAndLineSchedule>,
        SubscribeToLocationError,
    > {
        let db = SubscriptionDbAccess::new();
        todo!()
    }
}
