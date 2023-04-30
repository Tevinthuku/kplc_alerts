use entities::locations::{ExternalLocationId, LocationId};

use entities::power_interruptions::location::{NairobiTZDateTime, TimeFrame};
use entities::subscriptions::{AffectedSubscriber, SubscriberId};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SubscribeToLocationError {
    #[error("Internal error")]
    InternalError(#[from] anyhow::Error),
    #[error("Expected error: {0}")]
    ExpectedError(String),
}

pub struct SubscribeUseCase;

pub struct Line {
    pub name: String,
    pub time_frame: TimeFrame<NairobiTZDateTime>,
}

pub struct Notification {
    pub url: Url,
    pub line: Line,
    pub location_id_matched: LocationId,
    pub subscriber: AffectedSubscriber,
}

impl SubscribeUseCase {
    pub async fn subscribe_to_location(
        subscriber_id: SubscriberId,
        external_id: ExternalLocationId,
    ) -> Result<Option<Notification>, SubscribeToLocationError> {
        todo!()
    }
}
