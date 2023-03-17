use anyhow::Context;
use tasks::app;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app = app().await?;

    app.consume_from(&["fetch_location_details", "celery"])
        .await
        .context("Failed to consume tasks")
}
