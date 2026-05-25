use crate::providers::{ProviderConfigInput, ProviderRegistry};
pub use crate::providers::{
    bangumi::BangumiProviderConfig, browser_worker::BrowserWorkerProviderConfig,
    douban::DoubanProviderConfig, tmdb::TmdbProviderConfig,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NakoRuntimeConfig {
    pub base_url: Option<String>,
    pub addon_token: Option<String>,
    pub side_effects_enabled: bool,
    pub timeout_ms: u64,
}

impl NakoRuntimeConfig {
    pub const DEFAULT_TIMEOUT_MS: u64 = 10_000;

    #[must_use]
    pub fn from_env_lookup(mut lookup: impl FnMut(&str) -> Option<String>) -> Self {
        Self {
            base_url: lookup("NAKO_METADATA_SCRAPER_NAKO_BASE_URL").and_then(non_empty_trimmed),
            addon_token: lookup("NAKO_METADATA_SCRAPER_ADDON_TOKEN").and_then(non_empty_trimmed),
            side_effects_enabled: lookup("NAKO_METADATA_SCRAPER_SIDE_EFFECTS_ENABLED")
                .and_then(|value| parse_bool(&value))
                .unwrap_or(false),
            timeout_ms: lookup("NAKO_METADATA_SCRAPER_NAKO_TIMEOUT_MS")
                .and_then(|value| value.trim().parse::<u64>().ok())
                .filter(|value| *value > 0)
                .unwrap_or(Self::DEFAULT_TIMEOUT_MS),
        }
    }

    #[must_use]
    pub fn disabled() -> Self {
        Self {
            base_url: None,
            addon_token: None,
            side_effects_enabled: false,
            timeout_ms: Self::DEFAULT_TIMEOUT_MS,
        }
    }

    #[must_use]
    pub fn can_submit_side_effects(&self) -> bool {
        self.side_effects_enabled && self.base_url.is_some() && self.addon_token.is_some()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProviderId {
    Fixture,
    Tmdb,
    Bangumi,
    BrowserWorker,
    Douban,
}

impl ProviderId {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Fixture => "fixture",
            Self::Tmdb => "tmdb",
            Self::Bangumi => "bangumi",
            Self::BrowserWorker => "browser_worker",
            Self::Douban => "douban",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderConfig {
    pub id: ProviderId,
    pub enabled: bool,
    kind: ProviderConfigKind,
}

impl ProviderConfig {
    #[must_use]
    pub fn enabled(id: ProviderId) -> Self {
        Self::with_enabled(id, true)
    }

    #[must_use]
    pub fn disabled(id: ProviderId) -> Self {
        Self::with_enabled(id, false)
    }

    #[must_use]
    pub fn fixture(enabled: bool) -> Self {
        Self {
            id: ProviderId::Fixture,
            enabled,
            kind: ProviderConfigKind::Fixture,
        }
    }

    #[must_use]
    pub fn tmdb(enabled: bool, config: TmdbProviderConfig) -> Self {
        Self {
            id: ProviderId::Tmdb,
            enabled,
            kind: ProviderConfigKind::Tmdb(config),
        }
    }

    #[must_use]
    pub fn bangumi(enabled: bool, config: BangumiProviderConfig) -> Self {
        Self {
            id: ProviderId::Bangumi,
            enabled,
            kind: ProviderConfigKind::Bangumi(config),
        }
    }

    #[must_use]
    pub fn browser_worker(enabled: bool, config: BrowserWorkerProviderConfig) -> Self {
        Self {
            id: ProviderId::BrowserWorker,
            enabled,
            kind: ProviderConfigKind::BrowserWorker(config),
        }
    }

    #[must_use]
    pub fn douban(enabled: bool, config: DoubanProviderConfig) -> Self {
        Self {
            id: ProviderId::Douban,
            enabled,
            kind: ProviderConfigKind::Douban(config),
        }
    }

    #[must_use]
    pub fn tmdb_config(&self) -> Option<&TmdbProviderConfig> {
        match &self.kind {
            ProviderConfigKind::Tmdb(config) => Some(config),
            _ => None,
        }
    }

    #[must_use]
    pub fn bangumi_config(&self) -> Option<&BangumiProviderConfig> {
        match &self.kind {
            ProviderConfigKind::Bangumi(config) => Some(config),
            _ => None,
        }
    }

    #[must_use]
    pub fn browser_worker_config(&self) -> Option<&BrowserWorkerProviderConfig> {
        match &self.kind {
            ProviderConfigKind::BrowserWorker(config) => Some(config),
            _ => None,
        }
    }

    #[must_use]
    pub fn douban_config(&self) -> Option<&DoubanProviderConfig> {
        match &self.kind {
            ProviderConfigKind::Douban(config) => Some(config),
            _ => None,
        }
    }

    fn with_enabled(id: ProviderId, enabled: bool) -> Self {
        match id {
            ProviderId::Fixture => Self::fixture(enabled),
            ProviderId::Tmdb => Self::tmdb(enabled, TmdbProviderConfig::from_env_lookup(|_| None)),
            ProviderId::Bangumi => {
                Self::bangumi(enabled, BangumiProviderConfig::from_env_lookup(|_| None))
            }
            ProviderId::BrowserWorker => Self::browser_worker(
                enabled,
                BrowserWorkerProviderConfig::from_env_lookup(|_| None),
            ),
            ProviderId::Douban => {
                Self::douban(enabled, DoubanProviderConfig::from_env_lookup(|_| None))
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProviderConfigKind {
    Fixture,
    Tmdb(TmdbProviderConfig),
    Bangumi(BangumiProviderConfig),
    BrowserWorker(BrowserWorkerProviderConfig),
    Douban(DoubanProviderConfig),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Config {
    pub listen_addr: String,
    pub base_url: String,
    pub preferred_language: String,
    pub providers: Vec<ProviderConfig>,
    pub nako_runtime: NakoRuntimeConfig,
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
            providers: provider_configs_from_catalog(|name| lookup(name)),
            nako_runtime: NakoRuntimeConfig::from_env_lookup(|name| lookup(name)),
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

    #[must_use]
    pub fn provider_proxy_configured(&self, provider_id: ProviderId) -> bool {
        let Some(provider) = self.provider_config(provider_id) else {
            return false;
        };

        ProviderRegistry::catalog()
            .into_iter()
            .find(|entry| entry.id == provider_id)
            .is_some_and(|entry| (entry.proxy_configured)(provider))
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            listen_addr: "127.0.0.1:9100".to_owned(),
            base_url: "http://127.0.0.1:9100".to_owned(),
            preferred_language: "en-US".to_owned(),
            providers: provider_configs_from_catalog(|_| None),
            nako_runtime: NakoRuntimeConfig::disabled(),
        }
    }
}

fn provider_configs_from_catalog(
    mut lookup: impl FnMut(&str) -> Option<String>,
) -> Vec<ProviderConfig> {
    ProviderRegistry::catalog()
        .into_iter()
        .map(|entry| {
            let enabled = lookup(entry.enabled_env_var)
                .and_then(|value| parse_bool(&value))
                .unwrap_or(entry.default_enabled);
            (entry.load_config)(ProviderConfigInput {
                enabled,
                lookup: &mut lookup,
            })
        })
        .collect()
}

pub(crate) fn parse_bool(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

pub(crate) fn non_empty_trimmed(value: String) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_owned())
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
            config.providers[1].tmdb_config().unwrap().api_base_url,
            "https://api.themoviedb.org/3"
        );
        assert!(
            config.providers[1]
                .tmdb_config()
                .unwrap()
                .proxy_url
                .is_none()
        );
        assert_eq!(config.providers[2].id, ProviderId::Bangumi);
        assert!(!config.providers[2].enabled);
        let bangumi = config.providers[2].bangumi_config().unwrap();
        assert_eq!(bangumi.api_base_url, "https://api.bgm.tv");
        assert_eq!(bangumi.subject_types, vec![2]);
        assert!(!bangumi.include_nsfw);
        assert!(bangumi.proxy_url.is_none());
        assert_eq!(config.providers[3].id, ProviderId::BrowserWorker);
        assert!(!config.providers[3].enabled);
        let browser_worker = config.providers[3].browser_worker_config().unwrap();
        assert_eq!(browser_worker.base_url, "http://nako-browser-worker:3000");
        assert_eq!(browser_worker.extract_path, "/extract");
        assert_eq!(browser_worker.timeout_ms, 10_000);
        assert_eq!(config.providers[4].id, ProviderId::Douban);
        assert!(!config.providers[4].enabled);
        let douban = config.providers[4].douban_config().unwrap();
        assert_eq!(
            douban.search_base_url,
            "https://movie.douban.com/subject_search"
        );
        assert_eq!(
            douban.browser_worker_base_url,
            "http://nako-browser-worker:3000"
        );
        assert_eq!(douban.render_path, "/render");
        assert_eq!(douban.timeout_ms, 10_000);
        assert!(config.provider_enabled(ProviderId::Fixture));
        assert!(!config.provider_enabled(ProviderId::Tmdb));
        assert!(!config.provider_enabled(ProviderId::Bangumi));
        assert!(!config.provider_enabled(ProviderId::BrowserWorker));
        assert!(!config.provider_enabled(ProviderId::Douban));
        assert!(!config.provider_proxy_configured(ProviderId::Tmdb));
        assert!(!config.provider_proxy_configured(ProviderId::Bangumi));
    }

    #[test]
    fn config_from_env_lookup_overrides_provider_enabled_state() {
        let config = Config::from_env_lookup(|name| match name {
            "NAKO_METADATA_SCRAPER_LISTEN_ADDR" => Some("0.0.0.0:9200".to_owned()),
            "NAKO_METADATA_SCRAPER_BASE_URL" => Some("https://addon.example".to_owned()),
            "NAKO_METADATA_SCRAPER_LANGUAGE" => Some("zh-CN".to_owned()),
            "NAKO_METADATA_SCRAPER_NAKO_BASE_URL" => Some("https://nako.example".to_owned()),
            "NAKO_METADATA_SCRAPER_ADDON_TOKEN" => Some(" addon-token ".to_owned()),
            "NAKO_METADATA_SCRAPER_SIDE_EFFECTS_ENABLED" => Some("yes".to_owned()),
            "NAKO_METADATA_SCRAPER_NAKO_TIMEOUT_MS" => Some("2500".to_owned()),
            "NAKO_METADATA_SCRAPER_PROVIDER_FIXTURE_ENABLED" => Some("false".to_owned()),
            "NAKO_METADATA_SCRAPER_PROVIDER_TMDB_ENABLED" => Some("true".to_owned()),
            "NAKO_METADATA_SCRAPER_PROVIDER_BANGUMI_ENABLED" => Some("true".to_owned()),
            "NAKO_METADATA_SCRAPER_PROVIDER_BROWSER_WORKER_ENABLED" => Some("true".to_owned()),
            "NAKO_METADATA_SCRAPER_PROVIDER_DOUBAN_ENABLED" => Some("true".to_owned()),
            "NAKO_METADATA_SCRAPER_TMDB_READ_ACCESS_TOKEN" => Some("tmdb-token".to_owned()),
            "NAKO_METADATA_SCRAPER_TMDB_API_BASE_URL" => Some("https://tmdb.example/3".to_owned()),
            "NAKO_METADATA_SCRAPER_TMDB_LANGUAGE" => Some("ja-JP".to_owned()),
            "NAKO_METADATA_SCRAPER_TMDB_INCLUDE_ADULT" => Some("yes".to_owned()),
            "NAKO_METADATA_SCRAPER_TMDB_PROXY_URL" => {
                Some(" http://proxy.example:8080 ".to_owned())
            }
            "NAKO_METADATA_SCRAPER_BANGUMI_ACCESS_TOKEN" => Some("bangumi-token".to_owned()),
            "NAKO_METADATA_SCRAPER_BANGUMI_API_BASE_URL" => {
                Some("https://bangumi.example".to_owned())
            }
            "NAKO_METADATA_SCRAPER_BANGUMI_USER_AGENT" => {
                Some("Latias94/test-addon/0.1.0".to_owned())
            }
            "NAKO_METADATA_SCRAPER_BANGUMI_INCLUDE_NSFW" => Some("yes".to_owned()),
            "NAKO_METADATA_SCRAPER_BANGUMI_SUBJECT_TYPES" => Some("2,6".to_owned()),
            "NAKO_METADATA_SCRAPER_BANGUMI_PROXY_URL" => {
                Some(" http://proxy.example:8080 ".to_owned())
            }
            "NAKO_METADATA_SCRAPER_BROWSER_WORKER_BASE_URL" => {
                Some("http://browser-worker.example:3000".to_owned())
            }
            "NAKO_METADATA_SCRAPER_BROWSER_WORKER_EXTRACT_PATH" => Some("/extract".to_owned()),
            "NAKO_METADATA_SCRAPER_BROWSER_WORKER_TIMEOUT_MS" => Some("7500".to_owned()),
            "NAKO_METADATA_SCRAPER_DOUBAN_SEARCH_BASE_URL" => {
                Some("https://douban.example/subject_search".to_owned())
            }
            "NAKO_METADATA_SCRAPER_BROWSER_WORKER_RENDER_PATH" => Some("/render".to_owned()),
            "NAKO_METADATA_SCRAPER_DOUBAN_TIMEOUT_MS" => Some("6500".to_owned()),
            _ => None,
        });

        assert_eq!(config.listen_addr, "0.0.0.0:9200");
        assert_eq!(config.base_url, "https://addon.example");
        assert_eq!(config.preferred_language, "zh-CN");
        assert_eq!(
            config.nako_runtime,
            NakoRuntimeConfig {
                base_url: Some("https://nako.example".to_owned()),
                addon_token: Some("addon-token".to_owned()),
                side_effects_enabled: true,
                timeout_ms: 2500,
            }
        );
        assert!(config.nako_runtime.can_submit_side_effects());
        assert_eq!(
            config.providers[0],
            ProviderConfig::disabled(ProviderId::Fixture)
        );
        assert!(!config.provider_enabled(ProviderId::Fixture));
        assert!(config.provider_enabled(ProviderId::Tmdb));
        assert!(config.provider_enabled(ProviderId::Bangumi));
        let tmdb = config.providers[1].tmdb_config().unwrap();
        assert_eq!(tmdb.read_access_token.as_deref(), Some("tmdb-token"));
        assert_eq!(tmdb.api_base_url, "https://tmdb.example/3");
        assert_eq!(tmdb.language, "ja-JP");
        assert!(tmdb.include_adult);
        assert_eq!(tmdb.proxy_url.as_deref(), Some("http://proxy.example:8080"));
        let bangumi = config.providers[2].bangumi_config().unwrap();
        assert_eq!(bangumi.access_token.as_deref(), Some("bangumi-token"));
        assert_eq!(bangumi.api_base_url, "https://bangumi.example");
        assert_eq!(bangumi.user_agent, "Latias94/test-addon/0.1.0");
        assert!(bangumi.include_nsfw);
        assert_eq!(bangumi.subject_types, vec![2, 6]);
        assert_eq!(
            bangumi.proxy_url.as_deref(),
            Some("http://proxy.example:8080")
        );
        assert!(config.provider_proxy_configured(ProviderId::Tmdb));
        assert!(config.provider_proxy_configured(ProviderId::Bangumi));
        assert!(config.provider_enabled(ProviderId::BrowserWorker));
        let browser_worker = config.providers[3].browser_worker_config().unwrap();
        assert_eq!(
            browser_worker.base_url,
            "http://browser-worker.example:3000"
        );
        assert_eq!(browser_worker.extract_path, "/extract");
        assert_eq!(browser_worker.timeout_ms, 7500);
        assert!(config.provider_enabled(ProviderId::Douban));
        let douban = config.providers[4].douban_config().unwrap();
        assert_eq!(
            douban.search_base_url,
            "https://douban.example/subject_search"
        );
        assert_eq!(
            douban.browser_worker_base_url,
            "http://browser-worker.example:3000"
        );
        assert_eq!(douban.render_path, "/render");
        assert_eq!(douban.timeout_ms, 6500);
    }

    #[test]
    fn tmdb_config_trims_network_boundary_values() {
        let config = TmdbProviderConfig::from_env_lookup(|name| match name {
            "NAKO_METADATA_SCRAPER_TMDB_READ_ACCESS_TOKEN" => Some(" tmdb-token ".to_owned()),
            "NAKO_METADATA_SCRAPER_TMDB_API_BASE_URL" => {
                Some(" https://tmdb.example/3/ ".to_owned())
            }
            "NAKO_METADATA_SCRAPER_TMDB_LANGUAGE" => Some(" zh-CN ".to_owned()),
            _ => None,
        });

        assert_eq!(config.read_access_token.as_deref(), Some("tmdb-token"));
        assert_eq!(config.api_base_url, "https://tmdb.example/3/");
        assert_eq!(config.language, "zh-CN");
    }

    #[test]
    fn bangumi_config_trims_network_boundary_values() {
        let config = BangumiProviderConfig::from_env_lookup(|name| match name {
            "NAKO_METADATA_SCRAPER_BANGUMI_ACCESS_TOKEN" => Some(" bangumi-token ".to_owned()),
            "NAKO_METADATA_SCRAPER_BANGUMI_API_BASE_URL" => {
                Some(" https://bangumi.example ".to_owned())
            }
            "NAKO_METADATA_SCRAPER_BANGUMI_USER_AGENT" => {
                Some(" Latias94/test-addon/0.1.0 ".to_owned())
            }
            _ => None,
        });

        assert_eq!(config.access_token.as_deref(), Some("bangumi-token"));
        assert_eq!(config.api_base_url, "https://bangumi.example");
        assert_eq!(config.user_agent, "Latias94/test-addon/0.1.0");
    }

    #[test]
    fn invalid_bangumi_subject_types_fall_back_to_anime() {
        let config = Config::from_env_lookup(|name| match name {
            "NAKO_METADATA_SCRAPER_BANGUMI_SUBJECT_TYPES" => Some("2,5".to_owned()),
            _ => None,
        });

        assert_eq!(
            config.providers[2].bangumi_config().unwrap().subject_types,
            vec![2]
        );
        assert_eq!(
            config.providers[3]
                .browser_worker_config()
                .unwrap()
                .extract_path,
            "/extract"
        );
    }

    #[test]
    fn nako_runtime_defaults_to_no_side_effect_authority() {
        let config = Config::default();

        assert_eq!(config.nako_runtime, NakoRuntimeConfig::disabled());
        assert!(!config.nako_runtime.can_submit_side_effects());
    }
}
