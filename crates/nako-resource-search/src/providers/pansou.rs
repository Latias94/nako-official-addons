use std::time::Duration;

use anyhow::Context;
use async_trait::async_trait;

use super::descriptor::{
    ProviderCapability, ProviderConfigurationSchemaFragment, ProviderDescriptor,
};
use super::{ProviderSearchBatch, ResourceSearchProvider};
use crate::{
    Config, config::PansouProviderConfig, domain::ResourceSearchQuery, source_policy::SourcePolicy,
};

mod mapper;
mod wire;

use mapper::map_pansou_response;
use wire::{PansouApiResponse, build_pansou_request};

pub const PANSOU_COMPATIBLE_PROVIDER_ID: &str = "pansou_compatible";

const PANSOU_CAPABILITIES: &[ProviderCapability] = &[
    ProviderCapability::ResourceSearch,
    ProviderCapability::ExternalHttpSearch,
    ProviderCapability::CloudDriveLinks,
    ProviderCapability::MagnetLinks,
    ProviderCapability::Refresh,
    ProviderCapability::MergedLinkResponse,
];

pub const PANSOU_DESCRIPTOR: ProviderDescriptor = ProviderDescriptor {
    id: PANSOU_COMPATIBLE_PROVIDER_ID,
    display_name: "PanSou Compatible",
    source_policy: SourcePolicy::ExternalService,
    default_enabled: false,
    capabilities: PANSOU_CAPABILITIES,
    configuration_schema: pansou_configuration_schema,
};

fn pansou_configuration_schema(config: &Config) -> ProviderConfigurationSchemaFragment {
    ProviderConfigurationSchemaFragment {
        provider_id: PANSOU_COMPATIBLE_PROVIDER_ID,
        provider_enabled_default: config.pansou.enabled,
        settings_key: Some("pansou"),
        settings_schema: Some(serde_json::json!({
            "type": "object",
            "properties": {
                "base_url": {
                    "type": "string",
                    "default": config.pansou.base_url.clone().unwrap_or_default()
                },
                "source_type": {
                    "type": "string",
                    "default": config.pansou.source_type
                },
                "plugins": {
                    "type": "array",
                    "items": { "type": "string" },
                    "default": config.pansou.plugins
                },
                "cloud_types": {
                    "type": "array",
                    "items": { "type": "string" },
                    "default": config.pansou.cloud_types.iter().map(|link_type| link_type.as_str()).collect::<Vec<_>>()
                },
                "concurrency": {
                    "type": ["integer", "null"],
                    "default": config.pansou.concurrency,
                    "minimum": 1
                },
                "timeout_ms": {
                    "type": "integer",
                    "default": config.pansou.timeout_ms,
                    "minimum": 250,
                    "maximum": 60000
                }
            },
            "additionalProperties": false
        })),
    }
}

#[derive(Clone)]
pub struct PansouCompatibleProvider {
    config: PansouProviderConfig,
    client: reqwest::Client,
}

impl PansouCompatibleProvider {
    #[must_use]
    pub fn new(config: PansouProviderConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_millis(config.timeout_ms))
            .build()
            .expect("reqwest client with timeout builds");
        Self { config, client }
    }
}

