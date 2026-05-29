use std::time::Duration;

use axum::http::StatusCode;
use nako_addon_protocol::AddonEventRequest;
use reqwest::header::{CONTENT_TYPE, HeaderName, HeaderValue};

use crate::config::{HttpWebhookConfig, HttpWebhookConfigStatus};

pub const HTTP_WEBHOOK_PROVIDER_ID: &str = "http_webhook";
pub const HTTP_WEBHOOK_LIBRARY_SCANNED_SCHEMA: &str =
    "nako.official.notification-bridge.http-webhook.library-scanned.v1";

#[derive(Clone, Debug)]
pub struct HttpWebhookClient {
    client: reqwest::Client,
}

impl HttpWebhookClient {
    #[must_use]
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    pub async fn send_library_scanned_event(
        &self,
        config: &HttpWebhookConfig,
        request: &AddonEventRequest,
        payload_keys: &[String],
        summary: &str,
    ) -> Result<HttpWebhookSendOutcome, HttpWebhookSendError> {
        match config.status() {
            HttpWebhookConfigStatus::Disabled => {
                return Ok(HttpWebhookSendOutcome::SkippedDisabled);
            }
            HttpWebhookConfigStatus::MissingTargetUrl => {
                return Err(HttpWebhookSendError::configuration(
                    "missing_target_url",
                    "http_webhook_configuration_invalid",
                ));
            }
            HttpWebhookConfigStatus::InvalidTargetUrl => {
                return Err(HttpWebhookSendError::configuration(
                    "invalid_target_url",
                    "http_webhook_configuration_invalid",
                ));
            }
            HttpWebhookConfigStatus::Configured => {}
        }

        let target_url = config.target_url.as_deref().ok_or_else(|| {
            HttpWebhookSendError::configuration(
                "missing_target_url",
                "http_webhook_configuration_invalid",
            )
        })?;
        let body = serde_json::to_vec(&library_scanned_payload(request, payload_keys, summary))
            .map_err(|_| {
                HttpWebhookSendError::configuration(
                    "payload_serialization_failed",
                    "http_webhook_configuration_invalid",
                )
            })?;
        let mut builder = self
            .client
            .post(target_url)
            .timeout(Duration::from_millis(config.timeout_ms))
            .header(CONTENT_TYPE, "application/json")
            .body(body);

        if let Some(shared_secret) = config.shared_secret.as_deref() {
            let header_name = HeaderName::from_bytes(config.secret_header_name.as_bytes())
                .map_err(|_| {
                    HttpWebhookSendError::configuration(
                        "invalid_secret_header_name",
                        "http_webhook_configuration_invalid",
                    )
                })?;
            let header_value = HeaderValue::from_str(shared_secret).map_err(|_| {
                HttpWebhookSendError::configuration(
                    "invalid_shared_secret_header_value",
                    "http_webhook_configuration_invalid",
                )
            })?;
            builder = builder.header(header_name, header_value);
        }

        let response = builder
            .send()
            .await
            .map_err(|_| HttpWebhookSendError::retryable(None))?;
        let http_status = response.status().as_u16();

        if response.status().is_success() {
            Ok(HttpWebhookSendOutcome::Sent { http_status })
        } else if is_retryable_provider_status(http_status) {
            Err(HttpWebhookSendError::retryable(Some(http_status)))
        } else {
            Err(HttpWebhookSendError::non_retryable(http_status))
        }
    }
}

impl Default for HttpWebhookClient {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HttpWebhookSendOutcome {
    SkippedDisabled,
    Sent { http_status: u16 },
}

impl HttpWebhookSendOutcome {
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
                "id": HTTP_WEBHOOK_PROVIDER_ID,
                "status": "disabled",
                "send_path_enabled": false
            }),
            Self::Sent { http_status } => serde_json::json!({
                "id": HTTP_WEBHOOK_PROVIDER_ID,
                "status": "sent",
                "send_path_enabled": true,
                "http_status": http_status
            }),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HttpWebhookSendError {
    safe_error_code: &'static str,
    provider_status: &'static str,
    retryable: bool,
    provider_http_status: Option<u16>,
}

impl HttpWebhookSendError {
    fn configuration(
        provider_status: &'static str,
        safe_error_code: &'static str,
    ) -> HttpWebhookSendError {
        Self {
            safe_error_code,
            provider_status,
            retryable: false,
            provider_http_status: None,
        }
    }

    fn retryable(provider_http_status: Option<u16>) -> Self {
        Self {
            safe_error_code: "http_webhook_retryable_failure",
            provider_status: "retryable_failure",
            retryable: true,
            provider_http_status,
        }
    }

    fn non_retryable(provider_http_status: u16) -> Self {
        Self {
            safe_error_code: "http_webhook_non_retryable_failure",
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
            "provider_id": HTTP_WEBHOOK_PROVIDER_ID,
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
    serde_json::json!({
        "schema": HTTP_WEBHOOK_LIBRARY_SCANNED_SCHEMA,
        "summary": summary,
        "event": {
            "event_id": request.event_id,
            "event_kind": request.event_kind,
            "subject_kind": request.subject_kind,
            "subject_id": request.subject_id,
            "occurred_at": request.occurred_at,
            "attempt": request.attempt
        },
        "payload_keys": payload_keys
    })
}

const fn is_retryable_provider_status(status: u16) -> bool {
    status == 408 || status == 429 || (status >= 500 && status < 600)
}
