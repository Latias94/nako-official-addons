use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::Html,
    routing::{get, post},
};
use nako_addon_protocol::{
    ADDON_EXTERNAL_ACQUISITION_ACTION_REQUEST_SCHEMA, ADDON_PROTOCOL_VERSION,
    AddonExternalAcquisitionActionRequest, AddonHealthCheckRequest, AddonHealthCheckResponse,
    AddonHealthManifestFacts, AddonHealthStatus, AddonTaskRequest, AddonTaskResponse,
};
use tower_http::trace::TraceLayer;

use crate::{
    Config,
    manifest::{
        ACTION_TASK_ID, ACTION_TASK_PATH, ADDON_ID, ADDON_NAME, ADDON_VERSION, DIAGNOSTICS_PATH,
        addon_manifest, container_manifest,
    },
    runner::FixtureRunner,
};

#[derive(Clone)]
pub struct AppState {
    config: Config,
    runner: FixtureRunner,
}

pub fn router(config: Config) -> Router {
    let runner = FixtureRunner::new(config.clone());

    Router::new()
        .route("/manifest.json", get(manifest))
        .route("/health", post(health))
        .route(ACTION_TASK_PATH, post(external_acquisition_action))
        .route(DIAGNOSTICS_PATH, get(diagnostics))
        .layer(TraceLayer::new_for_http())
        .with_state(AppState { config, runner })
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
        && state.config.active_profile_count() > 0
    {
        AddonHealthStatus::Ok
    } else {
        AddonHealthStatus::Degraded
    };

    Json(AddonHealthCheckResponse {
        protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
        manifest_id: request.manifest_id,
        status,
        checked_at: "2026-05-29T00:00:00.000Z".to_owned(),
        manifest: AddonHealthManifestFacts {
            addon_version: ADDON_VERSION.to_owned(),
            resource_count: container_manifest().resources.len(),
        },
        diagnostics: state.runner.diagnostics(),
    })
}

