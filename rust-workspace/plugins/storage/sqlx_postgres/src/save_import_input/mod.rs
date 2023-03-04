mod counties;
use anyhow::Context;
use sqlx::query;
use std::{collections::HashMap, iter};

use crate::repository::Repository;
use crate::save_import_input::counties::{DbCounty, DbCountyId};
use async_trait::async_trait;
use futures::stream::FuturesUnordered;
use power_interuptions::location::{
    Area, County, FutureOrCurrentNairobiTZDateTime, NairobiTZDateTime, Region, TimeFrame,
};
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
            .iter()
            .flat_map(|region| region.counties.clone())
            .map(|county| {
                DbCounty::resolve_county_id(counties, &county.name).map(|id| (id, county))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut transaction = self
            .pool()
            .begin()
            .await
            .context("Failed to begin transaction")?;

        let source = SourceFile::save(&mut transaction, url).await?;

        // TODO: Flatten the counties area list

        for (county_id, county) in counties.into_iter() {
            let areas = AreaWithId::save_many(&mut transaction, county_id, county.areas)
                .await
                .context("Failed to save & return areas")?;

            BlackoutSchedule::save_many(&mut transaction, source.id, &areas).await?;

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

struct SourceFile {
    id: Uuid,
    url: Url,
}

impl SourceFile {
    async fn save(
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        url: &Url,
    ) -> anyhow::Result<Self> {
        let url = url.as_str();
        let source = sqlx::query!(
            "INSERT INTO public.source(url) VALUES ($1) ON CONFLICT DO NOTHING RETURNING id, url ",
            url
        )
        .fetch_one(&mut *transaction)
        .await
        .context("Failed to insert source")?;

        let url = Url::parse(&source.url).context("Failed to parse inserted URL")?;

        Ok(SourceFile { id: source.id, url })
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

struct BlackoutSchedule {
    id: Uuid,
    area_id: Uuid,
    start_time: NairobiTZDateTime,
    end_time: NairobiTZDateTime,
}

impl BlackoutSchedule {
    async fn save_many(
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        source_id: Uuid,
        areas: &[AreaWithId],
    ) -> anyhow::Result<()> {
        let area_ids = areas.iter().map(|area| area.id).collect::<Vec<_>>();
        let (start_times, end_times): (Vec<_>, Vec<_>) = areas
            .iter()
            .map(|area| {
                let time_frame = &area.area.time_frame;
                (time_frame.from.to_date_time(), time_frame.to.to_date_time())
            })
            .unzip();
        let source_ids = iter::repeat(source_id)
            .take(areas.len())
            .collect::<Vec<_>>();

        sqlx::query!(
            "
            INSERT INTO location.blackout_schedule(area_id, start_time, end_time, source_id) 
            SELECT * FROM UNNEST($1::uuid[], $2::timestamptz[], $3::timestamptz[], $4::uuid[]) ON CONFLICT DO NOTHING
            ",
            &area_ids[..],
            &start_times[..],
            &end_times[..],
            &source_ids[..]
        )
        .execute(&mut *transaction).await.context("Failed to save schedules")?;

        Ok(())
    }
}
