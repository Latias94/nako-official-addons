use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::Html,
    routing::{get, post},
};
use nako_addon_protocol::{
    ADDON_PROTOCOL_VERSION, AddonEventRequest, AddonEventResponse, AddonHealthCheckRequest,
    AddonHealthCheckResponse, AddonHealthManifestFacts, AddonHealthStatus, AddonResourceRequest,
    AddonResourceResponse, AddonTaskRequest, AddonTaskResponse,
};
use nako_official_addon_catalog::metadata_scraper;
use tower_http::trace::TraceLayer;

use crate::{
    Config,
    engine::{
        MetadataScrapeRuntime, ProviderRunPolicy,
        bulk::{BULK_METADATA_SCRAPE_TASK_ID, BULK_METADATA_SCRAPE_TASK_PATH},
    },
    manifest::{ADDON_ID, ADDON_VERSION, addon_manifest},
    nako_runtime::NakoRuntimeClient,
    nako_runtime::NakoRuntimeClientConfig,
    providers::{ProviderDiagnostics, ProviderRegistry},
};

#[derive(Clone)]
pub struct AppState {
    config: Config,
    metadata_runtime: MetadataScrapeRuntime,
    provider_diagnostics: ProviderDiagnostics,
}

pub fn router(config: Config) -> Router {
    let registry = ProviderRegistry::from_config(config.clone());
    let external_id_capabilities = registry.external_id_capabilities();
    let default_provider_field_policy = ProviderRegistry::default_provider_field_policy();
    let provider_assembly = registry.assemble();
    let provider_diagnostics = provider_assembly.diagnostics;
    let providers = provider_assembly.providers;
    let nako_runtime = NakoRuntimeClientConfig::from_runtime_config(&config.nako_runtime)
        .map(NakoRuntimeClient::new);
    let state = AppState {
        metadata_runtime:
            MetadataScrapeRuntime::with_external_id_capabilities_field_policy_and_run_policy(
                config.preferred_language.clone(),
                external_id_capabilities,
                default_provider_field_policy,
                ProviderRunPolicy::from_max_selected_providers(
                    config.provider_execution.max_selected_providers,
                ),
                providers,
                nako_runtime,
            ),
        provider_diagnostics,
        config,
    };

    Router::new()
        .route("/manifest.json", get(manifest))
        .route("/health", post(health))
        .route("/metadata", post(metadata))
        .route(BULK_METADATA_SCRAPE_TASK_PATH, post(bulk_metadata_scrape))
        .route(
            metadata_scraper::LIBRARY_SCANNED_EVENT_PATH,
            post(library_scanned_event),
        )
        .route("/ui/diagnostics", get(diagnostics))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn manifest(State(state): State<AppState>) -> Json<nako_addon_protocol::AddonManifest> {
    Json(addon_manifest(&state.config))
}

async fn health(
    State(state): State<AppState>,
    Json(request): Json<AddonHealthCheckRequest>,
) -> Json<AddonHealthCheckResponse> {
    let expected_status = if request.manifest_id == ADDON_ID {
        AddonHealthStatus::Ok
    } else {
        AddonHealthStatus::Degraded
    };
    let mut network_policy = state.provider_diagnostics.network_policy.clone();
    network_policy.insert(
        "browser_worker_render_proxy_policy_configured",
        state.config.rendered_page_proxy_policy_configured(),
    );
    network_policy.insert(
        "browser_worker_render_session_key_configured",
        state.config.rendered_page_session_key_configured(),
    );

    Json(AddonHealthCheckResponse {
        protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
        manifest_id: request.manifest_id,
        status: expected_status,
        checked_at: "2026-05-22T00:00:00.000Z".to_owned(),
        manifest: AddonHealthManifestFacts {
            addon_version: ADDON_VERSION.to_owned(),
            resource_count: 1,
        },
        diagnostics: serde_json::json!({
            "safe_note": "metadata scraper sidecar is reachable",
            "providers": state.provider_diagnostics.supported,
            "enabled_providers": state.provider_diagnostics.enabled,
            "disabled_providers": state.provider_diagnostics.disabled,
            "unavailable_providers": state.provider_diagnostics.unavailable,
            "network_policy": network_policy,
            "provider_execution_policy": {
                "max_selected_providers": state.config.provider_execution.max_selected_providers
            }
        }),
    })
}

async fn metadata(
    State(state): State<AppState>,
    Json(request): Json<AddonResourceRequest>,
) -> Json<AddonResourceResponse> {
    Json(state.metadata_runtime.scrape(request).await)
}

async fn bulk_metadata_scrape(
    State(state): State<AppState>,
    Json(request): Json<AddonTaskRequest>,
) -> Result<Json<AddonTaskResponse>, (StatusCode, Json<serde_json::Value>)> {
    match state.metadata_runtime.bulk_scrape(request).await {
        Ok(response) => Ok(Json(response)),
        Err(error) => Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": error.to_string(),
                "task_id": BULK_METADATA_SCRAPE_TASK_ID
            })),
        )),
    }
}

