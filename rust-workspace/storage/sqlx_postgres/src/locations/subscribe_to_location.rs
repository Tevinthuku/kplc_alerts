use std::iter;

use anyhow::Context;
use async_trait::async_trait;
use entities::subscriptions::SubscriberId;
use use_cases::subscriber_locations::{
    data::{LocationId, LocationInput},
    subscribe_to_location::SubscribeToLocationRepo,
};
use uuid::Uuid;

use crate::repository::Repository;

impl Repository {
    pub async fn subscribe_to_location(
        &self,
        subscriber: SubscriberId,
        location_id: LocationId,
    ) -> anyhow::Result<Uuid> {
        let subscriber = subscriber.inner();
        let location_id = location_id.into_inner();
        let _ = sqlx::query!(
            r#"
              INSERT INTO location.subscriber_locations (subscriber_id, location_id) 
              VALUES ($1, $2) ON CONFLICT DO NOTHING
            "#,
            subscriber,
            location_id
        )
        .execute(self.pool())
        .await
        .context("Failed to subscribe to location")?;

        let record = sqlx::query!(
            r#"
            SELECT id FROM location.subscriber_locations WHERE subscriber_id = $1 AND location_id = $2
            "#,
             subscriber,
              location_id
        ).fetch_one(self.pool()).await.context("Failed to get location")?;

        Ok(record.id)
    }

    pub async fn subscribe_to_adjuscent_location(
        &self,
        initial_location_id: Uuid,
        adjuscent_location_id: LocationId,
    ) -> anyhow::Result<()> {
        sqlx::query!(
            "
              INSERT INTO location.adjuscent_locations(initial_location_id, adjuscent_location_id) 
              VALUES ($1, $2) ON CONFLICT DO NOTHING
            ",
            initial_location_id,
            adjuscent_location_id.into_inner()
        )
        .execute(self.pool())
        .await
        .context("Failed to insert nearby location")?;

        Ok(())
    }
}

