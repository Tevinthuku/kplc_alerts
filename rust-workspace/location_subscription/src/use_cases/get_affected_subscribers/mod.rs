mod db_access;

use crate::data_transfer::{AffectedSubscriber, LocationMatchedAndLineSchedule};
use crate::use_cases::get_affected_subscribers::db_access::AffectedSubscribersDbAccess;
use entities::power_interruptions::location::NairobiTZDateTime;
use itertools::Itertools;
use std::collections::HashMap;
use url::Url;

pub struct AffectedSubscribersInteractor;

#[derive(Debug, Clone)]
pub struct TimeFrame {
    pub from: NairobiTZDateTime,
    pub to: NairobiTZDateTime,
}

#[derive(Debug)]
pub struct Area {
    pub name: String,
    pub time_frame: TimeFrame,
    pub locations: Vec<String>,
}

#[derive(Debug)]
pub struct County {
    pub name: String,
    pub areas: Vec<Area>,
}
#[derive(Debug)]
pub struct Region {
    pub name: String,
    pub counties: Vec<County>,
}

#[derive(Debug)]
pub struct ImportInput(pub HashMap<Url, Vec<Region>>);

impl AffectedSubscribersInteractor {
    pub async fn get_affected_subscribers_from_import(
        input: ImportInput,
    ) -> anyhow::Result<HashMap<AffectedSubscriber, Vec<LocationMatchedAndLineSchedule>>> {
        let db = AffectedSubscribersDbAccess::new();
        let mut result = HashMap::new();
        for (url, regions) in input.0.into_iter() {
            result.extend(
                db.get_affected_subscribers(url, &regions)
                    .await?
                    .into_iter(),
            )
        }

        Ok(result)
    }
}
