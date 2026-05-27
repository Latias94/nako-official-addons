use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::Html,
    routing::{get, post},
};
use nako_addon_protocol::{
    ADDON_PROTOCOL_VERSION, AddonHealthCheckRequest, AddonHealthCheckResponse,
    AddonHealthManifestFacts, AddonHealthStatus, AddonRendererAdapterProtocol,
    AddonRendererAdapterReadinessStatus, AddonRendererAdapterRequest, AddonRendererAdapterResponse,
    AddonResource, AddonResourceRequest, AddonResourceResponse,
};
use tower_http::trace::TraceLayer;

use crate::{
    Config,
    chromecast::{discover_targets, dispatch_command, readiness},
    manifest::{
        ADDON_ID, ADDON_VERSION, DIAGNOSTICS_PATH, RENDERER_ADAPTER_RESOURCE_PATH, addon_manifest,
        container_manifest,
    },
};

#[derive(Clone)]
pub struct AppState {
    config: Config,
}

pub fn router(config: Config) -> Router {
    Router::new()
        .route("/manifest.json", get(manifest))
        .route("/health", post(health))
        .route(RENDERER_ADAPTER_RESOURCE_PATH, post(renderer_adapter))
        .route(DIAGNOSTICS_PATH, get(diagnostics))
        .layer(TraceLayer::new_for_http())
        .with_state(AppState { config })
}

async fn manifest(State(state): State<AppState>) -> Json<nako_addon_protocol::AddonManifest> {
    Json(addon_manifest(&state.config))
}

async fn health(
    State(state): State<AppState>,
    Json(request): Json<AddonHealthCheckRequest>,
) -> Json<AddonHealthCheckResponse> {
    let readiness = readiness(&state.config);
    let status = if request.protocol_version == ADDON_PROTOCOL_VERSION
        && request.manifest_id == ADDON_ID
        && readiness.status == AddonRendererAdapterReadinessStatus::Ready
    {
        AddonHealthStatus::Ok
    } else {
        AddonHealthStatus::Degraded
    };

    Json(AddonHealthCheckResponse {
        protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
        manifest_id: request.manifest_id,
        status,
        checked_at: "2026-05-27T00:00:00.000Z".to_owned(),
        manifest: AddonHealthManifestFacts {
            addon_version: ADDON_VERSION.to_owned(),
            resource_count: container_manifest().resources.len(),
        },
        diagnostics: diagnostics_payload(&state.config, readiness),
    })
}

async fn renderer_adapter(
    State(state): State<AppState>,
    Json(request): Json<AddonResourceRequest>,
) -> Result<Json<AddonResourceResponse>, (StatusCode, Json<serde_json::Value>)> {
    validate_resource_envelope(&request)?;

    let payload = serde_json::from_value::<AddonRendererAdapterRequest>(request.payload.clone())
        .map_err(|_| safe_bad_request("invalid_renderer_adapter_payload"))?;

    let response_payload = match payload {
        AddonRendererAdapterRequest::InspectReadiness { protocol } => {
            ensure_chromecast_protocol(protocol)?;
            serde_json::to_value(AddonRendererAdapterResponse::Readiness {
                readiness: readiness(&state.config),
            })
            .expect("renderer readiness response serializes")
        }
        AddonRendererAdapterRequest::DiscoverTargets {
            protocol,
            timeout_ms,
        } => {
            ensure_chromecast_protocol(protocol)?;
            let targets = discover_targets(&state.config, timeout_ms).await;
            serde_json::to_value(AddonRendererAdapterResponse::Targets { targets })
                .expect("renderer target response serializes")
        }
        AddonRendererAdapterRequest::DispatchCommand { protocol, envelope } => {
            ensure_chromecast_protocol(protocol)?;
            let result = dispatch_command(&state.config, &envelope).await;
            serde_json::to_value(AddonRendererAdapterResponse::CommandResult { result })
                .expect("renderer command response serializes")
        }
    };

    Ok(Json(AddonResourceResponse {
        protocol_version: request.protocol_version,
        addon_id: request.addon_id,
        resource: request.resource,
        request_id: request.request_id,
        payload: response_payload,
        artifacts: Vec::new(),
    }))
}

