use crate::repository::Repository;
use anyhow::Context;
use async_trait::async_trait;
use entities::power_interruptions::location::{Area, TimeFrame};
use entities::subscriptions::AffectedSubscriber;
use entities::{
    power_interruptions::location::{AffectedLine, FutureOrCurrentNairobiTZDateTime, Region},
    subscriptions::SubscriberId,
};
use futures::{stream::FuturesUnordered, StreamExt};
use itertools::Itertools;
use std::collections::{HashMap, HashSet};
use use_cases::notifications::notify_subscribers::SubscriberRepo;
use uuid::Uuid;

#[async_trait]
impl SubscriberRepo for Repository {
    async fn get_affected_subscribers(
        &self,
        regions: &[Region],
    ) -> anyhow::Result<HashMap<AffectedSubscriber, Vec<AffectedLine>>> {
        let areas = regions
            .iter()
            .flat_map(|region| {
                region
                    .counties
                    .iter()
                    .flat_map(|county| &county.areas)
                    .collect_vec()
            })
            .collect_vec();

        let mut futures: FuturesUnordered<_> = areas
            .into_iter()
            .map(|area| self.get_location_subscribers(area))
            .collect();

        let mut result = vec![];

        while let Some(future_result) = futures.next().await {
            match future_result {
                Ok(area_results) => {
                    result.push(area_results);
                }
                Err(e) => {
                    // TODO: Refactor to tracing block
                    println!("Error searching locations {e:?}");
                }
            }
        }

        Ok(result
            .into_iter()
            .flat_map(|v| v.into_iter().map(|(subscriber, lines)| (subscriber, lines)))
            .collect())
    }
}

#[derive(Debug, Eq, PartialEq, Hash)]
struct SearcheableCandidate(String);

impl ToString for SearcheableCandidate {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}

impl AsRef<str> for SearcheableCandidate {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<&String> for SearcheableCandidate {
    fn from(value: &String) -> Self {
        let value = value.trim().replace(' ', " & ");
        SearcheableCandidate(value)
    }
}

#[derive(sqlx::FromRow, Debug)]
struct DbLocationSearchResults {
    search_query: String,
    location: String,
    id: uuid::Uuid,
}

impl Repository {
    async fn get_location_subscribers(
        &self,
        area: &Area<FutureOrCurrentNairobiTZDateTime>,
    ) -> anyhow::Result<HashMap<AffectedSubscriber, Vec<AffectedLine>>> {
        let pool = self.pool();
        let time_frame = area.time_frame.clone();
        let candidates = &area.locations;

        let mapping_of_searcheable_candidate_to_candidate = candidates
            .iter()
            .map(|candidate| (SearcheableCandidate::from(candidate), candidate.as_ref()))
            .collect::<HashMap<_, _>>();

        let searcheable_candidates = mapping_of_searcheable_candidate_to_candidate.keys();

        let searcheable_candidates = searcheable_candidates
            .map(|candidate| candidate.as_ref())
            .collect_vec();

        let (directly_affected_subscribers, mapping_of_subscriber_to_directly_affected_locations) =
            self.directly_affected_subscribers(
                &searcheable_candidates,
                &time_frame,
                &mapping_of_searcheable_candidate_to_candidate,
            )
            .await?;

        let potentially_affected_subscribers = self
            .nearby_locations_searcher(
                &searcheable_candidates,
                &time_frame,
                &mapping_of_searcheable_candidate_to_candidate,
                mapping_of_subscriber_to_directly_affected_locations,
            )
            .await?;

        Ok(directly_affected_subscribers
            .into_iter()
            .chain(potentially_affected_subscribers.into_iter())
            .collect())
    }

    async fn directly_affected_subscribers(
        &self,
        searcheable_candidates: &[&str],
        time_frame: &TimeFrame<FutureOrCurrentNairobiTZDateTime>,
        mapping_of_searcheable_candidate_to_original_candidate: &HashMap<
            SearcheableCandidate,
            &str,
        >,
    ) -> anyhow::Result<(
        HashMap<AffectedSubscriber, Vec<AffectedLine>>,
        HashMap<uuid::Uuid, Vec<uuid::Uuid>>,
    )> {
        let pool = self.pool();
        let time_frame = TimeFrame {
            from: time_frame.from.to_date_time(),
            to: time_frame.to.to_date_time(),
        };
        let primary_locations = sqlx::query_as::<_, DbLocationSearchResults>(
            "
            SELECT * FROM location.search_locations_primary_text($1::text[])
            ",
        )
        .bind(searcheable_candidates)
        .fetch_all(pool)
        .await
        .context("Failed to get primary search results from db")?;

        let location_ids = primary_locations
            .iter()
            .map(|location| location.id)
            .collect_vec();

        let primary_affected_subscribers = sqlx::query!(
            "
            SELECT subscriber_id, location_id FROM location.subscriber_locations WHERE location_id = ANY($1)
            ",
            &location_ids[..]
        ).fetch_all(pool).await.context("Failed to get subscribers subscribed to primary locations")?;

        let directly_affected_subscribers = primary_affected_subscribers
            .into_iter()
            .map(|record| (record.subscriber_id, record.location_id))
            .into_group_map();

        let location_ids_to_search_query = primary_locations
            .iter()
            .map(|data| (data.id, data.search_query.clone()))
            .collect::<HashMap<_, _>>();

        let mapping_of_searcheable_candidate_to_candidate =
            mapping_of_searcheable_candidate_to_original_candidate
                .iter()
                .map(|(searcheable_candidate, original_candidate)| {
                    (
                        searcheable_candidate.to_string(),
                        original_candidate.to_string(),
                    )
                })
                .collect::<HashMap<_, _>>();
        let result = directly_affected_subscribers
            .iter()
            .map(|(subscriber, location_ids)| {
                let locations = location_ids
                    .into_iter()
                    .filter_map(|location| {
                        location_ids_to_search_query
                            .get(location)
                            .and_then(|candidate| {
                                mapping_of_searcheable_candidate_to_candidate
                                    .get(candidate)
                                    .cloned()
                            })
                    })
                    .map(|location| AffectedLine {
                        line: location,
                        time_frame: TimeFrame {
                            from: time_frame.from,
                            to: time_frame.to,
                        },
                    })
                    .collect_vec();
                (
                    AffectedSubscriber::DirectlyAffected(SubscriberId::from(*subscriber)),
                    locations,
                )
            })
            .collect::<HashMap<_, _>>();

        Ok((result, directly_affected_subscribers))
    }

