use std::fmt;

use crate::template::{DEFAULT_SUMMARY_TEMPLATE, TemplateStatus, template_status};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Config {
    pub listen_addr: String,
    pub base_url: String,
    pub http_webhook: HttpWebhookConfig,
    pub discord_webhook: DiscordWebhookConfig,
    pub telegram: TelegramConfig,
    pub template: NotificationTemplateConfig,
    pub provider_attempt_history_capacity: usize,
}

impl Config {
    pub const DEFAULT_LISTEN_ADDR: &'static str = "127.0.0.1:9110";
    pub const DEFAULT_BASE_URL: &'static str = "http://127.0.0.1:9110";
    pub const DEFAULT_PROVIDER_ATTEMPT_HISTORY_CAPACITY: usize = 20;

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
            discord_webhook: DiscordWebhookConfig::from_env_lookup(|name| lookup(name)),
            telegram: TelegramConfig::from_env_lookup(|name| lookup(name)),
            template: NotificationTemplateConfig::from_env_lookup(|name| lookup(name)),
            provider_attempt_history_capacity: lookup(
                "NAKO_NOTIFICATION_BRIDGE_PROVIDER_ATTEMPT_HISTORY_CAPACITY",
            )
            .and_then(|value| parse_positive_usize(&value))
            .unwrap_or(Self::DEFAULT_PROVIDER_ATTEMPT_HISTORY_CAPACITY),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            listen_addr: Self::DEFAULT_LISTEN_ADDR.to_owned(),
            base_url: Self::DEFAULT_BASE_URL.to_owned(),
            http_webhook: HttpWebhookConfig::default(),
            discord_webhook: DiscordWebhookConfig::default(),
            telegram: TelegramConfig::default(),
            template: NotificationTemplateConfig::default(),
            provider_attempt_history_capacity: Self::DEFAULT_PROVIDER_ATTEMPT_HISTORY_CAPACITY,
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

#[derive(Clone, Eq, PartialEq)]
pub struct DiscordWebhookConfig {
    pub enabled: bool,
    pub webhook_url: Option<String>,
    pub timeout_ms: u64,
}

impl DiscordWebhookConfig {
    pub const DEFAULT_TIMEOUT_MS: u64 = 10_000;

    #[must_use]
    pub fn from_env_lookup(mut lookup: impl FnMut(&str) -> Option<String>) -> Self {
        Self {
            enabled: lookup("NAKO_NOTIFICATION_BRIDGE_DISCORD_WEBHOOK_ENABLED")
                .and_then(|value| parse_bool(&value))
                .unwrap_or(false),
            webhook_url: lookup("NAKO_NOTIFICATION_BRIDGE_DISCORD_WEBHOOK_URL")
                .and_then(non_empty_trimmed),
            timeout_ms: lookup("NAKO_NOTIFICATION_BRIDGE_DISCORD_WEBHOOK_TIMEOUT_MS")
                .and_then(|value| parse_positive_u64(&value))
                .unwrap_or(Self::DEFAULT_TIMEOUT_MS),
        }
    }

    #[must_use]
    pub fn status(&self) -> DiscordWebhookConfigStatus {
        if !self.enabled {
            return DiscordWebhookConfigStatus::Disabled;
        }

        let Some(webhook_url) = self.webhook_url.as_deref() else {
            return DiscordWebhookConfigStatus::MissingWebhookUrl;
        };

        if is_valid_http_url(webhook_url) {
            DiscordWebhookConfigStatus::Configured
        } else {
            DiscordWebhookConfigStatus::InvalidWebhookUrl
        }
    }

    #[must_use]
    pub fn webhook_url_configured(&self) -> bool {
        self.webhook_url.is_some()
    }

    #[must_use]
    pub fn webhook_url_valid(&self) -> bool {
        self.webhook_url.as_deref().is_some_and(is_valid_http_url)
    }

    #[must_use]
    pub fn send_path_enabled(&self) -> bool {
        self.status() == DiscordWebhookConfigStatus::Configured
    }
}

impl Default for DiscordWebhookConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            webhook_url: None,
            timeout_ms: Self::DEFAULT_TIMEOUT_MS,
        }
    }
}

impl fmt::Debug for DiscordWebhookConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DiscordWebhookConfig")
            .field("enabled", &self.enabled)
            .field("status", &self.status())
            .field("webhook_url_configured", &self.webhook_url_configured())
            .field("webhook_url_valid", &self.webhook_url_valid())
            .field("timeout_ms", &self.timeout_ms)
            .finish()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiscordWebhookConfigStatus {
    Disabled,
    MissingWebhookUrl,
    InvalidWebhookUrl,
    Configured,
}