async fn diagnostics(State(state): State<AppState>) -> Html<String> {
    let readiness = readiness(&state.config);
    let receiver_app_id_configured = yes_no_label(state.config.receiver_app_id_configured());
    let manual_devices_json_valid = yes_no_label(state.config.manual_devices_json_valid);
    let live_discovery_enabled = yes_no_label(state.config.live_discovery_enabled);
    let live_control_enabled = yes_no_label(state.config.live_control_enabled);
    let manual_device_count = state.config.manual_device_count();
    Html(format!(
        r#"<!doctype html>
<html lang="en">
<head><meta charset="utf-8"><title>Nako Chromecast Renderer</title></head>
<body>
  <h1>Nako Chromecast Renderer</h1>
  <p>Base URL: {}</p>
  <p>Renderer adapter resource: {}</p>
  <p>Readiness status: {}</p>
  <p>Readiness reason: {}</p>
  <p>Receiver app id configured: {receiver_app_id_configured}</p>
  <p>Manual device count: {manual_device_count}</p>
  <p>Manual devices JSON valid: {manual_devices_json_valid}</p>
  <p>Live discovery enabled: {live_discovery_enabled}</p>
  <p>Live control enabled: {live_control_enabled}</p>
  <p>Protocol library: oxicast</p>
  <p>This page is hosted by the Addon Sidecar and is not trusted Nako Admin UI.</p>
</body>
</html>"#,
        state.config.base_url,
        RENDERER_ADAPTER_RESOURCE_PATH,
        readiness.status.as_str(),
        readiness.reason_code
    ))
}

fn validate_resource_envelope(
    request: &AddonResourceRequest,
) -> Result<(), (StatusCode, Json<serde_json::Value>)> {
    if request.protocol_version != ADDON_PROTOCOL_VERSION {
        return Err(safe_bad_request("invalid_protocol_version"));
    }
    if request.addon_id != ADDON_ID {
        return Err(safe_bad_request("invalid_addon_id"));
    }
    if request.resource != AddonResource::RendererAdapter {
        return Err(safe_bad_request("invalid_resource"));
    }

    Ok(())
}

fn ensure_chromecast_protocol(
    protocol: AddonRendererAdapterProtocol,
) -> Result<(), (StatusCode, Json<serde_json::Value>)> {
    if protocol == AddonRendererAdapterProtocol::Chromecast {
        Ok(())
    } else {
        Err(safe_bad_request("unsupported_renderer_adapter_protocol"))
    }
}

fn diagnostics_payload(
    config: &Config,
    readiness: nako_addon_protocol::AddonRendererAdapterReadiness,
) -> serde_json::Value {
    serde_json::json!({
        "safe_note": "chromecast renderer adapter sidecar is reachable",
        "readiness": {
            "protocol": readiness.protocol.as_str(),
            "status": readiness.status.as_str(),
            "reason_code": readiness.reason_code,
            "safe_message": readiness.safe_message,
        },
        "receiver_app_id_configured": config.receiver_app_id_configured(),
        "manual_device_count": config.manual_device_count(),
        "manual_devices_json_valid": config.manual_devices_json_valid,
        "live_discovery_enabled": config.live_discovery_enabled,
        "live_control_enabled": config.live_control_enabled,
        "discovery_timeout_ms": config.discovery_timeout_ms,
        "command_timeout_ms": config.command_timeout_ms,
        "protocol_library": "oxicast"
    })
}

fn safe_bad_request(safe_error_code: &str) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::BAD_REQUEST,
        Json(serde_json::json!({
            "safe_error_code": safe_error_code,
            "retryable": false
        })),
    )
}

const fn yes_no_label(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}

