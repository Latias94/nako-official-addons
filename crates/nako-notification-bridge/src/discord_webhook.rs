use std::time::Duration;

use axum::http::StatusCode;
use nako_addon_protocol::AddonEventRequest;
use reqwest::header::CONTENT_TYPE;

use crate::config::{DiscordWebhookConfig, DiscordWebhookConfigStatus};

pub const DISCORD_WEBHOOK_PROVIDER_ID: &str = "discord_webhook";
pub const DISCORD_WEBHOOK_LIBRARY_SCANNED_SCHEMA: &str =
    "nako.official.notification-bridge.discord-webhook.library-scanned.v1";

#[derive(Clone, Debug)]
pub struct DiscordWebhookClient {
    client: reqwest::Client,
}

impl DiscordWebhookClient {
    #[must_use]
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    pub async fn send_library_scanned_event(
        &self,
        config: &DiscordWebhookConfig,
        request: &AddonEventRequest,
        payload_keys: &[String],
        summary: &str,
    ) -> Result<DiscordWebhookSendOutcome, DiscordWebhookSendError> {
        match config.status() {
            DiscordWebhookConfigStatus::Disabled => {
                return Ok(DiscordWebhookSendOutcome::SkippedDisabled);
            }
            DiscordWebhookConfigStatus::MissingWebhookUrl => {
                return Err(DiscordWebhookSendError::configuration(
                    "missing_webhook_url",
                    "discord_webhook_configuration_invalid",
                ));
            }
            DiscordWebhookConfigStatus::InvalidWebhookUrl => {
                return Err(DiscordWebhookSendError::configuration(
                    "invalid_webhook_url",
                    "discord_webhook_configuration_invalid",
                ));
            }
            DiscordWebhookConfigStatus::Configured => {}
        }

        let webhook_url = config.webhook_url.as_deref().ok_or_else(|| {
            DiscordWebhookSendError::configuration(
                "missing_webhook_url",
                "discord_webhook_configuration_invalid",
            )
        })?;
        let body = serde_json::to_vec(&library_scanned_payload(request, payload_keys, summary))
            .map_err(|_| {
                DiscordWebhookSendError::configuration(
                    "payload_serialization_failed",
                    "discord_webhook_configuration_invalid",
                )
            })?;
        let response = self
            .client
            .post(webhook_url)
            .timeout(Duration::from_millis(config.timeout_ms))
            .header(CONTENT_TYPE, "application/json")
            .body(body)
            .send()
            .await
            .map_err(|_| DiscordWebhookSendError::retryable(None))?;
        let http_status = response.status().as_u16();

        if response.status().is_success() {
            Ok(DiscordWebhookSendOutcome::Sent { http_status })
        } else if is_retryable_provider_status(http_status) {
            Err(DiscordWebhookSendError::retryable(Some(http_status)))
        } else {
            Err(DiscordWebhookSendError::non_retryable(http_status))
        }
    }
}

impl Default for DiscordWebhookClient {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DiscordWebhookSendOutcome {
    SkippedDisabled,
    Sent { http_status: u16 },
}

impl DiscordWebhookSendOutcome {
    #[must_use]
    pub const fn provider_status(&self) -> &'static str {
        match self {
            Self::SkippedDisabled => "disabled",
            Self::Sent { .. } => "sent",
        }
    }

    #[must_use]
    pub const fn provider_http_status(&self) -> Option<u16> {
        match self {
            Self::SkippedDisabled => None,
            Self::Sent { http_status } => Some(*http_status),
        }
    }

    #[must_use]
    pub const fn mode(&self) -> &'static str {
        match self {
            Self::SkippedDisabled => "ack_only",
            Self::Sent { .. } => "provider_send",
        }
    }

    #[must_use]
    pub fn provider_output(&self) -> serde_json::Value {
        match self {
            Self::SkippedDisabled => serde_json::json!({
                "id": DISCORD_WEBHOOK_PROVIDER_ID,
                "status": "disabled",
                "send_path_enabled": false
            }),
            Self::Sent { http_status } => serde_json::json!({
                "id": DISCORD_WEBHOOK_PROVIDER_ID,
                "status": "sent",
                "send_path_enabled": true,
                "http_status": http_status
            }),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiscordWebhookSendError {
    safe_error_code: &'static str,
    provider_status: &'static str,
    retryable: bool,
    provider_http_status: Option<u16>,
}

impl DiscordWebhookSendError {
    fn configuration(
        provider_status: &'static str,
        safe_error_code: &'static str,
    ) -> DiscordWebhookSendError {
        Self {
            safe_error_code,
            provider_status,
            retryable: false,
            provider_http_status: None,
        }
    }

    fn retryable(provider_http_status: Option<u16>) -> Self {
        Self {
            safe_error_code: "discord_webhook_retryable_failure",
            provider_status: "retryable_failure",
            retryable: true,
            provider_http_status,
        }
    }

    fn non_retryable(provider_http_status: u16) -> Self {
        Self {
            safe_error_code: "discord_webhook_non_retryable_failure",
            provider_status: "non_retryable_failure",
            retryable: false,
            provider_http_status: Some(provider_http_status),
        }
    }

    #[must_use]
    pub const fn provider_status(&self) -> &'static str {
        self.provider_status
    }

    #[must_use]
    pub const fn is_retryable(&self) -> bool {
        self.retryable
    }

    #[must_use]
    pub const fn provider_http_status(&self) -> Option<u16> {
        self.provider_http_status
    }

    #[must_use]
    pub const fn status_code(&self) -> StatusCode {
        if self.retryable {
            StatusCode::SERVICE_UNAVAILABLE
        } else if self.provider_http_status.is_none() {
            StatusCode::BAD_REQUEST
        } else {
            StatusCode::FAILED_DEPENDENCY
        }
    }

    #[must_use]
    pub fn safe_body(&self) -> serde_json::Value {
        serde_json::json!({
            "safe_error_code": self.safe_error_code,
            "provider_id": DISCORD_WEBHOOK_PROVIDER_ID,
            "provider_status": self.provider_status,
            "provider_http_status": self.provider_http_status,
            "retryable": self.retryable
        })
    }
}

fn library_scanned_payload(
    request: &AddonEventRequest,
    payload_keys: &[String],
    summary: &str,
) -> serde_json::Value {
    let payload_keys = if payload_keys.is_empty() {
        "none".to_owned()
    } else {
        payload_keys.join(", ")
    };

    serde_json::json!({
        "schema": DISCORD_WEBHOOK_LIBRARY_SCANNED_SCHEMA,
        "content": summary,
        "embeds": [
            {
                "title": "Nako library scanned",
                "fields": [
                    {
                        "name": "Event",
                        "value": request.event_kind,
                        "inline": true
                    },
                    {
                        "name": "Subject",
                        "value": format!("{} {}", request.subject_kind, request.subject_id),
                        "inline": true
                    },
                    {
                        "name": "Attempt",
                        "value": request.attempt.to_string(),
                        "inline": true
                    },
                    {
                        "name": "Occurred at",
                        "value": request.occurred_at,
                        "inline": false
                    },
                    {
                        "name": "Payload keys",
                        "value": payload_keys,
                        "inline": false
                    }
                ]
            }
        ]
    })
}

const fn is_retryable_provider_status(status: u16) -> bool {
    status == 408 || status == 429 || (status >= 500 && status < 600)
}
