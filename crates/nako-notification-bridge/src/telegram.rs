use std::time::Duration;

use axum::http::StatusCode;
use nako_addon_protocol::AddonEventRequest;
use reqwest::header::CONTENT_TYPE;

use crate::config::{TelegramConfig, TelegramConfigStatus};

pub const TELEGRAM_PROVIDER_ID: &str = "telegram";

#[derive(Clone, Debug)]
pub struct TelegramClient {
    client: reqwest::Client,
}

impl TelegramClient {
    #[must_use]
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    pub async fn send_library_scanned_event(
        &self,
        config: &TelegramConfig,
        _request: &AddonEventRequest,
        _payload_keys: &[String],
        summary: &str,
    ) -> Result<TelegramSendOutcome, TelegramSendError> {
        match config.status() {
            TelegramConfigStatus::Disabled => {
                return Ok(TelegramSendOutcome::SkippedDisabled);
            }
            TelegramConfigStatus::InvalidApiBaseUrl => {
                return Err(TelegramSendError::configuration("invalid_api_base_url"));
            }
            TelegramConfigStatus::MissingBotToken => {
                return Err(TelegramSendError::configuration("missing_bot_token"));
            }
            TelegramConfigStatus::MissingChatId => {
                return Err(TelegramSendError::configuration("missing_chat_id"));
            }
            TelegramConfigStatus::Configured => {}
        }

        let bot_token = config
            .bot_token
            .as_deref()
            .ok_or_else(|| TelegramSendError::configuration("missing_bot_token"))?;
        let chat_id = config
            .chat_id
            .as_deref()
            .ok_or_else(|| TelegramSendError::configuration("missing_chat_id"))?;
        let endpoint = format!(
            "{}/bot{bot_token}/sendMessage",
            config.api_base_url.trim_end_matches('/')
        );
        let body = serde_json::to_vec(&serde_json::json!({
            "chat_id": chat_id,
            "text": summary,
            "disable_web_page_preview": true
        }))
        .map_err(|_| TelegramSendError::configuration("payload_serialization_failed"))?;

        let response = self
            .client
            .post(endpoint)
            .timeout(Duration::from_millis(config.timeout_ms))
            .header(CONTENT_TYPE, "application/json")
            .body(body)
            .send()
            .await
            .map_err(|_| TelegramSendError::retryable(None))?;
        let http_status = response.status().as_u16();

        if response.status().is_success() {
            Ok(TelegramSendOutcome::Sent { http_status })
        } else if is_retryable_provider_status(http_status) {
            Err(TelegramSendError::retryable(Some(http_status)))
        } else {
            Err(TelegramSendError::non_retryable(http_status))
        }
    }
}

impl Default for TelegramClient {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TelegramSendOutcome {
    SkippedDisabled,
    Sent { http_status: u16 },
}

impl TelegramSendOutcome {
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
    pub fn provider_output(&self) -> serde_json::Value {
        match self {
            Self::SkippedDisabled => serde_json::json!({
                "id": TELEGRAM_PROVIDER_ID,
                "status": "disabled",
                "send_path_enabled": false
            }),
            Self::Sent { http_status } => serde_json::json!({
                "id": TELEGRAM_PROVIDER_ID,
                "status": "sent",
                "send_path_enabled": true,
                "http_status": http_status
            }),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TelegramSendError {
    safe_error_code: &'static str,
    provider_status: &'static str,
    retryable: bool,
    provider_http_status: Option<u16>,
}

impl TelegramSendError {
    fn configuration(provider_status: &'static str) -> TelegramSendError {
        Self {
            safe_error_code: "telegram_configuration_invalid",
            provider_status,
            retryable: false,
            provider_http_status: None,
        }
    }

    fn retryable(provider_http_status: Option<u16>) -> Self {
        Self {
            safe_error_code: "telegram_retryable_failure",
            provider_status: "retryable_failure",
            retryable: true,
            provider_http_status,
        }
    }

    fn non_retryable(provider_http_status: u16) -> Self {
        Self {
            safe_error_code: "telegram_non_retryable_failure",
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
            "provider_id": TELEGRAM_PROVIDER_ID,
            "provider_status": self.provider_status,
            "provider_http_status": self.provider_http_status,
            "retryable": self.retryable
        })
    }
}

const fn is_retryable_provider_status(status: u16) -> bool {
    status == 408 || status == 429 || (status >= 500 && status < 600)
}
