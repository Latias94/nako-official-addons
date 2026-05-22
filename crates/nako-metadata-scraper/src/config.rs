#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Config {
    pub listen_addr: String,
    pub base_url: String,
    pub preferred_language: String,
}

impl Config {
    #[must_use]
    pub fn from_env() -> Self {
        Self {
            listen_addr: std::env::var("NAKO_METADATA_SCRAPER_LISTEN_ADDR")
                .unwrap_or_else(|_| "127.0.0.1:9100".to_owned()),
            base_url: std::env::var("NAKO_METADATA_SCRAPER_BASE_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:9100".to_owned()),
            preferred_language: std::env::var("NAKO_METADATA_SCRAPER_LANGUAGE")
                .unwrap_or_else(|_| "en-US".to_owned()),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            listen_addr: "127.0.0.1:9100".to_owned(),
            base_url: "http://127.0.0.1:9100".to_owned(),
            preferred_language: "en-US".to_owned(),
        }
    }
}