async fn external_acquisition_action(
    State(state): State<AppState>,
    Json(request): Json<AddonTaskRequest>,
) -> Result<Json<AddonTaskResponse>, (StatusCode, Json<serde_json::Value>)> {
    validate_task_envelope(&request)?;
    let payload = decode_action_payload(request.payload.clone())?;
    let output = match state.runner.handle_action(payload).await {
        Ok(response) => response,
        Err(error) => error.to_response(),
    };

    Ok(Json(AddonTaskResponse {
        protocol_version: request.protocol_version,
        addon_id: request.addon_id,
        task_id: request.task_id,
        job_id: request.job_id,
        request_id: request.request_id,
        output: serde_json::to_value(output).expect("external acquisition response serializes"),
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
  <p>Action task: {ACTION_TASK_ID}</p>
  <p>Task path: {ACTION_TASK_PATH}</p>
  <p>Default runner profile: {}</p>
  <p>Fixture profile enabled: {}</p>
  <p>External network calls: no</p>
  <p>This fixture accepts only host-owned opaque target references.</p>
</body>
</html>"#,
        state.config.base_url,
        state.config.default_runner_profile_id,
        yes_no_label(state.config.fixture_profile_enabled)
    ))
}

fn validate_task_envelope(
    request: &AddonTaskRequest,
) -> Result<(), (StatusCode, Json<serde_json::Value>)> {
    if request.protocol_version != ADDON_PROTOCOL_VERSION {
        return Err(safe_bad_request("invalid_protocol_version"));
    }
    if request.addon_id != ADDON_ID {
        return Err(safe_bad_request("invalid_addon_id"));
    }
    if request.task_id != ACTION_TASK_ID {
        return Err(safe_bad_request("invalid_task_id"));
    }

    Ok(())
}

fn decode_action_payload(
    payload: serde_json::Value,
) -> Result<AddonExternalAcquisitionActionRequest, (StatusCode, Json<serde_json::Value>)> {
    let payload = serde_json::from_value::<AddonExternalAcquisitionActionRequest>(payload)
        .map_err(|_| safe_bad_request("invalid_action_payload"))?;
    if payload.schema != ADDON_EXTERNAL_ACQUISITION_ACTION_REQUEST_SCHEMA {
        return Err(safe_bad_request("invalid_action_schema"));
    }

    Ok(payload)
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
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use nako_addon_protocol::{
        AddonExternalAcquisitionActionResponse, AddonExternalAcquisitionActionStatus,
        AddonExternalAcquisitionOperation, AddonExternalAcquisitionRunnerState,
        AddonHealthCheckRequest, AddonManifest, AddonScope, validate_manifest,
    };
    use tower::ServiceExt;

    use super::*;
    use crate::manifest::ACTION_REQUEST_SCHEMA;

    #[tokio::test]
    async fn manifest_endpoint_returns_valid_external_acquisition_runner_manifest() {
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
        assert!(manifest.resources.is_empty());
        assert_eq!(manifest.tasks[0].id, ACTION_TASK_ID);
        assert_eq!(
            manifest.tasks[0].required_scopes,
            vec![AddonScope::AcquisitionActionRun]
        );
    }

    #[tokio::test]
    async fn action_task_enqueues_and_replays_by_idempotency_key() {
        let router = router(Config::default());
        let first = router
            .clone()
            .oneshot(task_request(action_payload(
                AddonExternalAcquisitionOperation::Enqueue,
                serde_json::json!({
                    "kind": "selected_link",
                    "selected_link_ref": "selected-link-secret"
                }),
                "idem-1",
            )))
            .await
            .unwrap();
        let second = router
            .oneshot(task_request(action_payload(
                AddonExternalAcquisitionOperation::Enqueue,
                serde_json::json!({
                    "kind": "selected_link",
                    "selected_link_ref": "selected-link-secret"
                }),
                "idem-1",
            )))
            .await
            .unwrap();

        assert_eq!(first.status(), StatusCode::OK);
        assert_eq!(second.status(), StatusCode::OK);
        let first = task_output(first).await;
        let second = task_output(second).await;

        assert_eq!(first.status, AddonExternalAcquisitionActionStatus::Accepted);
        assert_eq!(
            second.status,
            AddonExternalAcquisitionActionStatus::AlreadyExists
        );
        assert_eq!(first.runner_job_ref, second.runner_job_ref);
        assert_eq!(
            second
                .safe_facts
                .get("idempotent_replay")
                .map(String::as_str),
            Some("true")
        );
    }

    #[tokio::test]
    async fn action_task_cancels_and_queries_status() {
        let router = router(Config::default());
        let enqueued = router
            .clone()
            .oneshot(task_request(action_payload(
                AddonExternalAcquisitionOperation::Enqueue,
                serde_json::json!({
                    "kind": "intake_candidate",
                    "intake_candidate_ref": "candidate-secret"
                }),
                "idem-2",
            )))
            .await
            .unwrap();
        let runner_job_ref = task_output(enqueued).await.runner_job_ref.unwrap();

        let cancelled = router
            .clone()
            .oneshot(task_request(action_payload(
                AddonExternalAcquisitionOperation::Cancel,
                serde_json::json!({
                    "kind": "runner_job",
                    "runner_job_ref": runner_job_ref
                }),
                "idem-cancel-2",
            )))
            .await
            .unwrap();
        let status = router
            .oneshot(task_request(action_payload(
                AddonExternalAcquisitionOperation::QueryStatus,
                serde_json::json!({
                    "kind": "runner_job",
                    "runner_job_ref": runner_job_ref
                }),
                "idem-status-2",
            )))
            .await
            .unwrap();

        let cancelled = task_output(cancelled).await;
        let status = task_output(status).await;
        assert_eq!(
            cancelled.state,
            AddonExternalAcquisitionRunnerState::Cancelled
        );
        assert_eq!(status.state, AddonExternalAcquisitionRunnerState::Cancelled);
        assert_eq!(status.progress.unwrap().percent_milli, Some(100_000));
    }

    #[tokio::test]
    async fn action_task_rejects_raw_urls_and_passwords_without_echoing_them() {
        let response = router(Config::default())
            .oneshot(task_request(serde_json::json!({
                "schema": ACTION_REQUEST_SCHEMA,
                "target_ref": {
                    "kind": "selected_link",
                    "selected_link_ref": "selected-link-secret",
                    "raw_url": "magnet:?xt=urn:btih:secret"
                },
                "runner_profile_id": "fixture",
                "idempotency_key": "idem-secret",
                "operation": "enqueue",
                "password": "secret-code"
            })))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();
        assert!(text.contains("invalid_action_payload"));
        for forbidden in [
            "magnet:",
            "secret-code",
            "selected-link-secret",
            "idem-secret",
        ] {
            assert!(
                !text.contains(forbidden),
                "invalid action response leaked forbidden term: {forbidden}"
            );
        }
    }

    #[tokio::test]
    async fn health_and_diagnostics_are_redaction_safe() {
        let health_request = AddonHealthCheckRequest {
            protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
            manifest_id: ADDON_ID.to_owned(),
            request_id: "health-1".to_owned(),
            expected_addon_version: ADDON_VERSION.to_owned(),
            expected_resource_count: 0,
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
        assert_eq!(health.manifest.resource_count, 0);
        assert_eq!(health.diagnostics["external_network"], false);
        assert_eq!(
            health.diagnostics["profile_registry"][0]["runner_profile_id"],
            "fixture"
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
        assert!(text.contains("External network calls: no"));
        assert!(text.contains("opaque target references"));
        for forbidden in ["raw_url", "password", "Bearer ", "nako_at_", "magnet:"] {
            assert!(
                !text.contains(forbidden),
                "diagnostics leaked forbidden term: {forbidden}"
            );
        }
    }

    fn task_request(payload: serde_json::Value) -> Request<Body> {
        let request = AddonTaskRequest {
            protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
            addon_id: ADDON_ID.to_owned(),
            task_id: ACTION_TASK_ID.to_owned(),
            job_id: "job-1".to_owned(),
            request_id: "request-1".to_owned(),
            attempt: 1,
            retry_of_job_id: None,
            library_id: Some("library-1".to_owned()),
            source_id: Some("source-1".to_owned()),
            payload,
        };

        Request::builder()
            .method("POST")
            .uri(ACTION_TASK_PATH)
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&request).unwrap()))
            .unwrap()
    }

    fn action_payload(
        operation: AddonExternalAcquisitionOperation,
        target_ref: serde_json::Value,
        idempotency_key: &str,
    ) -> serde_json::Value {
        serde_json::json!({
            "schema": ACTION_REQUEST_SCHEMA,
            "target_ref": target_ref,
            "runner_profile_id": "fixture",
            "idempotency_key": idempotency_key,
            "operation": operation.as_str(),
            "audit_ref": "audit-ref"
        })
    }

    async fn task_output(
        response: axum::response::Response,
    ) -> AddonExternalAcquisitionActionResponse {
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let response: AddonTaskResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(response.addon_id, ADDON_ID);
        assert_eq!(response.task_id, ACTION_TASK_ID);
        serde_json::from_value(response.output).unwrap()
    }
}
