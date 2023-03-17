use crate::producer::Producer;
use anyhow::{anyhow, Context};
use async_trait::async_trait;
use entities::locations::ExternalLocationId;
use entities::subscriptions::SubscriberId;
use std::collections::HashMap;
use tasks::location_details::fetch_and_subscribe_to_locations;
use use_cases::subscriber_locations::data::{LocationId, LocationInput};
use use_cases::subscriber_locations::subscribe_to_location::LocationDetailsFinder;

impl Producer {
    async fn get_location_details(
        &self,
        subscriber_id: SubscriberId,
        location: LocationInput<ExternalLocationId>,
    ) -> anyhow::Result<()> {
        self.app
            .send_task(fetch_and_subscribe_to_locations::new(
                location.primary_id().to_owned(),
                location.nearby_locations,
                subscriber_id,
            ))
            .await
            .context("Failed to send task")?;

        Ok(())
    }
}

struct LocationInputAndCacheResultsWrapper(
    LocationInput<ExternalLocationId>,
    HashMap<ExternalLocationId, LocationId>,
);

impl TryFrom<LocationInputAndCacheResultsWrapper> for LocationInput<LocationId> {
    type Error = anyhow::Error;

    fn try_from(value: LocationInputAndCacheResultsWrapper) -> Result<Self, Self::Error> {
        let primary_id = value.0.primary_id();
        let primary_id = value.1.get(primary_id).ok_or(anyhow!(
            "Failed to find id for location with external identifier {primary_id:?}"
        ))?;
        let nearby_location_ids = value
            .0
            .nearby_locations
            .iter()
            .map(|location| {
                value.1.get(location).cloned().ok_or(anyhow!(
                    "Failed to find id for location with external identifier {primary_id:?}"
                ))
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(LocationInput {
            id: *primary_id,
            nearby_locations: nearby_location_ids,
        })
    }
}
