use async_trait::async_trait;
use nako_addon_protocol::AddonMetadataPatch;
use serde::{Deserialize, Serialize};

use crate::{
    Config,
    config::{BrowserWorkerProviderConfig, ProviderId},
    engine::{
        MetadataQuery, ProviderCandidateFacts, ProviderExternalId, ProviderMetadataCandidate,
    },
    providers::{
        MetadataProvider, ProviderBuildStatus,
        http_runtime::{
            ProviderHttpResult, ProviderHttpRuntime, ProviderHttpRuntimeConfig,
            ProviderHttpTransport, ReqwestProviderHttpTransport,
        },
        registry::ProviderCatalogEntry,
    },
};

pub const BROWSER_WORKER_PROVIDER_ID: &str = "browser_worker";
const BROWSER_WORKER_RENDERED_PAGE_CAPABILITY: &str = "rendered_page_extraction";

#[must_use]
pub(crate) fn catalog_entry() -> ProviderCatalogEntry {
    ProviderCatalogEntry {
        id: ProviderId::BrowserWorker,
        capabilities: &[
            "metadata_suggestion",
            BROWSER_WORKER_RENDERED_PAGE_CAPABILITY,
        ],
        secret_reference: None,
        build: build_provider,
    }
}

fn build_provider(config: &Config) -> ProviderBuildStatus {
    let Some(browser_worker_config) = config
        .provider_config(ProviderId::BrowserWorker)
        .and_then(|provider| provider.browser_worker.clone())
    else {
        return ProviderBuildStatus::Unavailable;
    };
    match BrowserWorkerMetadataProvider::new(browser_worker_config) {
        Ok(provider) => ProviderBuildStatus::Ready(Box::new(provider)),
        Err(_) => ProviderBuildStatus::Unavailable,
    }
}

#[derive(Clone, Debug)]
pub struct BrowserWorkerMetadataProvider<T = ReqwestProviderHttpTransport>
where
    T: ProviderHttpTransport,
{
    config: BrowserWorkerProviderConfig,
    runtime: ProviderHttpRuntime<T>,
}

impl BrowserWorkerMetadataProvider<ReqwestProviderHttpTransport> {
    pub fn new(config: BrowserWorkerProviderConfig) -> ProviderHttpResult<Self> {
        let runtime = ProviderHttpRuntime::new(ProviderHttpRuntimeConfig {
            timeout_ms: config.timeout_ms,
            ..ProviderHttpRuntimeConfig::default()
        })?;
        Ok(Self { config, runtime })
    }
}

impl<T> BrowserWorkerMetadataProvider<T>
where
    T: ProviderHttpTransport,
{
    #[must_use]
    pub fn with_runtime(
        config: BrowserWorkerProviderConfig,
        runtime: ProviderHttpRuntime<T>,
    ) -> Self {
        Self { config, runtime }
    }

    fn endpoint(&self, path: impl AsRef<str>) -> String {
        let path = path.as_ref();
        format!(
            "{}/{}",
            self.config.base_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }

    fn extract_request(&self, url: &str) -> BrowserWorkerExtractRequest {
        BrowserWorkerExtractRequest {
            url: url.to_owned(),
        }
    }
}

#[async_trait]
impl<T> MetadataProvider for BrowserWorkerMetadataProvider<T>
where
    T: ProviderHttpTransport,
{
    fn id(&self) -> ProviderId {
        ProviderId::BrowserWorker
    }

    async fn suggest(
        &self,
        query: &MetadataQuery,
    ) -> anyhow::Result<Vec<ProviderMetadataCandidate>> {
        let mut candidates = Vec::new();
        let source_urls = query
            .external_ids
            .iter()
            .filter(|external_id| is_browser_worker_source(&external_id.provider))
            .map(|external_id| external_id.value.clone())
            .collect::<Vec<_>>();

        for source_url in source_urls {
            let response = self
                .runtime
                .post_json(
                    BROWSER_WORKER_PROVIDER_ID,
                    "extract rendered page",
                    self.endpoint(&self.config.extract_path),
                    Vec::new(),
                    Vec::new(),
                    &self.extract_request(&source_url),
                )
                .await?;
            let extracted = BrowserWorkerExtractResponse::from_value(response.body)?;
            if extracted.status.as_deref() != Some("ok") {
                anyhow::bail!(
                    "browser worker returned non-ok status for {source_url}: {:?}",
                    extracted.status
                );
            }
            candidates.push(extracted.into_candidate(query, &source_url));
        }

        Ok(candidates)
    }
}

#[derive(Debug, Serialize)]
struct BrowserWorkerExtractRequest {
    url: String,
}

#[derive(Debug, Deserialize)]
struct BrowserWorkerExtractResponse {
    #[serde(default)]
    status: Option<String>,
    url: Option<String>,
    title: Option<String>,
    rendered_text: Option<String>,
    excerpt: Option<String>,
}

impl BrowserWorkerExtractResponse {
    fn from_value(value: serde_json::Value) -> anyhow::Result<Self> {
        serde_json::from_value(value).map_err(|error| {
            anyhow::anyhow!("failed to parse browser worker extract response: {error}")
        })
    }

    fn into_candidate(self, query: &MetadataQuery, source_url: &str) -> ProviderMetadataCandidate {
        let rendered_url = self
            .url
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| source_url.to_owned());
        let title = self
            .title
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| query.title.clone());
        let rendered_text = self
            .rendered_text
            .filter(|value| !value.trim().is_empty())
            .or(self.excerpt)
            .unwrap_or_default();

        ProviderMetadataCandidate {
            provider: BROWSER_WORKER_PROVIDER_ID.to_owned(),
            provider_id: format!("browser-worker:{source_url}"),
            patch: AddonMetadataPatch {
                title: Some(title.clone()),
                original_title: Some(query.title.clone()).filter(|value| value != &title),
                sort_title: Some(title.clone()),
                overview: (!rendered_text.is_empty()).then_some(rendered_text.clone()),
                release_date: None,
                runtime_minutes: None,
                tagline: Some("Browser worker rendered page".to_owned()),
                genres: None,
                tags: Some(vec![
                    BROWSER_WORKER_PROVIDER_ID.to_owned(),
                    BROWSER_WORKER_RENDERED_PAGE_CAPABILITY.to_owned(),
                ]),
            },
            facts: ProviderCandidateFacts {
                title: Some(title),
                alternate_titles: Vec::new(),
                release_year: None,
                language: Some(query.language.clone()),
                community_score_milli: None,
                community_vote_count: None,
                external_ids: vec![ProviderExternalId {
                    provider: BROWSER_WORKER_PROVIDER_ID.to_owned(),
                    value: rendered_url,
                }],
                provider_note: Some(
                    "Browser worker rendered a page and returned normalized text.".to_owned(),
                ),
            },
            artwork_candidates: Vec::new(),
        }
    }
}

