use nako_addon_protocol::AddonEventRequest;

use crate::{
    Config,
    attempt_history::{ProviderAttemptHistory, ProviderAttemptRecord},
    config::{DiscordWebhookConfig, HttpWebhookConfig},
    discord_webhook::{
        DISCORD_WEBHOOK_PROVIDER_ID, DiscordWebhookSendError, DiscordWebhookSendOutcome,
    },
    http_webhook::{HTTP_WEBHOOK_PROVIDER_ID, HttpWebhookSendError, HttpWebhookSendOutcome},
    template::DEFAULT_SUMMARY_TEMPLATE,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NotificationConfigurationStatus {
    AckOnly,
    ProviderSendReady,
    ProviderConfigurationInvalid,
    MultipleProviderSendPathsConfigured,
    TemplateInvalid,
}

impl NotificationConfigurationStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::AckOnly => "ack_only",
            Self::ProviderSendReady => "provider_send_ready",
            Self::ProviderConfigurationInvalid => "provider_configuration_invalid",
            Self::MultipleProviderSendPathsConfigured => "multiple_provider_send_paths_configured",
            Self::TemplateInvalid => "template_invalid",
        }
    }

    #[must_use]
    pub const fn is_degraded(self) -> bool {
        matches!(
            self,
            Self::ProviderConfigurationInvalid
                | Self::MultipleProviderSendPathsConfigured
                | Self::TemplateInvalid
        )
    }
}

#[derive(Clone, Copy, Debug)]
pub struct NotificationProviderRegistry<'a> {
    config: &'a Config,
}

impl<'a> NotificationProviderRegistry<'a> {
    #[must_use]
    pub const fn new(config: &'a Config) -> Self {
        Self { config }
    }

    #[must_use]
    pub fn diagnostics(&self) -> NotificationProviderDiagnostics {
        NotificationProviderDiagnostics {
            http_webhook: HttpWebhookProviderDiagnostics::from_config(&self.config.http_webhook),
            discord_webhook: DiscordWebhookProviderDiagnostics::from_config(
                &self.config.discord_webhook,
            ),
        }
    }

    #[must_use]
    pub fn send_path_count(&self) -> usize {
        (self.config.http_webhook.send_path_enabled() as usize)
            + (self.config.discord_webhook.send_path_enabled() as usize)
    }

    #[must_use]
    pub fn send_path_configured(&self) -> bool {
        self.send_path_count() > 0
    }

    #[must_use]
    pub fn any_provider_enabled(&self) -> bool {
        self.config.http_webhook.enabled || self.config.discord_webhook.enabled
    }

    #[must_use]
    pub fn summary_template(&self) -> &str {
        if self.any_provider_enabled() {
            self.config.template.summary_template.as_str()
        } else {
            DEFAULT_SUMMARY_TEMPLATE
        }
    }

    #[must_use]
    pub fn multiple_send_paths_error(&self) -> Option<serde_json::Value> {
        (self.send_path_count() > 1).then(|| {
            serde_json::json!({
                "safe_error_code": "multiple_notification_provider_send_paths_configured",
                "retryable": false
            })
        })
    }

