#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProviderId {
    Fixture,
    Tmdb,
}

impl ProviderId {
    pub const ALL: [Self; 2] = [Self::Fixture, Self::Tmdb];

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Fixture => "fixture",
            Self::Tmdb => "tmdb",
        }
    }

    #[must_use]
    pub const fn enabled_env_var(self) -> &'static str {
        match self {
            Self::Fixture => "NAKO_METADATA_SCRAPER_PROVIDER_FIXTURE_ENABLED",
            Self::Tmdb => "NAKO_METADATA_SCRAPER_PROVIDER_TMDB_ENABLED",
        }
    }

    #[must_use]
    pub const fn default_enabled(self) -> bool {
        match self {
            Self::Fixture => true,
            Self::Tmdb => false,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderConfig {
    pub id: ProviderId,
    pub enabled: bool,
    pub tmdb: Option<TmdbProviderConfig>,
}

impl ProviderConfig {
    #[must_use]
    pub const fn enabled(id: ProviderId) -> Self {
        Self {
            id,
            enabled: true,
            tmdb: None,
        }
    }

    #[must_use]
    pub const fn disabled(id: ProviderId) -> Self {
        Self {
            id,
            enabled: false,
            tmdb: None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TmdbProviderConfig {
    pub read_access_token: Option<String>,
    pub api_base_url: String,
    pub language: String,
    pub include_adult: bool,
}

impl TmdbProviderConfig {
    #[must_use]
    pub fn from_env_lookup(mut lookup: impl FnMut(&str) -> Option<String>) -> Self {
        Self {
            read_access_token: lookup("NAKO_METADATA_SCRAPER_TMDB_READ_ACCESS_TOKEN")
                .filter(|value| !value.trim().is_empty()),
            api_base_url: lookup("NAKO_METADATA_SCRAPER_TMDB_API_BASE_URL")
                .unwrap_or_else(|| "https://api.themoviedb.org/3".to_owned()),
            language: lookup("NAKO_METADATA_SCRAPER_TMDB_LANGUAGE")
                .unwrap_or_else(|| "en-US".to_owned()),
            include_adult: lookup("NAKO_METADATA_SCRAPER_TMDB_INCLUDE_ADULT")
                .and_then(|value| parse_bool(&value))
                .unwrap_or(false),
        }
    }

    #[must_use]
    pub const fn secret_field_id() -> &'static str {
        "tmdb_read_access_token"
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Config {
    pub listen_addr: String,
    pub base_url: String,
    pub preferred_language: String,
    pub providers: Vec<ProviderConfig>,
}

impl Config {
    #[must_use]
    pub fn from_env() -> Self {
        Self::from_env_lookup(|name| std::env::var(name).ok())
    }

    #[must_use]
    pub fn from_env_lookup(mut lookup: impl FnMut(&str) -> Option<String>) -> Self {
        Self {
            listen_addr: lookup("NAKO_METADATA_SCRAPER_LISTEN_ADDR")
                .unwrap_or_else(|| "127.0.0.1:9100".to_owned()),
            base_url: lookup("NAKO_METADATA_SCRAPER_BASE_URL")
                .unwrap_or_else(|| "http://127.0.0.1:9100".to_owned()),
            preferred_language: lookup("NAKO_METADATA_SCRAPER_LANGUAGE")
                .unwrap_or_else(|| "en-US".to_owned()),
            providers: ProviderId::ALL
                .into_iter()
                .map(|id| {
                    let enabled = lookup(id.enabled_env_var())
                        .and_then(|value| parse_bool(&value))
                        .unwrap_or_else(|| id.default_enabled());
                    ProviderConfig {
                        id,
                        enabled,
                        tmdb: (id == ProviderId::Tmdb)
                            .then(|| TmdbProviderConfig::from_env_lookup(|name| lookup(name))),
                    }
                })
                .collect(),
        }
    }

    #[must_use]
    pub fn provider_enabled(&self, provider_id: ProviderId) -> bool {
        self.providers
            .iter()
            .any(|provider| provider.id == provider_id && provider.enabled)
    }

    #[must_use]
    pub fn provider_config(&self, provider_id: ProviderId) -> Option<&ProviderConfig> {
        self.providers
            .iter()
            .find(|provider| provider.id == provider_id)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            listen_addr: "127.0.0.1:9100".to_owned(),
            base_url: "http://127.0.0.1:9100".to_owned(),
            preferred_language: "en-US".to_owned(),
            providers: ProviderId::ALL
                .into_iter()
                .map(|id| ProviderConfig {
                    id,
                    enabled: id.default_enabled(),
                    tmdb: (id == ProviderId::Tmdb)
                        .then(|| TmdbProviderConfig::from_env_lookup(|_| None)),
                })
                .collect(),
        }
    }
}

fn parse_bool(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_enables_only_fixture_provider() {
        let config = Config::default();

        assert_eq!(
            config.providers[0],
            ProviderConfig::enabled(ProviderId::Fixture)
        );
        assert_eq!(config.providers[1].id, ProviderId::Tmdb);
        assert!(!config.providers[1].enabled);
        assert_eq!(
            config.providers[1].tmdb.as_ref().unwrap().api_base_url,
            "https://api.themoviedb.org/3"
        );
        assert!(config.provider_enabled(ProviderId::Fixture));
        assert!(!config.provider_enabled(ProviderId::Tmdb));
    }

    #[test]
    fn config_from_env_lookup_overrides_fixture_provider_enabled_state() {
        let config = Config::from_env_lookup(|name| match name {
            "NAKO_METADATA_SCRAPER_LISTEN_ADDR" => Some("0.0.0.0:9200".to_owned()),
            "NAKO_METADATA_SCRAPER_BASE_URL" => Some("https://addon.example".to_owned()),
            "NAKO_METADATA_SCRAPER_LANGUAGE" => Some("zh-CN".to_owned()),
            "NAKO_METADATA_SCRAPER_PROVIDER_FIXTURE_ENABLED" => Some("false".to_owned()),
            "NAKO_METADATA_SCRAPER_PROVIDER_TMDB_ENABLED" => Some("true".to_owned()),
            "NAKO_METADATA_SCRAPER_TMDB_READ_ACCESS_TOKEN" => Some("tmdb-token".to_owned()),
            "NAKO_METADATA_SCRAPER_TMDB_API_BASE_URL" => Some("https://tmdb.example/3".to_owned()),
            "NAKO_METADATA_SCRAPER_TMDB_LANGUAGE" => Some("ja-JP".to_owned()),
            "NAKO_METADATA_SCRAPER_TMDB_INCLUDE_ADULT" => Some("yes".to_owned()),
            _ => None,
        });

        assert_eq!(config.listen_addr, "0.0.0.0:9200");
        assert_eq!(config.base_url, "https://addon.example");
        assert_eq!(config.preferred_language, "zh-CN");
        assert_eq!(
            config.providers[0],
            ProviderConfig::disabled(ProviderId::Fixture)
        );
        assert!(!config.provider_enabled(ProviderId::Fixture));
        assert!(config.provider_enabled(ProviderId::Tmdb));
        let tmdb = config.providers[1].tmdb.as_ref().unwrap();
        assert_eq!(tmdb.read_access_token.as_deref(), Some("tmdb-token"));
        assert_eq!(tmdb.api_base_url, "https://tmdb.example/3");
        assert_eq!(tmdb.language, "ja-JP");
        assert!(tmdb.include_adult);
    }
}
