use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::Html,
    routing::{get, post},
};
use nako_addon_protocol::{
    ADDON_PROTOCOL_VERSION, AddonEventRequest, AddonEventResponse, AddonHealthCheckRequest,
    AddonHealthCheckResponse, AddonHealthManifestFacts, AddonHealthStatus,
};
use tower_http::trace::TraceLayer;

use crate::{
    Config,
    attempt_history::{ProviderAttemptHistory, ProviderAttemptRecord},
    discord_webhook::{
        DISCORD_WEBHOOK_PROVIDER_ID, DiscordWebhookClient, DiscordWebhookSendError,
        DiscordWebhookSendOutcome,
    },
    http_webhook::{
        HTTP_WEBHOOK_PROVIDER_ID, HttpWebhookClient, HttpWebhookSendError, HttpWebhookSendOutcome,
    },
    manifest::{
        ADDON_ID, ADDON_VERSION, DIAGNOSTICS_PATH, LIBRARY_SCANNED_EVENT_KIND,
        LIBRARY_SCANNED_EVENT_PATH, LIBRARY_SCANNED_EVENT_SUBSCRIPTION_ID, addon_manifest,
    },
    template::{DEFAULT_SUMMARY_TEMPLATE, TemplateContext, render_template},
};

#[derive(Clone)]
pub struct AppState {
    config: Config,
    http_webhook: HttpWebhookClient,
    discord_webhook: DiscordWebhookClient,
    provider_attempt_history: ProviderAttemptHistory,
}

pub fn router(config: Config) -> Router {
    let provider_attempt_history =
        ProviderAttemptHistory::new(config.provider_attempt_history_capacity);

    Router::new()
        .route("/manifest.json", get(manifest))
        .route("/health", post(health))
        .route(LIBRARY_SCANNED_EVENT_PATH, post(library_scanned_event))
        .route(DIAGNOSTICS_PATH, get(diagnostics))
        .layer(TraceLayer::new_for_http())
        .with_state(AppState {
            config,
            http_webhook: HttpWebhookClient::new(),
            discord_webhook: DiscordWebhookClient::new(),
            provider_attempt_history,
        })
}

async fn manifest(State(state): State<AppState>) -> Json<nako_addon_protocol::AddonManifest> {
    Json(addon_manifest(&state.config))
}

async fn health(
    State(state): State<AppState>,
    Json(request): Json<AddonHealthCheckRequest>,
) -> Json<AddonHealthCheckResponse> {
    let configuration_status = notification_configuration_status(&state.config);
    let expected_status = if request.manifest_id == ADDON_ID && !configuration_status.is_degraded()
    {
        AddonHealthStatus::Ok
    } else {
        AddonHealthStatus::Degraded
    };
    let http_webhook = &state.config.http_webhook;
    let discord_webhook = &state.config.discord_webhook;
    let template = &state.config.template;

    Json(AddonHealthCheckResponse {
        protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
        manifest_id: request.manifest_id,
        status: expected_status,
        checked_at: "2026-05-25T00:00:00.000Z".to_owned(),
        manifest: AddonHealthManifestFacts {
            addon_version: ADDON_VERSION.to_owned(),
            resource_count: crate::manifest::container_manifest().resources.len(),
        },
        diagnostics: serde_json::json!({
            "safe_note": "notification bridge sidecar is reachable",
            "mode": "ack_only",
            "provider_fan_out": http_webhook.send_path_enabled() || discord_webhook.send_path_enabled(),
            "provider_send_path_count": provider_send_path_count(&state.config),
            "configuration_status": configuration_status.as_str(),
            "template": {
                "summary_template_configured": template.summary_template_configured(),
                "summary_template_valid": template.summary_template_valid(),
                "status": template.status().as_str()
            },
            "provider_attempt_history": {
                "capacity": state.provider_attempt_history.capacity(),
                "recent": state.provider_attempt_history.snapshot()
            },
            "providers": [
                {
                    "id": "http_webhook",
                    "enabled": http_webhook.enabled,
                    "status": http_webhook.status().as_str(),
                    "target_url_configured": http_webhook.target_url_configured(),
                    "target_url_valid": http_webhook.target_url_valid(),
                    "custom_secret_header_name_configured": http_webhook.custom_secret_header_name_configured(),
                    "shared_secret_configured": http_webhook.shared_secret_configured(),
                    "timeout_ms": http_webhook.timeout_ms,
                    "send_path_enabled": http_webhook.send_path_enabled()
                },
                {
                    "id": "discord_webhook",
                    "enabled": discord_webhook.enabled,
                    "status": discord_webhook.status().as_str(),
                    "webhook_url_configured": discord_webhook.webhook_url_configured(),
                    "webhook_url_valid": discord_webhook.webhook_url_valid(),
                    "timeout_ms": discord_webhook.timeout_ms,
                    "send_path_enabled": discord_webhook.send_path_enabled()
                }
            ]
        }),
    })
}

async fn library_scanned_event(
    State(state): State<AppState>,
    Json(request): Json<AddonEventRequest>,
) -> Result<Json<AddonEventResponse>, (StatusCode, Json<serde_json::Value>)> {
    if request.protocol_version != ADDON_PROTOCOL_VERSION
        || request.addon_id != ADDON_ID
        || request.subscription_id != LIBRARY_SCANNED_EVENT_SUBSCRIPTION_ID
        || request.event_kind != LIBRARY_SCANNED_EVENT_KIND
    {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "safe_error_code": "invalid_event_envelope"
            })),
        ));
    }

    let payload_keys = request
        .payload
        .as_object()
        .map(|object| {
            let mut keys = object.keys().cloned().collect::<Vec<_>>();
            keys.sort();
            keys
        })
        .unwrap_or_default();
    if state.config.http_webhook.send_path_enabled()
        && state.config.discord_webhook.send_path_enabled()
    {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "safe_error_code": "multiple_notification_provider_send_paths_configured",
                "retryable": false
            })),
        ));
    }
    let summary_template =
        if state.config.http_webhook.enabled || state.config.discord_webhook.enabled {
            state.config.template.summary_template.as_str()
        } else {
            DEFAULT_SUMMARY_TEMPLATE
        };
    let summary = render_template(
        summary_template,
        &TemplateContext {
            request: &request,
            payload_keys: &payload_keys,
        },
    )
    .map_err(invalid_template_error)?;

    let provider_outcome = match state
        .http_webhook
        .send_library_scanned_event(
            &state.config.http_webhook,
            &request,
            &payload_keys,
            &summary,
        )
        .await
    {
        Ok(outcome) => {
            record_http_webhook_outcome(&state.provider_attempt_history, &request, &outcome);
            outcome
        }
        Err(error) => {
            record_http_webhook_error(&state.provider_attempt_history, &request, &error);
            return Err((error.status_code(), Json(error.safe_body())));
        }
    };
    let discord_provider_outcome = match state
        .discord_webhook
        .send_library_scanned_event(
            &state.config.discord_webhook,
            &request,
            &payload_keys,
            &summary,
        )
        .await
    {
        Ok(outcome) => {
            record_discord_webhook_outcome(&state.provider_attempt_history, &request, &outcome);
            outcome
        }
        Err(error) => {
            record_discord_webhook_error(&state.provider_attempt_history, &request, &error);
            return Err((error.status_code(), Json(error.safe_body())));
        }
    };
    let provider_outputs = vec![
        provider_outcome.provider_output(),
        discord_provider_outcome.provider_output(),
    ];
    let primary_provider_output = provider_outputs
        .iter()
        .find(|provider| provider["send_path_enabled"] == true)
        .cloned()
        .unwrap_or_else(|| provider_outputs[0].clone());
    let mode = if provider_outcome.mode() == "provider_send"
        || discord_provider_outcome.mode() == "provider_send"
    {
        "provider_send"
    } else {
        "ack_only"
    };

    Ok(Json(AddonEventResponse {
        protocol_version: request.protocol_version,
        addon_id: request.addon_id,
        subscription_id: request.subscription_id,
        event_id: request.event_id,
        output: serde_json::json!({
            "schema": "nako.official.notification-bridge.library-scanned.event.v1",
            "accepted": true,
            "mode": mode,
            "attempt": request.attempt,
            "subject_kind": request.subject_kind,
            "subject_id": request.subject_id,
            "payload_keys": payload_keys,
            "provider": primary_provider_output,
            "providers": provider_outputs
        }),
    }))
}