fn is_browser_worker_source(provider: &str) -> bool {
    provider.eq_ignore_ascii_case(BROWSER_WORKER_PROVIDER_ID)
        || provider.eq_ignore_ascii_case("browser-worker")
        || provider.eq_ignore_ascii_case("browser_worker_url")
}

#[cfg(test)]
mod tests {
    use std::{
        collections::VecDeque,
        sync::{Arc, Mutex},
    };

    use crate::providers::http_runtime::{
        ProviderHttpRequest, ProviderHttpResponse, ProviderHttpRuntimeConfig,
    };

    use super::*;

    #[tokio::test]
    async fn browser_worker_provider_maps_rendered_page_candidates() {
        let transport = FakeTransport::default();
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: serde_json::json!({
                "status": "ok",
                "url": "http://browser-worker.example/final-page",
                "title": "Rendered Fixture",
                "rendered_text": "Browser worker fixture rendered by JavaScript",
                "excerpt": "Browser worker fixture rendered by JavaScript"
            })
            .to_string()
            .into_bytes(),
        }));
        let runtime = ProviderHttpRuntime::with_transport(
            ProviderHttpRuntimeConfig {
                retry_backoff_ms: 0,
                ..ProviderHttpRuntimeConfig::default()
            },
            transport.clone(),
        );
        let provider = BrowserWorkerMetadataProvider::with_runtime(
            BrowserWorkerProviderConfig {
                base_url: "http://browser-worker.example".to_owned(),
                extract_path: "/extract".to_owned(),
                timeout_ms: 10_000,
            },
            runtime,
        );

        let candidates = provider
            .suggest(&MetadataQuery {
                title: "Query Title".to_owned(),
                year: None,
                language: "en-US".to_owned(),
                external_ids: vec![crate::engine::QueryExternalId {
                    provider: "browser_worker".to_owned(),
                    value: "http://fixture.example/page".to_owned(),
                }],
            })
            .await
            .unwrap();

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].provider, "browser_worker");
        assert_eq!(
            candidates[0].provider_id,
            "browser-worker:http://fixture.example/page"
        );
        assert_eq!(
            candidates[0].patch.title.as_deref(),
            Some("Rendered Fixture")
        );
        assert_eq!(
            candidates[0].patch.overview.as_deref(),
            Some("Browser worker fixture rendered by JavaScript")
        );
        assert_eq!(
            candidates[0].facts.external_ids[0].value,
            "http://browser-worker.example/final-page"
        );

        let requests = transport.requests();
        assert_eq!(requests[0].url, "http://browser-worker.example/extract");
        let body: serde_json::Value =
            serde_json::from_slice(requests[0].json_body.as_ref().unwrap()).unwrap();
        assert_eq!(body["url"], "http://fixture.example/page");
    }

    #[derive(Clone, Default)]
    struct FakeTransport {
        responses: Arc<Mutex<VecDeque<ProviderHttpResult<ProviderHttpResponse>>>>,
        requests: Arc<Mutex<Vec<ProviderHttpRequest>>>,
    }

    impl FakeTransport {
        fn push(&self, response: ProviderHttpResult<ProviderHttpResponse>) {
            self.responses.lock().unwrap().push_back(response);
        }

        fn requests(&self) -> Vec<ProviderHttpRequest> {
            self.requests.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl ProviderHttpTransport for FakeTransport {
        async fn send(
            &self,
            request: ProviderHttpRequest,
            _config: ProviderHttpRuntimeConfig,
        ) -> ProviderHttpResult<ProviderHttpResponse> {
            self.requests.lock().unwrap().push(request);
            self.responses
                .lock()
                .unwrap()
                .pop_front()
                .unwrap_or_else(|| {
                    Err(
                        crate::providers::http_runtime::ProviderHttpError::Transport {
                            provider_id: BROWSER_WORKER_PROVIDER_ID,
                            operation: "fake",
                            message: "fake transport response queue was empty".to_owned(),
                            attempts: 0,
                        },
                    )
                })
        }
    }
}
