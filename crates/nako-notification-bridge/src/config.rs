use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Config {
    pub listen_addr: String,
    pub base_url: String,
    pub http_webhook: HttpWebhookConfig,
}

impl Config {
    pub const DEFAULT_LISTEN_ADDR: &'static str = "127.0.0.1:9110";
    pub const DEFAULT_BASE_URL: &'static str = "http://127.0.0.1:9110";

    #[must_use]
    pub fn from_env() -> Self {
        Self::from_env_lookup(|name| std::env::var(name).ok())
    }

    #[must_use]
    pub fn from_env_lookup(mut lookup: impl FnMut(&str) -> Option<String>) -> Self {
        Self {
            listen_addr: lookup("NAKO_NOTIFICATION_BRIDGE_LISTEN_ADDR")
                .and_then(non_empty_trimmed)
                .unwrap_or_else(|| Self::DEFAULT_LISTEN_ADDR.to_owned()),
            base_url: lookup("NAKO_NOTIFICATION_BRIDGE_BASE_URL")
                .and_then(non_empty_trimmed)
                .unwrap_or_else(|| Self::DEFAULT_BASE_URL.to_owned()),
            http_webhook: HttpWebhookConfig::from_env_lookup(|name| lookup(name)),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            listen_addr: Self::DEFAULT_LISTEN_ADDR.to_owned(),
            base_url: Self::DEFAULT_BASE_URL.to_owned(),
            http_webhook: HttpWebhookConfig::default(),
        }
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct HttpWebhookConfig {
    pub enabled: bool,
    pub target_url: Option<String>,
    pub secret_header_name: String,
    pub shared_secret: Option<String>,
    pub timeout_ms: u64,
}

impl HttpWebhookConfig {
    pub const DEFAULT_SECRET_HEADER_NAME: &'static str = "X-Nako-Notification-Secret";
    pub const DEFAULT_TIMEOUT_MS: u64 = 10_000;

    #[must_use]
    pub fn from_env_lookup(mut lookup: impl FnMut(&str) -> Option<String>) -> Self {
        Self {
            enabled: lookup("NAKO_NOTIFICATION_BRIDGE_HTTP_WEBHOOK_ENABLED")
                .and_then(|value| parse_bool(&value))
                .unwrap_or(false),
            target_url: lookup("NAKO_NOTIFICATION_BRIDGE_HTTP_WEBHOOK_URL")
                .and_then(non_empty_trimmed),
            secret_header_name: lookup("NAKO_NOTIFICATION_BRIDGE_HTTP_WEBHOOK_SECRET_HEADER_NAME")
                .and_then(non_empty_trimmed)
                .unwrap_or_else(|| Self::DEFAULT_SECRET_HEADER_NAME.to_owned()),
            shared_secret: lookup("NAKO_NOTIFICATION_BRIDGE_HTTP_WEBHOOK_SHARED_SECRET")
                .and_then(non_empty_trimmed),
            timeout_ms: lookup("NAKO_NOTIFICATION_BRIDGE_HTTP_WEBHOOK_TIMEOUT_MS")
                .and_then(|value| parse_positive_u64(&value))
                .unwrap_or(Self::DEFAULT_TIMEOUT_MS),
        }
    }

    #[must_use]
    pub fn status(&self) -> HttpWebhookConfigStatus {
        if !self.enabled {
            return HttpWebhookConfigStatus::Disabled;
        }

        let Some(target_url) = self.target_url.as_deref() else {
            return HttpWebhookConfigStatus::MissingTargetUrl;
        };

        if is_valid_http_url(target_url) {
            HttpWebhookConfigStatus::Configured
        } else {
            HttpWebhookConfigStatus::InvalidTargetUrl
        }
    }

    #[must_use]
    pub fn target_url_configured(&self) -> bool {
        self.target_url.is_some()
    }

    #[must_use]
    pub fn target_url_valid(&self) -> bool {
        self.target_url.as_deref().is_some_and(is_valid_http_url)
    }

    #[must_use]
    pub fn shared_secret_configured(&self) -> bool {
        self.shared_secret.is_some()
    }

    #[must_use]
    pub fn custom_secret_header_name_configured(&self) -> bool {
        self.secret_header_name != Self::DEFAULT_SECRET_HEADER_NAME
    }

    #[must_use]
    pub fn send_path_enabled(&self) -> bool {
        self.status() == HttpWebhookConfigStatus::Configured
    }
}

impl Default for HttpWebhookConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            target_url: None,
            secret_header_name: Self::DEFAULT_SECRET_HEADER_NAME.to_owned(),
            shared_secret: None,
            timeout_ms: Self::DEFAULT_TIMEOUT_MS,
        }
    }
}

