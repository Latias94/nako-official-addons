mod client;
mod enrichment;
mod mapper;
mod parser;

use async_trait::async_trait;

use crate::{
    Config,
    config::{ProviderConfig, ProviderId, non_empty_trimmed},
    engine::{
        ExternalIdValueKind, MetadataQuery, ProviderExternalIdCapability,
        ProviderMetadataCandidate,
        av::{AV_NUMBER_EXTERNAL_ID_PROVIDER, AvNumberRoute},
    },
    providers::{
        MetadataProvider, ProviderBuildStatus, ProviderConfigInput,
        http_runtime::{ProviderHttpTransport, ReqwestProviderHttpTransport},
        registry::ProviderCatalogEntry,
        rendered_page::{RenderedPageRuntime, RenderedPageSupportConfig},
    },
};

#[cfg(test)]
use crate::providers::rendered_av_fixture::{RenderedAvFixtureTransport, request_json_body};
#[cfg(test)]
use nako_addon_protocol::AddonArtworkKind;

pub const JAVDB_PROVIDER_ID: &str = "javdb";
const JAVDB_EXTERNAL_ID_CAPABILITIES: &[ProviderExternalIdCapability] = &[
    ProviderExternalIdCapability::new(
        JAVDB_PROVIDER_ID,
        ExternalIdValueKind::Opaque,
        true,
        true,
        &["javdb_id"],
        false,
    ),
    ProviderExternalIdCapability::new(
        AV_NUMBER_EXTERNAL_ID_PROVIDER,
        ExternalIdValueKind::Opaque,
        true,
        true,
        &["av_number"],
        false,
    ),
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JavdbProviderConfig {
    pub(crate) base_url: String,
    pub(crate) rendered_pages: RenderedPageSupportConfig,
    pub(crate) render_path: String,
}

impl JavdbProviderConfig {
    pub const DEFAULT_TIMEOUT_MS: u64 = 10_000;

    #[must_use]
    pub(crate) fn new(
        base_url: String,
        browser_worker_base_url: String,
        render_path: String,
        timeout_ms: u64,
    ) -> Self {
        Self {
            base_url,
            rendered_pages: RenderedPageSupportConfig::new(browser_worker_base_url, timeout_ms),
            render_path,
        }
    }

    #[must_use]
    pub fn from_env_lookup(mut lookup: impl FnMut(&str) -> Option<String>) -> Self {
        let base_url = lookup("NAKO_METADATA_SCRAPER_JAVDB_BASE_URL")
            .and_then(non_empty_trimmed)
            .unwrap_or_else(|| "https://javdb.com".to_owned());
        let browser_worker_base_url = lookup("NAKO_METADATA_SCRAPER_BROWSER_WORKER_BASE_URL")
            .and_then(non_empty_trimmed)
            .unwrap_or_else(|| "http://nako-browser-worker:3000".to_owned());
        let render_path = lookup("NAKO_METADATA_SCRAPER_BROWSER_WORKER_RENDER_PATH")
            .and_then(non_empty_trimmed)
            .unwrap_or_else(|| "/render".to_owned());
        let timeout_ms = lookup("NAKO_METADATA_SCRAPER_JAVDB_TIMEOUT_MS")
            .or_else(|| lookup("NAKO_METADATA_SCRAPER_BROWSER_WORKER_TIMEOUT_MS"))
            .and_then(|value| value.trim().parse::<u64>().ok())
            .filter(|value| *value > 0)
            .unwrap_or(Self::DEFAULT_TIMEOUT_MS);

        let mut config = Self::new(base_url, browser_worker_base_url, render_path, timeout_ms);
        config.rendered_pages = config.rendered_pages.with_env_defaults(|name| lookup(name));
        config
    }
}

#[must_use]
pub(crate) fn catalog_entry() -> ProviderCatalogEntry {
    ProviderCatalogEntry {
        id: ProviderId::Javdb,
        default_enabled: false,
        enabled_env_var: "NAKO_METADATA_SCRAPER_PROVIDER_JAVDB_ENABLED",
        capabilities: &[
            "metadata_suggestion",
            "av_number_search",
            "javdb_movie_search",
            "browser_worker_rendered_html",
        ],
        field_quality: crate::engine::ProviderFieldQualityDescriptor::new(500, 550, 500, 400),
        secret_reference: None,
        external_id_capabilities: JAVDB_EXTERNAL_ID_CAPABILITIES,
        load_config: load_config,
        proxy_configured: |_| false,
        network_policy_key: None,
        build: build_provider,
    }
}

fn load_config(input: ProviderConfigInput<'_>) -> ProviderConfig {
    let lookup = input.lookup;
    ProviderConfig::javdb(
        input.enabled,
        JavdbProviderConfig::from_env_lookup(|name| lookup(name)),
    )
}

fn build_provider(config: &Config) -> ProviderBuildStatus {
    let Some(javdb_config) = config
        .provider_config(ProviderId::Javdb)
        .and_then(|provider| provider.javdb_config().cloned())
    else {
        return ProviderBuildStatus::Unavailable;
    };
    match JavdbMetadataProvider::new(javdb_config) {
        Ok(provider) => ProviderBuildStatus::Ready(Box::new(provider)),
        Err(_) => ProviderBuildStatus::Unavailable,
    }
}

#[derive(Clone, Debug)]
pub struct JavdbMetadataProvider<T = ReqwestProviderHttpTransport>
where
    T: ProviderHttpTransport,
{
    config: JavdbProviderConfig,
    rendered_pages: RenderedPageRuntime<T>,
}

#[async_trait]
impl<T> MetadataProvider for JavdbMetadataProvider<T>
where
    T: ProviderHttpTransport,
{
    fn id(&self) -> ProviderId {
        ProviderId::Javdb
    }

    fn supports_av_route(&self, route: AvNumberRoute) -> bool {
        route != AvNumberRoute::Fc2
    }

    async fn suggest(
        &self,
        query: &MetadataQuery,
    ) -> anyhow::Result<Vec<ProviderMetadataCandidate>> {
        self.suggest_candidates(query).await
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        config::JavdbProviderConfig,
        engine::{MetadataQuery, QueryExternalId, av::AvNumberSource},
        providers::http_runtime::{ProviderHttpRuntime, ProviderHttpRuntimeConfig},
    };

    use super::*;

    #[tokio::test]
    async fn javdb_provider_uses_browser_worker_render_contract_for_av_search_and_detail() {
        let transport = RenderedAvFixtureTransport::new(JAVDB_PROVIDER_ID);
        transport.push_rendered_html(
            "https://javdb.example/search?q=SSNI-644&locale=zh",
            "JavDB Search",
            r#"
<!doctype html>
<html>
<body>
  <a class="box" href="/v/abc123">
    <div class="video-title">SSNI-644 Synthetic AV Title</div>
    <div class="meta">SSNI-644</div>
  </a>
  <a class="box" href="/v/other999">
    <div class="video-title">ABP-001 Other Title</div>
  </a>
</body>
</html>"#,
        );
        transport.push_rendered_html(
            "https://javdb.example/v/abc123",
            "SSNI-644 Synthetic AV Title",
            r#"
<!doctype html>
<html>
<head>
  <meta property="og:image" content="https://img.example/cover.jpg">
</head>
<body>
  <h2 class="title"><strong class="current-title">SSNI-644 Synthetic AV Title</strong></h2>
  <a class="copy-to-clipboard" data-clipboard-text="SSNI-644">SSNI-644</a>
  <div class="movie-panel-info">
    <span>日期:</span> 2024-05-01
    <span>時長:</span> 121 分鐘
    <span>片商:</span> Studio Alpha
    <span>發行:</span> Publisher Beta
    <span>系列:</span> Series Gamma
    <span>導演:</span> Director Delta
  </div>
  <a href="/actors/a1">Actor One</a>
  <a href="/actors/a2">Actor Two</a>
  <a href="/tags/t1">剧情</a>
  <a href="/tags/t2">制服</a>
  <strong class="score">4.6</strong>
  <span class="wanted">123 人想看</span>
  <div class="preview-images">
    <img src="//img.example/preview1.jpg">
    <img src="https://img.example/preview2.jpg">
  </div>
</body>
</html>"#,
        );
        let runtime = ProviderHttpRuntime::with_transport(
            ProviderHttpRuntimeConfig {
                retry_backoff_ms: 0,
                ..ProviderHttpRuntimeConfig::default()
            },
            transport.clone(),
        );
        let provider = JavdbMetadataProvider::with_runtime(
            JavdbProviderConfig::new(
                "https://javdb.example".to_owned(),
                "http://browser-worker.example".to_owned(),
                "/render".to_owned(),
                10_000,
            ),
            runtime,
        );

        let candidates = provider
            .suggest(&MetadataQuery {
                title: "SSNI-644".to_owned(),
                year: None,
                language: "zh-CN".to_owned(),
                external_ids: vec![],
            })
            .await
            .unwrap();

        assert_eq!(candidates.len(), 1);
        let candidate = &candidates[0];
        assert_eq!(candidate.provider, "javdb");
        assert_eq!(candidate.provider_id, "javdb:movie:abc123");
        assert_eq!(
            candidate.patch.title.as_deref(),
            Some("SSNI-644 Synthetic AV Title")
        );
        assert_eq!(candidate.patch.release_date.as_deref(), Some("2024-05-01"));
        assert_eq!(candidate.patch.runtime_minutes, Some(121));
        assert_eq!(
            candidate.patch.genres.as_ref().unwrap(),
            &vec!["剧情".to_owned(), "制服".to_owned()]
        );
        assert!(
            candidate
                .patch
                .tags
                .as_ref()
                .unwrap()
                .contains(&"av_number:SSNI-644".to_owned())
        );
        assert!(
            candidate.facts.external_ids.iter().any(|id| {
                id.provider == AV_NUMBER_EXTERNAL_ID_PROVIDER && id.value == "SSNI-644"
            })
        );
        assert!(candidate.facts.external_ids.iter().any(|id| {
            id.provider == "javdb_url" && id.value == "https://javdb.example/v/abc123"
        }));
        assert_eq!(candidate.facts.community_score_milli, Some(920));
        assert_eq!(candidate.facts.community_vote_count, Some(123));
        assert_eq!(
            candidate.facts.av.as_ref().unwrap().actors,
            vec!["Actor One".to_owned(), "Actor Two".to_owned()]
        );
        assert_eq!(
            candidate.facts.av.as_ref().unwrap().studio.as_deref(),
            Some("Studio Alpha")
        );
        assert_eq!(candidate.facts.av.as_ref().unwrap().wanted_count, Some(123));
        assert_eq!(
            candidate.facts.av.as_ref().unwrap().extrafanart_urls,
            vec![
                "https://img.example/preview1.jpg".to_owned(),
                "https://img.example/preview2.jpg".to_owned()
            ]
        );
        assert_eq!(candidate.artwork_candidates.len(), 3);
        assert_eq!(
            candidate.artwork_candidates[0].facts.kind,
            AddonArtworkKind::Poster
        );
        assert_eq!(
            candidate.artwork_candidates[1].facts.kind,
            AddonArtworkKind::Backdrop
        );

        let requests = transport.requests();
        assert_eq!(requests.len(), 2);
        assert_eq!(requests[0].url, "http://browser-worker.example/render");
        assert_eq!(requests[1].url, "http://browser-worker.example/render");
        let search_body = request_json_body(&requests[0]);
        assert_eq!(
            search_body["url"],
            "https://javdb.example/search?q=SSNI-644&locale=zh"
        );
        let detail_body = request_json_body(&requests[1]);
        assert_eq!(detail_body["url"], "https://javdb.example/v/abc123");
    }

    #[tokio::test]
    async fn javdb_provider_skips_requests_without_av_number() {
        let transport = RenderedAvFixtureTransport::new(JAVDB_PROVIDER_ID);
        let runtime = ProviderHttpRuntime::with_transport(
            ProviderHttpRuntimeConfig {
                retry_backoff_ms: 0,
                ..ProviderHttpRuntimeConfig::default()
            },
            transport.clone(),
        );
        let provider = JavdbMetadataProvider::with_runtime(
            JavdbProviderConfig::new(
                "https://javdb.example".to_owned(),
                "http://browser-worker.example".to_owned(),
                "/render".to_owned(),
                10_000,
            ),
            runtime,
        );

        let candidates = provider
            .suggest(&MetadataQuery {
                title: "The Matrix".to_owned(),
                year: Some(1999),
                language: "en-US".to_owned(),
                external_ids: vec![],
            })
            .await
            .unwrap();

        assert!(candidates.is_empty());
        assert!(transport.requests().is_empty());
    }

    #[tokio::test]
    async fn javdb_provider_uses_explicit_javdb_id_for_direct_detail_lookup() {
        let transport = RenderedAvFixtureTransport::new(JAVDB_PROVIDER_ID);
        transport.push_rendered_html(
            "https://javdb.example/v/abc123",
            "SSNI-644 Direct Lookup Title",
            r#"
<!doctype html>
<html>
<head>
  <meta property="og:image" content="https://img.example/direct-cover.jpg">
</head>
<body>
  <h2 class="title"><strong class="current-title">SSNI-644 Direct Lookup Title</strong></h2>
  <a class="copy-to-clipboard" data-clipboard-text="SSNI-644">SSNI-644</a>
  <div class="movie-panel-info">
    <span>日期:</span> 2024-05-01
    <span>時長:</span> 121 分鐘
  </div>
  <a href="/tags/t1">剧情</a>
</body>
</html>"#,
        );
        let runtime = ProviderHttpRuntime::with_transport(
            ProviderHttpRuntimeConfig {
                retry_backoff_ms: 0,
                ..ProviderHttpRuntimeConfig::default()
            },
            transport.clone(),
        );
        let provider = JavdbMetadataProvider::with_runtime(
            JavdbProviderConfig::new(
                "https://javdb.example".to_owned(),
                "http://browser-worker.example".to_owned(),
                "/render".to_owned(),
                10_000,
            ),
            runtime,
        );

        let candidates = provider
            .suggest(&MetadataQuery {
                title: "Untrusted Raw Title".to_owned(),
                year: None,
                language: "zh-CN".to_owned(),
                external_ids: vec![QueryExternalId {
                    provider: "javdb".to_owned(),
                    value: "abc123".to_owned(),
                }],
            })
            .await
            .unwrap();

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].provider_id, "javdb:movie:abc123");
        assert_eq!(
            candidates[0].patch.title.as_deref(),
            Some("SSNI-644 Direct Lookup Title")
        );
        assert!(
            candidates[0].facts.external_ids.iter().any(|id| {
                id.provider == AV_NUMBER_EXTERNAL_ID_PROVIDER && id.value == "SSNI-644"
            })
        );

        let requests = transport.requests();
        assert_eq!(requests.len(), 1);
        let body = request_json_body(&requests[0]);
        assert_eq!(body["url"], "https://javdb.example/v/abc123");
    }

    #[tokio::test]
    async fn javdb_provider_skips_fc2_numbers() {
        let transport = RenderedAvFixtureTransport::new(JAVDB_PROVIDER_ID);
        let runtime = ProviderHttpRuntime::with_transport(
            ProviderHttpRuntimeConfig {
                retry_backoff_ms: 0,
                ..ProviderHttpRuntimeConfig::default()
            },
            transport.clone(),
        );
        let provider = JavdbMetadataProvider::with_runtime(
            JavdbProviderConfig::new(
                "https://javdb.example".to_owned(),
                "http://browser-worker.example".to_owned(),
                "/render".to_owned(),
                10_000,
            ),
            runtime,
        );

        let candidates = provider
            .suggest(&MetadataQuery::from_payload(
                &serde_json::json!({"file_name": "FC2PPV-1723984.mp4"}),
                "zh-CN",
            ))
            .await
            .unwrap();

        assert!(candidates.is_empty());
        assert!(transport.requests().is_empty());
    }

    #[test]
    fn javdb_query_facts_can_be_derived_from_explicit_av_external_id() {
        let facts =
            crate::engine::av::facts_from_text("fc2ppv 1723984", AvNumberSource::AvNumber).unwrap();

        assert_eq!(facts.number, "FC2-1723984");
    }
}
