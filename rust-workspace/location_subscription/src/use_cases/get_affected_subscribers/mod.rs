use crate::data_transfer::AffectedSubscriberWithLocationMatchedAndLineSchedule;
use entities::power_interruptions::location::NairobiTZDateTime;
use std::collections::HashMap;
use url::Url;

pub struct AffectedSubscribersInteractor;

#[derive(Debug)]
pub struct Area {
    pub name: String,
    pub from: NairobiTZDateTime,
    pub to: NairobiTZDateTime,
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
    ) -> anyhow::Result<Vec<AffectedSubscriberWithLocationMatchedAndLineSchedule>> {
        todo!()
    }
}
