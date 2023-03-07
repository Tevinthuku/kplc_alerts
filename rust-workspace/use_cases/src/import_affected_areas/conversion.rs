use crate::import_affected_areas::{Area, County, Region};
use anyhow::{anyhow, Context};
use entities::power_interruptions::location::{
    Area as DomainArea, County as DomainCounty, FutureOrCurrentNairobiTZDateTime,
    Region as DomainRegion, TimeFrame,
};

impl TryFrom<Region> for DomainRegion<FutureOrCurrentNairobiTZDateTime> {
    type Error = anyhow::Error;

    fn try_from(value: Region) -> Result<Self, Self::Error> {
        let counties = value
            .counties
            .into_iter()
            .map(TryFrom::try_from)
            .collect::<Result<_, _>>()
            .with_context(|| format!("Region {}", value.name))?;
        Ok(Self {
            region: value.name,
            counties,
        })
    }
}

impl TryFrom<County> for DomainCounty<FutureOrCurrentNairobiTZDateTime> {
    type Error = anyhow::Error;

    fn try_from(value: County) -> Result<Self, Self::Error> {
        let areas = value
            .areas
            .into_iter()
            .map(TryFrom::try_from)
            .collect::<Result<_, _>>()
            .with_context(|| format!("County {}", value.name))?;
        Ok(DomainCounty {
            name: value.name,
            areas,
        })
    }
}

impl TryFrom<Area> for DomainArea<FutureOrCurrentNairobiTZDateTime> {
    type Error = anyhow::Error;

    fn try_from(value: Area) -> Result<Self, Self::Error> {
        let from = FutureOrCurrentNairobiTZDateTime::try_from(value.from)
            .map_err(|error| anyhow!(error))?;
        let to =
            FutureOrCurrentNairobiTZDateTime::try_from(value.to).map_err(|err| anyhow!(err))?;
        Ok(DomainArea {
            name: value.name,
            time_frame: TimeFrame { from, to },
            locations: value.locations,
        })
    }
}