#[async_trait]
impl SubscribeToLocationRepo for Repository {
    async fn subscribe(
        &self,
        subscriber: SubscriberId,
        location: LocationInput<LocationId>,
    ) -> anyhow::Result<()> {
        let mut transaction = self
            .pool()
            .begin()
            .await
            .context("Failed to begin transaction")?;

        let subscriber_id = subscriber.inner();

        let record = sqlx::query!(
            r#"
              INSERT INTO location.subscriber_locations (subscriber_id, location_id) 
              VALUES ($1, $2) ON CONFLICT DO NOTHING RETURNING id
            "#,
            subscriber_id,
            location.primary_id().into_inner()
        )
        .fetch_one(&mut *transaction)
        .await
        .context("Failed to subscribe to location")?;

        let nearby_locations = location
            .nearby_locations
            .into_iter()
            .map(|location| location.into_inner())
            .collect::<Vec<_>>();
        let take_initial_location = iter::repeat(record.id)
            .take(nearby_locations.len())
            .collect::<Vec<_>>();

        sqlx::query!(
            "
              INSERT INTO location.adjuscent_locations(initial_location_id, adjuscent_location_id) 
              SELECT * FROM UNNEST($1::uuid[], $2::uuid[])
            ",
            &take_initial_location[..],
            &nearby_locations[..],
        )
        .execute(&mut *transaction)
        .await
        .context("Failed to insert nearby locations")?;

        transaction
            .commit()
            .await
            .context("Failed to save location subscription changes")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use entities::{
        locations::{ExternalLocationId, LocationInput},
        power_interruptions::location::{Area, County, NairobiTZDateTime, Region, TimeFrame},
        subscriptions::{
            details::{SubscriberDetails, SubscriberExternalId},
            AffectedSubscriber, SubscriberId,
        },
    };
    use serde_json::Value;
    use use_cases::{
        authentication::SubscriberAuthenticationRepo,
        notifications::notify_subscribers::SubscriberRepo,
    };

    use crate::repository::Repository;

    fn generate_region() -> Region {
        Region {
            region: "Nairobi".to_string(),
            counties: vec![County {
                name: "Nairobi".to_string(),
                areas: vec![
                    Area {
                        name: "Garden City".to_string(),
                        time_frame: TimeFrame {
                            from: NairobiTZDateTime::today().try_into().unwrap(),
                            to: NairobiTZDateTime::today().try_into().unwrap(),
                        },
                        locations: vec![
                            "Will Mary Estate".to_string(),
                            "Garden City Mall".to_string(),
                        ],
                    },
                    Area {
                        name: "Lumumba".to_string(),
                        time_frame: TimeFrame {
                            from: NairobiTZDateTime::today().try_into().unwrap(),
                            to: NairobiTZDateTime::today().try_into().unwrap(),
                        },
                        locations: vec![
                            "Lumumba dr".to_string(),
                            "Pan Africa Christian University".to_string(),
                        ],
                    },
                ],
            }],
        }
    }

    async fn authenticate(repo: &Repository) -> SubscriberId {
        let external_id: SubscriberExternalId =
            "ChIJGdueTt0VLxgRk19ir6oE8I0".to_owned().try_into().unwrap();
        repo.create_or_update_subscriber(SubscriberDetails {
            name: "Tev".to_owned().try_into().unwrap(),
            email: "tevinthuku@gmail.com".to_owned().try_into().unwrap(),
            external_id: external_id.clone(),
        })
        .await
        .unwrap();

        repo.find_by_external_id(external_id).await.unwrap()
    }

    #[tokio::test]
    async fn test_searching_directly_affected_subscriber_works() {
        let repository = Repository::new_test_repo().await;
        let subscriber_id = authenticate(&repository).await;
        let contents = include_str!("mock_data/garden_city_details_response.json");
        let api_response: Value = serde_json::from_str(contents).unwrap();
        let location_id = repository
            .insert_location(LocationInput {
                name: "Garden City Mall".to_string(),
                external_id: ExternalLocationId::from("ChIJGdueTt0VLxgRk19ir6oE8I0".to_string()),
                address: "Thika Rd, Nairobi, Kenya".to_string(),
                api_response,
            })
            .await
            .unwrap();

        repository
            .subscribe_to_location(subscriber_id, location_id)
            .await
            .unwrap();

        let results = repository
            .get_affected_subscribers(&[generate_region()])
            .await
            .unwrap();
        println!("{results:?}");
        assert!(!results.is_empty());
        let key = AffectedSubscriber::DirectlyAffected(subscriber_id);
        assert!(results.contains_key(&key));
        let value = results.get(&key).unwrap().first().unwrap();
        assert_eq!(&value.line, "Garden City Mall")
    }

    #[tokio::test]
    async fn test_searching_api_response_results_in_potentially_affected_subscriber() {
        let repository = Repository::new_test_repo().await;
        let subscriber_id = authenticate(&repository).await;
        let contents = include_str!("mock_data/mi_vida_homes.json");
        let api_response: Value = serde_json::from_str(contents).unwrap();

        let location_id = repository
            .insert_location(LocationInput {
                name: "Mi Vida Homes".to_string(),
                external_id: ExternalLocationId::from("ChIJhVbiHlwVLxgRUzt5QN81vPA".to_string()),
                address: "Off exit, 7 Thika Rd, Nairobi, Kenya".to_string(),
                api_response,
            })
            .await
            .unwrap();

        repository
            .subscribe_to_location(subscriber_id, location_id)
            .await
            .unwrap();

        let results = repository
            .get_affected_subscribers(&[generate_region()])
            .await
            .unwrap();

        println!("{results:?}");

        assert!(!results.is_empty());
        let key = AffectedSubscriber::PotentiallyAffected(subscriber_id);
        assert!(results.contains_key(&key));
        let value = results.get(&key).unwrap().first().unwrap();
        assert_eq!(&value.line, "Garden City") // The area name
    }
}
