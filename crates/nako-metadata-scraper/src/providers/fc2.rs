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
        registry::{ProviderCatalogEntry, ProviderRenderedPageSupport},
        render_drift::{
            BrowserWorkerRenderDriftCase, DEFAULT_SAMPLE_FC2_AV_NUMBER,
            ProviderRenderDriftCaseDescriptor, RENDER_DRIFT_SAMPLE_FC2_AV_NUMBER_ENV_VAR,
            SLOW_LIVE_RENDER_DRIFT_SELECTOR_TIMEOUT_MS, SLOW_LIVE_RENDER_DRIFT_TIMEOUT_MS,
        },
        rendered_page::{RenderedPageRuntime, RenderedPageSupportConfig},
    },
};

#[cfg(test)]
use crate::providers::rendered_av_fixture::{RenderedAvFixtureTransport, request_json_body};
#[cfg(test)]
use nako_addon_protocol::AddonArtworkKind;

pub const FC2_PROVIDER_ID: &str = "fc2";
const FC2_EXTERNAL_ID_CAPABILITIES: &[ProviderExternalIdCapability] = &[
    ProviderExternalIdCapability::new(
        FC2_PROVIDER_ID,
        ExternalIdValueKind::Opaque,
        true,
        true,
        &["fc2_id"],
        false,
    ),
    ProviderExternalIdCapability::new(
        AV_NUMBER_EXTERNAL_ID_PROVIDER,
        ExternalIdValueKind::Opaque,
        true,
        true,
        &[],
        false,
    ),
];

fn fc2_detail_url(base_url: &str, article_id: &str) -> String {
    format!("{}/article/{}/", base_url.trim_end_matches('/'), article_id)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Fc2ProviderConfig {
    pub(crate) base_url: String,
    pub(crate) rendered_pages: RenderedPageSupportConfig,
    pub(crate) render_path: String,
}

impl Fc2ProviderConfig {
    pub const DEFAULT_TIMEOUT_MS: u64 = 60_000;

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
        let base_url = lookup("NAKO_METADATA_SCRAPER_FC2_BASE_URL")
            .and_then(non_empty_trimmed)
            .unwrap_or_else(|| "https://adult.contents.fc2.com".to_owned());
        let browser_worker_base_url = lookup("NAKO_METADATA_SCRAPER_BROWSER_WORKER_BASE_URL")
            .and_then(non_empty_trimmed)
            .unwrap_or_else(|| "http://nako-browser-worker:3000".to_owned());
        let render_path = lookup("NAKO_METADATA_SCRAPER_BROWSER_WORKER_RENDER_PATH")
            .and_then(non_empty_trimmed)
            .unwrap_or_else(|| "/render".to_owned());
        let timeout_ms = lookup("NAKO_METADATA_SCRAPER_FC2_TIMEOUT_MS")
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
        id: ProviderId::Fc2,
        default_enabled: false,
        enabled_env_var: "NAKO_METADATA_SCRAPER_PROVIDER_FC2_ENABLED",
        capabilities: &[
            "metadata_suggestion",
            "av_number_search",
            "fc2_direct_lookup",
            "browser_worker_rendered_html",
        ],
        field_quality: crate::engine::ProviderFieldQualityDescriptor::new(450, 350, 450, 300),
        secret_reference: None,
        external_id_capabilities: FC2_EXTERNAL_ID_CAPABILITIES,
        load_config: load_config,
        proxy_configured: |_| false,
        network_policy_key: None,
        rendered_page_support: Some(ProviderRenderedPageSupport::new(rendered_page_config)),
        render_drift_case: Some(ProviderRenderDriftCaseDescriptor::new(
            100,
            RENDER_DRIFT_SAMPLE_FC2_AV_NUMBER_ENV_VAR,
            DEFAULT_SAMPLE_FC2_AV_NUMBER,
            render_drift_case_from_config,
        )),
        build: build_provider,
    }
}

fn rendered_page_config(provider: &ProviderConfig) -> Option<&RenderedPageSupportConfig> {
    provider.fc2_config().map(|config| &config.rendered_pages)
}

fn render_drift_case_from_config(
    provider: &ProviderConfig,
    sample: &str,
) -> Option<BrowserWorkerRenderDriftCase> {
    provider
        .fc2_config()
        .map(|config| render_drift_case(config, sample))
}

