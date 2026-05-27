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
    AddonResourceResponse,
};
use tower_http::trace::TraceLayer;

use crate::{
    Config,
    domain::ResourceSearchRequest,
    engine::{ResourceSearchError, ResourceSearchRuntime},
    manifest::{
        ADDON_ID, ADDON_NAME, ADDON_VERSION, DIAGNOSTICS_PATH, RESOURCE_SEARCH_RESOURCE_PATH,
        addon_manifest, container_manifest,
    },
};

#[derive(Clone)]
pub struct AppState {
    config: Config,
    runtime: ResourceSearchRuntime,
}

pub fn router(config: Config) -> Router {
    let runtime = ResourceSearchRuntime::new(config.clone());

    Router::new()
        .route("/manifest.json", get(manifest))
        .route("/health", post(health))
        .route(RESOURCE_SEARCH_RESOURCE_PATH, post(resource_search))
        .route(DIAGNOSTICS_PATH, get(diagnostics))
        .layer(TraceLayer::new_for_http())
        .with_state(AppState { config, runtime })
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
        && state.runtime.provider_count() > 0
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
        diagnostics: diagnostics_payload(&state),
    })
}

async fn resource_search(
    State(state): State<AppState>,
    Json(request): Json<AddonResourceRequest>,
) -> Result<Json<AddonResourceResponse>, (StatusCode, Json<serde_json::Value>)> {
    validate_resource_envelope(&request)?;

    let payload = serde_json::from_value::<ResourceSearchRequest>(request.payload.clone())
        .map_err(|_| safe_bad_request("invalid_resource_search_payload"))?;
    let response = state
        .runtime
        .search(payload)
        .await
        .map_err(search_error_response)?;

    Ok(Json(AddonResourceResponse {
        protocol_version: request.protocol_version,
        addon_id: request.addon_id,
        resource: request.resource,
        request_id: request.request_id,
        payload: serde_json::to_value(response).expect("resource search response serializes"),
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
  <p>Resource path: {RESOURCE_SEARCH_RESOURCE_PATH}</p>
  <p>Protocol resource: automation alpha</p>
  <p>Configured provider count: {}</p>
  <p>Runtime provider count: {}</p>
  <p>Providers: {}</p>
  <p>Default limit: {}</p>
  <p>Max limit: {}</p>
  <p>Search timeout ms: {}</p>
  <p>This alpha sidecar will move to a dedicated resource_search protocol resource after the Nako host contract lands.</p>
</body>
</html>"#,
        state.config.base_url,
        state.config.enabled_provider_count(),
        state.runtime.provider_count(),
        state.runtime.provider_ids().join(", "),
        state.config.default_limit,
        state.config.max_limit,
        state.config.search_timeout_ms,
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
    if request.resource != AddonResource::Automation {
        return Err(safe_bad_request("invalid_resource"));
    }

    Ok(())
}

fn search_error_response(error: ResourceSearchError) -> (StatusCode, Json<serde_json::Value>) {
    match error {
        ResourceSearchError::EmptyQuery => safe_bad_request("empty_query"),
    }
}

fn diagnostics_payload(state: &AppState) -> serde_json::Value {
    serde_json::json!({
        "safe_note": "resource search sidecar is reachable",
        "protocol_resource": "automation_alpha",
        "future_protocol_resource": "resource_search",
        "configured_provider_count": state.config.enabled_provider_count(),
        "runtime_provider_count": state.runtime.provider_count(),
        "providers": state.runtime.provider_ids(),
        "default_limit": state.config.default_limit,
        "max_limit": state.config.max_limit,
        "search_timeout_ms": state.config.search_timeout_ms
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

#[cfg(test)]
mod tests {
    use axum::{body::Body, http::Request};
    use nako_addon_protocol::{AddonManifest, AddonScope, validate_manifest};
    use tower::ServiceExt;

    use crate::domain::{
        RESOURCE_SEARCH_RESPONSE_SCHEMA, ResourceLinkType, ResourceSearchResponse,
    };

    use super::*;

    #[tokio::test]
    async fn manifest_endpoint_returns_valid_resource_search_manifest() {
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
        assert_eq!(manifest.resources[0].kind, AddonResource::Automation);
        assert_eq!(manifest.scopes, vec![AddonScope::AutomationRun]);
    }

    #[tokio::test]
    async fn resource_search_returns_typed_results() {
        let response = router(Config::default())
            .oneshot(resource_request(serde_json::json!({
                "query": "Demo Movie",
                "limit": 10
            })))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let payload = resource_response_payload(response).await;
        let response: ResourceSearchResponse = serde_json::from_value(payload).unwrap();

        assert_eq!(response.schema, RESOURCE_SEARCH_RESPONSE_SCHEMA);
        assert_eq!(response.query, "Demo Movie");
        assert_eq!(response.total, 2);
        assert!(
            response
                .merged_by_type
                .contains_key(&ResourceLinkType::Quark)
        );
        assert!(
            response
                .merged_by_type
                .contains_key(&ResourceLinkType::Aliyun)
        );
    }

    #[tokio::test]
    async fn resource_search_rejects_empty_query() {
        let response = router(Config::default())
            .oneshot(resource_request(serde_json::json!({ "query": "   " })))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let payload: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(payload["safe_error_code"], "empty_query");
    }

    #[tokio::test]
    async fn resource_search_rejects_invalid_resource_envelope() {
        let request = AddonResourceRequest {
            protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
            addon_id: ADDON_ID.to_owned(),
            resource: AddonResource::Metadata,
            request_id: "request-1".to_owned(),
            payload: serde_json::json!({ "query": "Demo Movie" }),
        };

        let response = router(Config::default())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(RESOURCE_SEARCH_RESOURCE_PATH)
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
    async fn health_and_diagnostics_are_redaction_safe() {
        let health_request = AddonHealthCheckRequest {
            protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
            manifest_id: ADDON_ID.to_owned(),
            request_id: "health-1".to_owned(),
            expected_addon_version: ADDON_VERSION.to_owned(),
            expected_resource_count: 1,
        };

        let response = router(Config::default())
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
        let health: AddonHealthCheckResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(health.status, AddonHealthStatus::Ok);
        assert_eq!(
            health.diagnostics["future_protocol_resource"],
            "resource_search"
        );

        let response = router(Config::default())
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
        assert!(text.contains("resource_search"));
        assert!(!text.contains("password"));
    }

    fn resource_request(payload: serde_json::Value) -> Request<Body> {
        let request = AddonResourceRequest {
            protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
            addon_id: ADDON_ID.to_owned(),
            resource: AddonResource::Automation,
            request_id: "request-1".to_owned(),
            payload,
        };

        Request::builder()
            .method("POST")
            .uri(RESOURCE_SEARCH_RESOURCE_PATH)
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
        assert_eq!(response.resource, AddonResource::Automation);
        response.payload
    }
}
