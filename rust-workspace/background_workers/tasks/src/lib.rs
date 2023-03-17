use crate::location_details::{
    fetch_and_subscribe_to_locations, get_and_subscribe_to_nearby_location,
};
use anyhow::Context;
use celery::Celery;
use std::sync::Arc;

pub mod location_details;

const QUEUE_NAME: &str = "celery";

pub async fn app() -> anyhow::Result<Arc<Celery>> {
    celery::app!( // TODO: Fix address, pass it via configuration.
        broker = RedisBroker { std::env::var("REDIS_ADDR").unwrap_or_else(|_| "redis://127.0.0.1:6379/".into()) },
        tasks = [
            fetch_and_subscribe_to_locations,
            get_and_subscribe_to_nearby_location
        ],
        task_routes = [
            "*" => QUEUE_NAME,
            "fetch_location_details" => "fetch_location_details"
        ],
        prefetch_count = 2,
        heartbeat = Some(10)
    ).await.context("Failed to initialize app")
}
