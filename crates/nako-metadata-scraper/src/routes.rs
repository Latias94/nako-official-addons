use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::Html,
    routing::{get, post},
};
use nako_addon_protocol::{
    ADDON_PROTOCOL_VERSION, AddonHealthCheckRequest, AddonHealthCheckResponse,
    AddonHealthManifestFacts, AddonHealthStatus, AddonResourceRequest, AddonResourceResponse,
    AddonTaskRequest, AddonTaskResponse,
};
use tower_http::trace::TraceLayer;

use crate::{
    Config,
    config::ProviderId,
    engine::{
        MetadataScrapeRuntime,
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
    let provider_diagnostics = registry.diagnostics();
    let providers = registry.providers();
    let nako_runtime = NakoRuntimeClientConfig::from_runtime_config(&config.nako_runtime)
        .map(NakoRuntimeClient::new);
    let state = AppState {
        metadata_runtime: MetadataScrapeRuntime::new(
            config.preferred_language.clone(),
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
            "network_policy": {
                "tmdb_proxy_configured": state.config.provider_proxy_configured(ProviderId::Tmdb),
                "bangumi_proxy_configured": state.config.provider_proxy_configured(ProviderId::Bangumi)
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
        yes_no_label(state.config.provider_proxy_configured(ProviderId::Tmdb));
    let bangumi_proxy_configured =
        yes_no_label(state.config.provider_proxy_configured(ProviderId::Bangumi));
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

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use nako_addon_protocol::{
        AddonHealthCheckRequest, AddonResource, AddonScope, validate_manifest,
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
            ]
        );
        assert_eq!(manifest.tasks.len(), 1);
        assert_eq!(manifest.tasks[0].id, BULK_METADATA_SCRAPE_TASK_ID);
        assert_eq!(manifest.tasks[0].path, BULK_METADATA_SCRAPE_TASK_PATH);
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
            serde_json::json!(["fixture", "tmdb", "bangumi", "browser_worker", "douban"])
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
        let diagnostics = serde_json::to_string(&payload.diagnostics).unwrap();
        assert!(!diagnostics.contains("proxy.example"));
        assert!(!diagnostics.contains("user:pass"));
    }
}
