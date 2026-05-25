use async_trait::async_trait;
use nako_addon_protocol::AddonMetadataPatch;

use crate::{
    Config,
    config::{ProviderConfig, ProviderId},
    engine::{
        MetadataQuery, ProviderCandidateFacts, ProviderExternalId, ProviderMetadataCandidate,
        ProviderOutcome, QueryExternalIdAlias,
    },
    providers::{
        MetadataProvider, ProviderBuildStatus, ProviderConfigInput,
        http_runtime::{
            ProviderHttpResult, ProviderHttpRuntime, ProviderHttpTransport,
            ReqwestProviderHttpTransport,
        },
        registry::ProviderCatalogEntry,
        rendered_page::{RenderedPageRuntime, RenderedPageSupportConfig, RenderedTextPage},
    },
};

pub const BROWSER_WORKER_PROVIDER_ID: &str = "browser_worker";
const BROWSER_WORKER_RENDERED_PAGE_CAPABILITY: &str = "rendered_page_extraction";
const BROWSER_WORKER_EXTERNAL_ID_ALIASES: &[QueryExternalIdAlias] = &[
    QueryExternalIdAlias::new("browser_worker_url", BROWSER_WORKER_PROVIDER_ID, false),
    QueryExternalIdAlias::new("browser_worker_id", BROWSER_WORKER_PROVIDER_ID, false),
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BrowserWorkerProviderConfig {
    pub(crate) rendered_pages: RenderedPageSupportConfig,
    pub(crate) extract_path: String,
}

impl BrowserWorkerProviderConfig {
    pub const DEFAULT_TIMEOUT_MS: u64 = 10_000;

    #[must_use]
    pub(crate) fn new(base_url: String, extract_path: String, timeout_ms: u64) -> Self {
        Self {
            rendered_pages: RenderedPageSupportConfig::new(base_url, timeout_ms),
            extract_path,
        }
    }

    #[must_use]
    pub fn from_env_lookup(mut lookup: impl FnMut(&str) -> Option<String>) -> Self {
        Self::new(
            lookup("NAKO_METADATA_SCRAPER_BROWSER_WORKER_BASE_URL")
                .unwrap_or_else(|| "http://nako-browser-worker:3000".to_owned()),
            lookup("NAKO_METADATA_SCRAPER_BROWSER_WORKER_EXTRACT_PATH")
                .filter(|value| !value.trim().is_empty())
                .unwrap_or_else(|| "/extract".to_owned()),
            lookup("NAKO_METADATA_SCRAPER_BROWSER_WORKER_TIMEOUT_MS")
                .and_then(|value| value.trim().parse::<u64>().ok())
                .filter(|value| *value > 0)
                .unwrap_or(Self::DEFAULT_TIMEOUT_MS),
        )
    }
}

#[must_use]
pub(crate) fn catalog_entry() -> ProviderCatalogEntry {
    ProviderCatalogEntry {
        id: ProviderId::BrowserWorker,
        default_enabled: false,
        enabled_env_var: "NAKO_METADATA_SCRAPER_PROVIDER_BROWSER_WORKER_ENABLED",
        capabilities: &[
            "metadata_suggestion",
            BROWSER_WORKER_RENDERED_PAGE_CAPABILITY,
        ],
        secret_reference: None,
        external_id_aliases: BROWSER_WORKER_EXTERNAL_ID_ALIASES,
        load_config: load_config,
        proxy_configured: |_| false,
        network_policy_key: None,
        build: build_provider,
    }
}

fn load_config(input: ProviderConfigInput<'_>) -> ProviderConfig {
    let lookup = input.lookup;
    ProviderConfig::browser_worker(
        input.enabled,
        BrowserWorkerProviderConfig::from_env_lookup(|name| lookup(name)),
    )
}

fn build_provider(config: &Config) -> ProviderBuildStatus {
    let Some(browser_worker_config) = config
        .provider_config(ProviderId::BrowserWorker)
        .and_then(|provider| provider.browser_worker_config().cloned())
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
    rendered_pages: RenderedPageRuntime<T>,
}

impl BrowserWorkerMetadataProvider<ReqwestProviderHttpTransport> {
    pub fn new(config: BrowserWorkerProviderConfig) -> ProviderHttpResult<Self> {
        let rendered_pages = RenderedPageRuntime::new(config.rendered_pages.clone())?;
        Ok(Self {
            config,
            rendered_pages,
        })
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
        let rendered_pages =
            RenderedPageRuntime::with_runtime(config.rendered_pages.clone(), runtime);
        Self {
            config,
            rendered_pages,
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
            let extracted = self
                .rendered_pages
                .extract_text(
                    BROWSER_WORKER_PROVIDER_ID,
                    "extract rendered page",
                    &self.config.extract_path,
                    &source_url,
                )
                .await?;
            candidates.push(rendered_text_candidate(extracted, query, &source_url));
        }

        Ok(candidates)
    }
}

fn rendered_text_candidate(
    rendered_page: RenderedTextPage,
    query: &MetadataQuery,
    source_url: &str,
) -> ProviderMetadataCandidate {
    let title = rendered_page
        .title
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| query.title.clone());
    let rendered_text = rendered_page
        .rendered_text
        .filter(|value| !value.trim().is_empty())
        .or(rendered_page.excerpt)
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
                value: rendered_page.final_url,
            }],
            provider_outcomes: vec![ProviderOutcome::BrowserWorkerRenderedText],
            provider_note: None,
        },
        artwork_candidates: Vec::new(),
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
            BrowserWorkerProviderConfig::new(
                "http://browser-worker.example".to_owned(),
                "/extract".to_owned(),
                10_000,
            ),
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