#[cfg(test)]
mod tests {
    use axum::{body::Body, http::Request};
    use nako_addon_protocol::{
        AddonManifest, AddonRendererAdapterCommand, AddonRendererAdapterCommandEnvelope,
        AddonRendererAdapterCommandState, AddonRendererAdapterTransport,
        AddonRendererAdapterTransportMode, AddonRendererAdapterTransportUrl,
        AddonRendererAdapterTransportUrlKind, AddonScope, validate_manifest,
    };
    use tower::ServiceExt;

    use super::*;
    use crate::ManualChromecastDevice;

    #[tokio::test]
    async fn manifest_endpoint_returns_valid_chromecast_renderer_manifest() {
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
        let manifest: AddonManifest = serde_json::from_slice(&body).unwrap();

        validate_manifest(&manifest).unwrap();
        assert_eq!(manifest.id, ADDON_ID);
        assert_eq!(manifest.resources[0].kind, AddonResource::RendererAdapter);
        assert_eq!(
            manifest.scopes,
            vec![
                AddonScope::RendererAdapterRead,
                AddonScope::RendererAdapterControl
            ]
        );
    }

    #[tokio::test]
    async fn renderer_adapter_readiness_resource_returns_safe_payload() {
        let response = router(config_with_manual_device())
            .oneshot(resource_request(
                AddonRendererAdapterRequest::InspectReadiness {
                    protocol: AddonRendererAdapterProtocol::Chromecast,
                },
            ))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let payload = resource_response_payload(response).await;
        let response: AddonRendererAdapterResponse = serde_json::from_value(payload).unwrap();

        let AddonRendererAdapterResponse::Readiness { readiness } = response else {
            panic!("expected readiness response");
        };
        assert_eq!(readiness.status, AddonRendererAdapterReadinessStatus::Ready);
        assert_eq!(readiness.reason_code, "target_source_configured");
    }

