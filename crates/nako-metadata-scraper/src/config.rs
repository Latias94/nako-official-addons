use crate::providers::{ProviderConfigInput, ProviderRegistry};
pub use crate::providers::{
    bangumi::BangumiProviderConfig, browser_worker::BrowserWorkerProviderConfig,
    dmm::DmmProviderConfig, douban::DoubanProviderConfig, fc2::Fc2ProviderConfig,
    javbus::JavbusProviderConfig, javdb::JavdbProviderConfig, javlibrary::JavlibraryProviderConfig,
    mgstage::MgstageProviderConfig, prestige::PrestigeProviderConfig, tmdb::TmdbProviderConfig,
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

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ProviderExecutionConfig {
    pub max_selected_providers: Option<usize>,
}

impl ProviderExecutionConfig {
    #[must_use]
    pub fn from_env_lookup(mut lookup: impl FnMut(&str) -> Option<String>) -> Self {
        Self {
            max_selected_providers: lookup(
                "NAKO_METADATA_SCRAPER_PROVIDER_MAX_SELECTED_PER_REQUEST",
            )
            .and_then(|value| parse_positive_usize(&value)),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProviderId {
    Fixture,
    Tmdb,
    Bangumi,
    BrowserWorker,
    Douban,
    Javdb,
    Dmm,
    Fc2,
    Javbus,
    Javlibrary,
    Mgstage,
    Prestige,
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
            Self::Javdb => "javdb",
            Self::Dmm => "dmm",
            Self::Fc2 => "fc2",
            Self::Javbus => "javbus",
            Self::Javlibrary => "javlibrary",
            Self::Mgstage => "mgstage",
            Self::Prestige => "prestige",
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
    pub fn javdb(enabled: bool, config: JavdbProviderConfig) -> Self {
        Self {
            id: ProviderId::Javdb,
            enabled,
            kind: ProviderConfigKind::Javdb(config),
        }
    }

    #[must_use]
    pub fn dmm(enabled: bool, config: DmmProviderConfig) -> Self {
        Self {
            id: ProviderId::Dmm,
            enabled,
            kind: ProviderConfigKind::Dmm(config),
        }
    }

    #[must_use]
    pub fn fc2(enabled: bool, config: Fc2ProviderConfig) -> Self {
        Self {
            id: ProviderId::Fc2,
            enabled,
            kind: ProviderConfigKind::Fc2(config),
        }
    }

    #[must_use]
    pub fn javbus(enabled: bool, config: JavbusProviderConfig) -> Self {
        Self {
            id: ProviderId::Javbus,
            enabled,
            kind: ProviderConfigKind::Javbus(config),
        }
    }

    #[must_use]
    pub fn javlibrary(enabled: bool, config: JavlibraryProviderConfig) -> Self {
        Self {
            id: ProviderId::Javlibrary,
            enabled,
            kind: ProviderConfigKind::Javlibrary(config),
        }
    }

    #[must_use]
    pub fn mgstage(enabled: bool, config: MgstageProviderConfig) -> Self {
        Self {
            id: ProviderId::Mgstage,
            enabled,
            kind: ProviderConfigKind::Mgstage(config),
        }
    }

    #[must_use]
    pub fn prestige(enabled: bool, config: PrestigeProviderConfig) -> Self {
        Self {
            id: ProviderId::Prestige,
            enabled,
            kind: ProviderConfigKind::Prestige(config),
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

    #[must_use]
    pub fn javdb_config(&self) -> Option<&JavdbProviderConfig> {
        match &self.kind {
            ProviderConfigKind::Javdb(config) => Some(config),
            _ => None,
        }
    }

    #[must_use]
    pub fn dmm_config(&self) -> Option<&DmmProviderConfig> {
        match &self.kind {
            ProviderConfigKind::Dmm(config) => Some(config),
            _ => None,
        }
    }

    #[must_use]
    pub fn fc2_config(&self) -> Option<&Fc2ProviderConfig> {
        match &self.kind {
            ProviderConfigKind::Fc2(config) => Some(config),
            _ => None,
        }
    }

    #[must_use]
    pub fn javbus_config(&self) -> Option<&JavbusProviderConfig> {
        match &self.kind {
            ProviderConfigKind::Javbus(config) => Some(config),
            _ => None,
        }
    }

    #[must_use]
    pub fn javlibrary_config(&self) -> Option<&JavlibraryProviderConfig> {
        match &self.kind {
            ProviderConfigKind::Javlibrary(config) => Some(config),
            _ => None,
        }
    }

    #[must_use]
    pub fn mgstage_config(&self) -> Option<&MgstageProviderConfig> {
        match &self.kind {
            ProviderConfigKind::Mgstage(config) => Some(config),
            _ => None,
        }
    }

    #[must_use]
    pub fn prestige_config(&self) -> Option<&PrestigeProviderConfig> {
        match &self.kind {
            ProviderConfigKind::Prestige(config) => Some(config),
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
            ProviderId::Javdb => {
                Self::javdb(enabled, JavdbProviderConfig::from_env_lookup(|_| None))
            }
            ProviderId::Dmm => Self::dmm(enabled, DmmProviderConfig::from_env_lookup(|_| None)),
            ProviderId::Fc2 => Self::fc2(enabled, Fc2ProviderConfig::from_env_lookup(|_| None)),
            ProviderId::Javbus => {
                Self::javbus(enabled, JavbusProviderConfig::from_env_lookup(|_| None))
            }
            ProviderId::Javlibrary => {
                Self::javlibrary(enabled, JavlibraryProviderConfig::from_env_lookup(|_| None))
            }
            ProviderId::Mgstage => {
                Self::mgstage(enabled, MgstageProviderConfig::from_env_lookup(|_| None))
            }
            ProviderId::Prestige => {
                Self::prestige(enabled, PrestigeProviderConfig::from_env_lookup(|_| None))
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
    Javdb(JavdbProviderConfig),
    Dmm(DmmProviderConfig),
    Fc2(Fc2ProviderConfig),
    Javbus(JavbusProviderConfig),
    Javlibrary(JavlibraryProviderConfig),
    Mgstage(MgstageProviderConfig),
    Prestige(PrestigeProviderConfig),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Config {
    pub listen_addr: String,
    pub base_url: String,
    pub preferred_language: String,
    pub providers: Vec<ProviderConfig>,
    pub provider_execution: ProviderExecutionConfig,
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
            provider_execution: ProviderExecutionConfig::from_env_lookup(|name| lookup(name)),
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

    #[must_use]
    pub fn rendered_page_proxy_policy_configured(&self) -> bool {
        self.providers
            .iter()
            .any(ProviderConfig::rendered_page_proxy_policy_configured)
    }

    #[must_use]
    pub fn rendered_page_session_key_configured(&self) -> bool {
        self.providers
            .iter()
            .any(ProviderConfig::rendered_page_session_key_configured)
    }
}

impl ProviderConfig {
    fn rendered_page_proxy_policy_configured(&self) -> bool {
        match &self.kind {
            ProviderConfigKind::BrowserWorker(config) => {
                config.rendered_pages.proxy_policy_configured()
            }
            ProviderConfigKind::Douban(config) => config.rendered_pages.proxy_policy_configured(),
            ProviderConfigKind::Javdb(config) => config.rendered_pages.proxy_policy_configured(),
            ProviderConfigKind::Dmm(config) => config.rendered_pages.proxy_policy_configured(),
            ProviderConfigKind::Fc2(config) => config.rendered_pages.proxy_policy_configured(),
            ProviderConfigKind::Javbus(config) => config.rendered_pages.proxy_policy_configured(),
            ProviderConfigKind::Javlibrary(config) => {
                config.rendered_pages.proxy_policy_configured()
            }
            ProviderConfigKind::Mgstage(config) => config.rendered_pages.proxy_policy_configured(),
            ProviderConfigKind::Prestige(_) => false,
            ProviderConfigKind::Fixture
            | ProviderConfigKind::Tmdb(_)
            | ProviderConfigKind::Bangumi(_) => false,
        }
    }

    fn rendered_page_session_key_configured(&self) -> bool {
        match &self.kind {
            ProviderConfigKind::BrowserWorker(config) => {
                config.rendered_pages.session_key_configured()
            }
            ProviderConfigKind::Douban(config) => config.rendered_pages.session_key_configured(),
            ProviderConfigKind::Javdb(config) => config.rendered_pages.session_key_configured(),
            ProviderConfigKind::Dmm(config) => config.rendered_pages.session_key_configured(),
            ProviderConfigKind::Fc2(config) => config.rendered_pages.session_key_configured(),
            ProviderConfigKind::Javbus(config) => config.rendered_pages.session_key_configured(),
            ProviderConfigKind::Javlibrary(config) => {
                config.rendered_pages.session_key_configured()
            }
            ProviderConfigKind::Mgstage(config) => config.rendered_pages.session_key_configured(),
            ProviderConfigKind::Prestige(_) => false,
            ProviderConfigKind::Fixture
            | ProviderConfigKind::Tmdb(_)
            | ProviderConfigKind::Bangumi(_) => false,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            listen_addr: "127.0.0.1:9100".to_owned(),
            base_url: "http://127.0.0.1:9100".to_owned(),
            preferred_language: "en-US".to_owned(),
            providers: provider_configs_from_catalog(|_| None),
            provider_execution: ProviderExecutionConfig::default(),
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

fn parse_positive_usize(value: &str) -> Option<usize> {
    value
        .trim()
        .parse::<usize>()
        .ok()
        .filter(|value| *value > 0)
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
        assert_eq!(
            browser_worker.rendered_pages.base_url,
            "http://nako-browser-worker:3000"
        );
        assert_eq!(browser_worker.extract_path, "/extract");
        assert_eq!(browser_worker.rendered_pages.timeout_ms, 10_000);
        assert_eq!(config.providers[4].id, ProviderId::Douban);
        assert!(!config.providers[4].enabled);
        let douban = config.providers[4].douban_config().unwrap();
        assert_eq!(
            douban.search_base_url,
            "https://movie.douban.com/subject_search"
        );
        assert_eq!(
            douban.rendered_pages.base_url,
            "http://nako-browser-worker:3000"
        );
        assert_eq!(douban.render_path, "/render");
        assert_eq!(douban.rendered_pages.timeout_ms, 10_000);
        assert_eq!(config.providers[5].id, ProviderId::Javdb);
        assert!(!config.providers[5].enabled);
        let javdb = config.providers[5].javdb_config().unwrap();
        assert_eq!(javdb.base_url, "https://javdb.com");
        assert_eq!(
            javdb.rendered_pages.base_url,
            "http://nako-browser-worker:3000"
        );
        assert_eq!(javdb.render_path, "/render");
        assert_eq!(javdb.rendered_pages.timeout_ms, 10_000);
        assert_eq!(config.providers[6].id, ProviderId::Dmm);
        assert!(!config.providers[6].enabled);
        let dmm = config.providers[6].dmm_config().unwrap();
        assert_eq!(dmm.base_url, "https://www.dmm.co.jp");
        assert_eq!(
            dmm.rendered_pages.base_url,
            "http://nako-browser-worker:3000"
        );
        assert_eq!(dmm.render_path, "/render");
        assert_eq!(dmm.rendered_pages.timeout_ms, 10_000);
        assert_eq!(config.providers[7].id, ProviderId::Fc2);
        assert!(!config.providers[7].enabled);
        let fc2 = config.providers[7].fc2_config().unwrap();
        assert_eq!(fc2.base_url, "https://adult.contents.fc2.com");
        assert_eq!(
            fc2.rendered_pages.base_url,
            "http://nako-browser-worker:3000"
        );
        assert_eq!(fc2.render_path, "/render");
        assert_eq!(fc2.rendered_pages.timeout_ms, 10_000);
        assert_eq!(config.providers[8].id, ProviderId::Javbus);
        assert!(!config.providers[8].enabled);
        let javbus = config.providers[8].javbus_config().unwrap();
        assert_eq!(javbus.base_url, "https://www.javbus.com");
        assert_eq!(
            javbus.rendered_pages.base_url,
            "http://nako-browser-worker:3000"
        );
        assert_eq!(javbus.render_path, "/render");
        assert_eq!(javbus.rendered_pages.timeout_ms, 10_000);
        assert_eq!(config.providers[9].id, ProviderId::Javlibrary);
        assert!(!config.providers[9].enabled);
        let javlibrary = config.providers[9].javlibrary_config().unwrap();
        assert_eq!(javlibrary.base_url, "https://www.javlibrary.com");
        assert_eq!(javlibrary.language_path, "cn");
        assert_eq!(
            javlibrary.rendered_pages.base_url,
            "http://nako-browser-worker:3000"
        );
        assert_eq!(javlibrary.render_path, "/render");
        assert_eq!(javlibrary.rendered_pages.timeout_ms, 10_000);
        assert_eq!(config.providers[10].id, ProviderId::Mgstage);
        assert!(!config.providers[10].enabled);
        let mgstage = config.providers[10].mgstage_config().unwrap();
        assert_eq!(mgstage.base_url, "https://www.mgstage.com");
        assert_eq!(
            mgstage.rendered_pages.base_url,
            "http://nako-browser-worker:3000"
        );
        assert_eq!(mgstage.render_path, "/render");
        assert_eq!(mgstage.rendered_pages.timeout_ms, 10_000);
        assert_eq!(config.providers[11].id, ProviderId::Prestige);
        assert!(!config.providers[11].enabled);
        let prestige = config.providers[11].prestige_config().unwrap();
        assert_eq!(prestige.base_url, "https://www.prestige-av.com");
        assert_eq!(prestige.timeout_ms, 10_000);
        assert!(prestige.proxy_url.is_none());
        assert!(config.provider_enabled(ProviderId::Fixture));
        assert!(!config.provider_enabled(ProviderId::Tmdb));
        assert!(!config.provider_enabled(ProviderId::Bangumi));
        assert!(!config.provider_enabled(ProviderId::BrowserWorker));
        assert!(!config.provider_enabled(ProviderId::Douban));
        assert!(!config.provider_enabled(ProviderId::Javdb));
        assert!(!config.provider_enabled(ProviderId::Dmm));
        assert!(!config.provider_enabled(ProviderId::Fc2));
        assert!(!config.provider_enabled(ProviderId::Javbus));
        assert!(!config.provider_enabled(ProviderId::Javlibrary));
        assert!(!config.provider_enabled(ProviderId::Mgstage));
        assert!(!config.provider_enabled(ProviderId::Prestige));
        assert!(!config.provider_proxy_configured(ProviderId::Tmdb));
        assert!(!config.provider_proxy_configured(ProviderId::Bangumi));
        assert!(!config.provider_proxy_configured(ProviderId::Prestige));
        assert_eq!(
            config.provider_execution,
            ProviderExecutionConfig::default()
        );
        assert!(!config.rendered_page_proxy_policy_configured());
        assert!(!config.rendered_page_session_key_configured());
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
            "NAKO_METADATA_SCRAPER_PROVIDER_JAVDB_ENABLED" => Some("true".to_owned()),
            "NAKO_METADATA_SCRAPER_PROVIDER_DMM_ENABLED" => Some("true".to_owned()),
            "NAKO_METADATA_SCRAPER_PROVIDER_FC2_ENABLED" => Some("true".to_owned()),
            "NAKO_METADATA_SCRAPER_PROVIDER_JAVBUS_ENABLED" => Some("true".to_owned()),
            "NAKO_METADATA_SCRAPER_PROVIDER_JAVLIBRARY_ENABLED" => Some("true".to_owned()),
            "NAKO_METADATA_SCRAPER_PROVIDER_MGSTAGE_ENABLED" => Some("true".to_owned()),
            "NAKO_METADATA_SCRAPER_PROVIDER_PRESTIGE_ENABLED" => Some("true".to_owned()),
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
            "NAKO_METADATA_SCRAPER_BROWSER_WORKER_PROXY_POLICY" => Some("required".to_owned()),
            "NAKO_METADATA_SCRAPER_BROWSER_WORKER_SESSION_KEY" => Some(" av-batch ".to_owned()),
            "NAKO_METADATA_SCRAPER_DOUBAN_SEARCH_BASE_URL" => {
                Some("https://douban.example/subject_search".to_owned())
            }
            "NAKO_METADATA_SCRAPER_BROWSER_WORKER_RENDER_PATH" => Some("/render".to_owned()),
            "NAKO_METADATA_SCRAPER_DOUBAN_TIMEOUT_MS" => Some("6500".to_owned()),
            "NAKO_METADATA_SCRAPER_JAVDB_BASE_URL" => Some("https://javdb.example".to_owned()),
            "NAKO_METADATA_SCRAPER_JAVDB_TIMEOUT_MS" => Some("5500".to_owned()),
            "NAKO_METADATA_SCRAPER_DMM_BASE_URL" => Some("https://dmm.example".to_owned()),
            "NAKO_METADATA_SCRAPER_DMM_TIMEOUT_MS" => Some("3500".to_owned()),
            "NAKO_METADATA_SCRAPER_FC2_BASE_URL" => Some("https://fc2.example".to_owned()),
            "NAKO_METADATA_SCRAPER_FC2_TIMEOUT_MS" => Some("4500".to_owned()),
            "NAKO_METADATA_SCRAPER_JAVBUS_BASE_URL" => Some("https://javbus.example".to_owned()),
            "NAKO_METADATA_SCRAPER_JAVBUS_TIMEOUT_MS" => Some("8500".to_owned()),
            "NAKO_METADATA_SCRAPER_JAVLIBRARY_BASE_URL" => {
                Some("https://javlibrary.example".to_owned())
            }
            "NAKO_METADATA_SCRAPER_JAVLIBRARY_LANGUAGE" => Some("ja".to_owned()),
            "NAKO_METADATA_SCRAPER_JAVLIBRARY_TIMEOUT_MS" => Some("9500".to_owned()),
            "NAKO_METADATA_SCRAPER_MGSTAGE_BASE_URL" => Some("https://mgstage.example".to_owned()),
            "NAKO_METADATA_SCRAPER_MGSTAGE_TIMEOUT_MS" => Some("10500".to_owned()),
            "NAKO_METADATA_SCRAPER_PRESTIGE_BASE_URL" => {
                Some("https://prestige.example".to_owned())
            }
            "NAKO_METADATA_SCRAPER_PRESTIGE_TIMEOUT_MS" => Some("11500".to_owned()),
            "NAKO_METADATA_SCRAPER_PRESTIGE_PROXY_URL" => {
                Some(" http://prestige-proxy.example:8080 ".to_owned())
            }
            "NAKO_METADATA_SCRAPER_PROVIDER_MAX_SELECTED_PER_REQUEST" => Some("2".to_owned()),
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
            config.provider_execution,
            ProviderExecutionConfig {
                max_selected_providers: Some(2)
            }
        );
        assert!(config.rendered_page_proxy_policy_configured());
        assert!(config.rendered_page_session_key_configured());
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
            browser_worker.rendered_pages.base_url,
            "http://browser-worker.example:3000"
        );
        assert_eq!(browser_worker.extract_path, "/extract");
        assert_eq!(browser_worker.rendered_pages.timeout_ms, 7500);
        assert!(config.provider_enabled(ProviderId::Douban));
        let douban = config.providers[4].douban_config().unwrap();
        assert_eq!(
            douban.search_base_url,
            "https://douban.example/subject_search"
        );
        assert_eq!(
            douban.rendered_pages.base_url,
            "http://browser-worker.example:3000"
        );
        assert_eq!(douban.render_path, "/render");
        assert_eq!(douban.rendered_pages.timeout_ms, 6500);
        assert!(config.provider_enabled(ProviderId::Javdb));
        let javdb = config.providers[5].javdb_config().unwrap();
        assert_eq!(javdb.base_url, "https://javdb.example");
        assert_eq!(
            javdb.rendered_pages.base_url,
            "http://browser-worker.example:3000"
        );
        assert_eq!(javdb.render_path, "/render");
        assert_eq!(javdb.rendered_pages.timeout_ms, 5500);
        assert!(config.provider_enabled(ProviderId::Dmm));
        let dmm = config.providers[6].dmm_config().unwrap();
        assert_eq!(dmm.base_url, "https://dmm.example");
        assert_eq!(
            dmm.rendered_pages.base_url,
            "http://browser-worker.example:3000"
        );
        assert_eq!(dmm.render_path, "/render");
        assert_eq!(dmm.rendered_pages.timeout_ms, 3500);
        assert!(config.provider_enabled(ProviderId::Fc2));
        let fc2 = config.providers[7].fc2_config().unwrap();
        assert_eq!(fc2.base_url, "https://fc2.example");
        assert_eq!(
            fc2.rendered_pages.base_url,
            "http://browser-worker.example:3000"
        );
        assert_eq!(fc2.render_path, "/render");
        assert_eq!(fc2.rendered_pages.timeout_ms, 4500);
        assert!(config.provider_enabled(ProviderId::Javbus));
        let javbus = config.providers[8].javbus_config().unwrap();
        assert_eq!(javbus.base_url, "https://javbus.example");
        assert_eq!(
            javbus.rendered_pages.base_url,
            "http://browser-worker.example:3000"
        );
        assert_eq!(javbus.render_path, "/render");
        assert_eq!(javbus.rendered_pages.timeout_ms, 8500);
        assert!(config.provider_enabled(ProviderId::Javlibrary));
        let javlibrary = config.providers[9].javlibrary_config().unwrap();
        assert_eq!(javlibrary.base_url, "https://javlibrary.example");
        assert_eq!(javlibrary.language_path, "ja");
        assert_eq!(
            javlibrary.rendered_pages.base_url,
            "http://browser-worker.example:3000"
        );
        assert_eq!(javlibrary.render_path, "/render");
        assert_eq!(javlibrary.rendered_pages.timeout_ms, 9500);
        assert!(config.provider_enabled(ProviderId::Mgstage));
        let mgstage = config.providers[10].mgstage_config().unwrap();
        assert_eq!(mgstage.base_url, "https://mgstage.example");
        assert_eq!(
            mgstage.rendered_pages.base_url,
            "http://browser-worker.example:3000"
        );
        assert_eq!(mgstage.render_path, "/render");
        assert_eq!(mgstage.rendered_pages.timeout_ms, 10500);
        assert!(config.provider_enabled(ProviderId::Prestige));
        let prestige = config.providers[11].prestige_config().unwrap();
        assert_eq!(prestige.base_url, "https://prestige.example");
        assert_eq!(prestige.timeout_ms, 11500);
        assert_eq!(
            prestige.proxy_url.as_deref(),
            Some("http://prestige-proxy.example:8080")
        );
        assert!(config.provider_proxy_configured(ProviderId::Prestige));
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