async fn library_scanned_event(
    Json(request): Json<AddonEventRequest>,
) -> Result<Json<AddonEventResponse>, (StatusCode, Json<serde_json::Value>)> {
    if request.protocol_version != ADDON_PROTOCOL_VERSION
        || request.addon_id != ADDON_ID
        || request.subscription_id != metadata_scraper::LIBRARY_SCANNED_EVENT_SUBSCRIPTION_ID
        || request.event_kind != metadata_scraper::LIBRARY_SCANNED_EVENT_KIND
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

    Ok(Json(AddonEventResponse {
        protocol_version: request.protocol_version,
        addon_id: request.addon_id,
        subscription_id: request.subscription_id,
        event_id: request.event_id,
        output: serde_json::json!({
            "schema": "nako.official.metadata-scraper.library-scanned.event.v1",
            "accepted": true,
            "attempt": request.attempt,
            "subject_kind": request.subject_kind,
            "subject_id": request.subject_id,
            "payload_keys": payload_keys
        }),
    }))
}

async fn diagnostics(State(state): State<AppState>) -> Html<String> {
    let enabled_providers = provider_list_label(&state.provider_diagnostics.enabled);
    let supported_provider_ids = state
        .provider_diagnostics
        .supported
        .iter()
        .map(|provider| provider.id)
        .collect::<Vec<_>>();
    let supported_providers = provider_list_label(&supported_provider_ids);
    let tmdb_proxy_configured =
        network_policy_label(&state.provider_diagnostics, "tmdb_proxy_configured");
    let bangumi_proxy_configured =
        network_policy_label(&state.provider_diagnostics, "bangumi_proxy_configured");
    let prestige_proxy_configured =
        network_policy_label(&state.provider_diagnostics, "prestige_proxy_configured");
    let render_proxy_policy_configured =
        yes_no_label(state.config.rendered_page_proxy_policy_configured());
    let render_session_key_configured =
        yes_no_label(state.config.rendered_page_session_key_configured());
    let max_selected_providers = state
        .config
        .provider_execution
        .max_selected_providers
        .map_or_else(|| "(none)".to_owned(), |value| value.to_string());
    Html(format!(
        r#"<!doctype html>
<html lang="en">
<head><meta charset="utf-8"><title>Nako Metadata Scraper</title></head>
<body>
  <h1>Nako Metadata Scraper</h1>
  <p>Base URL: {}</p>
  <p>Supported providers: {supported_providers}</p>
  <p>Enabled providers: {enabled_providers}</p>
  <p>TMDB proxy configured: {tmdb_proxy_configured}</p>
  <p>Bangumi proxy configured: {bangumi_proxy_configured}</p>
  <p>Prestige proxy configured: {prestige_proxy_configured}</p>
  <p>Browser render proxy policy configured: {render_proxy_policy_configured}</p>
  <p>Browser render session key configured: {render_session_key_configured}</p>
  <p>Provider max selected per request: {max_selected_providers}</p>
  <p>This page is hosted by the Addon Sidecar and is not trusted Nako Admin UI.</p>
</body>
</html>"#,
        state.config.base_url
    ))
}

fn provider_list_label(providers: &[&str]) -> String {
    if providers.is_empty() {
        "(none)".to_owned()
    } else {
        providers.join(", ")
    }
}

const fn yes_no_label(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}

