use chrono::{DateTime, NaiveDate, NaiveDateTime, TimeZone, Utc};
use chrono_tz::Africa::Nairobi;
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
/// NairobiTZDateTime stores the time as `DateTime<UTC>` for easier serialization
/// and deserialization
pub struct NairobiTZDateTime(DateTime<Utc>);

impl NairobiTZDateTime {
    pub fn today() -> Self {
        NairobiTZDateTime(Utc::now())
    }

    pub fn date(&self) -> NaiveDate {
        self.0.date_naive()
    }

    pub fn to_date_time(&self) -> DateTime<Tz> {
        Nairobi.from_utc_datetime(&self.0.naive_utc())
    }
}

impl From<DateTime<Utc>> for NairobiTZDateTime {
    fn from(data: DateTime<Utc>) -> NairobiTZDateTime {
        NairobiTZDateTime(data)
    }
}

impl TryFrom<NaiveDateTime> for NairobiTZDateTime {
    type Error = String;

    fn try_from(value: NaiveDateTime) -> Result<Self, Self::Error> {
        Nairobi
            .from_local_datetime(&value)
            .single()
            .ok_or_else(|| "Failed to convert {value} to Nairobi timezone".to_string())
            .map(|date_time| {
                let date_time = date_time.naive_utc();
                let date_time = Utc.from_utc_datetime(&date_time);
                Self(date_time)
            })
    }
}

#[derive(Debug, Clone)]
pub struct FutureOrCurrentNairobiTZDateTime(NairobiTZDateTime);

impl AsRef<NairobiTZDateTime> for FutureOrCurrentNairobiTZDateTime {
    fn as_ref(&self) -> &NairobiTZDateTime {
        &self.0
    }
}

impl FutureOrCurrentNairobiTZDateTime {
    pub fn to_date_time(&self) -> DateTime<Tz> {
        self.0.to_date_time()
    }
}

impl TryFrom<NairobiTZDateTime> for FutureOrCurrentNairobiTZDateTime {
    type Error = String;

    fn try_from(provided_date: NairobiTZDateTime) -> Result<Self, Self::Error> {
        let today = NairobiTZDateTime::today();
        if provided_date.date() < today.date() {
            return Err(format!(
                "The date provided already passed {:?}",
                provided_date
            ));
        }
        Ok(FutureOrCurrentNairobiTZDateTime(provided_date))
    }
}

impl From<&FutureOrCurrentNairobiTZDateTime> for NairobiTZDateTime {
    fn from(value: &FutureOrCurrentNairobiTZDateTime) -> Self {
        value.0.clone()
    }
}
