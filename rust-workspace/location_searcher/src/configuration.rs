use anyhow::Context;
use secrecy::Secret;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct LocationSearcherConfig {
    pub host: String,
    pub api_key: Secret<String>,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub location_searcher: LocationSearcherConfig,
}

impl Settings {
    pub fn parse() -> anyhow::Result<Settings> {
        //TODO:  This config should go to shared-kernel soon
        let base_path =
            std::env::current_dir().context("Failed to determine the current directory")?;
        let configuration_directory = base_path.join("configuration");
        let file = if cfg!(test) { "test.yaml" } else { "base.yaml" };
        let settings = config::Config::builder()
            .add_source(config::File::from(configuration_directory.join(file)))
            .add_source(
                config::Environment::with_prefix("APP")
                    .prefix_separator("_")
                    .separator("__"),
            )
            .build()
            .context("Failed to build configuration")?;

        settings
            .try_deserialize::<Settings>()
            .context("Failed to deserialize settings to location_searcher settings")
    }
}
