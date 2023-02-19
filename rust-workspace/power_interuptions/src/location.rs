use chrono::{Datelike, Days, NaiveDate, NaiveTime, Utc};
use std::collections::HashMap;
use url::Url;
#[derive(Debug)]
pub struct County<T> {
    pub name: String,
    pub areas: Vec<Area<T>>,
}

#[derive(Debug)]
pub struct Region<T = FutureOrCurrentDate> {
    pub region: String,
    pub counties: Vec<County<T>>,
}

#[derive(Debug)]
pub struct FutureOrCurrentDate(NaiveDate);

#[derive(Debug)]
pub struct Area<T> {
    pub lines: Vec<String>,
    pub date: T,
    pub time_frame: TimeFrame,
    pub locations: Vec<String>,
}

pub struct ImportInput(pub HashMap<Url, Vec<Region<FutureOrCurrentDate>>>);

impl TryFrom<NaiveDate> for FutureOrCurrentDate {
    type Error = String;

    fn try_from(provided_date: NaiveDate) -> Result<Self, Self::Error> {
        let today = Utc::now().date_naive();
        if provided_date < today {
            return Err(format!(
                "The date provided already passed {}",
                provided_date
            ));
        }
        Ok(FutureOrCurrentDate(provided_date))
    }
}

#[derive(Clone, Debug)]
pub struct TimeFrame {
    pub from: NaiveTime,
    pub to: NaiveTime,
}

#[derive(Clone)]
pub struct LocationWithDateAndTime {
    location: String,
    area: String,
    county: String,
    date: NaiveDate,
    time_frame: TimeFrame,
}