    #[must_use]
    pub fn configuration_status(&self) -> NotificationConfigurationStatus {
        let send_path_count = self.send_path_count();
        if send_path_count > 1 {
            return NotificationConfigurationStatus::MultipleProviderSendPathsConfigured;
        }

        let provider_configuration_invalid = (self.config.http_webhook.enabled
            && !self.config.http_webhook.send_path_enabled())
            || (self.config.discord_webhook.enabled
                && !self.config.discord_webhook.send_path_enabled());
        if provider_configuration_invalid {
            return NotificationConfigurationStatus::ProviderConfigurationInvalid;
        }

        if self.any_provider_enabled() && !self.config.template.summary_template_valid() {
            return NotificationConfigurationStatus::TemplateInvalid;
        }

        if send_path_count == 1 {
            NotificationConfigurationStatus::ProviderSendReady
        } else {
            NotificationConfigurationStatus::AckOnly
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NotificationProviderDiagnostics {
    pub http_webhook: HttpWebhookProviderDiagnostics,
    pub discord_webhook: DiscordWebhookProviderDiagnostics,
}

impl NotificationProviderDiagnostics {
    #[must_use]
    pub fn to_json_array(self) -> Vec<serde_json::Value> {
        vec![self.http_webhook.to_json(), self.discord_webhook.to_json()]
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HttpWebhookProviderDiagnostics {
    pub enabled: bool,
    pub status: &'static str,
    pub target_url_configured: bool,
    pub target_url_valid: bool,
    pub custom_secret_header_name_configured: bool,
    pub shared_secret_configured: bool,
    pub timeout_ms: u64,
    pub send_path_enabled: bool,
}

impl HttpWebhookProviderDiagnostics {
    #[must_use]
    pub fn from_config(config: &HttpWebhookConfig) -> Self {
        Self {
            enabled: config.enabled,
            status: config.status().as_str(),
            target_url_configured: config.target_url_configured(),
            target_url_valid: config.target_url_valid(),
            custom_secret_header_name_configured: config.custom_secret_header_name_configured(),
            shared_secret_configured: config.shared_secret_configured(),
            timeout_ms: config.timeout_ms,
            send_path_enabled: config.send_path_enabled(),
        }
    }

    #[must_use]
    pub fn to_json(self) -> serde_json::Value {
        serde_json::json!({
            "id": HTTP_WEBHOOK_PROVIDER_ID,
            "enabled": self.enabled,
            "status": self.status,
            "target_url_configured": self.target_url_configured,
            "target_url_valid": self.target_url_valid,
            "custom_secret_header_name_configured": self.custom_secret_header_name_configured,
            "shared_secret_configured": self.shared_secret_configured,
            "timeout_ms": self.timeout_ms,
            "send_path_enabled": self.send_path_enabled
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DiscordWebhookProviderDiagnostics {
    pub enabled: bool,
    pub status: &'static str,
    pub webhook_url_configured: bool,
    pub webhook_url_valid: bool,
    pub timeout_ms: u64,
    pub send_path_enabled: bool,
}

impl DiscordWebhookProviderDiagnostics {
    #[must_use]
    pub fn from_config(config: &DiscordWebhookConfig) -> Self {
        Self {
            enabled: config.enabled,
            status: config.status().as_str(),
            webhook_url_configured: config.webhook_url_configured(),
            webhook_url_valid: config.webhook_url_valid(),
            timeout_ms: config.timeout_ms,
            send_path_enabled: config.send_path_enabled(),
        }
    }

    #[must_use]
    pub fn to_json(self) -> serde_json::Value {
        serde_json::json!({
            "id": DISCORD_WEBHOOK_PROVIDER_ID,
            "enabled": self.enabled,
            "status": self.status,
            "webhook_url_configured": self.webhook_url_configured,
            "webhook_url_valid": self.webhook_url_valid,
            "timeout_ms": self.timeout_ms,
            "send_path_enabled": self.send_path_enabled
        })
    }
}

pub trait ProviderAttemptOutcomeFacts {
    fn provider_id(&self) -> &'static str;
    fn provider_status(&self) -> &'static str;
    fn provider_http_status(&self) -> Option<u16>;
    fn should_record_attempt(&self) -> bool;
}

pub trait ProviderAttemptErrorFacts {
    fn provider_id(&self) -> &'static str;
    fn provider_status(&self) -> &'static str;
    fn is_retryable(&self) -> bool;
    fn provider_http_status(&self) -> Option<u16>;
}

impl ProviderAttemptOutcomeFacts for HttpWebhookSendOutcome {
    fn provider_id(&self) -> &'static str {
        HTTP_WEBHOOK_PROVIDER_ID
    }

    fn provider_status(&self) -> &'static str {
        self.provider_status()
    }

    fn provider_http_status(&self) -> Option<u16> {
        self.provider_http_status()
    }

    fn should_record_attempt(&self) -> bool {
        !matches!(self, Self::SkippedDisabled)
    }
}

impl ProviderAttemptErrorFacts for HttpWebhookSendError {
    fn provider_id(&self) -> &'static str {
        HTTP_WEBHOOK_PROVIDER_ID
    }

    fn provider_status(&self) -> &'static str {
        self.provider_status()
    }

    fn is_retryable(&self) -> bool {
        self.is_retryable()
    }

    fn provider_http_status(&self) -> Option<u16> {
        self.provider_http_status()
    }
}

impl ProviderAttemptOutcomeFacts for DiscordWebhookSendOutcome {
    fn provider_id(&self) -> &'static str {
        DISCORD_WEBHOOK_PROVIDER_ID
    }

    fn provider_status(&self) -> &'static str {
        self.provider_status()
    }

    fn provider_http_status(&self) -> Option<u16> {
        self.provider_http_status()
    }

    fn should_record_attempt(&self) -> bool {
        !matches!(self, Self::SkippedDisabled)
    }
}

impl ProviderAttemptErrorFacts for DiscordWebhookSendError {
    fn provider_id(&self) -> &'static str {
        DISCORD_WEBHOOK_PROVIDER_ID
    }

    fn provider_status(&self) -> &'static str {
        self.provider_status()
    }

    fn is_retryable(&self) -> bool {
        self.is_retryable()
    }

    fn provider_http_status(&self) -> Option<u16> {
        self.provider_http_status()
    }
}

pub fn record_provider_outcome(
    history: &ProviderAttemptHistory,
    request: &AddonEventRequest,
    outcome: &impl ProviderAttemptOutcomeFacts,
) {
    if !outcome.should_record_attempt() {
        return;
    }

    history.record(ProviderAttemptRecord::new(
        outcome.provider_id(),
        request,
        outcome.provider_status(),
        false,
        outcome.provider_http_status(),
    ));
}

pub fn record_provider_error(
    history: &ProviderAttemptHistory,
    request: &AddonEventRequest,
    error: &impl ProviderAttemptErrorFacts,
) {
    history.record(ProviderAttemptRecord::new(
        error.provider_id(),
        request,
        error.provider_status(),
        error.is_retryable(),
        error.provider_http_status(),
    ));
}

#[must_use]
pub fn select_primary_provider_output(provider_outputs: &[serde_json::Value]) -> serde_json::Value {
    provider_outputs
        .iter()
        .find(|provider| provider["send_path_enabled"] == true)
        .cloned()
        .or_else(|| provider_outputs.first().cloned())
        .unwrap_or_else(|| {
            serde_json::json!({
                "id": "none",
                "status": "disabled",
                "send_path_enabled": false
            })
        })
}

#[cfg(test)]
mod tests {
    use nako_addon_protocol::{ADDON_PROTOCOL_VERSION, AddonEventRequest};

