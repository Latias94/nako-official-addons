use std::{fmt, str::FromStr};

use crate::domain::ResourceLinkType;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Config {
    pub listen_addr: String,
    pub base_url: String,
    pub fixture_provider_enabled: bool,
    pub pansou: PansouProviderConfig,
    pub default_limit: usize,
    pub max_limit: usize,
    pub search_timeout_ms: u64,
}

impl Config {
    pub const DEFAULT_LISTEN_ADDR: &'static str = "127.0.0.1:9130";
    pub const DEFAULT_BASE_URL: &'static str = "http://127.0.0.1:9130";
    pub const DEFAULT_LIMIT: usize = 20;
    pub const DEFAULT_MAX_LIMIT: usize = 100;
    pub const DEFAULT_SEARCH_TIMEOUT_MS: u64 = 10_000;

    #[must_use]
    pub fn from_env() -> Self {
        Self::from_env_lookup(|name| std::env::var(name).ok())
    }

    #[must_use]
    pub fn from_env_lookup(mut lookup: impl FnMut(&str) -> Option<String>) -> Self {
        let default_limit = lookup("NAKO_RESOURCE_SEARCH_DEFAULT_LIMIT")
            .and_then(|value| parse_positive_usize(&value))
            .map(|value| value.clamp(1, Self::DEFAULT_MAX_LIMIT))
            .unwrap_or(Self::DEFAULT_LIMIT);
        let max_limit = lookup("NAKO_RESOURCE_SEARCH_MAX_LIMIT")
            .and_then(|value| parse_positive_usize(&value))
            .map(|value| value.clamp(1, 500))
            .unwrap_or(Self::DEFAULT_MAX_LIMIT)
            .max(default_limit);

        Self {
            listen_addr: lookup("NAKO_RESOURCE_SEARCH_LISTEN_ADDR")
                .and_then(non_empty_trimmed)
                .unwrap_or_else(|| Self::DEFAULT_LISTEN_ADDR.to_owned()),
            base_url: lookup("NAKO_RESOURCE_SEARCH_BASE_URL")
                .and_then(non_empty_trimmed)
                .unwrap_or_else(|| Self::DEFAULT_BASE_URL.to_owned()),
            fixture_provider_enabled: lookup("NAKO_RESOURCE_SEARCH_FIXTURE_PROVIDER_ENABLED")
                .and_then(|value| parse_bool(&value))
                .unwrap_or(true),
            pansou: PansouProviderConfig::from_env_lookup(&mut lookup),
            default_limit,
            max_limit,
            search_timeout_ms: lookup("NAKO_RESOURCE_SEARCH_TIMEOUT_MS")
                .and_then(|value| parse_positive_u64(&value))
                .map(|value| value.clamp(250, 60_000))
                .unwrap_or(Self::DEFAULT_SEARCH_TIMEOUT_MS),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            listen_addr: Self::DEFAULT_LISTEN_ADDR.to_owned(),
            base_url: Self::DEFAULT_BASE_URL.to_owned(),
            fixture_provider_enabled: true,
            pansou: PansouProviderConfig::default(),
            default_limit: Self::DEFAULT_LIMIT,
            max_limit: Self::DEFAULT_MAX_LIMIT,
            search_timeout_ms: Self::DEFAULT_SEARCH_TIMEOUT_MS,
        }
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct PansouProviderConfig {
    pub enabled: bool,
    pub base_url: Option<String>,
    pub bearer_token: Option<String>,
    pub source_type: String,
    pub plugins: Vec<String>,
    pub cloud_types: Vec<ResourceLinkType>,
    pub concurrency: Option<u16>,
    pub timeout_ms: u64,
}

impl PansouProviderConfig {
    pub const DEFAULT_SOURCE_TYPE: &'static str = "all";
    pub const DEFAULT_TIMEOUT_MS: u64 = 10_000;

    #[must_use]
    pub fn from_env_lookup(lookup: &mut impl FnMut(&str) -> Option<String>) -> Self {
        Self {
            enabled: lookup("NAKO_RESOURCE_SEARCH_PANSOU_PROVIDER_ENABLED")
                .and_then(|value| parse_bool(&value))
                .unwrap_or(false),
            base_url: lookup("NAKO_RESOURCE_SEARCH_PANSOU_BASE_URL").and_then(normalize_base_url),
            bearer_token: lookup("NAKO_RESOURCE_SEARCH_PANSOU_TOKEN").and_then(non_empty_trimmed),
            source_type: lookup("NAKO_RESOURCE_SEARCH_PANSOU_SOURCE_TYPE")
                .and_then(non_empty_trimmed)
                .unwrap_or_else(|| Self::DEFAULT_SOURCE_TYPE.to_owned()),
            plugins: lookup("NAKO_RESOURCE_SEARCH_PANSOU_PLUGINS")
                .map(|value| parse_csv(&value))
                .unwrap_or_default(),
            cloud_types: lookup("NAKO_RESOURCE_SEARCH_PANSOU_CLOUD_TYPES")
                .map(|value| parse_link_types_csv(&value))
                .unwrap_or_default(),
            concurrency: lookup("NAKO_RESOURCE_SEARCH_PANSOU_CONCURRENCY")
                .and_then(|value| parse_positive_u16(&value)),
            timeout_ms: lookup("NAKO_RESOURCE_SEARCH_PANSOU_TIMEOUT_MS")
                .and_then(|value| parse_positive_u64(&value))
                .map(|value| value.clamp(250, 60_000))
                .unwrap_or(Self::DEFAULT_TIMEOUT_MS),
        }
    }

    #[must_use]
    pub fn is_active(&self) -> bool {
        self.enabled && self.base_url.is_some()
    }

    #[must_use]
    pub fn bearer_token_configured(&self) -> bool {
        self.bearer_token
            .as_deref()
            .is_some_and(|token| !token.trim().is_empty())
    }
}

impl Default for PansouProviderConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            base_url: None,
            bearer_token: None,
            source_type: Self::DEFAULT_SOURCE_TYPE.to_owned(),
            plugins: Vec::new(),
            cloud_types: Vec::new(),
            concurrency: None,
            timeout_ms: Self::DEFAULT_TIMEOUT_MS,
        }
    }
}