async fn diagnostics(State(state): State<AppState>) -> Html<String> {
    let http_webhook = &state.config.http_webhook;
    let discord_webhook = &state.config.discord_webhook;
    let template = &state.config.template;
    let http_webhook_enabled = yes_no_label(http_webhook.enabled);
    let http_webhook_target_configured = yes_no_label(http_webhook.target_url_configured());
    let http_webhook_target_valid = yes_no_label(http_webhook.target_url_valid());
    let http_webhook_shared_secret_configured =
        yes_no_label(http_webhook.shared_secret_configured());
    let http_webhook_send_path_enabled = yes_no_label(http_webhook.send_path_enabled());
    let discord_webhook_enabled = yes_no_label(discord_webhook.enabled);
    let discord_webhook_url_configured = yes_no_label(discord_webhook.webhook_url_configured());
    let discord_webhook_url_valid = yes_no_label(discord_webhook.webhook_url_valid());
    let discord_webhook_send_path_enabled = yes_no_label(discord_webhook.send_path_enabled());
    let provider_send_configured =
        yes_no_label(http_webhook.send_path_enabled() || discord_webhook.send_path_enabled());
    let provider_send_path_count = provider_send_path_count(&state.config);
    let configuration_status = notification_configuration_status(&state.config);
    let summary_template_configured = yes_no_label(template.summary_template_configured());
    let summary_template_valid = yes_no_label(template.summary_template_valid());
    let provider_attempt_history_count = state.provider_attempt_history.snapshot().len();
    let provider_attempt_history_capacity = state.provider_attempt_history.capacity();
    Html(format!(
        r#"<!doctype html>
<html lang="en">
<head><meta charset="utf-8"><title>Nako Notification Bridge</title></head>
<body>
  <h1>Nako Notification Bridge</h1>
  <p>Base URL: {}</p>
  <p>Mode: ack only</p>
  <p>Provider send configured: {provider_send_configured}</p>
  <p>Provider send path count: {provider_send_path_count}</p>
  <p>Configuration status: {}</p>
  <p>Summary template status: {}</p>
  <p>Summary template configured: {summary_template_configured}</p>
  <p>Summary template valid: {summary_template_valid}</p>
  <p>Provider attempt history count: {provider_attempt_history_count}</p>
  <p>Provider attempt history capacity: {provider_attempt_history_capacity}</p>
  <p>HTTP webhook provider status: {}</p>
  <p>HTTP webhook enabled: {http_webhook_enabled}</p>
  <p>HTTP webhook target configured: {http_webhook_target_configured}</p>
  <p>HTTP webhook target valid: {http_webhook_target_valid}</p>
  <p>HTTP webhook shared secret configured: {http_webhook_shared_secret_configured}</p>
  <p>HTTP webhook send path enabled: {http_webhook_send_path_enabled}</p>
  <p>Discord webhook provider status: {}</p>
  <p>Discord webhook enabled: {discord_webhook_enabled}</p>
  <p>Discord webhook URL configured: {discord_webhook_url_configured}</p>
  <p>Discord webhook URL valid: {discord_webhook_url_valid}</p>
  <p>Discord webhook send path enabled: {discord_webhook_send_path_enabled}</p>
  <p>This page is hosted by the Addon Sidecar and is not trusted Nako Admin UI.</p>
</body>
</html>"#,
        state.config.base_url,
        configuration_status.as_str(),
        template.status().as_str(),
        http_webhook.status().as_str(),
        discord_webhook.status().as_str()
    ))
}

fn invalid_template_error(
    error: crate::template::TemplateError,
) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::BAD_REQUEST,
        Json(serde_json::json!({
            "safe_error_code": "notification_template_invalid",
            "template_error": error.safe_code(),
            "retryable": false
        })),
    )
}

fn record_http_webhook_outcome(
    history: &ProviderAttemptHistory,
    request: &AddonEventRequest,
    outcome: &HttpWebhookSendOutcome,
) {
    if matches!(outcome, HttpWebhookSendOutcome::SkippedDisabled) {
        return;
    }

    history.record(ProviderAttemptRecord::new(
        HTTP_WEBHOOK_PROVIDER_ID,
        request,
        outcome.provider_status(),
        false,
        outcome.provider_http_status(),
    ));
}

fn record_http_webhook_error(
    history: &ProviderAttemptHistory,
    request: &AddonEventRequest,
    error: &HttpWebhookSendError,
) {
    history.record(ProviderAttemptRecord::new(
        HTTP_WEBHOOK_PROVIDER_ID,
        request,
        error.provider_status(),
        error.is_retryable(),
        error.provider_http_status(),
    ));
}

fn record_discord_webhook_outcome(
    history: &ProviderAttemptHistory,
    request: &AddonEventRequest,
    outcome: &DiscordWebhookSendOutcome,
) {
    if matches!(outcome, DiscordWebhookSendOutcome::SkippedDisabled) {
        return;
    }

    history.record(ProviderAttemptRecord::new(
        DISCORD_WEBHOOK_PROVIDER_ID,
        request,
        outcome.provider_status(),
        false,
        outcome.provider_http_status(),
    ));
}

fn record_discord_webhook_error(
    history: &ProviderAttemptHistory,
    request: &AddonEventRequest,
    error: &DiscordWebhookSendError,
) {
    history.record(ProviderAttemptRecord::new(
        DISCORD_WEBHOOK_PROVIDER_ID,
        request,
        error.provider_status(),
        error.is_retryable(),
        error.provider_http_status(),
    ));
}