    use super::*;

    fn request() -> AddonEventRequest {
        AddonEventRequest {
            protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
            addon_id: "nako.official.notification-bridge".to_owned(),
            subscription_id: "library-scanned-notification".to_owned(),
            event_id: "event-1".to_owned(),
            event_kind: "library.scanned".to_owned(),
            subject_kind: "library".to_owned(),
            subject_id: "library-1".to_owned(),
            occurred_at: "2026-05-25T00:00:00.000Z".to_owned(),
            attempt: 1,
            payload: serde_json::json!({
                "secret": "nako_at_should_not_echo",
                "source_id": "source-1"
            }),
        }
    }

    #[test]
    fn registry_reports_configuration_status_from_provider_facts() {
        let config = Config::default();
        let registry = NotificationProviderRegistry::new(&config);
        assert_eq!(
            registry.configuration_status(),
            NotificationConfigurationStatus::AckOnly
        );
        assert_eq!(registry.send_path_count(), 0);

        let config = Config::from_env_lookup(|name| match name {
            "NAKO_NOTIFICATION_BRIDGE_HTTP_WEBHOOK_ENABLED" => Some("true".to_owned()),
            "NAKO_NOTIFICATION_BRIDGE_HTTP_WEBHOOK_URL" => {
                Some("https://hooks.example/internal/path".to_owned())
            }
            _ => None,
        });
        let registry = NotificationProviderRegistry::new(&config);
        assert_eq!(
            registry.configuration_status(),
            NotificationConfigurationStatus::ProviderSendReady
        );
        assert_eq!(registry.send_path_count(), 1);

        let config = Config::from_env_lookup(|name| match name {
            "NAKO_NOTIFICATION_BRIDGE_HTTP_WEBHOOK_ENABLED" => Some("true".to_owned()),
            "NAKO_NOTIFICATION_BRIDGE_HTTP_WEBHOOK_URL" => Some("file:///tmp/hook".to_owned()),
            _ => None,
        });
        let registry = NotificationProviderRegistry::new(&config);
        assert_eq!(
            registry.configuration_status(),
            NotificationConfigurationStatus::ProviderConfigurationInvalid
        );
        assert_eq!(registry.send_path_count(), 0);

        let config = Config::from_env_lookup(|name| match name {
            "NAKO_NOTIFICATION_BRIDGE_DISCORD_WEBHOOK_ENABLED" => Some("true".to_owned()),
            "NAKO_NOTIFICATION_BRIDGE_DISCORD_WEBHOOK_URL" => {
                Some("https://discord.example/api/webhooks/secret".to_owned())
            }
            "NAKO_NOTIFICATION_BRIDGE_TEMPLATE_SUMMARY" => Some("{{payload.secret}}".to_owned()),
            _ => None,
        });
        let registry = NotificationProviderRegistry::new(&config);
        assert_eq!(
            registry.configuration_status(),
            NotificationConfigurationStatus::TemplateInvalid
        );
        assert_eq!(registry.send_path_count(), 1);
    }

