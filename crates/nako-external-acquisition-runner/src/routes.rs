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
    materialization::FixtureActionContext,
    runner::FixtureRunner,
};

#[derive(Clone)]
pub struct AppState {
    config: Config,
    runner: FixtureRunner,
}

pub fn router(config: Config) -> Router {
    let runner = FixtureRunner::new(config.clone());
    router_with_runner(config, runner)
}

fn router_with_runner(config: Config, runner: FixtureRunner) -> Router {
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
    let action_context = FixtureActionContext::new(request.job_id.clone(), request.task_id.clone());
    let output = match state.runner.handle_action(action_context, payload).await {
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
  <p>Transmission profile configured: {}</p>
  <p>External network calls: fixture no-op by default; production profiles only after host materialization.</p>
  <p>This fixture accepts only host-owned opaque target references.</p>
</body>
</html>"#,
        state.config.base_url,
        state.config.default_runner_profile_id,
        yes_no_label(state.config.fixture_profile_enabled),
        yes_no_label(state.config.transmission.enabled)
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
    use std::{
        collections::BTreeMap,
        sync::{Arc, Mutex as StdMutex},
    };

    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use nako_addon_protocol::{
        AddonExternalAcquisitionActionResponse, AddonExternalAcquisitionActionStatus,
        AddonExternalAcquisitionMaterializationRequest,
        AddonExternalAcquisitionMaterializationResponse, AddonExternalAcquisitionMaterializedLink,
        AddonExternalAcquisitionOperation, AddonExternalAcquisitionRunnerState,
        AddonExternalAcquisitionTargetRef, AddonHealthCheckRequest, AddonManifest,
        AddonResourceLinkType, AddonScope, validate_manifest,
    };
    use tower::ServiceExt;

    use super::*;
    use crate::{
        manifest::ACTION_REQUEST_SCHEMA,
        materialization::{ExternalAcquisitionMaterializer, MaterializationError},
    };

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
    async fn action_task_materialization_uses_job_context_and_redacts_output() {
        let materializer = RecordingMaterializer::with_response(materialization_response());
        let runner =
            FixtureRunner::with_materializer(Config::default(), Arc::new(materializer.clone()));
        let response = router_with_runner(Config::default(), runner)
            .oneshot(task_request(action_payload(
                AddonExternalAcquisitionOperation::Enqueue,
                serde_json::json!({
                    "kind": "selected_link",
                    "selected_link_ref": "selected-link-secret"
                }),
                "idem-route-materialization",
            )))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();
        let response: AddonTaskResponse = serde_json::from_str(&text).unwrap();
        let output: AddonExternalAcquisitionActionResponse =
            serde_json::from_value(response.output).unwrap();

        assert_eq!(
            output.status,
            AddonExternalAcquisitionActionStatus::Accepted
        );
        assert_eq!(
            output
                .safe_facts
                .get("materialization_client")
                .map(String::as_str),
            Some("recording")
        );
        assert_eq!(
            output
                .safe_facts
                .get("materialized_link_type")
                .map(String::as_str),
            Some("ed2k")
        );

        for forbidden in [
            "ed2k://|file|secret",
            "route-secret-code",
            "route-materialization-secret",
            "selected-link-secret",
            "idem-route-materialization",
        ] {
            assert!(
                !text.contains(forbidden),
                "task response leaked forbidden term: {forbidden}"
            );
        }

        let requests = materializer.requests();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].job_id, "job-1");
        assert_eq!(requests[0].declaration_id, ACTION_TASK_ID);
        assert_eq!(requests[0].idempotency_key, "idem-route-materialization");
        assert_eq!(requests[0].audit_ref, "audit-ref");
        assert_eq!(
            requests[0].target_ref,
            AddonExternalAcquisitionTargetRef::SelectedLink {
                selected_link_ref: "selected-link-secret".to_owned()
            }
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
        assert_eq!(
            health.diagnostics["profile_registry"][1]["runner_profile_id"],
            "transmission"
        );
        assert_eq!(
            health.diagnostics["profile_registry"][1]["implementation_status"],
            "configuration_ready"
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
        assert!(text.contains("External network calls: fixture no-op by default"));
        assert!(text.contains("opaque target references"));
        for forbidden in ["raw_url", "password", "Bearer ", "nako_at_", "magnet:"] {
            assert!(
                !text.contains(forbidden),
                "diagnostics leaked forbidden term: {forbidden}"
            );
        }
    }

    #[tokio::test]
    async fn diagnostics_reports_transmission_profile_without_secrets() {
        let mut config = Config::default();
        config.transmission.enabled = true;
        config.transmission.rpc_url = "http://runner:secret@transmission.local/rpc".to_owned();
        config.transmission.username = Some("runner".to_owned());
        config.transmission.password = Some("transmission-password-secret".to_owned());

        let diagnostics = FixtureRunner::new(config).diagnostics();
        assert_eq!(diagnostics["external_network"], true);
        assert_eq!(
            diagnostics["profile_registry"][1]["runner_profile_id"],
            "transmission"
        );
        assert_eq!(diagnostics["profile_registry"][1]["auth_configured"], true);
        assert_eq!(
            diagnostics["profile_registry"][1]["endpoint_configured"],
            true
        );

        let text = serde_json::to_string(&diagnostics).unwrap();
        for forbidden in [
            "transmission-password-secret",
            "runner:secret",
            "transmission.local",
            "http://runner",
        ] {
            assert!(
                !text.contains(forbidden),
                "diagnostics leaked forbidden Transmission config term: {forbidden}"
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

    #[derive(Clone, Debug)]
    struct RecordingMaterializer {
        requests: Arc<StdMutex<Vec<AddonExternalAcquisitionMaterializationRequest>>>,
        response: Result<AddonExternalAcquisitionMaterializationResponse, MaterializationError>,
    }

    impl RecordingMaterializer {
        fn with_response(response: AddonExternalAcquisitionMaterializationResponse) -> Self {
            Self {
                requests: Arc::default(),
                response: Ok(response),
            }
        }

        fn requests(&self) -> Vec<AddonExternalAcquisitionMaterializationRequest> {
            self.requests.lock().unwrap().clone()
        }
    }

    #[async_trait::async_trait]
    impl ExternalAcquisitionMaterializer for RecordingMaterializer {
        fn safe_client_kind(&self) -> &'static str {
            "recording"
        }

        async fn materialize(
            &self,
            request: AddonExternalAcquisitionMaterializationRequest,
        ) -> Result<AddonExternalAcquisitionMaterializationResponse, MaterializationError> {
            self.requests.lock().unwrap().push(request);
            self.response.clone()
        }
    }

    fn materialization_response() -> AddonExternalAcquisitionMaterializationResponse {
        AddonExternalAcquisitionMaterializationResponse {
            schema: nako_addon_protocol::ADDON_EXTERNAL_ACQUISITION_MATERIALIZATION_RESPONSE_SCHEMA
                .to_owned(),
            materialization_ref: "route-materialization-secret".to_owned(),
            target_ref: AddonExternalAcquisitionTargetRef::SelectedLink {
                selected_link_ref: "selected-link-secret".to_owned(),
            },
            expires_at: "2026-05-29T00:01:00.000Z".to_owned(),
            material: AddonExternalAcquisitionMaterializedLink {
                link_type: AddonResourceLinkType::Ed2k,
                uri: "ed2k://|file|secret|1|abcdef|/".to_owned(),
                password: Some("route-secret-code".to_owned()),
            },
            safe_facts: BTreeMap::new(),
        }
    }
}