impl fmt::Debug for PansouProviderConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PansouProviderConfig")
            .field("enabled", &self.enabled)
            .field("base_url", &self.base_url)
            .field("bearer_token_configured", &self.bearer_token_configured())
            .field("source_type", &self.source_type)
            .field("plugins", &self.plugins)
            .field("cloud_types", &self.cloud_types)
            .field("concurrency", &self.concurrency)
            .field("timeout_ms", &self.timeout_ms)
            .finish()
    }
}

fn non_empty_trimmed(value: String) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_owned())
    }
}

fn normalize_base_url(value: String) -> Option<String> {
    let value = non_empty_trimmed(value)?;
    let value = value.trim_end_matches('/').to_owned();
    if value.starts_with("http://") || value.starts_with("https://") {
        Some(value)
    } else {
        None
    }
}

fn parse_csv(value: &str) -> Vec<String> {
    value
        .split(',')
        .filter_map(|part| non_empty_trimmed(part.to_owned()))
        .collect()
}

fn parse_link_types_csv(value: &str) -> Vec<ResourceLinkType> {
    value
        .split(',')
        .filter_map(|part| ResourceLinkType::from_str(part).ok())
        .collect()
}

fn parse_bool(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

fn parse_positive_u64(value: &str) -> Option<u64> {
    value.trim().parse::<u64>().ok().filter(|value| *value > 0)
}

fn parse_positive_u16(value: &str) -> Option<u16> {
    value.trim().parse::<u16>().ok().filter(|value| *value > 0)
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
    fn default_config_enables_fixture_search_only() {
        let config = Config::default();

        assert_eq!(config.listen_addr, Config::DEFAULT_LISTEN_ADDR);
        assert_eq!(config.base_url, Config::DEFAULT_BASE_URL);
        assert!(config.fixture_provider_enabled);
        assert!(!config.pansou.is_active());
        assert_eq!(config.default_limit, Config::DEFAULT_LIMIT);
        assert_eq!(config.max_limit, Config::DEFAULT_MAX_LIMIT);
    }

    #[test]
    fn config_reads_environment_with_bounds() {
        let config = Config::from_env_lookup(|name| match name {
            "NAKO_RESOURCE_SEARCH_LISTEN_ADDR" => Some(" 0.0.0.0:9130 ".to_owned()),
            "NAKO_RESOURCE_SEARCH_BASE_URL" => Some(" http://resource-search.local ".to_owned()),
            "NAKO_RESOURCE_SEARCH_FIXTURE_PROVIDER_ENABLED" => Some("false".to_owned()),
            "NAKO_RESOURCE_SEARCH_DEFAULT_LIMIT" => Some("600".to_owned()),
            "NAKO_RESOURCE_SEARCH_MAX_LIMIT" => Some("4".to_owned()),
            "NAKO_RESOURCE_SEARCH_TIMEOUT_MS" => Some("120000".to_owned()),
            _ => None,
        });

        assert_eq!(config.listen_addr, "0.0.0.0:9130");
        assert_eq!(config.base_url, "http://resource-search.local");
        assert!(!config.fixture_provider_enabled);
        assert!(!config.pansou.enabled);
        assert_eq!(config.default_limit, Config::DEFAULT_MAX_LIMIT);
        assert_eq!(config.max_limit, Config::DEFAULT_MAX_LIMIT);
        assert_eq!(config.search_timeout_ms, 60_000);
    }

    #[test]
    fn pansou_provider_requires_enablement_and_base_url() {
        let config = Config::from_env_lookup(|name| match name {
            "NAKO_RESOURCE_SEARCH_PANSOU_PROVIDER_ENABLED" => Some("true".to_owned()),
            "NAKO_RESOURCE_SEARCH_PANSOU_BASE_URL" => Some(" http://127.0.0.1:8888/ ".to_owned()),
            "NAKO_RESOURCE_SEARCH_PANSOU_TOKEN" => Some(" secret-token ".to_owned()),
            "NAKO_RESOURCE_SEARCH_PANSOU_SOURCE_TYPE" => Some("plugin".to_owned()),
            "NAKO_RESOURCE_SEARCH_PANSOU_PLUGINS" => Some("jikepan, pansearch ".to_owned()),
            "NAKO_RESOURCE_SEARCH_PANSOU_CLOUD_TYPES" => Some("quark,magnet,web".to_owned()),
            "NAKO_RESOURCE_SEARCH_PANSOU_CONCURRENCY" => Some("4".to_owned()),
            "NAKO_RESOURCE_SEARCH_PANSOU_TIMEOUT_MS" => Some("500".to_owned()),
            _ => None,
        });

        assert!(config.pansou.is_active());
        assert_eq!(
            config.pansou.base_url.as_deref(),
            Some("http://127.0.0.1:8888")
        );
        assert_eq!(config.pansou.source_type, "plugin");
        assert_eq!(
            config.pansou.plugins,
            vec!["jikepan".to_owned(), "pansearch".to_owned()]
        );
        assert_eq!(
            config.pansou.cloud_types,
            vec![
                ResourceLinkType::Quark,
                ResourceLinkType::Magnet,
                ResourceLinkType::Web
            ]
        );
        assert_eq!(config.pansou.concurrency, Some(4));
        assert_eq!(config.pansou.timeout_ms, 500);

        let debug = format!("{config:?}");
        assert!(debug.contains("bearer_token_configured"));
        assert!(!debug.contains("secret-token"));
    }

    #[test]
    fn pansou_provider_ignores_invalid_base_url() {
        let config = Config::from_env_lookup(|name| match name {
            "NAKO_RESOURCE_SEARCH_PANSOU_PROVIDER_ENABLED" => Some("true".to_owned()),
            "NAKO_RESOURCE_SEARCH_PANSOU_BASE_URL" => Some("file:///tmp/pansou".to_owned()),
            _ => None,
        });

        assert!(!config.pansou.is_active());
    }
}