    async fn nearby_locations_searcher(
        &self,
        searcheable_candidates: &[&str],
        time_frame: &TimeFrame<FutureOrCurrentNairobiTZDateTime>,
        mapping_of_searcheable_candidate_to_original_candidate: &HashMap<
            SearcheableCandidate,
            &str,
        >,
        mapping_of_subscriber_to_directly_affected_locations: HashMap<Uuid, Vec<Uuid>>,
    ) -> anyhow::Result<HashMap<AffectedSubscriber, Vec<AffectedLine>>> {
        let pool = self.pool();
        let time_frame = TimeFrame {
            from: time_frame.from.to_date_time(),
            to: time_frame.to.to_date_time(),
        };
        let nearby_locations = sqlx::query_as::<_, DbLocationSearchResults>(
            "
                SELECT * FROM location.search_locations_secondary_text($1::text[])
                ",
        )
        .bind(searcheable_candidates)
        .fetch_all(pool)
        .await
        .context("Failed to get nearby location search results from db")?;
        let location_ids = nearby_locations
            .iter()
            .map(|location| location.id)
            .collect_vec();

        // TODO: Set search path on pool;
        let nearby_location_subscribers = sqlx::query!(
            "
            SELECT subscriber_id, adjuscent_location_id FROM location.adjuscent_locations 
            INNER JOIN location.subscriber_locations ON location.adjuscent_locations.initial_location_id = location.subscriber_locations.id 
            WHERE location.adjuscent_locations.adjuscent_location_id = ANY($1)
            ",
                &location_ids[..]
            ).fetch_all(pool).await.context("Failed to get subscribers subscribed to nearby locations")?;

        let nearby_location_subscribers = nearby_location_subscribers
            .into_iter()
            .map(|record| (record.subscriber_id, record.adjuscent_location_id))
            .into_group_map();

        let nearby_location_subscribers = nearby_location_subscribers
            .into_iter()
            .map(|(subscriber_id, nearby_location_ids)| {
                let primary_subscribers = mapping_of_subscriber_to_directly_affected_locations
                    .get(&subscriber_id)
                    .cloned()
                    .unwrap_or_default();
                let nearby_location_ids: HashSet<_> =
                    HashSet::from_iter(nearby_location_ids.into_iter());

                let locations_not_in_primary_locations: Vec<_> = nearby_location_ids
                    .difference(&HashSet::from_iter(primary_subscribers.into_iter()))
                    .cloned()
                    .collect();

                (subscriber_id, locations_not_in_primary_locations)
            })
            .collect::<HashMap<_, _>>();

        let location_ids_to_search_query = nearby_locations
            .iter()
            .map(|data| (data.id, data.search_query.clone()))
            .collect::<HashMap<_, _>>();

        let mapping_of_searcheable_candidate_to_candidate =
            mapping_of_searcheable_candidate_to_original_candidate
                .iter()
                .map(|(searcheable_candidate, original_candidate)| {
                    (
                        searcheable_candidate.to_string(),
                        original_candidate.to_string(),
                    )
                })
                .collect::<HashMap<_, _>>();
        let result = nearby_location_subscribers
            .iter()
            .map(|(subscriber, location_ids)| {
                let locations = location_ids
                    .into_iter()
                    .filter_map(|location| {
                        location_ids_to_search_query
                            .get(&location)
                            .and_then(|candidate| {
                                mapping_of_searcheable_candidate_to_candidate
                                    .get(candidate)
                                    .cloned()
                            })
                    })
                    .map(|location| AffectedLine {
                        line: location,
                        time_frame: TimeFrame {
                            from: time_frame.from,
                            to: time_frame.to,
                        },
                    })
                    .collect_vec();
                (
                    AffectedSubscriber::PotentiallyAffected(SubscriberId::from(*subscriber)),
                    locations,
                )
            })
            .collect::<HashMap<_, _>>();

        Ok(result)
    }
}
