mod counties;
use anyhow::Context;
use sqlx::query;
use std::{collections::HashMap, iter};

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
            .map(|(url, regions)| self.save_regions(url, regions, &counties))
            .collect();

        todo!()
    }
}

impl Repository {
    async fn save_regions(
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
            .collect::<Result<Vec<_>, _>>()?;
        let mut transaction = self
            .pool()
            .begin()
            .await
            .context("Failed to begin transaction")?;

        for (county_id, county) in counties.into_iter() {
            let areas = AreaWithId::save_many(&mut transaction, county_id, county.areas)
                .await
                .context("Failed to save & return areas")?;

            let lines = areas
                .iter()
                .flat_map(|data| {
                    data.area.locations.iter().map(|line| DbLine {
                        area_id: data.id,
                        name: line.clone(),
                    })
                })
                .collect::<Vec<_>>();

            DbLine::save_many(&mut transaction, lines)
                .await
                .context("Failed to save lines")?;
        }

        todo!()
    }
}

struct AreaWithId {
    id: Uuid,
    area: Area<FutureOrCurrentNairobiTZDateTime>,
}

impl AreaWithId {
    async fn save_many(
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        county_id: DbCountyId,
        areas: Vec<Area<FutureOrCurrentNairobiTZDateTime>>,
    ) -> Result<Vec<AreaWithId>, sqlx::Error> {
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
        let inserted_areas = query!(
            "SELECT * FROM location.area WHERE name = ANY($1)",
            &area_names[..]
        )
        .fetch_all(&mut *transaction)
        .await?;
        let mapping_of_area_name_to_id = inserted_areas
            .into_iter()
            .map(|record| (record.name, record.id))
            .collect::<HashMap<_, _>>();
        Ok(areas
            .into_iter()
            .filter_map(|area| {
                mapping_of_area_name_to_id
                    .get(&area.name)
                    .map(|id| AreaWithId { id: *id, area })
            })
            .collect::<Vec<_>>())
    }
}

struct DbLine {
    area_id: Uuid,
    name: String,
}

impl DbLine {
    async fn save_many(
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        lines: Vec<DbLine>,
    ) -> Result<(), sqlx::Error> {
        let line_names = lines
            .iter()
            .map(|line| line.name.clone())
            .collect::<Vec<_>>();
        let line_area_ids = lines.iter().map(|line| line.area_id).collect::<Vec<_>>();

        sqlx::query!(
            "
            INSERT INTO location.line(name, area_id)
            SELECT * FROM UNNEST($1::text[], $2::uuid[]) ON CONFLICT DO NOTHING
            ",
            &line_names[..],
            &line_area_ids[..]
        )
        .execute(&mut *transaction)
        .await?;

        Ok(())
    }
}