fn load_config(input: ProviderConfigInput<'_>) -> ProviderConfig {
    let lookup = input.lookup;
    ProviderConfig::fc2(
        input.enabled,
        Fc2ProviderConfig::from_env_lookup(|name| lookup(name)),
    )
}

fn build_provider(config: &Config) -> ProviderBuildStatus {
    let Some(fc2_config) = config
        .provider_config(ProviderId::Fc2)
        .and_then(|provider| provider.fc2_config().cloned())
    else {
        return ProviderBuildStatus::Unavailable;
    };
    match Fc2MetadataProvider::new(fc2_config) {
        Ok(provider) => ProviderBuildStatus::Ready(Box::new(provider)),
        Err(_) => ProviderBuildStatus::Unavailable,
    }
}

#[must_use]
pub(crate) fn render_drift_case(
    config: &Fc2ProviderConfig,
    av_number: &str,
) -> BrowserWorkerRenderDriftCase {
    let article_id = parser::article_id_from_av_number(av_number)
        .unwrap_or_else(|| av_number.trim().trim_start_matches("FC2-").to_owned());
    let render_timeout_ms = config
        .rendered_pages
        .timeout_ms
        .max(SLOW_LIVE_RENDER_DRIFT_TIMEOUT_MS);
    let selector_timeout_ms = config
        .rendered_pages
        .timeout_ms
        .max(SLOW_LIVE_RENDER_DRIFT_SELECTOR_TIMEOUT_MS);
    BrowserWorkerRenderDriftCase::new("fc2-detail", fc2_detail_url(&config.base_url, &article_id))
        .with_selector("h1, .items_article_info, .items_article_HeadInfo")
        .with_selector_timeout_ms(selector_timeout_ms)
        .with_rendered_page_defaults(&config.rendered_pages)
        .with_render_timeout_ms(render_timeout_ms)
        .with_min_text_bytes(100)
        .with_min_html_bytes(500)
}

#[derive(Clone, Debug)]
pub struct Fc2MetadataProvider<T = ReqwestProviderHttpTransport>
where
    T: ProviderHttpTransport,
{
    config: Fc2ProviderConfig,
    rendered_pages: RenderedPageRuntime<T>,
}

