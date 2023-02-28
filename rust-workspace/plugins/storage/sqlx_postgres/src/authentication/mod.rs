use crate::repository::Repository;
use anyhow::Context;
use async_trait::async_trait;
use subscriber::subscriber::details::SubscriberDetails;
use use_cases::authentication::SubscriberAuthenticationRepo;

#[async_trait]
impl SubscriberAuthenticationRepo for Repository {
    async fn create_or_update_subscriber(
        &self,
        subscriber: SubscriberDetails,
    ) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
        INSERT INTO public.subscriber (name, email, external_id) 
        VALUES ($1, $2, $3) 
        ON CONFLICT (external_id) 
        DO UPDATE SET name = EXCLUDED.name, email = EXCLUDED.email, last_login = now();
        "#,
            subscriber.name.as_ref(),
            subscriber.email.as_ref(),
            subscriber.external_id.as_ref()
        )
        .execute(self.pool())
        .await
        .context("Failed to create or insert subscriber")
        .map(|_| ())
    }
}
