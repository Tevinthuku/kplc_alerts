mod counties;
use anyhow::Context;
use sqlx::query;
use std::iter;

use crate::repository::Repository;
use crate::save_import_input::counties::{DbCounty, DbCountyId};
use async_trait::async_trait;
use futures::stream::{iter, FuturesUnordered};
use power_interuptions::location::{Area, County, FutureOrCurrentNairobiTZDateTime, Region};
use std::error::Error;
use url::Url;
use use_cases::import_planned_blackouts::SaveBlackOutsRepo;
use uuid::Uuid;

#[async_trait]
impl SaveBlackOutsRepo for Repository {
    async fn save_blackouts(
        &self,
        data: &power_interuptions::location::ImportInput,
    ) -> anyhow::Result<()> {
        let counties = self.get_counties().await?;
        let results: FuturesUnordered<_> = data
            .0
            .iter()
            .map(|(url, regions)| self.save_file_regions(url, regions, &counties))
            .collect();

        todo!()
    }
}

impl Repository {
    async fn save_file_regions(
        &self,
        url: &Url,
        regions: &[Region],
        counties: &[DbCounty],
    ) -> anyhow::Result<()> {
        let counties = regions
            .into_iter()
            .flat_map(|region| region.counties.clone())
            .map(|county| {
                DbCounty::resolve_county_id(&counties, &county.name).map(|id| (id, county))
            })
            .collect::<Result<Vec<_>, _>>();
        todo!()
    }
}

struct DbArea {
    id: Uuid,
    name: String,
    county_id: DbCountyId,
}

impl DbArea {
    async fn save_many(
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        county_id: DbCountyId,
        areas: Vec<Area<FutureOrCurrentNairobiTZDateTime>>,
    ) -> Result<(), sqlx::Error> {
        let area_names = areas
            .iter()
            .map(|area| area.name.clone())
            .collect::<Vec<_>>();
        let county_ids = iter::repeat(county_id.into_inner())
            .take(area_names.len())
            .collect::<Vec<_>>();
        sqlx::query!(
            "
            INSERT INTO location.area(name, county_id)
            SELECT * FROM UNNEST($1::text[], $2::uuid[]) ON CONFLICT DO NOTHING
            ",
            &area_names[..],
            &county_ids[..]
        )
        .execute(&mut *transaction)
        .await?;
        let areas = query!(
            "SELECT * FROM location.area WHERE name = ANY($1)",
            &area_names[..]
        )
        .fetch_all(&mut *transaction)
        .await?;

        for area in areas {}

        Ok(())
    }
}