const fn yes_no_label(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum NotificationConfigurationStatus {
    AckOnly,
    ProviderSendReady,
    ProviderConfigurationInvalid,
    MultipleProviderSendPathsConfigured,
    TemplateInvalid,
}

impl NotificationConfigurationStatus {
    const fn as_str(self) -> &'static str {
        match self {
            Self::AckOnly => "ack_only",
            Self::ProviderSendReady => "provider_send_ready",
            Self::ProviderConfigurationInvalid => "provider_configuration_invalid",
            Self::MultipleProviderSendPathsConfigured => "multiple_provider_send_paths_configured",
            Self::TemplateInvalid => "template_invalid",
        }
    }

    const fn is_degraded(self) -> bool {
        matches!(
            self,
            Self::ProviderConfigurationInvalid
                | Self::MultipleProviderSendPathsConfigured
                | Self::TemplateInvalid
        )
    }
}

fn notification_configuration_status(config: &Config) -> NotificationConfigurationStatus {
    let send_path_count = provider_send_path_count(config);
    if send_path_count > 1 {
        return NotificationConfigurationStatus::MultipleProviderSendPathsConfigured;
    }

    let provider_configuration_invalid = (config.http_webhook.enabled
        && !config.http_webhook.send_path_enabled())
        || (config.discord_webhook.enabled && !config.discord_webhook.send_path_enabled());
    if provider_configuration_invalid {
        return NotificationConfigurationStatus::ProviderConfigurationInvalid;
    }

    let provider_enabled = config.http_webhook.enabled || config.discord_webhook.enabled;
    if provider_enabled && !config.template.summary_template_valid() {
        return NotificationConfigurationStatus::TemplateInvalid;
    }

    if send_path_count == 1 {
        NotificationConfigurationStatus::ProviderSendReady
    } else {
        NotificationConfigurationStatus::AckOnly
    }
}