#[async_trait]
impl ResourceSearchProvider for PansouCompatibleProvider {
    fn id(&self) -> &'static str {
        PANSOU_COMPATIBLE_PROVIDER_ID
    }

    fn priority(&self) -> u16 {
        900
    }

    async fn search(&self, query: &ResourceSearchQuery) -> anyhow::Result<ProviderSearchBatch> {
        let base_url = self
            .config
            .base_url
            .as_deref()
            .context("pansou compatible base URL is not configured")?;
        let request = build_pansou_request(&self.config, query);
        let request_body = serde_json::to_vec(&request)
            .context("pansou compatible search request did not serialize")?;
        let mut builder = self
            .client
            .post(format!("{base_url}/api/search"))
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(request_body);

        if let Some(token) = self.config.bearer_token.as_deref() {
            builder = builder.bearer_auth(token);
        }

        let response = builder
            .send()
            .await
            .context("pansou compatible search request failed")?
            .error_for_status()
            .context("pansou compatible search returned an HTTP error")?
            .bytes()
            .await
            .context("pansou compatible search response body failed")?;
        let response = serde_json::from_slice::<PansouApiResponse>(&response)
            .context("pansou compatible search response was not valid JSON")?;

        let results = response
            .into_success_data()?
            .map(|data| map_pansou_response(query, data))
            .unwrap_or_default();

        Ok(ProviderSearchBatch::complete(self.id(), results))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use axum::{Json, Router, extract::State, http::HeaderMap, routing::post};
    use tokio::net::TcpListener;

    use crate::domain::{ResourceLinkType, ResourceSearchQuery};

    use super::*;

    #[derive(Clone, Debug, Default)]
    struct PansouFixtureState {
        requests: Arc<Mutex<Vec<CapturedPansouRequest>>>,
    }

    #[derive(Clone, Debug)]
    struct CapturedPansouRequest {
        auth_header: Option<String>,
        body: serde_json::Value,
    }

    async fn record_pansou_search(
        State(state): State<PansouFixtureState>,
        headers: HeaderMap,
        Json(body): Json<serde_json::Value>,
    ) -> Json<serde_json::Value> {
        let auth_header = headers
            .get(reqwest::header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned);
        state
            .requests
            .lock()
            .unwrap()
            .push(CapturedPansouRequest { auth_header, body });

        Json(serde_json::json!({
            "code": 0,
            "message": "ok",
            "data": {
                "results": [{
                    "message_id": "m1",
                    "unique_id": "u1",
                    "channel": "movies",
                    "title": "Demo Movie 1080p",
                    "content": "mock content",
                    "links": [{
                        "type": "quark",
                        "url": "https://pan.quark.cn/s/demo",
                        "password": "1234",
                        "work_title": "disc 1"
                    }],
                    "tags": ["mock"],
                    "images": ["https://example.test/poster.jpg"]
                }]
            }
        }))
    }

    async fn spawn_pansou_fixture() -> (String, PansouFixtureState, tokio::task::JoinHandle<()>) {
        let state = PansouFixtureState::default();
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let app = Router::new()
            .route("/api/search", post(record_pansou_search))
            .with_state(state.clone());
        let handle = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        (format!("http://{addr}"), state, handle)
    }

    #[tokio::test]
    async fn provider_search_calls_pansou_http_api_and_maps_response() {
        let (base_url, state, handle) = spawn_pansou_fixture().await;
        let provider = PansouCompatibleProvider::new(PansouProviderConfig {
            enabled: true,
            base_url: Some(base_url),
            bearer_token: Some("secret-token".to_owned()),
            source_type: "plugin".to_owned(),
            plugins: vec!["jikepan".to_owned()],
            cloud_types: Vec::new(),
            concurrency: Some(3),
            timeout_ms: 1_000,
        });
        let mut query = ResourceSearchQuery::free_text("Demo Movie", 20);
        query.link_types = vec![ResourceLinkType::Quark];
        query.refresh = true;
        query.ext = serde_json::json!({ "season": 1 });

        let batch = provider.search(&query).await.unwrap();

        assert_eq!(batch.provider_id, PANSOU_COMPATIBLE_PROVIDER_ID);
        assert_eq!(batch.results.len(), 1);
        let result = &batch.results[0];
        assert_eq!(result.id, "u1");
        assert_eq!(result.title, "Demo Movie 1080p");
        assert_eq!(result.source, "pansou:movies");
        assert_eq!(result.links.len(), 1);
        assert_eq!(result.links[0].link_type, ResourceLinkType::Quark);
        assert_eq!(
            result.links[0].url,
            "https://pan.quark.cn/s/demo".to_owned()
        );
        assert_eq!(result.links[0].password.as_deref(), Some("1234"));
        assert_eq!(result.links[0].note.as_deref(), Some("disc 1"));
        assert_eq!(result.tags, vec!["mock"]);

        let requests = state.requests.lock().unwrap();
        assert_eq!(requests.len(), 1);
        assert_eq!(
            requests[0].auth_header.as_deref(),
            Some("Bearer secret-token")
        );
        assert_eq!(requests[0].body["kw"], "Demo Movie");
        assert_eq!(requests[0].body["conc"], 3);
        assert_eq!(requests[0].body["refresh"], true);
        assert_eq!(requests[0].body["src"], "plugin");
        assert_eq!(requests[0].body["plugins"], serde_json::json!(["jikepan"]));
        assert_eq!(
            requests[0].body["cloud_types"],
            serde_json::json!(["quark"])
        );
        assert_eq!(requests[0].body["ext"], serde_json::json!({ "season": 1 }));

        handle.abort();
    }
}
