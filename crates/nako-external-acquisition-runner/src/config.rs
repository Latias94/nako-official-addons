use nako_official_addon_catalog::external_acquisition_runner;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Config {
    pub listen_addr: String,
    pub base_url: String,
    pub fixture_profile_enabled: bool,
    pub default_runner_profile_id: String,
}

impl Config {
    pub const DEFAULT_LISTEN_ADDR: &'static str = "127.0.0.1:9160";
    pub const DEFAULT_BASE_URL: &'static str = "http://127.0.0.1:9160";

    #[must_use]
    pub fn from_env() -> Self {
        Self::from_env_lookup(|name| std::env::var(name).ok())
    }

    #[must_use]
    pub fn from_env_lookup(mut lookup: impl FnMut(&str) -> Option<String>) -> Self {
        Self {
            listen_addr: lookup("NAKO_EXTERNAL_ACQUISITION_RUNNER_LISTEN_ADDR")
                .and_then(non_empty_trimmed)
                .unwrap_or_else(|| Self::DEFAULT_LISTEN_ADDR.to_owned()),
            base_url: lookup("NAKO_EXTERNAL_ACQUISITION_RUNNER_BASE_URL")
                .and_then(non_empty_trimmed)
                .unwrap_or_else(|| Self::DEFAULT_BASE_URL.to_owned()),
            fixture_profile_enabled: lookup(
                "NAKO_EXTERNAL_ACQUISITION_RUNNER_FIXTURE_PROFILE_ENABLED",
            )
            .and_then(|value| parse_bool(&value))
            .unwrap_or(true),
            default_runner_profile_id: lookup(
                "NAKO_EXTERNAL_ACQUISITION_RUNNER_DEFAULT_PROFILE_ID",
            )
            .and_then(non_empty_trimmed)
            .unwrap_or_else(|| external_acquisition_runner::DEFAULT_RUNNER_PROFILE_ID.to_owned()),
        }
    }

    #[must_use]
    pub fn active_profile_count(&self) -> usize {
        usize::from(self.fixture_profile_enabled)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            listen_addr: Self::DEFAULT_LISTEN_ADDR.to_owned(),
            base_url: Self::DEFAULT_BASE_URL.to_owned(),
            fixture_profile_enabled: true,
            default_runner_profile_id: external_acquisition_runner::DEFAULT_RUNNER_PROFILE_ID
                .to_owned(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_enables_fixture_profile_only() {
        let config = Config::default();

        assert_eq!(config.listen_addr, Config::DEFAULT_LISTEN_ADDR);
        assert_eq!(config.base_url, Config::DEFAULT_BASE_URL);
        assert!(config.fixture_profile_enabled);
        assert_eq!(config.default_runner_profile_id, "fixture");
        assert_eq!(config.active_profile_count(), 1);
    }

    #[test]
    fn config_reads_environment() {
        let config = Config::from_env_lookup(|name| match name {
            "NAKO_EXTERNAL_ACQUISITION_RUNNER_LISTEN_ADDR" => Some(" 0.0.0.0:9160 ".to_owned()),
            "NAKO_EXTERNAL_ACQUISITION_RUNNER_BASE_URL" => Some(" http://runner.local ".to_owned()),
            "NAKO_EXTERNAL_ACQUISITION_RUNNER_FIXTURE_PROFILE_ENABLED" => Some("false".to_owned()),
            "NAKO_EXTERNAL_ACQUISITION_RUNNER_DEFAULT_PROFILE_ID" => {
                Some(" fixture-alt ".to_owned())
            }
            _ => None,
        });

        assert_eq!(config.listen_addr, "0.0.0.0:9160");
        assert_eq!(config.base_url, "http://runner.local");
        assert!(!config.fixture_profile_enabled);
        assert_eq!(config.default_runner_profile_id, "fixture-alt");
        assert_eq!(config.active_profile_count(), 0);
    }
}