    #[tokio::test]
    async fn renderer_adapter_manual_discovery_returns_target_without_host() {
        let response = router(config_with_manual_device())
            .oneshot(resource_request(
                AddonRendererAdapterRequest::DiscoverTargets {
                    protocol: AddonRendererAdapterProtocol::Chromecast,
                    timeout_ms: Some(250),
                },
            ))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let payload = resource_response_payload(response).await;
        let response: AddonRendererAdapterResponse =
            serde_json::from_value(payload.clone()).unwrap();

        let AddonRendererAdapterResponse::Targets { targets } = response else {
            panic!("expected targets response");
        };
        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].stable_device_id, "living-room");
        assert_eq!(
            targets[0].target_kind,
            AddonRendererAdapterProtocol::Chromecast
        );

        let json = serde_json::to_string(&payload).unwrap();
        assert!(!json.contains("192.168.1.50"));
    }

    #[tokio::test]
    async fn renderer_adapter_dispatch_command_accepts_cast_safe_transport() {
        let response = router(config_with_manual_device())
            .oneshot(resource_request(
                AddonRendererAdapterRequest::DispatchCommand {
                    protocol: AddonRendererAdapterProtocol::Chromecast,
                    envelope: play_envelope("https://nako.local/cast/media-ticket"),
                },
            ))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let payload = resource_response_payload(response).await;
        let response: AddonRendererAdapterResponse =
            serde_json::from_value(payload.clone()).unwrap();

        let AddonRendererAdapterResponse::CommandResult { result } = response else {
            panic!("expected command result response");
        };
        assert_eq!(result.state, AddonRendererAdapterCommandState::Accepted);
        assert_eq!(result.safe_reason_code.as_deref(), Some("plan_only"));

        let json = serde_json::to_string(&payload).unwrap();
        assert!(!json.contains("media-ticket"));
    }

    #[tokio::test]
    async fn renderer_adapter_rejects_forbidden_transport_facts() {
        let response = router(config_with_manual_device())
            .oneshot(resource_request(
                AddonRendererAdapterRequest::DispatchCommand {
                    protocol: AddonRendererAdapterProtocol::Chromecast,
                    envelope: play_envelope("file:///media/movie.mp4"),
                },
            ))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let payload = resource_response_payload(response).await;
        let response: AddonRendererAdapterResponse = serde_json::from_value(payload).unwrap();

        let AddonRendererAdapterResponse::CommandResult { result } = response else {
            panic!("expected command result response");
        };
        assert_eq!(result.state, AddonRendererAdapterCommandState::Rejected);
        assert_eq!(
            result.safe_reason_code.as_deref(),
            Some("unsafe_transport_url")
        );
    }

    #[tokio::test]
    async fn renderer_adapter_rejects_invalid_resource_envelope() {
        let request = AddonResourceRequest {
            protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
            addon_id: ADDON_ID.to_owned(),
            resource: AddonResource::Metadata,
            request_id: "request-1".to_owned(),
            payload: serde_json::to_value(AddonRendererAdapterRequest::InspectReadiness {
                protocol: AddonRendererAdapterProtocol::Chromecast,
            })
            .unwrap(),
        };

        let response = router(config_with_manual_device())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(RENDERER_ADAPTER_RESOURCE_PATH)
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
        let payload: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(payload["safe_error_code"], "invalid_resource");
    }

    #[tokio::test]
    async fn health_and_diagnostics_do_not_leak_configured_hosts() {
        let config = config_with_manual_device();
        let health_request = AddonHealthCheckRequest {
            protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
            manifest_id: ADDON_ID.to_owned(),
            request_id: "health-1".to_owned(),
            expected_addon_version: ADDON_VERSION.to_owned(),
            expected_resource_count: 1,
        };
        let response = router(config.clone())
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
        assert!(!text.contains("192.168.1.50"));

        let response = router(config)
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
        assert!(!text.contains("192.168.1.50"));
        assert!(text.contains("Manual device count: 1"));
    }

    fn resource_request(payload: AddonRendererAdapterRequest) -> Request<Body> {
        let request = AddonResourceRequest {
            protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
            addon_id: ADDON_ID.to_owned(),
            resource: AddonResource::RendererAdapter,
            request_id: "request-1".to_owned(),
            payload: serde_json::to_value(payload).unwrap(),
        };

        Request::builder()
            .method("POST")
            .uri(RENDERER_ADAPTER_RESOURCE_PATH)
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&request).unwrap()))
            .unwrap()
    }

    async fn resource_response_payload(response: axum::response::Response) -> serde_json::Value {
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let response: AddonResourceResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(response.addon_id, ADDON_ID);
        assert_eq!(response.resource, AddonResource::RendererAdapter);
        response.payload
    }

    fn config_with_manual_device() -> Config {
        Config {
            manual_devices: vec![ManualChromecastDevice {
                stable_device_id: "living-room".to_owned(),
                display_name: "Living Room".to_owned(),
                host: "192.168.1.50".to_owned(),
                port: 8009,
                model: Some("Chromecast".to_owned()),
            }],
            ..Config::default()
        }
    }

    fn play_envelope(url: &str) -> AddonRendererAdapterCommandEnvelope {
        AddonRendererAdapterCommandEnvelope {
            adapter_id: ADDON_ID.to_owned(),
            stable_device_id: "living-room".to_owned(),
            target_kind: AddonRendererAdapterProtocol::Chromecast,
            renderer_session_id: "renderer-session-1".to_owned(),
            playback_session_id: "playback-session-1".to_owned(),
            source_id: "source-1".to_owned(),
            command: AddonRendererAdapterCommand::Play,
            position_ms: Some(12_000),
            volume_percent: Some(50),
            transport: AddonRendererAdapterTransport {
                mode: AddonRendererAdapterTransportMode::Direct,
                expires_at: "2026-05-27T12:00:00.000Z".to_owned(),
                urls: vec![AddonRendererAdapterTransportUrl {
                    kind: AddonRendererAdapterTransportUrlKind::Stream,
                    url: url.to_owned(),
                    content_type: "video/mp4".to_owned(),
                    supports_range_requests: true,
                }],
            },
        }
    }
}
