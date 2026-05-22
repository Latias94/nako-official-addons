use std::sync::Arc;

use axum::{
    Json, Router,
    extract::State,
    response::Html,
    routing::{get, post},
};
use nako_addon_protocol::{
    ADDON_PROTOCOL_VERSION, AddonArtifact, AddonHealthCheckRequest, AddonHealthCheckResponse,
    AddonHealthManifestFacts, AddonHealthStatus, AddonResourceRequest, AddonResourceResponse,
};
use tower_http::trace::TraceLayer;

use crate::{
    Config,
    engine::MetadataQuery,
    manifest::{ADDON_ID, ADDON_VERSION, addon_manifest},
    providers::{MetadataProvider, default_providers},
};

#[derive(Clone)]
pub struct AppState {
    config: Config,
    providers: Arc<Vec<Box<dyn MetadataProvider>>>,
}

#[must_use]
pub fn router(config: Config) -> Router {
    let state = AppState {
        config,
        providers: Arc::new(default_providers()),
    };

    Router::new()
        .route("/manifest.json", get(manifest))
        .route("/health", post(health))
        .route("/metadata", post(metadata))
        .route("/ui/diagnostics", get(diagnostics))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn manifest(State(state): State<AppState>) -> Json<nako_addon_protocol::AddonManifest> {
    Json(addon_manifest(state.config.base_url))
}

async fn health(Json(request): Json<AddonHealthCheckRequest>) -> Json<AddonHealthCheckResponse> {
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
            "providers": ["fixture"]
        }),
    })
}

async fn metadata(
    State(state): State<AppState>,
    Json(request): Json<AddonResourceRequest>,
) -> Json<AddonResourceResponse> {
    let query = MetadataQuery::from_payload(&request.payload, &state.config.preferred_language);
    let mut candidates = Vec::new();

    for provider in state.providers.iter() {
        match provider.suggest(&query).await {
            Ok(mut provider_candidates) => candidates.append(&mut provider_candidates),
            Err(error) => {
                tracing::warn!(provider = provider.id(), %error, "metadata provider failed")
            }
        }
    }

    candidates.sort_by(|left, right| right.confidence_milli.cmp(&left.confidence_milli));
    let payload = serde_json::json!({
        "query": {
            "title": query.title,
            "year": query.year,
            "language": query.language
        },
        "candidates": candidates
    });

    Json(AddonResourceResponse {
        protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
        addon_id: request.addon_id,
        resource: request.resource,
        request_id: request.request_id,
        payload: payload.clone(),
        artifacts: vec![AddonArtifact {
            kind: "metadata_suggestion".to_owned(),
            payload,
        }],
    })
}

async fn diagnostics(State(state): State<AppState>) -> Html<String> {
    Html(format!(
        r#"<!doctype html>
<html lang="en">
<head><meta charset="utf-8"><title>Nako Metadata Scraper</title></head>
<body>
  <h1>Nako Metadata Scraper</h1>
  <p>Base URL: {}</p>
  <p>Providers: fixture</p>
  <p>This page is hosted by the Addon Sidecar and is not trusted Nako Admin UI.</p>
</body>
</html>"#,
        state.config.base_url
    ))
}

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use nako_addon_protocol::{AddonResource, AddonScope, validate_manifest};
    use tower::ServiceExt;

    use super::*;

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
        assert_eq!(
            manifest.scopes,
            vec![
                AddonScope::ItemMetadataRead,
                AddonScope::ItemMetadataSuggest
            ]
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
}
