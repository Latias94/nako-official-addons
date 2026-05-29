use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::Html,
    routing::{get, post},
};
use nako_addon_protocol::{
    ADDON_PROTOCOL_VERSION, AddonHealthCheckRequest, AddonHealthCheckResponse,
    AddonHealthManifestFacts, AddonHealthStatus, AddonResource, AddonResourceRequest,
    AddonResourceResponse, AddonSubtitleSearchRequest,
};
use tower_http::trace::TraceLayer;

use crate::{
    Config,
    manifest::{
        ADDON_ID, ADDON_NAME, ADDON_VERSION, DIAGNOSTICS_PATH, SUBTITLE_REQUEST_SCHEMA,
        SUBTITLE_RESOURCE_PATH, addon_manifest, container_manifest,
    },
    subtitles::search_subtitles,
};

#[derive(Clone)]
pub struct AppState {
    config: Config,
}

pub fn router(config: Config) -> Router {
    Router::new()
        .route("/manifest.json", get(manifest))
        .route("/health", post(health))
        .route(SUBTITLE_RESOURCE_PATH, post(subtitle))
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
    let status = if request.protocol_version == ADDON_PROTOCOL_VERSION
        && request.manifest_id == ADDON_ID
        && state.config.active_provider_count() > 0
    {
        AddonHealthStatus::Ok
    } else {
        AddonHealthStatus::Degraded
    };

    Json(AddonHealthCheckResponse {
        protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
        manifest_id: request.manifest_id,
        status,
        checked_at: "2026-05-28T00:00:00.000Z".to_owned(),
        manifest: AddonHealthManifestFacts {
            addon_version: ADDON_VERSION.to_owned(),
            resource_count: container_manifest().resources.len(),
        },
        diagnostics: diagnostics_payload(&state.config),
    })
}

async fn subtitle(
    State(state): State<AppState>,
    Json(request): Json<AddonResourceRequest>,
) -> Result<Json<AddonResourceResponse>, (StatusCode, Json<serde_json::Value>)> {
    validate_resource_envelope(&request)?;
    let payload = serde_json::from_value::<AddonSubtitleSearchRequest>(request.payload.clone())
        .map_err(|_| safe_bad_request("invalid_subtitle_payload"))?;
    if payload.schema != SUBTITLE_REQUEST_SCHEMA {
        return Err(safe_bad_request("invalid_subtitle_schema"));
    }
    let response = search_subtitles(&state.config, payload)
        .ok_or_else(|| safe_bad_request("empty_subtitle_query"))?;

    Ok(Json(AddonResourceResponse {
        protocol_version: request.protocol_version,
        addon_id: request.addon_id,
        resource: request.resource,
        request_id: request.request_id,
        payload: serde_json::to_value(response).expect("subtitle response serializes"),
        artifacts: Vec::new(),
    }))
}

async fn diagnostics(State(state): State<AppState>) -> Html<String> {
    Html(format!(
        r#"<!doctype html>
<html lang="en">
<head><meta charset="utf-8"><title>{ADDON_NAME}</title></head>
<body>
  <h1>{ADDON_NAME}</h1>
  <p>Base URL: {}</p>
  <p>Subtitle resource: {SUBTITLE_RESOURCE_PATH}</p>
  <p>Fixture provider enabled: {}</p>
  <p>Default language: {}</p>
  <p>Default limit: {}</p>
  <p>Max limit: {}</p>
  <p>This sidecar is read-only and does not write subtitle files.</p>
</body>
</html>"#,
        state.config.base_url,
        yes_no_label(state.config.fixture_provider_enabled),
        state.config.default_language,
        state.config.default_limit,
        state.config.max_limit
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
    if request.resource != AddonResource::Subtitle {
        return Err(safe_bad_request("invalid_resource"));
    }

    Ok(())
}

fn diagnostics_payload(config: &Config) -> serde_json::Value {
    serde_json::json!({
        "safe_note": "subtitle provider sidecar is reachable",
        "provider_registry": [{
            "provider_id": "fixture",
            "active": config.fixture_provider_enabled,
            "source_policy": "official_fixture"
        }],
        "active_provider_count": config.active_provider_count(),
        "default_language": config.default_language,
        "default_limit": config.default_limit,
        "max_limit": config.max_limit,
        "read_only": true
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
    use nako_addon_protocol::{AddonManifest, AddonScope, validate_manifest};
    use tower::ServiceExt;

    use super::*;

    #[tokio::test]
    async fn manifest_endpoint_returns_valid_subtitle_manifest() {
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
        assert_eq!(manifest.resources[0].kind, AddonResource::Subtitle);
        assert_eq!(manifest.scopes, vec![AddonScope::SubtitleRead]);
    }

    #[tokio::test]
    async fn subtitle_resource_returns_inline_fixture_candidates() {
        let response = router(Config::default())
            .oneshot(subtitle_request(serde_json::json!({
                "schema": SUBTITLE_REQUEST_SCHEMA,
                "query": "Demo Movie",
                "languages": ["zh-CN", "en"],
                "limit": 2
            })))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let payload = resource_response_payload(response).await;
        assert_eq!(
            payload["schema"],
            nako_addon_protocol::ADDON_SUBTITLE_RESPONSE_SCHEMA
        );
        assert_eq!(payload["total"], 2);
        assert_eq!(payload["subtitles"][0]["source"], "fixture");
        assert_eq!(payload["subtitles"][0]["delivery"]["kind"], "inline");
    }

    #[tokio::test]
    async fn subtitle_resource_rejects_invalid_resource_envelope() {
        let mut request = resource_envelope(serde_json::json!({
            "schema": SUBTITLE_REQUEST_SCHEMA,
            "query": "Demo Movie"
        }));
        request.resource = AddonResource::Metadata;

        let response = router(Config::default())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(SUBTITLE_RESOURCE_PATH)
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&request).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn health_and_diagnostics_are_redaction_safe() {
        let response = router(Config::default())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/health")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::to_vec(&AddonHealthCheckRequest {
                            protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
                            manifest_id: ADDON_ID.to_owned(),
                            request_id: "health-1".to_owned(),
                            expected_addon_version: ADDON_VERSION.to_owned(),
                            expected_resource_count: 1,
                        })
                        .unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();
        assert!(!text.contains("Bearer "));
        assert!(!text.contains("nako_at_"));
        assert!(!text.contains("file://"));
    }

    fn subtitle_request(payload: serde_json::Value) -> Request<Body> {
        Request::builder()
            .method("POST")
            .uri(SUBTITLE_RESOURCE_PATH)
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::to_vec(&resource_envelope(payload)).unwrap(),
            ))
            .unwrap()
    }

    fn resource_envelope(payload: serde_json::Value) -> AddonResourceRequest {
        AddonResourceRequest {
            protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
            addon_id: ADDON_ID.to_owned(),
            resource: AddonResource::Subtitle,
            request_id: "req-subtitle".to_owned(),
            payload,
        }
    }

    async fn resource_response_payload(response: axum::response::Response) -> serde_json::Value {
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let response: AddonResourceResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(response.resource, AddonResource::Subtitle);
        response.payload
    }
}