impl DiscordWebhookConfigStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Disabled => "disabled",
            Self::MissingWebhookUrl => "missing_webhook_url",
            Self::InvalidWebhookUrl => "invalid_webhook_url",
            Self::Configured => "configured",
        }
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct TelegramConfig {
    pub enabled: bool,
    pub api_base_url: String,
    api_base_url_configured: bool,
    pub bot_token: Option<String>,
    pub chat_id: Option<String>,
    pub timeout_ms: u64,
}

impl TelegramConfig {
    pub const DEFAULT_API_BASE_URL: &'static str = "https://api.telegram.org";
    pub const DEFAULT_TIMEOUT_MS: u64 = 10_000;

    #[must_use]
    pub fn from_env_lookup(mut lookup: impl FnMut(&str) -> Option<String>) -> Self {
        let configured_api_base_url =
            lookup("NAKO_NOTIFICATION_BRIDGE_TELEGRAM_API_BASE_URL").and_then(non_empty_trimmed);
        Self {
            enabled: lookup("NAKO_NOTIFICATION_BRIDGE_TELEGRAM_ENABLED")
                .and_then(|value| parse_bool(&value))
                .unwrap_or(false),
            api_base_url: configured_api_base_url
                .clone()
                .unwrap_or_else(|| Self::DEFAULT_API_BASE_URL.to_owned()),
            api_base_url_configured: configured_api_base_url.is_some(),
            bot_token: lookup("NAKO_NOTIFICATION_BRIDGE_TELEGRAM_BOT_TOKEN")
                .and_then(non_empty_trimmed),
            chat_id: lookup("NAKO_NOTIFICATION_BRIDGE_TELEGRAM_CHAT_ID")
                .and_then(non_empty_trimmed),
            timeout_ms: lookup("NAKO_NOTIFICATION_BRIDGE_TELEGRAM_TIMEOUT_MS")
                .and_then(|value| parse_positive_u64(&value))
                .unwrap_or(Self::DEFAULT_TIMEOUT_MS),
        }
    }

    #[must_use]
    pub fn status(&self) -> TelegramConfigStatus {
        if !self.enabled {
            return TelegramConfigStatus::Disabled;
        }

        if !is_valid_http_url(&self.api_base_url) {
            return TelegramConfigStatus::InvalidApiBaseUrl;
        }

        if self.bot_token.is_none() {
            return TelegramConfigStatus::MissingBotToken;
        }

        if self.chat_id.is_none() {
            return TelegramConfigStatus::MissingChatId;
        }

        TelegramConfigStatus::Configured
    }

    #[must_use]
    pub const fn api_base_url_configured(&self) -> bool {
        self.api_base_url_configured
    }

    #[must_use]
    pub fn api_base_url_valid(&self) -> bool {
        is_valid_http_url(&self.api_base_url)
    }

    #[must_use]
    pub fn bot_token_configured(&self) -> bool {
        self.bot_token.is_some()
    }

    #[must_use]
    pub fn chat_id_configured(&self) -> bool {
        self.chat_id.is_some()
    }

    #[must_use]
    pub fn send_path_enabled(&self) -> bool {
        self.status() == TelegramConfigStatus::Configured
    }
}

impl Default for TelegramConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            api_base_url: Self::DEFAULT_API_BASE_URL.to_owned(),
            api_base_url_configured: false,
            bot_token: None,
            chat_id: None,
            timeout_ms: Self::DEFAULT_TIMEOUT_MS,
        }
    }
}

impl fmt::Debug for TelegramConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TelegramConfig")
            .field("enabled", &self.enabled)
            .field("status", &self.status())
            .field("api_base_url_configured", &self.api_base_url_configured())
            .field("api_base_url_valid", &self.api_base_url_valid())
            .field("bot_token_configured", &self.bot_token_configured())
            .field("chat_id_configured", &self.chat_id_configured())
            .field("timeout_ms", &self.timeout_ms)
            .finish()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TelegramConfigStatus {
    Disabled,
    InvalidApiBaseUrl,
    MissingBotToken,
    MissingChatId,
    Configured,
}

impl TelegramConfigStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Disabled => "disabled",
            Self::InvalidApiBaseUrl => "invalid_api_base_url",
            Self::MissingBotToken => "missing_bot_token",
            Self::MissingChatId => "missing_chat_id",
            Self::Configured => "configured",
        }
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct NotificationTemplateConfig {
    pub summary_template: String,
    summary_template_configured: bool,
}