    #[test]
    fn registry_reports_multiple_send_paths_without_leaking_provider_values() {
        let config = Config::from_env_lookup(|name| match name {
            "NAKO_NOTIFICATION_BRIDGE_HTTP_WEBHOOK_ENABLED" => Some("true".to_owned()),
            "NAKO_NOTIFICATION_BRIDGE_HTTP_WEBHOOK_URL" => {
                Some("https://hooks.example/internal/path".to_owned())
            }
            "NAKO_NOTIFICATION_BRIDGE_HTTP_WEBHOOK_SHARED_SECRET" => {
                Some("webhook-secret-should-not-appear".to_owned())
            }
            "NAKO_NOTIFICATION_BRIDGE_DISCORD_WEBHOOK_ENABLED" => Some("true".to_owned()),
            "NAKO_NOTIFICATION_BRIDGE_DISCORD_WEBHOOK_URL" => {
                Some("https://discord.example/api/webhooks/secret".to_owned())
            }
            _ => None,
        });
        let registry = NotificationProviderRegistry::new(&config);

        assert_eq!(registry.send_path_count(), 2);
        assert_eq!(
            registry.configuration_status(),
            NotificationConfigurationStatus::MultipleProviderSendPathsConfigured
        );
        assert_eq!(
            registry.multiple_send_paths_error().unwrap()["safe_error_code"],
            "multiple_notification_provider_send_paths_configured"
        );

        let diagnostics = serde_json::to_string(&registry.diagnostics().to_json_array()).unwrap();
        assert!(!diagnostics.contains("hooks.example"));
        assert!(!diagnostics.contains("discord.example"));
        assert!(!diagnostics.contains("api/webhooks"));
        assert!(!diagnostics.contains("webhook-secret-should-not-appear"));
    }

    #[test]
    fn registry_records_attempt_history_only_for_real_provider_attempts() {
        let history = ProviderAttemptHistory::new(4);

        record_provider_outcome(
            &history,
            &request(),
            &HttpWebhookSendOutcome::SkippedDisabled,
        );
        assert!(history.snapshot().is_empty());

        record_provider_outcome(
            &history,
            &request(),
            &DiscordWebhookSendOutcome::Sent { http_status: 202 },
        );

        let records = history.snapshot();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].provider_id, DISCORD_WEBHOOK_PROVIDER_ID);
        assert_eq!(records[0].provider_status, "sent");
        assert_eq!(records[0].provider_http_status, Some(202));
    }

    #[test]
    fn registry_selects_primary_provider_output_from_enabled_send_path() {
        let selected = select_primary_provider_output(&[
            serde_json::json!({
                "id": "http_webhook",
                "status": "disabled",
                "send_path_enabled": false
            }),
            serde_json::json!({
                "id": "discord_webhook",
                "status": "sent",
                "send_path_enabled": true,
                "http_status": 202
            }),
        ]);

        assert_eq!(selected["id"], "discord_webhook");
        assert_eq!(selected["status"], "sent");
        assert_eq!(selected["http_status"], 202);
    }
}