impl fmt::Debug for HttpWebhookConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("HttpWebhookConfig")
            .field("enabled", &self.enabled)
            .field("status", &self.status())
            .field("target_url_configured", &self.target_url_configured())
            .field("target_url_valid", &self.target_url_valid())
            .field(
                "custom_secret_header_name_configured",
                &self.custom_secret_header_name_configured(),
            )
            .field("shared_secret_configured", &self.shared_secret_configured())
            .field("timeout_ms", &self.timeout_ms)
            .finish()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HttpWebhookConfigStatus {
    Disabled,
    MissingTargetUrl,
    InvalidTargetUrl,
    Configured,
}

impl HttpWebhookConfigStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Disabled => "disabled",
            Self::MissingTargetUrl => "missing_target_url",
            Self::InvalidTargetUrl => "invalid_target_url",
            Self::Configured => "configured",
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

fn parse_positive_u64(value: &str) -> Option<u64> {
    value.trim().parse::<u64>().ok().filter(|value| *value > 0)
}

fn is_valid_http_url(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    let Some(rest) = lower
        .strip_prefix("http://")
        .or_else(|| lower.strip_prefix("https://"))
    else {
        return false;
    };

    !rest.is_empty() && !rest.starts_with('/')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_keeps_http_webhook_disabled() {
        let config = Config::default();

        assert_eq!(config.listen_addr, Config::DEFAULT_LISTEN_ADDR);
        assert_eq!(config.base_url, Config::DEFAULT_BASE_URL);
        assert!(!config.http_webhook.enabled);
        assert_eq!(
            config.http_webhook.status(),
            HttpWebhookConfigStatus::Disabled
        );
        assert!(!config.http_webhook.target_url_configured());
        assert!(!config.http_webhook.shared_secret_configured());
        assert!(!config.http_webhook.custom_secret_header_name_configured());
    }

    #[test]
    fn config_from_env_lookup_reads_http_webhook_contract_without_exposing_values() {
        let config = Config::from_env_lookup(|name| match name {
            "NAKO_NOTIFICATION_BRIDGE_LISTEN_ADDR" => Some(" 0.0.0.0:9110 ".to_owned()),
            "NAKO_NOTIFICATION_BRIDGE_BASE_URL" => Some(" http://bridge.local ".to_owned()),
            "NAKO_NOTIFICATION_BRIDGE_HTTP_WEBHOOK_ENABLED" => Some("yes".to_owned()),
            "NAKO_NOTIFICATION_BRIDGE_HTTP_WEBHOOK_URL" => {
                Some(" https://hooks.example/internal/path ".to_owned())
            }
            "NAKO_NOTIFICATION_BRIDGE_HTTP_WEBHOOK_SECRET_HEADER_NAME" => {
                Some(" X-Custom-Secret ".to_owned())
            }
            "NAKO_NOTIFICATION_BRIDGE_HTTP_WEBHOOK_SHARED_SECRET" => {
                Some(" should-not-appear-in-debug ".to_owned())
            }
            "NAKO_NOTIFICATION_BRIDGE_HTTP_WEBHOOK_TIMEOUT_MS" => Some("2500".to_owned()),
            _ => None,
        });

        assert_eq!(config.listen_addr, "0.0.0.0:9110");
        assert_eq!(config.base_url, "http://bridge.local");
        assert!(config.http_webhook.enabled);
        assert_eq!(
            config.http_webhook.status(),
            HttpWebhookConfigStatus::Configured
        );
        assert!(config.http_webhook.target_url_configured());
        assert!(config.http_webhook.target_url_valid());
        assert!(config.http_webhook.shared_secret_configured());
        assert!(config.http_webhook.custom_secret_header_name_configured());
        assert_eq!(config.http_webhook.timeout_ms, 2500);

        let debug = format!("{:?}", config.http_webhook);
        assert!(!debug.contains("hooks.example"));
        assert!(!debug.contains("should-not-appear-in-debug"));
        assert!(!debug.contains("X-Custom-Secret"));
    }

    #[test]
    fn enabled_http_webhook_requires_a_valid_http_target_url() {
        let missing = HttpWebhookConfig {
            enabled: true,
            ..HttpWebhookConfig::default()
        };
        assert_eq!(missing.status(), HttpWebhookConfigStatus::MissingTargetUrl);

        let invalid = HttpWebhookConfig::from_env_lookup(|name| match name {
            "NAKO_NOTIFICATION_BRIDGE_HTTP_WEBHOOK_ENABLED" => Some("true".to_owned()),
            "NAKO_NOTIFICATION_BRIDGE_HTTP_WEBHOOK_URL" => Some("file:///tmp/hook".to_owned()),
            _ => None,
        });
        assert_eq!(invalid.status(), HttpWebhookConfigStatus::InvalidTargetUrl);
        assert!(invalid.target_url_configured());
        assert!(!invalid.target_url_valid());
    }
}