impl NotificationTemplateConfig {
    #[must_use]
    pub fn from_env_lookup(mut lookup: impl FnMut(&str) -> Option<String>) -> Self {
        if let Some(summary_template) =
            lookup("NAKO_NOTIFICATION_BRIDGE_TEMPLATE_SUMMARY").and_then(non_empty_trimmed)
        {
            return Self {
                summary_template,
                summary_template_configured: true,
            };
        }

        Self::default()
    }

    #[must_use]
    pub fn status(&self) -> TemplateStatus {
        template_status(&self.summary_template)
    }

    #[must_use]
    pub fn summary_template_configured(&self) -> bool {
        self.summary_template_configured
    }

    #[must_use]
    pub fn summary_template_valid(&self) -> bool {
        self.status() == TemplateStatus::Valid
    }
}

impl Default for NotificationTemplateConfig {
    fn default() -> Self {
        Self {
            summary_template: DEFAULT_SUMMARY_TEMPLATE.to_owned(),
            summary_template_configured: false,
        }
    }
}

impl fmt::Debug for NotificationTemplateConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("NotificationTemplateConfig")
            .field(
                "summary_template_configured",
                &self.summary_template_configured(),
            )
            .field("summary_template_valid", &self.summary_template_valid())
            .field("status", &self.status())
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

fn parse_positive_u64(value: &str) -> Option<u64> {
    value.trim().parse::<u64>().ok().filter(|value| *value > 0)
}

