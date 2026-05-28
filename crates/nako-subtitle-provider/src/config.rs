#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Config {
    pub listen_addr: String,
    pub base_url: String,
    pub fixture_provider_enabled: bool,
    pub default_language: String,
    pub default_limit: usize,
    pub max_limit: usize,
}

impl Config {
    pub const DEFAULT_LISTEN_ADDR: &'static str = "127.0.0.1:9140";
    pub const DEFAULT_BASE_URL: &'static str = "http://127.0.0.1:9140";
    pub const DEFAULT_LANGUAGE: &'static str = "en";
    pub const DEFAULT_LIMIT: usize = 10;
    pub const DEFAULT_MAX_LIMIT: usize = 50;

    #[must_use]
    pub fn from_env() -> Self {
        Self::from_env_lookup(|name| std::env::var(name).ok())
    }

    #[must_use]
    pub fn from_env_lookup(mut lookup: impl FnMut(&str) -> Option<String>) -> Self {
        let default_limit = lookup("NAKO_SUBTITLE_PROVIDER_DEFAULT_LIMIT")
            .and_then(|value| parse_positive_usize(&value))
            .map(|value| value.clamp(1, Self::DEFAULT_MAX_LIMIT))
            .unwrap_or(Self::DEFAULT_LIMIT);
        let max_limit = lookup("NAKO_SUBTITLE_PROVIDER_MAX_LIMIT")
            .and_then(|value| parse_positive_usize(&value))
            .map(|value| value.clamp(1, 200))
            .unwrap_or(Self::DEFAULT_MAX_LIMIT)
            .max(default_limit);

        Self {
            listen_addr: lookup("NAKO_SUBTITLE_PROVIDER_LISTEN_ADDR")
                .and_then(non_empty_trimmed)
                .unwrap_or_else(|| Self::DEFAULT_LISTEN_ADDR.to_owned()),
            base_url: lookup("NAKO_SUBTITLE_PROVIDER_BASE_URL")
                .and_then(non_empty_trimmed)
                .unwrap_or_else(|| Self::DEFAULT_BASE_URL.to_owned()),
            fixture_provider_enabled: lookup("NAKO_SUBTITLE_PROVIDER_FIXTURE_PROVIDER_ENABLED")
                .and_then(|value| parse_bool(&value))
                .unwrap_or(true),
            default_language: lookup("NAKO_SUBTITLE_PROVIDER_DEFAULT_LANGUAGE")
                .and_then(non_empty_trimmed)
                .unwrap_or_else(|| Self::DEFAULT_LANGUAGE.to_owned()),
            default_limit,
            max_limit,
        }
    }

    #[must_use]
    pub fn active_provider_count(&self) -> usize {
        usize::from(self.fixture_provider_enabled)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            listen_addr: Self::DEFAULT_LISTEN_ADDR.to_owned(),
            base_url: Self::DEFAULT_BASE_URL.to_owned(),
            fixture_provider_enabled: true,
            default_language: Self::DEFAULT_LANGUAGE.to_owned(),
            default_limit: Self::DEFAULT_LIMIT,
            max_limit: Self::DEFAULT_MAX_LIMIT,
        }
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

fn parse_bool(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
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
    fn default_config_enables_fixture_provider_only() {
        let config = Config::default();

        assert_eq!(config.listen_addr, Config::DEFAULT_LISTEN_ADDR);
        assert_eq!(config.base_url, Config::DEFAULT_BASE_URL);
        assert!(config.fixture_provider_enabled);
        assert_eq!(config.default_language, "en");
        assert_eq!(config.active_provider_count(), 1);
    }

    #[test]
    fn config_reads_environment_with_bounds() {
        let config = Config::from_env_lookup(|name| match name {
            "NAKO_SUBTITLE_PROVIDER_LISTEN_ADDR" => Some(" 0.0.0.0:9140 ".to_owned()),
            "NAKO_SUBTITLE_PROVIDER_BASE_URL" => Some(" http://subtitle.local ".to_owned()),
            "NAKO_SUBTITLE_PROVIDER_FIXTURE_PROVIDER_ENABLED" => Some("false".to_owned()),
            "NAKO_SUBTITLE_PROVIDER_DEFAULT_LANGUAGE" => Some(" zh-CN ".to_owned()),
            "NAKO_SUBTITLE_PROVIDER_DEFAULT_LIMIT" => Some("500".to_owned()),
            "NAKO_SUBTITLE_PROVIDER_MAX_LIMIT" => Some("2".to_owned()),
            _ => None,
        });

        assert_eq!(config.listen_addr, "0.0.0.0:9140");
        assert_eq!(config.base_url, "http://subtitle.local");
        assert!(!config.fixture_provider_enabled);
        assert_eq!(config.default_language, "zh-CN");
        assert_eq!(config.default_limit, Config::DEFAULT_MAX_LIMIT);
        assert_eq!(config.max_limit, Config::DEFAULT_MAX_LIMIT);
    }
}