fn provider_send_path_count(config: &Config) -> usize {
    (config.http_webhook.send_path_enabled() as usize)
        + (config.discord_webhook.send_path_enabled() as usize)
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use axum::{
        body::Body,
        extract::State as AxumState,
        http::{HeaderMap, Request, StatusCode},
        routing::post as axum_post,
    };
    use nako_addon_protocol::{
        AddonEventRequest, AddonEventResponse, AddonHealthCheckRequest, AddonHealthStatus,
        AddonScope, validate_manifest,
    };
    use tokio::net::TcpListener;
    use tower::ServiceExt;

    use super::*;

    #[derive(Clone, Debug, Default)]
    struct FixtureState {
        requests: Arc<Mutex<Vec<FixtureRequest>>>,
        response_status: StatusCode,
    }

    #[derive(Clone, Debug)]
    struct FixtureRequest {
        secret_header: Option<String>,
        body: serde_json::Value,
    }

    async fn record_webhook(
        AxumState(state): AxumState<FixtureState>,
        headers: HeaderMap,
        Json(body): Json<serde_json::Value>,
    ) -> StatusCode {
        let secret_header = headers
            .get("x-test-secret")
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned);
        state.requests.lock().unwrap().push(FixtureRequest {
            secret_header,
            body,
        });

        state.response_status
    }

    async fn spawn_http_webhook_fixture() -> (String, FixtureState, tokio::task::JoinHandle<()>) {
        spawn_http_webhook_fixture_with_status(StatusCode::ACCEPTED).await
    }

    async fn spawn_http_webhook_fixture_with_status(
        response_status: StatusCode,
    ) -> (String, FixtureState, tokio::task::JoinHandle<()>) {
        let state = FixtureState {
            requests: Arc::new(Mutex::new(Vec::new())),
            response_status,
        };
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let app = Router::new()
            .route("/hook", axum_post(record_webhook))
            .with_state(state.clone());
        let handle = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        (format!("http://{addr}/hook"), state, handle)
    }

    #[tokio::test]
    async fn manifest_endpoint_returns_valid_notification_manifest() {
        let response = router(Config::default())
            .oneshot(
                Request::builder()
                    .uri("/manifest.json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let manifest: nako_addon_protocol::AddonManifest = serde_json::from_slice(&body).unwrap();

        validate_manifest(&manifest).unwrap();
        assert_eq!(manifest.id, ADDON_ID);
        assert_eq!(manifest.scopes, vec![AddonScope::WebhookEventRead]);
        assert_eq!(manifest.resources.len(), 1);
        assert_eq!(
            manifest.resources[0].kind,
            nako_addon_protocol::AddonResource::Webhook
        );
        assert!(manifest.tasks.is_empty());
        assert!(manifest.secret_reference_fields.is_empty());
        assert_eq!(manifest.event_subscriptions.len(), 1);
        assert_eq!(
            manifest.event_subscriptions[0].id,
            LIBRARY_SCANNED_EVENT_SUBSCRIPTION_ID
        );
        assert_eq!(
            manifest.event_subscriptions[0].event_kind,
            LIBRARY_SCANNED_EVENT_KIND
        );
        assert_eq!(
            manifest.event_subscriptions[0].path,
            LIBRARY_SCANNED_EVENT_PATH
        );
    }

    #[tokio::test]
    async fn library_scanned_event_endpoint_acknowledges_without_echoing_payload_values() {
        let request = AddonEventRequest {
            protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
            addon_id: ADDON_ID.to_owned(),
            subscription_id: LIBRARY_SCANNED_EVENT_SUBSCRIPTION_ID.to_owned(),
            event_id: "event-1".to_owned(),
            event_kind: LIBRARY_SCANNED_EVENT_KIND.to_owned(),
            subject_kind: "library".to_owned(),
            subject_id: "library-1".to_owned(),
            occurred_at: "2026-05-25T00:00:00.000Z".to_owned(),
            attempt: 1,
            payload: serde_json::json!({
                "library_id": "library-1",
                "source_id": "source-1",
                "secret": "nako_at_should_not_echo"
            }),
        };
        let response = router(Config::default())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(LIBRARY_SCANNED_EVENT_PATH)
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&request).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();
        let payload: AddonEventResponse = serde_json::from_str(&text).unwrap();

        assert_eq!(payload.addon_id, ADDON_ID);
        assert_eq!(
            payload.subscription_id,
            LIBRARY_SCANNED_EVENT_SUBSCRIPTION_ID
        );
        assert_eq!(payload.event_id, "event-1");
        assert_eq!(
            payload.output["schema"],
            "nako.official.notification-bridge.library-scanned.event.v1"
        );
        assert_eq!(payload.output["accepted"], true);
        assert_eq!(payload.output["mode"], "ack_only");
        assert_eq!(
            payload.output["payload_keys"],
            serde_json::json!(["library_id", "secret", "source_id"])
        );
        assert!(!text.contains("nako_at_should_not_echo"));
        assert!(!text.contains("source-1"));
    }

    #[tokio::test]
    async fn library_scanned_event_endpoint_sends_http_webhook_payload_without_raw_event_values() {
        let (target_url, fixture, handle) = spawn_http_webhook_fixture().await;
        let request = AddonEventRequest {
            protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
            addon_id: ADDON_ID.to_owned(),
            subscription_id: LIBRARY_SCANNED_EVENT_SUBSCRIPTION_ID.to_owned(),
            event_id: "event-1".to_owned(),
            event_kind: LIBRARY_SCANNED_EVENT_KIND.to_owned(),
            subject_kind: "library".to_owned(),
            subject_id: "library-1".to_owned(),
            occurred_at: "2026-05-25T00:00:00.000Z".to_owned(),
            attempt: 1,
            payload: serde_json::json!({
                "library_id": "library-1",
                "source_id": "source-1",
                "secret": "nako_at_should_not_echo"
            }),
        };
        let response = router(Config::from_env_lookup(|name| match name {
            "NAKO_NOTIFICATION_BRIDGE_HTTP_WEBHOOK_ENABLED" => Some("true".to_owned()),
            "NAKO_NOTIFICATION_BRIDGE_HTTP_WEBHOOK_URL" => Some(target_url.clone()),
            "NAKO_NOTIFICATION_BRIDGE_HTTP_WEBHOOK_SECRET_HEADER_NAME" => {
                Some("X-Test-Secret".to_owned())
            }
            "NAKO_NOTIFICATION_BRIDGE_HTTP_WEBHOOK_SHARED_SECRET" => {
                Some("fixture-shared-secret".to_owned())
            }
            _ => None,
        }))
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(LIBRARY_SCANNED_EVENT_PATH)
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();
        let payload: AddonEventResponse = serde_json::from_str(&text).unwrap();

        assert_eq!(payload.output["mode"], "provider_send");
        assert_eq!(payload.output["provider"]["id"], "http_webhook");
        assert_eq!(payload.output["provider"]["status"], "sent");
        assert_eq!(payload.output["provider"]["http_status"], 202);
        assert!(!text.contains("nako_at_should_not_echo"));
        assert!(!text.contains("source-1"));
        assert!(!text.contains("fixture-shared-secret"));

        let requests = fixture.requests.lock().unwrap();
        assert_eq!(requests.len(), 1);
        assert_eq!(
            requests[0].secret_header.as_deref(),
            Some("fixture-shared-secret")
        );
        assert_eq!(
            requests[0].body["schema"],
            "nako.official.notification-bridge.http-webhook.library-scanned.v1"
        );
        assert_eq!(requests[0].body["event"]["event_id"], "event-1");
        assert_eq!(
            requests[0].body["payload_keys"],
            serde_json::json!(["library_id", "secret", "source_id"])
        );
        let webhook_body = serde_json::to_string(&requests[0].body).unwrap();
        assert!(!webhook_body.contains("nako_at_should_not_echo"));
        assert!(!webhook_body.contains("source-1"));

        handle.abort();
    }

    #[tokio::test]
    async fn library_scanned_event_endpoint_sends_discord_webhook_payload_without_raw_event_values()
    {
        let (target_url, fixture, handle) = spawn_http_webhook_fixture().await;
        let request = AddonEventRequest {
            protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
            addon_id: ADDON_ID.to_owned(),
            subscription_id: LIBRARY_SCANNED_EVENT_SUBSCRIPTION_ID.to_owned(),
            event_id: "event-1".to_owned(),
            event_kind: LIBRARY_SCANNED_EVENT_KIND.to_owned(),
            subject_kind: "library".to_owned(),
            subject_id: "library-1".to_owned(),
            occurred_at: "2026-05-25T00:00:00.000Z".to_owned(),
            attempt: 1,
            payload: serde_json::json!({
                "library_id": "library-1",
                "source_id": "source-1",
                "secret": "nako_at_should_not_echo"
            }),
        };
        let response = router(Config::from_env_lookup(|name| match name {
            "NAKO_NOTIFICATION_BRIDGE_DISCORD_WEBHOOK_ENABLED" => Some("true".to_owned()),
            "NAKO_NOTIFICATION_BRIDGE_DISCORD_WEBHOOK_URL" => Some(target_url.clone()),
            _ => None,
        }))
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(LIBRARY_SCANNED_EVENT_PATH)
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();
        let payload: AddonEventResponse = serde_json::from_str(&text).unwrap();

        assert_eq!(payload.output["mode"], "provider_send");
        assert_eq!(payload.output["provider"]["id"], "discord_webhook");
        assert_eq!(payload.output["provider"]["status"], "sent");
        assert_eq!(payload.output["provider"]["http_status"], 202);
        assert_eq!(payload.output["providers"][0]["id"], "http_webhook");
        assert_eq!(payload.output["providers"][0]["status"], "disabled");
        assert_eq!(payload.output["providers"][1]["id"], "discord_webhook");
        assert_eq!(payload.output["providers"][1]["status"], "sent");
        assert!(!text.contains("nako_at_should_not_echo"));
        assert!(!text.contains("source-1"));
        assert!(!text.contains("127.0.0.1"));

        let requests = fixture.requests.lock().unwrap();
        assert_eq!(requests.len(), 1);
        assert_eq!(
            requests[0].body["schema"],
            "nako.official.notification-bridge.discord-webhook.library-scanned.v1"
        );
        assert_eq!(
            requests[0].body["content"],
            "Nako library.scanned event for library library-1"
        );
        assert_eq!(
            requests[0].body["embeds"][0]["title"],
            "Nako library scanned"
        );
        let webhook_body = serde_json::to_string(&requests[0].body).unwrap();
        assert!(webhook_body.contains("library_id, secret, source_id"));
        assert!(!webhook_body.contains("nako_at_should_not_echo"));
        assert!(!webhook_body.contains("source-1"));

        handle.abort();
    }

    #[tokio::test]
    async fn library_scanned_event_endpoint_uses_safe_template_without_raw_payload_values() {
        let (target_url, fixture, handle) = spawn_http_webhook_fixture().await;
        let request = AddonEventRequest {
            protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
            addon_id: ADDON_ID.to_owned(),
            subscription_id: LIBRARY_SCANNED_EVENT_SUBSCRIPTION_ID.to_owned(),
            event_id: "event-1".to_owned(),
            event_kind: LIBRARY_SCANNED_EVENT_KIND.to_owned(),
            subject_kind: "library".to_owned(),
            subject_id: "library-1".to_owned(),
            occurred_at: "2026-05-25T00:00:00.000Z".to_owned(),
            attempt: 1,
            payload: serde_json::json!({
                "library_id": "library-1",
                "source_id": "source-1",
                "secret": "nako_at_should_not_echo"
            }),
        };
        let response = router(Config::from_env_lookup(|name| match name {
            "NAKO_NOTIFICATION_BRIDGE_DISCORD_WEBHOOK_ENABLED" => Some("true".to_owned()),
            "NAKO_NOTIFICATION_BRIDGE_DISCORD_WEBHOOK_URL" => Some(target_url.clone()),
            "NAKO_NOTIFICATION_BRIDGE_TEMPLATE_SUMMARY" => {
                Some("{{event_kind}} keys={{payload_keys}} attempt={{attempt}}".to_owned())
            }
            _ => None,
        }))
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(LIBRARY_SCANNED_EVENT_PATH)
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let requests = fixture.requests.lock().unwrap();
        assert_eq!(requests.len(), 1);
        assert_eq!(
            requests[0].body["content"],
            "library.scanned keys=library_id, secret, source_id attempt=1"
        );
        let webhook_body = serde_json::to_string(&requests[0].body).unwrap();
        assert!(!webhook_body.contains("nako_at_should_not_echo"));
        assert!(!webhook_body.contains("source-1"));

        handle.abort();
    }

    #[tokio::test]
    async fn library_scanned_event_endpoint_rejects_invalid_template_before_provider_send() {
        let (target_url, fixture, handle) = spawn_http_webhook_fixture().await;
        let request = AddonEventRequest {
            protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
            addon_id: ADDON_ID.to_owned(),
            subscription_id: LIBRARY_SCANNED_EVENT_SUBSCRIPTION_ID.to_owned(),
            event_id: "event-1".to_owned(),
            event_kind: LIBRARY_SCANNED_EVENT_KIND.to_owned(),
            subject_kind: "library".to_owned(),
            subject_id: "library-1".to_owned(),
            occurred_at: "2026-05-25T00:00:00.000Z".to_owned(),
            attempt: 1,
            payload: serde_json::json!({
                "library_id": "library-1",
                "secret": "nako_at_should_not_echo"
            }),
        };
        let response = router(Config::from_env_lookup(|name| match name {
            "NAKO_NOTIFICATION_BRIDGE_DISCORD_WEBHOOK_ENABLED" => Some("true".to_owned()),
            "NAKO_NOTIFICATION_BRIDGE_DISCORD_WEBHOOK_URL" => Some(target_url.clone()),
            "NAKO_NOTIFICATION_BRIDGE_TEMPLATE_SUMMARY" => Some("{{payload.secret}}".to_owned()),
            _ => None,
        }))
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(LIBRARY_SCANNED_EVENT_PATH)
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();
        let payload: serde_json::Value = serde_json::from_str(&text).unwrap();

        assert_eq!(payload["safe_error_code"], "notification_template_invalid");
        assert_eq!(payload["template_error"], "unknown_template_token");
        assert_eq!(payload["retryable"], false);
        assert!(!text.contains("payload.secret"));
        assert!(!text.contains("nako_at_should_not_echo"));
        assert_eq!(fixture.requests.lock().unwrap().len(), 0);

        handle.abort();
    }

    #[tokio::test]
    async fn provider_attempt_history_records_safe_recent_provider_send_outcomes() {
        let (target_url, fixture, handle) = spawn_http_webhook_fixture().await;
        let app = router(Config::from_env_lookup(|name| match name {
            "NAKO_NOTIFICATION_BRIDGE_DISCORD_WEBHOOK_ENABLED" => Some("true".to_owned()),
            "NAKO_NOTIFICATION_BRIDGE_DISCORD_WEBHOOK_URL" => Some(target_url.clone()),
            "NAKO_NOTIFICATION_BRIDGE_PROVIDER_ATTEMPT_HISTORY_CAPACITY" => Some("4".to_owned()),
            _ => None,
        }));
        let request = AddonEventRequest {
            protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
            addon_id: ADDON_ID.to_owned(),
            subscription_id: LIBRARY_SCANNED_EVENT_SUBSCRIPTION_ID.to_owned(),
            event_id: "event-1".to_owned(),
            event_kind: LIBRARY_SCANNED_EVENT_KIND.to_owned(),
            subject_kind: "library".to_owned(),
            subject_id: "library-1".to_owned(),
            occurred_at: "2026-05-25T00:00:00.000Z".to_owned(),
            attempt: 1,
            payload: serde_json::json!({
                "library_id": "library-1",
                "source_id": "source-1",
                "secret": "nako_at_should_not_echo"
            }),
        };
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(LIBRARY_SCANNED_EVENT_PATH)
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&request).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(fixture.requests.lock().unwrap().len(), 1);

        let health_request = AddonHealthCheckRequest {
            protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
            manifest_id: ADDON_ID.to_owned(),
            request_id: "health-1".to_owned(),
            expected_addon_version: ADDON_VERSION.to_owned(),
            expected_resource_count: crate::manifest::container_manifest().resources.len(),
        };
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/health")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&health_request).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();
        let payload: AddonHealthCheckResponse = serde_json::from_str(&text).unwrap();
        let attempts = payload.diagnostics["provider_attempt_history"]["recent"]
            .as_array()
            .unwrap();

        assert_eq!(
            payload.diagnostics["provider_attempt_history"]["capacity"],
            4
        );
        assert_eq!(attempts.len(), 1);
        assert_eq!(attempts[0]["provider_id"], "discord_webhook");
        assert_eq!(attempts[0]["provider_status"], "sent");
        assert_eq!(attempts[0]["provider_http_status"], 202);
        assert!(!text.contains("nako_at_should_not_echo"));
        assert!(!text.contains("source-1"));
        assert!(!text.contains("127.0.0.1"));

        handle.abort();
    }

    #[tokio::test]
    async fn library_scanned_event_endpoint_rejects_multiple_provider_send_paths_without_sending() {
        let (http_target_url, http_fixture, http_handle) = spawn_http_webhook_fixture().await;
        let (discord_target_url, discord_fixture, discord_handle) =
            spawn_http_webhook_fixture().await;
        let request = AddonEventRequest {
            protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
            addon_id: ADDON_ID.to_owned(),
            subscription_id: LIBRARY_SCANNED_EVENT_SUBSCRIPTION_ID.to_owned(),
            event_id: "event-1".to_owned(),
            event_kind: LIBRARY_SCANNED_EVENT_KIND.to_owned(),
            subject_kind: "library".to_owned(),
            subject_id: "library-1".to_owned(),
            occurred_at: "2026-05-25T00:00:00.000Z".to_owned(),
            attempt: 1,
            payload: serde_json::json!({
                "library_id": "library-1",
                "secret": "nako_at_should_not_echo"
            }),
        };
        let response = router(Config::from_env_lookup(|name| match name {
            "NAKO_NOTIFICATION_BRIDGE_HTTP_WEBHOOK_ENABLED" => Some("true".to_owned()),
            "NAKO_NOTIFICATION_BRIDGE_HTTP_WEBHOOK_URL" => Some(http_target_url.clone()),
            "NAKO_NOTIFICATION_BRIDGE_DISCORD_WEBHOOK_ENABLED" => Some("true".to_owned()),
            "NAKO_NOTIFICATION_BRIDGE_DISCORD_WEBHOOK_URL" => Some(discord_target_url.clone()),
            _ => None,
        }))
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(LIBRARY_SCANNED_EVENT_PATH)
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();
        let payload: serde_json::Value = serde_json::from_str(&text).unwrap();

        assert_eq!(
            payload["safe_error_code"],
            "multiple_notification_provider_send_paths_configured"
        );
        assert_eq!(payload["retryable"], false);
        assert!(!text.contains("nako_at_should_not_echo"));
        assert!(!text.contains("127.0.0.1"));
        assert_eq!(http_fixture.requests.lock().unwrap().len(), 0);
        assert_eq!(discord_fixture.requests.lock().unwrap().len(), 0);

        http_handle.abort();
        discord_handle.abort();
    }

    #[tokio::test]
    async fn library_scanned_event_endpoint_returns_retryable_safe_failure_for_rate_limited_http_webhook()
     {
        let (target_url, fixture, handle) =
            spawn_http_webhook_fixture_with_status(StatusCode::TOO_MANY_REQUESTS).await;
        let request = AddonEventRequest {
            protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
            addon_id: ADDON_ID.to_owned(),
            subscription_id: LIBRARY_SCANNED_EVENT_SUBSCRIPTION_ID.to_owned(),
            event_id: "event-1".to_owned(),
            event_kind: LIBRARY_SCANNED_EVENT_KIND.to_owned(),
            subject_kind: "library".to_owned(),
            subject_id: "library-1".to_owned(),
            occurred_at: "2026-05-25T00:00:00.000Z".to_owned(),
            attempt: 1,
            payload: serde_json::json!({
                "library_id": "library-1",
                "secret": "nako_at_should_not_echo"
            }),
        };
        let response = router(Config::from_env_lookup(|name| match name {
            "NAKO_NOTIFICATION_BRIDGE_HTTP_WEBHOOK_ENABLED" => Some("true".to_owned()),
            "NAKO_NOTIFICATION_BRIDGE_HTTP_WEBHOOK_URL" => Some(target_url.clone()),
            "NAKO_NOTIFICATION_BRIDGE_HTTP_WEBHOOK_SHARED_SECRET" => {
                Some("fixture-shared-secret".to_owned())
            }
            _ => None,
        }))
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(LIBRARY_SCANNED_EVENT_PATH)
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();
        let payload: serde_json::Value = serde_json::from_str(&text).unwrap();

        assert_eq!(payload["safe_error_code"], "http_webhook_retryable_failure");
        assert_eq!(payload["provider_id"], "http_webhook");
        assert_eq!(payload["provider_status"], "retryable_failure");
        assert_eq!(payload["provider_http_status"], 429);
        assert_eq!(payload["retryable"], true);
        assert!(!text.contains("nako_at_should_not_echo"));
        assert!(!text.contains("fixture-shared-secret"));
        assert!(!text.contains("127.0.0.1"));

        let requests = fixture.requests.lock().unwrap();
        assert_eq!(requests.len(), 1);
        let webhook_body = serde_json::to_string(&requests[0].body).unwrap();
        assert!(!webhook_body.contains("nako_at_should_not_echo"));

        handle.abort();
    }

    #[tokio::test]
    async fn library_scanned_event_endpoint_returns_non_retryable_safe_failure_for_provider_rejection()
     {
        let (target_url, fixture, handle) =
            spawn_http_webhook_fixture_with_status(StatusCode::BAD_REQUEST).await;
        let request = AddonEventRequest {
            protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
            addon_id: ADDON_ID.to_owned(),
            subscription_id: LIBRARY_SCANNED_EVENT_SUBSCRIPTION_ID.to_owned(),
            event_id: "event-1".to_owned(),
            event_kind: LIBRARY_SCANNED_EVENT_KIND.to_owned(),
            subject_kind: "library".to_owned(),
            subject_id: "library-1".to_owned(),
            occurred_at: "2026-05-25T00:00:00.000Z".to_owned(),
            attempt: 1,
            payload: serde_json::json!({
                "library_id": "library-1",
                "secret": "nako_at_should_not_echo"
            }),
        };
        let response = router(Config::from_env_lookup(|name| match name {
            "NAKO_NOTIFICATION_BRIDGE_HTTP_WEBHOOK_ENABLED" => Some("true".to_owned()),
            "NAKO_NOTIFICATION_BRIDGE_HTTP_WEBHOOK_URL" => Some(target_url.clone()),
            _ => None,
        }))
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(LIBRARY_SCANNED_EVENT_PATH)
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

        assert_eq!(response.status(), StatusCode::FAILED_DEPENDENCY);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();
        let payload: serde_json::Value = serde_json::from_str(&text).unwrap();

        assert_eq!(
            payload["safe_error_code"],
            "http_webhook_non_retryable_failure"
        );
        assert_eq!(payload["provider_status"], "non_retryable_failure");
        assert_eq!(payload["provider_http_status"], 400);
        assert_eq!(payload["retryable"], false);
        assert!(!text.contains("nako_at_should_not_echo"));
        assert!(!text.contains("127.0.0.1"));
        assert_eq!(fixture.requests.lock().unwrap().len(), 1);

        handle.abort();
    }

    #[tokio::test]
    async fn library_scanned_event_endpoint_rejects_wrong_envelope_with_safe_code() {
        let request = AddonEventRequest {
            protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
            addon_id: ADDON_ID.to_owned(),
            subscription_id: "wrong-subscription".to_owned(),
            event_id: "event-1".to_owned(),
            event_kind: LIBRARY_SCANNED_EVENT_KIND.to_owned(),
            subject_kind: "library".to_owned(),
            subject_id: "library-1".to_owned(),
            occurred_at: "2026-05-25T00:00:00.000Z".to_owned(),
            attempt: 1,
            payload: serde_json::json!({
                "secret": "nako_at_should_not_echo"
            }),
        };
        let response = router(Config::default())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(LIBRARY_SCANNED_EVENT_PATH)
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&request).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();

        assert!(text.contains("invalid_event_envelope"));
        assert!(!text.contains("nako_at_should_not_echo"));
    }

    #[tokio::test]
    async fn health_endpoint_reports_ack_only_mode() {
        let request = AddonHealthCheckRequest {
            protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
            manifest_id: ADDON_ID.to_owned(),
            request_id: "health-1".to_owned(),
            expected_addon_version: ADDON_VERSION.to_owned(),
            expected_resource_count: crate::manifest::container_manifest().resources.len(),
        };
        let response = router(Config::default())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/health")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&request).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let payload: AddonHealthCheckResponse = serde_json::from_slice(&body).unwrap();

        assert_eq!(payload.status, AddonHealthStatus::Ok);
        assert_eq!(
            payload.manifest.resource_count,
            crate::manifest::container_manifest().resources.len()
        );
        assert_eq!(payload.diagnostics["mode"], "ack_only");
        assert_eq!(payload.diagnostics["provider_fan_out"], false);
        assert_eq!(payload.diagnostics["provider_send_path_count"], 0);
        assert_eq!(payload.diagnostics["configuration_status"], "ack_only");
        assert_eq!(
            payload.diagnostics["template"]["summary_template_configured"],
            false
        );
        assert_eq!(
            payload.diagnostics["template"]["summary_template_valid"],
            true
        );
        assert_eq!(payload.diagnostics["template"]["status"], "valid");
        assert_eq!(
            payload.diagnostics["provider_attempt_history"]["capacity"],
            20
        );
        assert_eq!(
            payload.diagnostics["provider_attempt_history"]["recent"]
                .as_array()
                .unwrap()
                .len(),
            0
        );
        assert_eq!(
            payload.diagnostics["providers"].as_array().unwrap().len(),
            2
        );
        assert_eq!(payload.diagnostics["providers"][0]["id"], "http_webhook");
        assert_eq!(payload.diagnostics["providers"][0]["enabled"], false);
        assert_eq!(payload.diagnostics["providers"][0]["status"], "disabled");
        assert_eq!(
            payload.diagnostics["providers"][0]["send_path_enabled"],
            false
        );
        assert_eq!(payload.diagnostics["providers"][1]["id"], "discord_webhook");
        assert_eq!(payload.diagnostics["providers"][1]["enabled"], false);
        assert_eq!(payload.diagnostics["providers"][1]["status"], "disabled");
        assert_eq!(
            payload.diagnostics["providers"][1]["send_path_enabled"],
            false
        );
    }

    #[tokio::test]
    async fn health_endpoint_reports_http_webhook_config_without_leaking_values() {
        let request = AddonHealthCheckRequest {
            protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
            manifest_id: ADDON_ID.to_owned(),
            request_id: "health-1".to_owned(),
            expected_addon_version: ADDON_VERSION.to_owned(),
            expected_resource_count: crate::manifest::container_manifest().resources.len(),
        };
        let response = router(Config::from_env_lookup(|name| match name {
            "NAKO_NOTIFICATION_BRIDGE_HTTP_WEBHOOK_ENABLED" => Some("true".to_owned()),
            "NAKO_NOTIFICATION_BRIDGE_HTTP_WEBHOOK_URL" => {
                Some("https://hooks.example/internal/path".to_owned())
            }
            "NAKO_NOTIFICATION_BRIDGE_HTTP_WEBHOOK_SECRET_HEADER_NAME" => {
                Some("X-Leaky-Header".to_owned())
            }
            "NAKO_NOTIFICATION_BRIDGE_HTTP_WEBHOOK_SHARED_SECRET" => {
                Some("webhook-secret-should-not-appear".to_owned())
            }
            _ => None,
        }))
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/health")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let payload: AddonHealthCheckResponse = serde_json::from_slice(&body).unwrap();

        assert_eq!(payload.diagnostics["provider_fan_out"], true);
        assert_eq!(payload.diagnostics["provider_send_path_count"], 1);
        assert_eq!(
            payload.diagnostics["configuration_status"],
            "provider_send_ready"
        );
        assert_eq!(payload.diagnostics["providers"][0]["enabled"], true);
        assert_eq!(payload.diagnostics["providers"][0]["status"], "configured");
        assert_eq!(
            payload.diagnostics["providers"][0]["target_url_configured"],
            true
        );
        assert_eq!(
            payload.diagnostics["providers"][0]["target_url_valid"],
            true
        );
        assert_eq!(
            payload.diagnostics["providers"][0]["custom_secret_header_name_configured"],
            true
        );
        assert_eq!(
            payload.diagnostics["providers"][0]["shared_secret_configured"],
            true
        );
        assert_eq!(
            payload.diagnostics["providers"][0]["send_path_enabled"],
            true
        );

        let diagnostics = serde_json::to_string(&payload.diagnostics).unwrap();
        assert!(!diagnostics.contains("hooks.example"));
        assert!(!diagnostics.contains("webhook-secret-should-not-appear"));
        assert!(!diagnostics.contains("X-Leaky-Header"));
    }

    #[tokio::test]
    async fn health_endpoint_reports_discord_webhook_config_without_leaking_values() {
        let request = AddonHealthCheckRequest {
            protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
            manifest_id: ADDON_ID.to_owned(),
            request_id: "health-1".to_owned(),
            expected_addon_version: ADDON_VERSION.to_owned(),
            expected_resource_count: crate::manifest::container_manifest().resources.len(),
        };
        let response = router(Config::from_env_lookup(|name| match name {
            "NAKO_NOTIFICATION_BRIDGE_DISCORD_WEBHOOK_ENABLED" => Some("true".to_owned()),
            "NAKO_NOTIFICATION_BRIDGE_DISCORD_WEBHOOK_URL" => {
                Some("https://discord.example/api/webhooks/secret".to_owned())
            }
            _ => None,
        }))
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/health")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let payload: AddonHealthCheckResponse = serde_json::from_slice(&body).unwrap();

        assert_eq!(payload.diagnostics["provider_fan_out"], true);
        assert_eq!(payload.diagnostics["provider_send_path_count"], 1);
        assert_eq!(
            payload.diagnostics["configuration_status"],
            "provider_send_ready"
        );
        assert_eq!(payload.diagnostics["providers"][1]["id"], "discord_webhook");
        assert_eq!(payload.diagnostics["providers"][1]["enabled"], true);
        assert_eq!(payload.diagnostics["providers"][1]["status"], "configured");
        assert_eq!(
            payload.diagnostics["providers"][1]["webhook_url_configured"],
            true
        );
        assert_eq!(
            payload.diagnostics["providers"][1]["webhook_url_valid"],
            true
        );
        assert_eq!(
            payload.diagnostics["providers"][1]["send_path_enabled"],
            true
        );

        let diagnostics = serde_json::to_string(&payload.diagnostics).unwrap();
        assert!(!diagnostics.contains("discord.example"));
        assert!(!diagnostics.contains("api/webhooks"));
    }

    #[tokio::test]
    async fn health_endpoint_degrades_multiple_provider_send_path_configuration() {
        let request = AddonHealthCheckRequest {
            protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
            manifest_id: ADDON_ID.to_owned(),
            request_id: "health-1".to_owned(),
            expected_addon_version: ADDON_VERSION.to_owned(),
            expected_resource_count: crate::manifest::container_manifest().resources.len(),
        };
        let response = router(Config::from_env_lookup(|name| match name {
            "NAKO_NOTIFICATION_BRIDGE_HTTP_WEBHOOK_ENABLED" => Some("true".to_owned()),
            "NAKO_NOTIFICATION_BRIDGE_HTTP_WEBHOOK_URL" => {
                Some("https://hooks.example/internal/path".to_owned())
            }
            "NAKO_NOTIFICATION_BRIDGE_DISCORD_WEBHOOK_ENABLED" => Some("true".to_owned()),
            "NAKO_NOTIFICATION_BRIDGE_DISCORD_WEBHOOK_URL" => {
                Some("https://discord.example/api/webhooks/secret".to_owned())
            }
            _ => None,
        }))
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/health")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();
        let payload: AddonHealthCheckResponse = serde_json::from_str(&text).unwrap();

        assert_eq!(payload.status, AddonHealthStatus::Degraded);
        assert_eq!(payload.diagnostics["provider_send_path_count"], 2);
        assert_eq!(
            payload.diagnostics["configuration_status"],
            "multiple_provider_send_paths_configured"
        );
        assert!(!text.contains("hooks.example"));
        assert!(!text.contains("discord.example"));
        assert!(!text.contains("api/webhooks"));
    }

    #[tokio::test]
    async fn health_endpoint_degrades_invalid_provider_configuration_without_leaking_values() {
        let request = AddonHealthCheckRequest {
            protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
            manifest_id: ADDON_ID.to_owned(),
            request_id: "health-1".to_owned(),
            expected_addon_version: ADDON_VERSION.to_owned(),
            expected_resource_count: crate::manifest::container_manifest().resources.len(),
        };
        let response = router(Config::from_env_lookup(|name| match name {
            "NAKO_NOTIFICATION_BRIDGE_HTTP_WEBHOOK_ENABLED" => Some("true".to_owned()),
            "NAKO_NOTIFICATION_BRIDGE_HTTP_WEBHOOK_URL" => Some("file:///tmp/nako-hook".to_owned()),
            "NAKO_NOTIFICATION_BRIDGE_HTTP_WEBHOOK_SHARED_SECRET" => {
                Some("webhook-secret-should-not-appear".to_owned())
            }
            _ => None,
        }))
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/health")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();
        let payload: AddonHealthCheckResponse = serde_json::from_str(&text).unwrap();

        assert_eq!(payload.status, AddonHealthStatus::Degraded);
        assert_eq!(payload.diagnostics["provider_fan_out"], false);
        assert_eq!(payload.diagnostics["provider_send_path_count"], 0);
        assert_eq!(
            payload.diagnostics["configuration_status"],
            "provider_configuration_invalid"
        );
        assert_eq!(payload.diagnostics["providers"][0]["enabled"], true);
        assert_eq!(
            payload.diagnostics["providers"][0]["status"],
            "invalid_target_url"
        );
        assert_eq!(
            payload.diagnostics["providers"][0]["send_path_enabled"],
            false
        );
        assert!(!text.contains("file:///tmp/nako-hook"));
        assert!(!text.contains("webhook-secret-should-not-appear"));
    }

    #[tokio::test]
    async fn health_endpoint_degrades_enabled_provider_with_invalid_template() {
        let request = AddonHealthCheckRequest {
            protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
            manifest_id: ADDON_ID.to_owned(),
            request_id: "health-1".to_owned(),
            expected_addon_version: ADDON_VERSION.to_owned(),
            expected_resource_count: crate::manifest::container_manifest().resources.len(),
        };
        let response = router(Config::from_env_lookup(|name| match name {
            "NAKO_NOTIFICATION_BRIDGE_DISCORD_WEBHOOK_ENABLED" => Some("true".to_owned()),
            "NAKO_NOTIFICATION_BRIDGE_DISCORD_WEBHOOK_URL" => {
                Some("https://discord.example/api/webhooks/secret".to_owned())
            }
            "NAKO_NOTIFICATION_BRIDGE_TEMPLATE_SUMMARY" => Some("{{payload.secret}}".to_owned()),
            _ => None,
        }))
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/health")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();
        let payload: AddonHealthCheckResponse = serde_json::from_str(&text).unwrap();

        assert_eq!(payload.status, AddonHealthStatus::Degraded);
        assert_eq!(payload.diagnostics["provider_send_path_count"], 1);
        assert_eq!(
            payload.diagnostics["configuration_status"],
            "template_invalid"
        );
        assert_eq!(
            payload.diagnostics["template"]["summary_template_configured"],
            true
        );
        assert_eq!(
            payload.diagnostics["template"]["summary_template_valid"],
            false
        );
        assert_eq!(payload.diagnostics["template"]["status"], "invalid");
        assert!(!text.contains("payload.secret"));
        assert!(!text.contains("discord.example"));
        assert!(!text.contains("api/webhooks"));
    }

    #[tokio::test]
    async fn health_endpoint_reports_template_status_without_leaking_template_text() {
        let request = AddonHealthCheckRequest {
            protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
            manifest_id: ADDON_ID.to_owned(),
            request_id: "health-1".to_owned(),
            expected_addon_version: ADDON_VERSION.to_owned(),
            expected_resource_count: crate::manifest::container_manifest().resources.len(),
        };
        let response = router(Config::from_env_lookup(|name| match name {
            "NAKO_NOTIFICATION_BRIDGE_TEMPLATE_SUMMARY" => {
                Some("{{event_kind}} secret-literal-should-not-appear".to_owned())
            }
            _ => None,
        }))
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/health")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let payload: AddonHealthCheckResponse = serde_json::from_slice(&body).unwrap();

        assert_eq!(
            payload.diagnostics["template"]["summary_template_configured"],
            true
        );
        assert_eq!(
            payload.diagnostics["template"]["summary_template_valid"],
            true
        );
        assert_eq!(payload.diagnostics["template"]["status"], "valid");
        let diagnostics = serde_json::to_string(&payload.diagnostics).unwrap();
        assert!(!diagnostics.contains("secret-literal-should-not-appear"));
    }

    #[tokio::test]
    async fn diagnostics_page_reports_http_webhook_status_without_leaking_values() {
        let response = router(Config::from_env_lookup(|name| match name {
            "NAKO_NOTIFICATION_BRIDGE_HTTP_WEBHOOK_ENABLED" => Some("true".to_owned()),
            "NAKO_NOTIFICATION_BRIDGE_HTTP_WEBHOOK_URL" => {
                Some("https://hooks.example/internal/path".to_owned())
            }
            "NAKO_NOTIFICATION_BRIDGE_HTTP_WEBHOOK_SHARED_SECRET" => {
                Some("webhook-secret-should-not-appear".to_owned())
            }
            _ => None,
        }))
        .oneshot(
            Request::builder()
                .uri(DIAGNOSTICS_PATH)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();

        assert!(text.contains("HTTP webhook provider status: configured"));
        assert!(text.contains("HTTP webhook enabled: yes"));
        assert!(text.contains("HTTP webhook target configured: yes"));
        assert!(text.contains("HTTP webhook shared secret configured: yes"));
        assert!(text.contains("HTTP webhook send path enabled: yes"));
        assert!(!text.contains("hooks.example"));
        assert!(!text.contains("webhook-secret-should-not-appear"));
    }

    #[tokio::test]
    async fn diagnostics_page_reports_discord_webhook_status_without_leaking_values() {
        let response = router(Config::from_env_lookup(|name| match name {
            "NAKO_NOTIFICATION_BRIDGE_DISCORD_WEBHOOK_ENABLED" => Some("true".to_owned()),
            "NAKO_NOTIFICATION_BRIDGE_DISCORD_WEBHOOK_URL" => {
                Some("https://discord.example/api/webhooks/secret".to_owned())
            }
            _ => None,
        }))
        .oneshot(
            Request::builder()
                .uri(DIAGNOSTICS_PATH)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();

        assert!(text.contains("Provider send configured: yes"));
        assert!(text.contains("Provider send path count: 1"));
        assert!(text.contains("Configuration status: provider_send_ready"));
        assert!(text.contains("Discord webhook provider status: configured"));
        assert!(text.contains("Discord webhook enabled: yes"));
        assert!(text.contains("Discord webhook URL configured: yes"));
        assert!(text.contains("Discord webhook URL valid: yes"));
        assert!(text.contains("Discord webhook send path enabled: yes"));
        assert!(!text.contains("discord.example"));
        assert!(!text.contains("api/webhooks"));
    }
}
