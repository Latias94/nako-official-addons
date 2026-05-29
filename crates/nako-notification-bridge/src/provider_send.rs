use axum::http::StatusCode;
use nako_addon_protocol::AddonEventRequest;

use crate::{
    Config,
    attempt_history::ProviderAttemptHistory,
    discord_webhook::{DiscordWebhookClient, DiscordWebhookSendOutcome},
    http_webhook::{HttpWebhookClient, HttpWebhookSendOutcome},
    provider_registry::{record_provider_error, record_provider_outcome},
    telegram::{TelegramClient, TelegramSendOutcome},
};

#[derive(Clone)]
pub struct NotificationProviderClients {
    http_webhook: HttpWebhookClient,
    discord_webhook: DiscordWebhookClient,
    telegram: TelegramClient,
}

impl NotificationProviderClients {
    #[must_use]
    pub fn new() -> Self {
        Self {
            http_webhook: HttpWebhookClient::new(),
            discord_webhook: DiscordWebhookClient::new(),
            telegram: TelegramClient::new(),
        }
    }
}

impl Default for NotificationProviderClients {
    fn default() -> Self {
        Self::new()
    }
}

pub async fn send_library_scanned_event_to_providers(
    config: &Config,
    clients: &NotificationProviderClients,
    provider_attempt_history: &ProviderAttemptHistory,
    request: &AddonEventRequest,
    payload_keys: &[String],
    summary: &str,
) -> Result<ProviderSendOutcomes, ProviderSendFailure> {
    let http_webhook = match clients
        .http_webhook
        .send_library_scanned_event(&config.http_webhook, request, payload_keys, summary)
        .await
    {
        Ok(outcome) => {
            record_provider_outcome(provider_attempt_history, request, &outcome);
            outcome
        }
        Err(error) => {
            record_provider_error(provider_attempt_history, request, &error);
            return Err(ProviderSendFailure::new(
                error.status_code(),
                error.safe_body(),
            ));
        }
    };
    let discord_webhook = match clients
        .discord_webhook
        .send_library_scanned_event(&config.discord_webhook, request, payload_keys, summary)
        .await
    {
        Ok(outcome) => {
            record_provider_outcome(provider_attempt_history, request, &outcome);
            outcome
        }
        Err(error) => {
            record_provider_error(provider_attempt_history, request, &error);
            return Err(ProviderSendFailure::new(
                error.status_code(),
                error.safe_body(),
            ));
        }
    };
    let telegram = match clients
        .telegram
        .send_library_scanned_event(&config.telegram, request, payload_keys, summary)
        .await
    {
        Ok(outcome) => {
            record_provider_outcome(provider_attempt_history, request, &outcome);
            outcome
        }
        Err(error) => {
            record_provider_error(provider_attempt_history, request, &error);
            return Err(ProviderSendFailure::new(
                error.status_code(),
                error.safe_body(),
            ));
        }
    };

    Ok(ProviderSendOutcomes {
        http_webhook,
        discord_webhook,
        telegram,
    })
}

#[derive(Debug)]
pub struct ProviderSendFailure {
    status_code: StatusCode,
    safe_body: serde_json::Value,
}

impl ProviderSendFailure {
    #[must_use]
    pub const fn new(status_code: StatusCode, safe_body: serde_json::Value) -> Self {
        Self {
            status_code,
            safe_body,
        }
    }

    #[must_use]
    pub const fn status_code(&self) -> StatusCode {
        self.status_code
    }

    #[must_use]
    pub fn into_safe_body(self) -> serde_json::Value {
        self.safe_body
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderSendOutcomes {
    http_webhook: HttpWebhookSendOutcome,
    discord_webhook: DiscordWebhookSendOutcome,
    telegram: TelegramSendOutcome,
}

impl ProviderSendOutcomes {
    #[must_use]
    pub fn provider_outputs(&self) -> Vec<serde_json::Value> {
        vec![
            self.http_webhook.provider_output(),
            self.discord_webhook.provider_output(),
            self.telegram.provider_output(),
        ]
    }

    #[must_use]
    pub const fn mode(&self) -> &'static str {
        if matches!(self.http_webhook, HttpWebhookSendOutcome::Sent { .. })
            || matches!(self.discord_webhook, DiscordWebhookSendOutcome::Sent { .. })
            || matches!(self.telegram, TelegramSendOutcome::Sent { .. })
        {
            "provider_send"
        } else {
            "ack_only"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_send_outcomes_report_mode_and_outputs() {
        let outcomes = ProviderSendOutcomes {
            http_webhook: HttpWebhookSendOutcome::SkippedDisabled,
            discord_webhook: DiscordWebhookSendOutcome::Sent { http_status: 202 },
            telegram: TelegramSendOutcome::SkippedDisabled,
        };

        assert_eq!(outcomes.mode(), "provider_send");
        let outputs = outcomes.provider_outputs();
        assert_eq!(outputs.len(), 3);
        assert_eq!(outputs[1]["id"], "discord_webhook");
        assert_eq!(outputs[1]["status"], "sent");
        assert_eq!(outputs[1]["http_status"], 202);
    }
}