#[async_trait]
impl<T> MetadataProvider for Fc2MetadataProvider<T>
where
    T: ProviderHttpTransport,
{
    fn id(&self) -> ProviderId {
        ProviderId::Fc2
    }

    fn supports_av_route(&self, route: AvNumberRoute) -> bool {
        route == AvNumberRoute::Fc2
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
        config::Fc2ProviderConfig,
        engine::{MetadataQuery, QueryExternalId},
        providers::http_runtime::{ProviderHttpRuntime, ProviderHttpRuntimeConfig},
    };

    use super::*;

    #[tokio::test]
    async fn fc2_provider_uses_browser_worker_render_contract_for_direct_lookup() {
        let transport = RenderedAvFixtureTransport::new(FC2_PROVIDER_ID);
        transport.push_rendered_html(
            "https://fc2.example/article/1723984/",
            "FC2-1723984 Synthetic Title",
            r#"
<!doctype html>
<html>
<head>
  <meta property="og:image" content="https://img.example/fc2-cover.jpg">
</head>
<body>
  <h1>FC2-1723984 Synthetic Title</h1>
  <div class="items_article_info">
    <p><span>販売日:</span> 2024-04-20</p>
    <p><span>販売者:</span> FC2 Seller</p>
    <p><span>収録時間:</span> 88分</p>
  </div>
  <section class="items_article_Comment">Synthetic outline.</section>
  <a href="/genre/1">素人</a>
  <a href="/tag/2">個人撮影</a>
  <img class="items_article_MainitemThumb" src="//img.example/fc2-thumb.jpg">
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
        let provider = Fc2MetadataProvider::with_runtime(
            Fc2ProviderConfig::new(
                "https://fc2.example".to_owned(),
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

        assert_eq!(candidates.len(), 1);
        let candidate = &candidates[0];
        assert_eq!(candidate.provider, "fc2");
        assert_eq!(candidate.provider_id, "fc2:article:1723984");
        assert_eq!(
            candidate.patch.title.as_deref(),
            Some("FC2-1723984 Synthetic Title")
        );
        assert_eq!(candidate.patch.release_date.as_deref(), Some("2024-04-20"));
        assert_eq!(candidate.patch.runtime_minutes, Some(88));
        assert_eq!(
            candidate.patch.overview.as_deref(),
            Some("Synthetic outline.")
        );
        assert_eq!(
            candidate.facts.av.as_ref().unwrap().publisher.as_deref(),
            Some("FC2 Seller")
        );
        assert_eq!(
            candidate.facts.av.as_ref().unwrap().thumb_url.as_deref(),
            Some("https://img.example/fc2-cover.jpg")
        );
        assert!(candidate.facts.external_ids.iter().any(|id| {
            id.provider == AV_NUMBER_EXTERNAL_ID_PROVIDER && id.value == "FC2-1723984"
        }));
        assert_eq!(candidate.facts.community_vote_count, None);
        assert_eq!(candidate.artwork_candidates.len(), 1);
        assert_eq!(
            candidate.artwork_candidates[0].facts.kind,
            AddonArtworkKind::Poster
        );
        assert_eq!(
            candidate.artwork_candidates[0].facts.source_url,
            "https://img.example/fc2-cover.jpg"
        );

        let requests = transport.requests();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].url, "http://browser-worker.example/render");
        let body = request_json_body(&requests[0]);
        assert_eq!(body["url"], "https://fc2.example/article/1723984/");
    }

    #[tokio::test]
    async fn fc2_provider_skips_non_fc2_numbers() {
        let transport = RenderedAvFixtureTransport::new(FC2_PROVIDER_ID);
        let runtime = ProviderHttpRuntime::with_transport(
            ProviderHttpRuntimeConfig {
                retry_backoff_ms: 0,
                ..ProviderHttpRuntimeConfig::default()
            },
            transport.clone(),
        );
        let provider = Fc2MetadataProvider::with_runtime(
            Fc2ProviderConfig::new(
                "https://fc2.example".to_owned(),
                "http://browser-worker.example".to_owned(),
                "/render".to_owned(),
                10_000,
            ),
            runtime,
        );

        let candidates = provider
            .suggest(&MetadataQuery::from_payload(
                &serde_json::json!({"file_name": "SSNI-00644.mp4"}),
                "zh-CN",
            ))
            .await
            .unwrap();

        assert!(candidates.is_empty());
        assert!(transport.requests().is_empty());
    }

    #[tokio::test]
    async fn fc2_provider_uses_explicit_fc2_id_for_direct_detail_lookup() {
        let transport = RenderedAvFixtureTransport::new(FC2_PROVIDER_ID);
        transport.push_rendered_html(
            "https://fc2.example/article/1723984/",
            "FC2-1723984 Direct Lookup Title",
            r#"
<!doctype html>
<html>
<head>
  <meta property="og:image" content="https://img.example/fc2-direct-cover.jpg">
</head>
<body>
  <h1>FC2-1723984 Direct Lookup Title</h1>
  <div class="items_article_info">
    <span>販売日:</span> 2024-04-20
    <span>販売者:</span> FC2 Seller
    <span>収録時間:</span> 88分
  </div>
  <section class="items_article_Comment">Direct lookup outline.</section>
  <a href="/genre/1">素人</a>
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
        let provider = Fc2MetadataProvider::with_runtime(
            Fc2ProviderConfig::new(
                "https://fc2.example".to_owned(),
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
                    provider: "fc2".to_owned(),
                    value: "1723984".to_owned(),
                }],
            })
            .await
            .unwrap();

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].provider_id, "fc2:article:1723984");
        assert_eq!(
            candidates[0].patch.title.as_deref(),
            Some("FC2-1723984 Direct Lookup Title")
        );
        assert!(candidates[0].facts.external_ids.iter().any(|id| {
            id.provider == AV_NUMBER_EXTERNAL_ID_PROVIDER && id.value == "FC2-1723984"
        }));

        let requests = transport.requests();
        assert_eq!(requests.len(), 1);
        let body = request_json_body(&requests[0]);
        assert_eq!(body["url"], "https://fc2.example/article/1723984/");
    }
}