fn parse_positive_usize(value: &str) -> Option<usize> {
    value
        .trim()
        .parse::<usize>()
        .ok()
        .filter(|value| *value > 0)
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
        assert!(!config.discord_webhook.enabled);
        assert_eq!(
            config.discord_webhook.status(),
            DiscordWebhookConfigStatus::Disabled
        );
        assert!(!config.discord_webhook.webhook_url_configured());
        assert!(!config.telegram.enabled);
        assert_eq!(config.telegram.status(), TelegramConfigStatus::Disabled);
        assert!(!config.telegram.api_base_url_configured());
        assert!(config.telegram.api_base_url_valid());
        assert!(!config.telegram.bot_token_configured());
        assert!(!config.telegram.chat_id_configured());
        assert!(!config.template.summary_template_configured());
        assert!(config.template.summary_template_valid());
        assert_eq!(
            config.provider_attempt_history_capacity,
            Config::DEFAULT_PROVIDER_ATTEMPT_HISTORY_CAPACITY
        );
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
            "NAKO_NOTIFICATION_BRIDGE_DISCORD_WEBHOOK_ENABLED" => Some("true".to_owned()),
            "NAKO_NOTIFICATION_BRIDGE_DISCORD_WEBHOOK_URL" => {
                Some(" https://discord.example/api/webhooks/secret ".to_owned())
            }
            "NAKO_NOTIFICATION_BRIDGE_DISCORD_WEBHOOK_TIMEOUT_MS" => Some("3000".to_owned()),
            "NAKO_NOTIFICATION_BRIDGE_TELEGRAM_ENABLED" => Some("true".to_owned()),
            "NAKO_NOTIFICATION_BRIDGE_TELEGRAM_API_BASE_URL" => {
                Some(" https://api.telegram.example ".to_owned())
            }
            "NAKO_NOTIFICATION_BRIDGE_TELEGRAM_BOT_TOKEN" => {
                Some(" telegram-token-should-not-appear ".to_owned())
            }
            "NAKO_NOTIFICATION_BRIDGE_TELEGRAM_CHAT_ID" => {
                Some(" telegram-chat-should-not-appear ".to_owned())
            }
            "NAKO_NOTIFICATION_BRIDGE_TELEGRAM_TIMEOUT_MS" => Some("3500".to_owned()),
            "NAKO_NOTIFICATION_BRIDGE_TEMPLATE_SUMMARY" => {
                Some(" {{event_kind}} secret-literal-should-not-appear ".to_owned())
            }
            "NAKO_NOTIFICATION_BRIDGE_PROVIDER_ATTEMPT_HISTORY_CAPACITY" => Some("5".to_owned()),
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
        assert!(config.discord_webhook.enabled);
        assert_eq!(
            config.discord_webhook.status(),
            DiscordWebhookConfigStatus::Configured
        );
        assert!(config.discord_webhook.webhook_url_configured());
        assert!(config.discord_webhook.webhook_url_valid());
        assert_eq!(config.discord_webhook.timeout_ms, 3000);
        assert!(config.telegram.enabled);
        assert_eq!(config.telegram.status(), TelegramConfigStatus::Configured);
        assert!(config.telegram.api_base_url_configured());
        assert!(config.telegram.api_base_url_valid());
        assert!(config.telegram.bot_token_configured());
        assert!(config.telegram.chat_id_configured());
        assert_eq!(config.telegram.timeout_ms, 3500);
        assert!(config.template.summary_template_configured());
        assert!(config.template.summary_template_valid());
        assert_eq!(config.provider_attempt_history_capacity, 5);

        let debug = format!("{:?}", config.http_webhook);
        assert!(!debug.contains("hooks.example"));
        assert!(!debug.contains("should-not-appear-in-debug"));
        assert!(!debug.contains("X-Custom-Secret"));
        let debug = format!("{:?}", config.discord_webhook);
        assert!(!debug.contains("discord.example"));
        let debug = format!("{:?}", config.telegram);
        assert!(!debug.contains("api.telegram.example"));
        assert!(!debug.contains("telegram-token-should-not-appear"));
        assert!(!debug.contains("telegram-chat-should-not-appear"));
        let debug = format!("{:?}", config.template);
        assert!(!debug.contains("secret-literal-should-not-appear"));
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

    #[test]
    fn enabled_discord_webhook_requires_a_valid_http_webhook_url() {
        let missing = DiscordWebhookConfig {
            enabled: true,
            ..DiscordWebhookConfig::default()
        };
        assert_eq!(
            missing.status(),
            DiscordWebhookConfigStatus::MissingWebhookUrl
        );

        let invalid = DiscordWebhookConfig::from_env_lookup(|name| match name {
            "NAKO_NOTIFICATION_BRIDGE_DISCORD_WEBHOOK_ENABLED" => Some("true".to_owned()),
            "NAKO_NOTIFICATION_BRIDGE_DISCORD_WEBHOOK_URL" => Some("file:///tmp/hook".to_owned()),
            _ => None,
        });
        assert_eq!(
            invalid.status(),
            DiscordWebhookConfigStatus::InvalidWebhookUrl
        );
        assert!(invalid.webhook_url_configured());
        assert!(!invalid.webhook_url_valid());
    }

    #[test]
    fn enabled_telegram_requires_valid_base_url_bot_token_and_chat_id() {
        let missing_token = TelegramConfig {
            enabled: true,
            chat_id: Some("chat-1".to_owned()),
            ..TelegramConfig::default()
        };
        assert_eq!(
            missing_token.status(),
            TelegramConfigStatus::MissingBotToken
        );

        let missing_chat = TelegramConfig {
            enabled: true,
            bot_token: Some("token-1".to_owned()),
            ..TelegramConfig::default()
        };
        assert_eq!(missing_chat.status(), TelegramConfigStatus::MissingChatId);

        let invalid = TelegramConfig::from_env_lookup(|name| match name {
            "NAKO_NOTIFICATION_BRIDGE_TELEGRAM_ENABLED" => Some("true".to_owned()),
            "NAKO_NOTIFICATION_BRIDGE_TELEGRAM_API_BASE_URL" => {
                Some("file:///tmp/telegram".to_owned())
            }
            "NAKO_NOTIFICATION_BRIDGE_TELEGRAM_BOT_TOKEN" => Some("token-1".to_owned()),
            "NAKO_NOTIFICATION_BRIDGE_TELEGRAM_CHAT_ID" => Some("chat-1".to_owned()),
            _ => None,
        });
        assert_eq!(invalid.status(), TelegramConfigStatus::InvalidApiBaseUrl);
        assert!(invalid.api_base_url_configured());
        assert!(!invalid.api_base_url_valid());
    }

    #[test]
    fn configured_summary_template_reports_validity_without_exposing_template_text() {
        let valid = NotificationTemplateConfig::from_env_lookup(|name| match name {
            "NAKO_NOTIFICATION_BRIDGE_TEMPLATE_SUMMARY" => Some("{{event_kind}}".to_owned()),
            _ => None,
        });
        assert!(valid.summary_template_configured());
        assert!(valid.summary_template_valid());

        let invalid = NotificationTemplateConfig::from_env_lookup(|name| match name {
            "NAKO_NOTIFICATION_BRIDGE_TEMPLATE_SUMMARY" => Some("{{payload.source_id}}".to_owned()),
            _ => None,
        });
        assert!(invalid.summary_template_configured());
        assert!(!invalid.summary_template_valid());
        assert_eq!(invalid.status(), TemplateStatus::Invalid);

        let debug = format!("{invalid:?}");
        assert!(!debug.contains("payload.source_id"));
    }
}
