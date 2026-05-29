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
    attempt_history::ProviderAttemptHistory,
    diagnostics::render_diagnostics_page,
    manifest::{
        ADDON_ID, ADDON_VERSION, DIAGNOSTICS_PATH, LIBRARY_SCANNED_EVENT_KIND,
        LIBRARY_SCANNED_EVENT_PATH, LIBRARY_SCANNED_EVENT_SUBSCRIPTION_ID, PROVIDER_TEST_SEND_PATH,
        PROVIDER_TEST_SEND_RESPONSE_SCHEMA, addon_manifest,
    },
    provider_registry::{NotificationProviderRegistry, select_primary_provider_output},
    provider_send::{
        NotificationProviderClients, ProviderSendFailure, send_library_scanned_event_to_providers,
    },
    template::{TemplateContext, render_template},
};

#[derive(Clone)]
pub struct AppState {
    config: Config,
    provider_clients: NotificationProviderClients,
    provider_attempt_history: ProviderAttemptHistory,
}

pub fn router(config: Config) -> Router {
    let provider_attempt_history =
        ProviderAttemptHistory::new(config.provider_attempt_history_capacity);

    Router::new()
        .route("/manifest.json", get(manifest))
        .route("/health", post(health))
        .route(LIBRARY_SCANNED_EVENT_PATH, post(library_scanned_event))
        .route(PROVIDER_TEST_SEND_PATH, post(provider_test_send))
        .route(DIAGNOSTICS_PATH, get(diagnostics))
        .layer(TraceLayer::new_for_http())
        .with_state(AppState {
            config,
            provider_clients: NotificationProviderClients::new(),
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
    let providers = NotificationProviderRegistry::new(&state.config);
    let configuration_status = providers.configuration_status();
    let expected_status = if request.manifest_id == ADDON_ID && !configuration_status.is_degraded()
    {
        AddonHealthStatus::Ok
    } else {
        AddonHealthStatus::Degraded
    };
    let template = &state.config.template;
    let provider_diagnostics = providers.diagnostics();

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
            "provider_fan_out": providers.send_path_configured(),
            "provider_send_path_count": providers.send_path_count(),
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
            "providers": provider_diagnostics.to_json_array()
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

    let payload_keys = sorted_payload_keys(&request);
    let providers = NotificationProviderRegistry::new(&state.config);
    if let Some(error) = providers.multiple_send_paths_error() {
        return Err((StatusCode::BAD_REQUEST, Json(error)));
    }
    let summary = render_template(
        providers.summary_template(),
        &TemplateContext {
            request: &request,
            payload_keys: &payload_keys,
        },
    )
    .map_err(invalid_template_error)?;

    let provider_send = send_library_scanned_event_to_providers(
        &state.config,
        &state.provider_clients,
        &state.provider_attempt_history,
        &request,
        &payload_keys,
        &summary,
    )
    .await
    .map_err(provider_send_route_error)?;
    let provider_outputs = provider_send.provider_outputs();
    let primary_provider_output = select_primary_provider_output(&provider_outputs);

    Ok(Json(AddonEventResponse {
        protocol_version: request.protocol_version,
        addon_id: request.addon_id,
        subscription_id: request.subscription_id,
        event_id: request.event_id,
        output: serde_json::json!({
            "schema": "nako.official.notification-bridge.library-scanned.event.v1",
            "accepted": true,
            "mode": provider_send.mode(),
            "attempt": request.attempt,
            "subject_kind": request.subject_kind,
            "subject_id": request.subject_id,
            "payload_keys": payload_keys,
            "provider": primary_provider_output,
            "providers": provider_outputs
        }),
    }))
}

async fn provider_test_send(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let providers = NotificationProviderRegistry::new(&state.config);
    if let Some(error) = providers.test_send_preflight_error() {
        return Err((StatusCode::BAD_REQUEST, Json(error)));
    }

    let request = provider_test_send_request();
    let payload_keys = sorted_payload_keys(&request);
    let summary = render_template(
        providers.summary_template(),
        &TemplateContext {
            request: &request,
            payload_keys: &payload_keys,
        },
    )
    .map_err(invalid_template_error)?;
    let provider_send = send_library_scanned_event_to_providers(
        &state.config,
        &state.provider_clients,
        &state.provider_attempt_history,
        &request,
        &payload_keys,
        &summary,
    )
    .await
    .map_err(provider_send_route_error)?;
    let provider_outputs = provider_send.provider_outputs();
    let primary_provider_output = select_primary_provider_output(&provider_outputs);

    Ok(Json(serde_json::json!({
        "schema": PROVIDER_TEST_SEND_RESPONSE_SCHEMA,
        "accepted": true,
        "mode": provider_send.mode(),
        "provider_send_path_count": providers.send_path_count(),
        "configuration_status": providers.configuration_status().as_str(),
        "provider": primary_provider_output,
        "providers": provider_outputs
    })))
}

fn provider_send_route_error(error: ProviderSendFailure) -> (StatusCode, Json<serde_json::Value>) {
    (error.status_code(), Json(error.into_safe_body()))
}

fn sorted_payload_keys(request: &AddonEventRequest) -> Vec<String> {
    request
        .payload
        .as_object()
        .map(|object| {
            let mut keys = object.keys().cloned().collect::<Vec<_>>();
            keys.sort();
            keys
        })
        .unwrap_or_default()
}

fn provider_test_send_request() -> AddonEventRequest {
    AddonEventRequest {
        protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
        addon_id: ADDON_ID.to_owned(),
        subscription_id: LIBRARY_SCANNED_EVENT_SUBSCRIPTION_ID.to_owned(),
        event_id: "notification-provider-test-send".to_owned(),
        event_kind: LIBRARY_SCANNED_EVENT_KIND.to_owned(),
        subject_kind: "library".to_owned(),
        subject_id: "provider-test".to_owned(),
        occurred_at: "2026-05-25T00:00:00.000Z".to_owned(),
        attempt: 1,
        payload: serde_json::json!({
            "test_send": true
        }),
    }
}

async fn diagnostics(State(state): State<AppState>) -> Html<String> {
    Html(render_diagnostics_page(
        &state.config,
        &state.provider_attempt_history,
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
        path: String,
        secret_header: Option<String>,
        body: serde_json::Value,
    }

    async fn record_webhook(
        AxumState(state): AxumState<FixtureState>,
        uri: axum::http::Uri,
        headers: HeaderMap,
        Json(body): Json<serde_json::Value>,
    ) -> StatusCode {
        let secret_header = headers
            .get("x-test-secret")
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned);
        state.requests.lock().unwrap().push(FixtureRequest {
            path: uri.path().to_owned(),
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
            .fallback(axum_post(record_webhook))
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
    async fn library_scanned_event_endpoint_sends_telegram_payload_without_raw_event_values() {
        let (target_url, fixture, handle) = spawn_http_webhook_fixture().await;
        let api_base_url = target_url.strip_suffix("/hook").unwrap().to_owned();
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
            "NAKO_NOTIFICATION_BRIDGE_TELEGRAM_ENABLED" => Some("true".to_owned()),
            "NAKO_NOTIFICATION_BRIDGE_TELEGRAM_API_BASE_URL" => Some(api_base_url.clone()),
            "NAKO_NOTIFICATION_BRIDGE_TELEGRAM_BOT_TOKEN" => {
                Some("telegram-token-should-not-appear".to_owned())
            }
            "NAKO_NOTIFICATION_BRIDGE_TELEGRAM_CHAT_ID" => {
                Some("telegram-chat-should-not-appear".to_owned())
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
        assert_eq!(payload.output["provider"]["id"], "telegram");
        assert_eq!(payload.output["provider"]["status"], "sent");
        assert!(!text.contains("127.0.0.1"));
        assert!(!text.contains("telegram-token-should-not-appear"));
        assert!(!text.contains("telegram-chat-should-not-appear"));
        assert!(!text.contains("nako_at_should_not_echo"));

        let requests = fixture.requests.lock().unwrap();
        assert_eq!(requests.len(), 1);
        assert_eq!(
            requests[0].path,
            "/bottelegram-token-should-not-appear/sendMessage"
        );
        assert_eq!(
            requests[0].body["chat_id"],
            "telegram-chat-should-not-appear"
        );
        assert_eq!(
            requests[0].body["text"],
            "Nako library.scanned event for library library-1"
        );
        let telegram_body = serde_json::to_string(&requests[0].body).unwrap();
        assert!(!telegram_body.contains("nako_at_should_not_echo"));

        handle.abort();
    }

    #[tokio::test]
    async fn provider_test_send_endpoint_sends_http_webhook_without_leaking_values() {
        let (target_url, fixture, handle) = spawn_http_webhook_fixture().await;
        let response = router(Config::from_env_lookup(|name| match name {
            "NAKO_NOTIFICATION_BRIDGE_HTTP_WEBHOOK_ENABLED" => Some("true".to_owned()),
            "NAKO_NOTIFICATION_BRIDGE_HTTP_WEBHOOK_URL" => Some(target_url.clone()),
            "NAKO_NOTIFICATION_BRIDGE_HTTP_WEBHOOK_SECRET_HEADER_NAME" => {
                Some("x-test-secret".to_owned())
            }
            "NAKO_NOTIFICATION_BRIDGE_HTTP_WEBHOOK_SHARED_SECRET" => {
                Some("test-secret-should-not-appear".to_owned())
            }
            _ => None,
        }))
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/providers/test-send")
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
        let payload: serde_json::Value = serde_json::from_str(&text).unwrap();

        assert_eq!(
            payload["schema"],
            "nako.official.notification-bridge.provider-test-send.v1"
        );
        assert_eq!(payload["accepted"], true);
        assert_eq!(payload["mode"], "provider_send");
        assert_eq!(payload["provider"]["id"], "http_webhook");
        assert_eq!(payload["provider"]["status"], "sent");
        assert_eq!(payload["provider"]["send_path_enabled"], true);
        assert_eq!(payload["provider_send_path_count"], 1);
        assert_eq!(payload["configuration_status"], "provider_send_ready");
        assert!(!text.contains("127.0.0.1"));
        assert!(!text.contains("test-secret-should-not-appear"));

        let requests = fixture.requests.lock().unwrap();
        assert_eq!(requests.len(), 1);
        assert_eq!(
            requests[0].secret_header.as_deref(),
            Some("test-secret-should-not-appear")
        );
        assert_eq!(
            requests[0].body["schema"],
            "nako.official.notification-bridge.http-webhook.library-scanned.v1"
        );
        assert_eq!(
            requests[0].body["event"]["event_id"],
            "notification-provider-test-send"
        );
        assert_eq!(requests[0].body["event"]["subject_id"], "provider-test");
        let webhook_body = serde_json::to_string(&requests[0].body).unwrap();
        assert!(!webhook_body.contains("test-secret-should-not-appear"));

        handle.abort();
    }

    #[tokio::test]
    async fn provider_test_send_endpoint_sends_telegram_without_leaking_values() {
        let (target_url, fixture, handle) = spawn_http_webhook_fixture().await;
        let api_base_url = target_url.strip_suffix("/hook").unwrap().to_owned();
        let response = router(Config::from_env_lookup(|name| match name {
            "NAKO_NOTIFICATION_BRIDGE_TELEGRAM_ENABLED" => Some("true".to_owned()),
            "NAKO_NOTIFICATION_BRIDGE_TELEGRAM_API_BASE_URL" => Some(api_base_url.clone()),
            "NAKO_NOTIFICATION_BRIDGE_TELEGRAM_BOT_TOKEN" => {
                Some("test-token-should-not-appear".to_owned())
            }
            "NAKO_NOTIFICATION_BRIDGE_TELEGRAM_CHAT_ID" => {
                Some("test-chat-should-not-appear".to_owned())
            }
            _ => None,
        }))
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/providers/test-send")
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
        let payload: serde_json::Value = serde_json::from_str(&text).unwrap();

        assert_eq!(payload["mode"], "provider_send");
        assert_eq!(payload["provider"]["id"], "telegram");
        assert_eq!(payload["provider"]["status"], "sent");
        assert_eq!(payload["provider_send_path_count"], 1);
        assert_eq!(payload["configuration_status"], "provider_send_ready");
        assert!(!text.contains("127.0.0.1"));
        assert!(!text.contains("test-token-should-not-appear"));
        assert!(!text.contains("test-chat-should-not-appear"));

        let requests = fixture.requests.lock().unwrap();
        assert_eq!(requests.len(), 1);
        assert_eq!(
            requests[0].path,
            "/bottest-token-should-not-appear/sendMessage"
        );
        assert_eq!(requests[0].body["chat_id"], "test-chat-should-not-appear");
        assert_eq!(
            requests[0].body["text"],
            "Nako library.scanned event for library provider-test"
        );

        handle.abort();
    }

    #[tokio::test]
    async fn provider_test_send_endpoint_fails_closed_without_provider_send_path() {
        let response = router(Config::default())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/providers/test-send")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let payload: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(
            payload["safe_error_code"],
            "no_notification_provider_send_path_configured"
        );
        assert_eq!(payload["configuration_status"], "ack_only");
        assert_eq!(payload["provider_send_path_count"], 0);
        assert_eq!(payload["retryable"], false);
    }

    #[tokio::test]
    async fn provider_test_send_endpoint_fails_closed_for_multiple_provider_send_paths() {
        let (http_target_url, http_fixture, http_handle) = spawn_http_webhook_fixture().await;
        let (discord_target_url, discord_fixture, discord_handle) =
            spawn_http_webhook_fixture().await;
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
                .uri("/providers/test-send")
                .body(Body::empty())
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
        assert_eq!(
            payload["configuration_status"],
            "multiple_provider_send_paths_configured"
        );
        assert_eq!(payload["provider_send_path_count"], 2);
        assert_eq!(payload["retryable"], false);
        assert!(!text.contains("127.0.0.1"));
        assert_eq!(http_fixture.requests.lock().unwrap().len(), 0);
        assert_eq!(discord_fixture.requests.lock().unwrap().len(), 0);

        http_handle.abort();
        discord_handle.abort();
    }

    #[tokio::test]
    async fn provider_test_send_endpoint_fails_closed_for_invalid_provider_configuration() {
        let response = router(Config::from_env_lookup(|name| match name {
            "NAKO_NOTIFICATION_BRIDGE_HTTP_WEBHOOK_ENABLED" => Some("true".to_owned()),
            "NAKO_NOTIFICATION_BRIDGE_HTTP_WEBHOOK_URL" => Some("file:///tmp/hook".to_owned()),
            "NAKO_NOTIFICATION_BRIDGE_HTTP_WEBHOOK_SHARED_SECRET" => {
                Some("test-secret-should-not-appear".to_owned())
            }
            _ => None,
        }))
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/providers/test-send")
                .body(Body::empty())
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
            "notification_provider_configuration_invalid"
        );
        assert_eq!(
            payload["configuration_status"],
            "provider_configuration_invalid"
        );
        assert_eq!(payload["provider_send_path_count"], 0);
        assert_eq!(payload["retryable"], false);
        assert!(!text.contains("file:///tmp/hook"));
        assert!(!text.contains("test-secret-should-not-appear"));
    }

    #[tokio::test]
    async fn provider_test_send_endpoint_fails_closed_for_invalid_enabled_provider_template() {
        let (target_url, fixture, handle) = spawn_http_webhook_fixture().await;
        let response = router(Config::from_env_lookup(|name| match name {
            "NAKO_NOTIFICATION_BRIDGE_HTTP_WEBHOOK_ENABLED" => Some("true".to_owned()),
            "NAKO_NOTIFICATION_BRIDGE_HTTP_WEBHOOK_URL" => Some(target_url.clone()),
            "NAKO_NOTIFICATION_BRIDGE_TEMPLATE_SUMMARY" => Some("{{payload.secret}}".to_owned()),
            _ => None,
        }))
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/providers/test-send")
                .body(Body::empty())
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
        assert_eq!(payload["configuration_status"], "template_invalid");
        assert_eq!(payload["provider_send_path_count"], 1);
        assert_eq!(payload["retryable"], false);
        assert!(!text.contains("payload.secret"));
        assert!(!text.contains("127.0.0.1"));
        assert_eq!(fixture.requests.lock().unwrap().len(), 0);

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
            3
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
        assert_eq!(payload.diagnostics["providers"][2]["id"], "telegram");
        assert_eq!(payload.diagnostics["providers"][2]["enabled"], false);
        assert_eq!(payload.diagnostics["providers"][2]["status"], "disabled");
        assert_eq!(
            payload.diagnostics["providers"][2]["send_path_enabled"],
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
    async fn health_endpoint_reports_telegram_config_without_leaking_values() {
        let request = AddonHealthCheckRequest {
            protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
            manifest_id: ADDON_ID.to_owned(),
            request_id: "health-1".to_owned(),
            expected_addon_version: ADDON_VERSION.to_owned(),
            expected_resource_count: crate::manifest::container_manifest().resources.len(),
        };
        let response = router(Config::from_env_lookup(|name| match name {
            "NAKO_NOTIFICATION_BRIDGE_TELEGRAM_ENABLED" => Some("true".to_owned()),
            "NAKO_NOTIFICATION_BRIDGE_TELEGRAM_API_BASE_URL" => {
                Some("https://api.telegram.example".to_owned())
            }
            "NAKO_NOTIFICATION_BRIDGE_TELEGRAM_BOT_TOKEN" => {
                Some("telegram-token-should-not-appear".to_owned())
            }
            "NAKO_NOTIFICATION_BRIDGE_TELEGRAM_CHAT_ID" => {
                Some("telegram-chat-should-not-appear".to_owned())
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
        assert_eq!(payload.diagnostics["providers"][2]["id"], "telegram");
        assert_eq!(payload.diagnostics["providers"][2]["enabled"], true);
        assert_eq!(payload.diagnostics["providers"][2]["status"], "configured");
        assert_eq!(
            payload.diagnostics["providers"][2]["api_base_url_configured"],
            true
        );
        assert_eq!(
            payload.diagnostics["providers"][2]["api_base_url_valid"],
            true
        );
        assert_eq!(
            payload.diagnostics["providers"][2]["bot_token_configured"],
            true
        );
        assert_eq!(
            payload.diagnostics["providers"][2]["chat_id_configured"],
            true
        );
        assert_eq!(
            payload.diagnostics["providers"][2]["send_path_enabled"],
            true
        );

        let diagnostics = serde_json::to_string(&payload.diagnostics).unwrap();
        assert!(!diagnostics.contains("api.telegram.example"));
        assert!(!diagnostics.contains("telegram-token-should-not-appear"));
        assert!(!diagnostics.contains("telegram-chat-should-not-appear"));
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

    #[tokio::test]
    async fn diagnostics_page_reports_telegram_status_without_leaking_values() {
        let response = router(Config::from_env_lookup(|name| match name {
            "NAKO_NOTIFICATION_BRIDGE_TELEGRAM_ENABLED" => Some("true".to_owned()),
            "NAKO_NOTIFICATION_BRIDGE_TELEGRAM_API_BASE_URL" => {
                Some("https://api.telegram.example".to_owned())
            }
            "NAKO_NOTIFICATION_BRIDGE_TELEGRAM_BOT_TOKEN" => {
                Some("telegram-token-should-not-appear".to_owned())
            }
            "NAKO_NOTIFICATION_BRIDGE_TELEGRAM_CHAT_ID" => {
                Some("telegram-chat-should-not-appear".to_owned())
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
        assert!(text.contains("Telegram provider status: configured"));
        assert!(text.contains("Telegram enabled: yes"));
        assert!(text.contains("Telegram API base URL configured: yes"));
        assert!(text.contains("Telegram API base URL valid: yes"));
        assert!(text.contains("Telegram bot token configured: yes"));
        assert!(text.contains("Telegram chat id configured: yes"));
        assert!(text.contains("Telegram send path enabled: yes"));
        assert!(!text.contains("api.telegram.example"));
        assert!(!text.contains("telegram-token-should-not-appear"));
        assert!(!text.contains("telegram-chat-should-not-appear"));
    }
}
