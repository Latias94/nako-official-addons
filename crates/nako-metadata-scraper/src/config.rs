#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProviderId {
    Fixture,
    Tmdb,
    Bangumi,
}

impl ProviderId {
    pub const ALL: [Self; 3] = [Self::Fixture, Self::Tmdb, Self::Bangumi];

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Fixture => "fixture",
            Self::Tmdb => "tmdb",
            Self::Bangumi => "bangumi",
        }
    }

    #[must_use]
    pub const fn enabled_env_var(self) -> &'static str {
        match self {
            Self::Fixture => "NAKO_METADATA_SCRAPER_PROVIDER_FIXTURE_ENABLED",
            Self::Tmdb => "NAKO_METADATA_SCRAPER_PROVIDER_TMDB_ENABLED",
            Self::Bangumi => "NAKO_METADATA_SCRAPER_PROVIDER_BANGUMI_ENABLED",
        }
    }

    #[must_use]
    pub const fn default_enabled(self) -> bool {
        match self {
            Self::Fixture => true,
            Self::Tmdb | Self::Bangumi => false,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderConfig {
    pub id: ProviderId,
    pub enabled: bool,
    pub tmdb: Option<TmdbProviderConfig>,
    pub bangumi: Option<BangumiProviderConfig>,
}

impl ProviderConfig {
    #[must_use]
    pub const fn enabled(id: ProviderId) -> Self {
        Self {
            id,
            enabled: true,
            tmdb: None,
            bangumi: None,
        }
    }

    #[must_use]
    pub const fn disabled(id: ProviderId) -> Self {
        Self {
            id,
            enabled: false,
            tmdb: None,
            bangumi: None,
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
pub struct BangumiProviderConfig {
    pub access_token: Option<String>,
    pub api_base_url: String,
    pub user_agent: String,
    pub include_nsfw: bool,
    pub subject_types: Vec<u8>,
}

impl BangumiProviderConfig {
    #[must_use]
    pub fn from_env_lookup(mut lookup: impl FnMut(&str) -> Option<String>) -> Self {
        Self {
            access_token: lookup("NAKO_METADATA_SCRAPER_BANGUMI_ACCESS_TOKEN")
                .filter(|value| !value.trim().is_empty()),
            api_base_url: lookup("NAKO_METADATA_SCRAPER_BANGUMI_API_BASE_URL")
                .unwrap_or_else(|| "https://api.bgm.tv".to_owned()),
            user_agent: lookup("NAKO_METADATA_SCRAPER_BANGUMI_USER_AGENT")
                .filter(|value| !value.trim().is_empty())
                .unwrap_or_else(Self::default_user_agent),
            include_nsfw: lookup("NAKO_METADATA_SCRAPER_BANGUMI_INCLUDE_NSFW")
                .and_then(|value| parse_bool(&value))
                .unwrap_or(false),
            subject_types: lookup("NAKO_METADATA_SCRAPER_BANGUMI_SUBJECT_TYPES")
                .and_then(|value| parse_bangumi_subject_types(&value))
                .unwrap_or_else(|| vec![2]),
        }
    }

    #[must_use]
    pub fn default_user_agent() -> String {
        format!(
            "Latias94/nako-official-addons/nako-metadata-scraper/{} (https://github.com/Latias94/nako-official-addons)",
            env!("CARGO_PKG_VERSION")
        )
    }

    #[must_use]
    pub const fn secret_field_id() -> &'static str {
        "bangumi_access_token"
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
                        bangumi: (id == ProviderId::Bangumi)
                            .then(|| BangumiProviderConfig::from_env_lookup(|name| lookup(name))),
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
                    bangumi: (id == ProviderId::Bangumi)
                        .then(|| BangumiProviderConfig::from_env_lookup(|_| None)),
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

fn parse_bangumi_subject_types(value: &str) -> Option<Vec<u8>> {
    let mut subject_types = Vec::new();
    for item in value.split(',') {
        let subject_type = item.trim().parse::<u8>().ok()?;
        if !matches!(subject_type, 1 | 2 | 3 | 4 | 6) {
            return None;
        }
        subject_types.push(subject_type);
    }
    (!subject_types.is_empty()).then_some(subject_types)
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
        assert_eq!(config.providers[2].id, ProviderId::Bangumi);
        assert!(!config.providers[2].enabled);
        let bangumi = config.providers[2].bangumi.as_ref().unwrap();
        assert_eq!(bangumi.api_base_url, "https://api.bgm.tv");
        assert_eq!(bangumi.subject_types, vec![2]);
        assert!(!bangumi.include_nsfw);
        assert!(config.provider_enabled(ProviderId::Fixture));
        assert!(!config.provider_enabled(ProviderId::Tmdb));
        assert!(!config.provider_enabled(ProviderId::Bangumi));
    }

    #[test]
    fn config_from_env_lookup_overrides_provider_enabled_state() {
        let config = Config::from_env_lookup(|name| match name {
            "NAKO_METADATA_SCRAPER_LISTEN_ADDR" => Some("0.0.0.0:9200".to_owned()),
            "NAKO_METADATA_SCRAPER_BASE_URL" => Some("https://addon.example".to_owned()),
            "NAKO_METADATA_SCRAPER_LANGUAGE" => Some("zh-CN".to_owned()),
            "NAKO_METADATA_SCRAPER_PROVIDER_FIXTURE_ENABLED" => Some("false".to_owned()),
            "NAKO_METADATA_SCRAPER_PROVIDER_TMDB_ENABLED" => Some("true".to_owned()),
            "NAKO_METADATA_SCRAPER_PROVIDER_BANGUMI_ENABLED" => Some("true".to_owned()),
            "NAKO_METADATA_SCRAPER_TMDB_READ_ACCESS_TOKEN" => Some("tmdb-token".to_owned()),
            "NAKO_METADATA_SCRAPER_TMDB_API_BASE_URL" => Some("https://tmdb.example/3".to_owned()),
            "NAKO_METADATA_SCRAPER_TMDB_LANGUAGE" => Some("ja-JP".to_owned()),
            "NAKO_METADATA_SCRAPER_TMDB_INCLUDE_ADULT" => Some("yes".to_owned()),
            "NAKO_METADATA_SCRAPER_BANGUMI_ACCESS_TOKEN" => Some("bangumi-token".to_owned()),
            "NAKO_METADATA_SCRAPER_BANGUMI_API_BASE_URL" => {
                Some("https://bangumi.example".to_owned())
            }
            "NAKO_METADATA_SCRAPER_BANGUMI_USER_AGENT" => {
                Some("Latias94/test-addon/0.1.0".to_owned())
            }
            "NAKO_METADATA_SCRAPER_BANGUMI_INCLUDE_NSFW" => Some("yes".to_owned()),
            "NAKO_METADATA_SCRAPER_BANGUMI_SUBJECT_TYPES" => Some("2,6".to_owned()),
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
        assert!(config.provider_enabled(ProviderId::Bangumi));
        let tmdb = config.providers[1].tmdb.as_ref().unwrap();
        assert_eq!(tmdb.read_access_token.as_deref(), Some("tmdb-token"));
        assert_eq!(tmdb.api_base_url, "https://tmdb.example/3");
        assert_eq!(tmdb.language, "ja-JP");
        assert!(tmdb.include_adult);
        let bangumi = config.providers[2].bangumi.as_ref().unwrap();
        assert_eq!(bangumi.access_token.as_deref(), Some("bangumi-token"));
        assert_eq!(bangumi.api_base_url, "https://bangumi.example");
        assert_eq!(bangumi.user_agent, "Latias94/test-addon/0.1.0");
        assert!(bangumi.include_nsfw);
        assert_eq!(bangumi.subject_types, vec![2, 6]);
    }

    #[test]
    fn invalid_bangumi_subject_types_fall_back_to_anime() {
        let config = Config::from_env_lookup(|name| match name {
            "NAKO_METADATA_SCRAPER_BANGUMI_SUBJECT_TYPES" => Some("2,5".to_owned()),
            _ => None,
        });

        assert_eq!(
            config.providers[2].bangumi.as_ref().unwrap().subject_types,
            vec![2]
        );
    }
}
