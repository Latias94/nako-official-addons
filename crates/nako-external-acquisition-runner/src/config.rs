use std::fmt;

use nako_official_addon_catalog::external_acquisition_runner;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Config {
    pub listen_addr: String,
    pub base_url: String,
    pub fixture_profile_enabled: bool,
    pub default_runner_profile_id: String,
    pub nako_materialization: NakoMaterializationConfig,
}

#[derive(Clone, Eq, PartialEq)]
pub struct NakoMaterializationConfig {
    pub enabled: bool,
    pub base_url: Option<String>,
    pub addon_token: Option<String>,
    pub timeout_ms: u64,
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
            nako_materialization: NakoMaterializationConfig::from_env_lookup(|name| lookup(name)),
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
            nako_materialization: NakoMaterializationConfig::disabled(),
        }
    }
}

impl NakoMaterializationConfig {
    pub const DEFAULT_TIMEOUT_MS: u64 = 10_000;

    #[must_use]
    pub fn from_env_lookup(mut lookup: impl FnMut(&str) -> Option<String>) -> Self {
        Self {
            enabled: lookup("NAKO_EXTERNAL_ACQUISITION_RUNNER_MATERIALIZATION_ENABLED")
                .and_then(|value| parse_bool(&value))
                .unwrap_or(false),
            base_url: lookup("NAKO_EXTERNAL_ACQUISITION_RUNNER_NAKO_BASE_URL")
                .and_then(non_empty_trimmed),
            addon_token: lookup("NAKO_EXTERNAL_ACQUISITION_RUNNER_ADDON_TOKEN")
                .and_then(non_empty_trimmed),
            timeout_ms: lookup("NAKO_EXTERNAL_ACQUISITION_RUNNER_NAKO_TIMEOUT_MS")
                .and_then(|value| value.trim().parse::<u64>().ok())
                .filter(|value| *value > 0)
                .unwrap_or(Self::DEFAULT_TIMEOUT_MS),
        }
    }

    #[must_use]
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            base_url: None,
            addon_token: None,
            timeout_ms: Self::DEFAULT_TIMEOUT_MS,
        }
    }

    #[must_use]
    pub fn can_request_host_materialization(&self) -> bool {
        self.enabled && self.base_url.is_some() && self.addon_token.is_some()
    }

    #[must_use]
    pub fn runtime_client_config(&self) -> Option<nako_addon_client::NakoRuntimeClientConfig> {
        self.can_request_host_materialization().then(|| {
            nako_addon_client::NakoRuntimeClientConfig {
                base_url: self.base_url.clone().expect("checked by can_request"),
                addon_token: self.addon_token.clone().expect("checked by can_request"),
                timeout_ms: self.timeout_ms,
            }
        })
    }
}

impl fmt::Debug for NakoMaterializationConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("NakoMaterializationConfig")
            .field("enabled", &self.enabled)
            .field("base_url", &self.base_url)
            .field(
                "addon_token",
                &self.addon_token.as_ref().map(|_| "<redacted>"),
            )
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
        assert_eq!(
            config.nako_materialization,
            NakoMaterializationConfig::disabled()
        );
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
            "NAKO_EXTERNAL_ACQUISITION_RUNNER_MATERIALIZATION_ENABLED" => Some("true".to_owned()),
            "NAKO_EXTERNAL_ACQUISITION_RUNNER_NAKO_BASE_URL" => {
                Some(" https://nako.example ".to_owned())
            }
            "NAKO_EXTERNAL_ACQUISITION_RUNNER_ADDON_TOKEN" => {
                Some(" addon-token-secret ".to_owned())
            }
            "NAKO_EXTERNAL_ACQUISITION_RUNNER_NAKO_TIMEOUT_MS" => Some("2500".to_owned()),
            _ => None,
        });

        assert_eq!(config.listen_addr, "0.0.0.0:9160");
        assert_eq!(config.base_url, "http://runner.local");
        assert!(!config.fixture_profile_enabled);
        assert_eq!(config.default_runner_profile_id, "fixture-alt");
        assert_eq!(config.active_profile_count(), 0);
        assert_eq!(
            config.nako_materialization,
            NakoMaterializationConfig {
                enabled: true,
                base_url: Some("https://nako.example".to_owned()),
                addon_token: Some("addon-token-secret".to_owned()),
                timeout_ms: 2500,
            }
        );
        assert!(
            config
                .nako_materialization
                .runtime_client_config()
                .is_some()
        );
        assert!(!format!("{config:?}").contains("addon-token-secret"));
    }
}
