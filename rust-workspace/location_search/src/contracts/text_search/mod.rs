mod db_access;

struct TextSearcher;

impl TextSearcher {
    pub fn new() -> Self {
        TextSearcher
    }
}

mod search {
    use crate::config::SETTINGS_CONFIG;
    use anyhow::Context;
    use secrecy::ExposeSecret;
    use url::Url;

    pub fn generate_search_url(text: String) -> anyhow::Result<Url> {
        let path_details = "/place/autocomplete/json";
        let host_with_path = &format!("{}{}", SETTINGS_CONFIG.location.host, path_details);
        Url::parse_with_params(
            host_with_path,
            &[
                ("key", SETTINGS_CONFIG.location.api_key.expose_secret()),
                ("input", &text),
                ("components", &"country:ke".to_string()),
            ],
        )
        .context("Failed to parse url")
    }
}
