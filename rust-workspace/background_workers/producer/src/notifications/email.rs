use anyhow::{bail, Context};
use async_trait::async_trait;
use celery::Celery;
use entities::notifications::{DeliveryStrategy, Notification};
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use std::sync::Arc;
pub struct EmailStrategy {
    app: Arc<Celery>,
}

#[async_trait]
impl DeliveryStrategy for EmailStrategy {
    async fn deliver(&self, notifications: Vec<Notification>) -> anyhow::Result<()> {
        let mut futures: FuturesUnordered<_> = notifications
            .into_iter()
            .map(|notification| {
                self.app
                    .send_task(send_email_notification::new(notification))
            })
            .collect();

        let mut errors = vec![];
        while let Some(result) = futures.next().await {
            if let Err(e) = result {
                errors.push(e);
                // TODO: Setup logging
                println!("Error sending notification: {e:?}")
            }
        }

        if !errors.is_empty() {
            bail!("There were errors while sending the email notification tasks {errors:?}")
        }

        Ok(())
    }
}
