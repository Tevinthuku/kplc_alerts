use anyhow::anyhow;
use anyhow::Context;
use lazy_static::lazy_static;
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;
use serde_aux::field_attributes::deserialize_number_from_string;
use sqlx::postgres::{PgConnectOptions, PgPool};
use sqlx::postgres::{PgPoolOptions, PgSslMode};
use sqlx::{Connection, Executor, PgConnection};
use std::sync::Arc;
use uuid::Uuid;

lazy_static! {
    static ref DB_URL: String = "postgres://postgres:postgres@localhost/blackout".to_string();
}

#[derive(Clone)]
pub struct Repository {
    pg_pool: Arc<PgPool>,
}

#[derive(Debug, Deserialize)]
struct DbSettings {
    host: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    port: u16,
    username: String,
    password: Secret<String>,
    database_name: String,
    require_ssl: bool,
}

impl DbSettings {
    fn without_db(&self) -> PgConnectOptions {
        let ssl_mode = if self.require_ssl {
            PgSslMode::Require
        } else {
            PgSslMode::Prefer
        };
        PgConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(self.password.expose_secret())
            .port(self.port)
            .ssl_mode(ssl_mode)
    }

    fn with_db(&self) -> PgConnectOptions {
        let mut options = self.without_db();
        options.database(&self.database_name)
    }
}

impl Repository {
    fn configuration() -> anyhow::Result<DbSettings> {
        let base_path =
            std::env::current_dir().context("Failed to determine the current directory")?;
        let configuration_directory = base_path.join("configuration");
        let settings = config::Config::builder()
            .add_source(config::File::from(
                configuration_directory.join("base.yaml"),
            ))
            .add_source(config::Environment::with_prefix("APP").separator("_"))
            .build()
            .context("Failed to build configuration")?;
        settings
            .try_deserialize::<DbSettings>()
            .context("Failed to deserialize settings to db settings")
    }
    pub fn pool(&self) -> &PgPool {
        self.pg_pool.as_ref()
    }
    #[cfg(not(test))]
    pub async fn new() -> anyhow::Result<Self> {
        let configuration = Repository::configuration()?;
        let pg_connection = configuration.with_db();
        let pg_pool = PgPool::connect_with(pg_connection)
            .await
            .context("Failed to connect to DB")
            .map(Arc::new)?;

        sqlx::migrate!()
            .run(&*pg_pool)
            .await
            .context("Failed to run migration")?;

        Ok(Self { pg_pool })
    }
    #[cfg(test)]
    pub async fn new_test_repo() -> Self {
        let configuration = Repository::configuration().unwrap();

        let mut connection_options = configuration.without_db();

        let mut connection = PgConnection::connect_with(&connection_options)
            .await
            .expect("Failed to connect to Postgres");

        let db_name = Uuid::new_v4();
        connection
            .execute(&*format!(r#"CREATE DATABASE "{}";"#, db_name))
            .await
            .expect("Failed to create database.");

        let connection_with_db_name = connection_options.database(&db_name.to_string());

        // Migrate database
        let connection_pool = PgPool::connect_with(connection_with_db_name)
            .await
            .expect("Failed to connect to Postgres.");
        sqlx::migrate!("./migrations")
            .run(&connection_pool)
            .await
            .expect("Failed to migrate the database");

        Self {
            pg_pool: Arc::new(connection_pool),
        }
    }
}
