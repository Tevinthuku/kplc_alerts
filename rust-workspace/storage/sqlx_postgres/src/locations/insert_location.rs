use crate::repository::Repository;
use anyhow::Context;
use entities::locations::LocationInput;

impl Repository {
    pub async fn insert_location(&self, location: LocationInput) -> anyhow::Result<()> {
        // locations
        let pool = self.pool();

        sqlx::query!(
            "
            INSERT INTO location.locations (name) VALUES ($1) ON CONFLICT DO NOTHING
            ",
            location.name
        )
        .execute(pool)
        .await
        .context("Failed to insert location")?;

        Ok(())
    }
}
