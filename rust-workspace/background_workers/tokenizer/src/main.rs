use anyhow::Result;
use futures_util::StreamExt as _;
use redis::aio::ConnectionLike;
use redis::{AsyncCommands, RedisResult};
use serde::de::Unexpected::Option;
use serde::Deserialize;
use std::time::Duration;
use tokio::time::interval;

#[derive(Debug, Deserialize)]
struct ExternalApi {
    pub rate_per_second: f64,
    pub name: String,
}

const SUFFIX: &str = "EXTERNAL_API";

impl ExternalApi {
    fn token_bucket(&self) -> String {
        format!("{}_{}", self.name.to_uppercase(), SUFFIX)
    }

    fn expiration(&self) -> f64 {
        if self.rate_per_second > 1. {
            1.0
        } else {
            1. / self.rate_per_second
        }
    }

    /// Schedules to periodically (and infinitely) put the required number of tokens into the
    /// appropriate token bucket.
    ///
    /// As the actual content of the token is irrelevant and we care about its presence, we forgo
    /// actually inserting any objects, and instead have a (periodically reset by this method)
    /// counter.
    /// Then each consumer is assumed to operate using the atomic `DECR` redis operator, which
    /// decrements the value by 1 and returns it. If the value is >= 0, then we consider the
    /// operation successful and the token received.
    /// The only way the token value goes back up is through this method, which simply sets it to
    /// the required number with the appropriate periodicity.
    ///
    /// The big advantage of this, is that we're basically infinitely scalable and really only
    /// limited by the redis integer representation per time period, plus how well the atomic
    /// `DECR` operation scales (again on redis side).
    async fn schedule<C: ConnectionLike>(&self, mut con: C) -> RedisResult<()> {
        let expiration = self.expiration();
        let count: usize;
        let mut ticker = if expiration > 1.0 {
            count = 1;
            interval(Duration::from_secs_f64(expiration))
        } else {
            count = self.rate_per_second.floor() as usize;
            // This is the duration that is needed to replenish the floor(rate_per_second) tokens.
            // It is also the closest duration to 1 second for the given rate that can be waited to
            // replenish an integer amount of tokens exactly according to the rate.
            interval(Duration::from_secs_f64(
                self.rate_per_second.floor() / self.rate_per_second,
            ))
        };
        loop {
            // This ticks ever required interval - either one second, or less often if the rate is
            // slower
            ticker.tick().await;
            println!(
                "Putting {} tokens into {} token bucket",
                count,
                self.token_bucket()
            );
            redis::Cmd::set(self.token_bucket(), count)
                .query_async(&mut con)
                .await?;
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let client = redis::Client::open("redis://127.0.0.1:6379/")?
        .get_multiplexed_tokio_connection()
        .await?;

    let apis = vec![ExternalApi {
        rate_per_second: 100.0,
        name: "location".to_string(),
    }];

    let fut: Vec<_> = apis
        .iter()
        .map(|api| api.schedule(client.clone()))
        .collect();
    futures::future::try_join_all(fut).await?;
    Ok(())
}