fn network_policy_label(diagnostics: &ProviderDiagnostics, key: &str) -> &'static str {
    yes_no_label(
        diagnostics
            .network_policy
            .get(key)
            .copied()
            .unwrap_or(false),
    )
}

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use nako_addon_protocol::{
        AddonEventRequest, AddonEventResponse, AddonHealthCheckRequest, AddonResource, AddonScope,
        validate_manifest,
    };
    use tower::ServiceExt;

    use super::*;
    use crate::config::{ProviderConfig, ProviderId};

    #[tokio::test]
    async fn manifest_endpoint_returns_valid_manifest() {
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
        assert_eq!(manifest.resources[0].kind, AddonResource::Metadata);
        let schema = &manifest.configuration_schema.as_ref().unwrap().schema;
        assert_eq!(
            schema["properties"]["providers"]["properties"]["tmdb"]["default"],
            false
        );
        assert_eq!(
            schema["properties"]["providers"]["properties"]["bangumi"]["default"],
            false
        );
        assert_eq!(
            manifest.scopes,
            vec![
                AddonScope::ItemMetadataRead,
                AddonScope::ItemMetadataSuggest,
                AddonScope::AutomationRun,
                AddonScope::WebhookEventRead,
            ]
        );
        assert_eq!(manifest.tasks.len(), 1);
        assert_eq!(manifest.tasks[0].id, BULK_METADATA_SCRAPE_TASK_ID);
        assert_eq!(manifest.tasks[0].path, BULK_METADATA_SCRAPE_TASK_PATH);
        assert_eq!(manifest.event_subscriptions.len(), 1);
        assert_eq!(
            manifest.event_subscriptions[0].id,
            metadata_scraper::LIBRARY_SCANNED_EVENT_SUBSCRIPTION_ID
        );
        assert_eq!(
            manifest.event_subscriptions[0].event_kind,
            metadata_scraper::LIBRARY_SCANNED_EVENT_KIND
        );
        assert_eq!(
            manifest.event_subscriptions[0].path,
            metadata_scraper::LIBRARY_SCANNED_EVENT_PATH
        );
    }

    #[tokio::test]
    async fn metadata_endpoint_returns_candidate_suggestions() {
        let request = AddonResourceRequest {
            protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
            addon_id: ADDON_ID.to_owned(),
            resource: AddonResource::Metadata,
            request_id: "request-1".to_owned(),
            payload: serde_json::json!({"title":"The Matrix", "year": 1999}),
        };
        let response = router(Config::default())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/metadata")
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
        let payload: AddonResourceResponse = serde_json::from_slice(&body).unwrap();

        assert_eq!(payload.request_id, "request-1");
        assert_eq!(payload.artifacts[0].kind, "metadata_suggestion");
        assert_eq!(
            payload.payload["candidates"][0]["patch"]["title"],
            "The Matrix (1999)"
        );
    }

    #[tokio::test]
    async fn metadata_endpoint_respects_configured_provider_enablement() {
        let request = AddonResourceRequest {
            protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
            addon_id: ADDON_ID.to_owned(),
            resource: AddonResource::Metadata,
            request_id: "request-1".to_owned(),
            payload: serde_json::json!({"title":"The Matrix", "year": 1999}),
        };
        let response = router(Config {
            providers: vec![ProviderConfig::disabled(ProviderId::Fixture)],
            ..Config::default()
        })
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/metadata")
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
        let payload: AddonResourceResponse = serde_json::from_slice(&body).unwrap();

        assert!(payload.payload["candidates"].as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn bulk_metadata_scrape_endpoint_returns_planned_batch_output() {
        let request = AddonTaskRequest {
            protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
            addon_id: ADDON_ID.to_owned(),
            task_id: BULK_METADATA_SCRAPE_TASK_ID.to_owned(),
            job_id: "job-1".to_owned(),
            request_id: "task-request-1".to_owned(),
            attempt: 1,
            retry_of_job_id: None,
            library_id: Some("library-1".to_owned()),
            source_id: Some("source-1".to_owned()),
            payload: serde_json::json!({
                "batch_size": 1,
                "items": [
                    {"title": "The Matrix", "year": 1999},
                    {"title": "Inception", "year": 2010}
                ]
            }),
        };
        let response = router(Config::default())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(BULK_METADATA_SCRAPE_TASK_PATH)
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
        let payload: AddonTaskResponse = serde_json::from_slice(&body).unwrap();

        assert_eq!(payload.task_id, BULK_METADATA_SCRAPE_TASK_ID);
        assert_eq!(payload.request_id, "task-request-1");
        assert_eq!(
            payload.output["schema"],
            "nako.official.metadata-scraper.bulk-metadata-scrape.result.v1"
        );
        assert_eq!(payload.output["processed_items"], 1);
        assert_eq!(payload.output["remaining_items"], 1);
        assert_eq!(payload.output["next_cursor"], 1);
        assert_eq!(
            payload.output["items"][0]["payload"]["query"]["title"],
            "The Matrix"
        );
    }

    #[tokio::test]
    async fn library_scanned_event_endpoint_accepts_event_without_echoing_payload_values() {
        let request = AddonEventRequest {
            protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
            addon_id: ADDON_ID.to_owned(),
            subscription_id: metadata_scraper::LIBRARY_SCANNED_EVENT_SUBSCRIPTION_ID.to_owned(),
            event_id: "event-1".to_owned(),
            event_kind: metadata_scraper::LIBRARY_SCANNED_EVENT_KIND.to_owned(),
            subject_kind: "library".to_owned(),
            subject_id: "library-1".to_owned(),
            occurred_at: "2026-05-25T00:00:00.000Z".to_owned(),
            attempt: 1,
            payload: serde_json::json!({
                "library_id": "library-1",
                "secret": "nako_at_should_not_echo"
            }),
        };
        let response = router(Config::default())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(metadata_scraper::LIBRARY_SCANNED_EVENT_PATH)
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
            metadata_scraper::LIBRARY_SCANNED_EVENT_SUBSCRIPTION_ID
        );
        assert_eq!(payload.event_id, "event-1");
        assert_eq!(
            payload.output["schema"],
            "nako.official.metadata-scraper.library-scanned.event.v1"
        );
        assert_eq!(payload.output["accepted"], true);
        assert_eq!(
            payload.output["payload_keys"],
            serde_json::json!(["library_id", "secret"])
        );
        assert!(!text.contains("nako_at_should_not_echo"));
    }

    #[tokio::test]
    async fn health_endpoint_reports_configured_provider_diagnostics() {
        let request = AddonHealthCheckRequest {
            protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
            manifest_id: ADDON_ID.to_owned(),
            request_id: "health-1".to_owned(),
            expected_addon_version: ADDON_VERSION.to_owned(),
            expected_resource_count: 1,
        };
        let response = router(Config {
            providers: vec![ProviderConfig::disabled(ProviderId::Fixture)],
            ..Config::default()
        })
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
        assert_eq!(payload.diagnostics["providers"][0]["id"], "fixture");
        assert_eq!(payload.diagnostics["providers"][0]["status"], "disabled");
        assert_eq!(
            payload.diagnostics["enabled_providers"],
            serde_json::json!([])
        );
        assert_eq!(
            payload.diagnostics["disabled_providers"],
            serde_json::json!([
                "fixture",
                "tmdb",
                "bangumi",
                "browser_worker",
                "douban",
                "javdb",
                "dmm",
                "fc2",
                "fc2ppvdb",
                "caribbean",
                "1pondo",
                "10musume",
                "javbus",
                "javlibrary",
                "mgstage",
                "prestige"
            ])
        );
        assert_eq!(
            payload.diagnostics["unavailable_providers"],
            serde_json::json!([])
        );
        assert_eq!(
            payload.diagnostics["network_policy"]["tmdb_proxy_configured"],
            false
        );
        assert_eq!(
            payload.diagnostics["network_policy"]["bangumi_proxy_configured"],
            false
        );
        assert_eq!(
            payload.diagnostics["network_policy"]["prestige_proxy_configured"],
            false
        );
    }

    #[tokio::test]
    async fn health_endpoint_reports_proxy_policy_without_leaking_urls() {
        let request = AddonHealthCheckRequest {
            protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
            manifest_id: ADDON_ID.to_owned(),
            request_id: "health-1".to_owned(),
            expected_addon_version: ADDON_VERSION.to_owned(),
            expected_resource_count: 1,
        };
        let response = router(Config::from_env_lookup(|name| match name {
            "NAKO_METADATA_SCRAPER_TMDB_PROXY_URL" => {
                Some("http://user:pass@proxy.example:8080".to_owned())
            }
            "NAKO_METADATA_SCRAPER_BANGUMI_PROXY_URL" => {
                Some("http://proxy.example:8080".to_owned())
            }
            "NAKO_METADATA_SCRAPER_PRESTIGE_PROXY_URL" => {
                Some("http://prestige-proxy.example:8080".to_owned())
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
            payload.diagnostics["network_policy"]["tmdb_proxy_configured"],
            true
        );
        assert_eq!(
            payload.diagnostics["network_policy"]["bangumi_proxy_configured"],
            true
        );
        assert_eq!(
            payload.diagnostics["network_policy"]["prestige_proxy_configured"],
            true
        );
        let diagnostics = serde_json::to_string(&payload.diagnostics).unwrap();
        assert!(!diagnostics.contains("proxy.example"));
        assert!(!diagnostics.contains("prestige-proxy.example"));
        assert!(!diagnostics.contains("user:pass"));
    }
}
