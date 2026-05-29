mod client;
mod enrichment;
mod mapper;
mod parser;

use async_trait::async_trait;
use nako_addon_protocol::AddonSecretReferenceFieldDeclaration;

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
        registry::{
            ProviderCatalogEntry, ProviderDefaultFieldPreference, ProviderRenderedPageSupport,
        },
        render_drift::{
            BrowserWorkerRenderDriftCase, DEFAULT_SAMPLE_AV_NUMBER,
            ProviderRenderDriftCaseDescriptor, RENDER_DRIFT_SAMPLE_DMM_AV_NUMBER_ENV_VAR,
            SLOW_LIVE_RENDER_DRIFT_SELECTOR_TIMEOUT_MS, SLOW_LIVE_RENDER_DRIFT_TIMEOUT_MS,
        },
        rendered_page::{RenderedPageRuntime, RenderedPageSupportConfig},
    },
};

#[cfg(test)]
use crate::providers::rendered_av_fixture::{RenderedAvFixtureTransport, request_json_body};
#[cfg(test)]
use nako_addon_protocol::AddonArtworkKind;

pub const DMM_PROVIDER_ID: &str = "dmm";
pub(super) const DMM_URL_EXTERNAL_ID_PROVIDER: &str = "dmm_url";
pub(crate) const DMM_COOKIE_ENV_VAR: &str = "NAKO_METADATA_SCRAPER_DMM_COOKIE";
const DEFAULT_DMM_COOKIE: &str = "age_check_done=1";
const DMM_EXTERNAL_ID_CAPABILITIES: &[ProviderExternalIdCapability] = &[
    ProviderExternalIdCapability::new(
        DMM_PROVIDER_ID,
        ExternalIdValueKind::Opaque,
        true,
        true,
        &["dmm_id"],
        false,
    ),
    ProviderExternalIdCapability::new(
        DMM_URL_EXTERNAL_ID_PROVIDER,
        ExternalIdValueKind::Url,
        true,
        true,
        &["dmm_url"],
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
const DEFAULT_FIELD_PREFERENCES: &[ProviderDefaultFieldPreference] = &[
    ProviderDefaultFieldPreference::title(30),
    ProviderDefaultFieldPreference::outline(20),
    ProviderDefaultFieldPreference::trailer(20),
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DmmProviderConfig {
    pub(crate) base_url: String,
    pub(crate) rendered_pages: RenderedPageSupportConfig,
    pub(crate) render_path: String,
    pub(crate) cookie: Option<String>,
}

impl DmmProviderConfig {
    pub const DEFAULT_TIMEOUT_MS: u64 = 30_000;

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
            cookie: Some(DEFAULT_DMM_COOKIE.to_owned()),
        }
    }

    #[must_use]
    pub fn from_env_lookup(mut lookup: impl FnMut(&str) -> Option<String>) -> Self {
        let base_url = lookup("NAKO_METADATA_SCRAPER_DMM_BASE_URL")
            .and_then(non_empty_trimmed)
            .unwrap_or_else(|| "https://www.dmm.co.jp".to_owned());
        let browser_worker_base_url = lookup("NAKO_METADATA_SCRAPER_BROWSER_WORKER_BASE_URL")
            .and_then(non_empty_trimmed)
            .unwrap_or_else(|| "http://nako-browser-worker:3000".to_owned());
        let render_path = lookup("NAKO_METADATA_SCRAPER_BROWSER_WORKER_RENDER_PATH")
            .and_then(non_empty_trimmed)
            .unwrap_or_else(|| "/render".to_owned());
        let timeout_ms = lookup("NAKO_METADATA_SCRAPER_DMM_TIMEOUT_MS")
            .or_else(|| lookup("NAKO_METADATA_SCRAPER_BROWSER_WORKER_TIMEOUT_MS"))
            .and_then(|value| value.trim().parse::<u64>().ok())
            .filter(|value| *value > 0)
            .unwrap_or(Self::DEFAULT_TIMEOUT_MS);

        let mut config = Self::new(base_url, browser_worker_base_url, render_path, timeout_ms);
        config.rendered_pages = config.rendered_pages.with_env_defaults(|name| lookup(name));
        config.cookie = lookup(DMM_COOKIE_ENV_VAR)
            .and_then(non_empty_trimmed)
            .or_else(|| config.cookie.clone());
        config
    }

    #[must_use]
    pub const fn secret_field_id() -> &'static str {
        "dmm_cookie"
    }
}

#[must_use]
pub(crate) fn catalog_entry() -> ProviderCatalogEntry {
    ProviderCatalogEntry {
        id: ProviderId::Dmm,
        default_enabled: false,
        enabled_env_var: "NAKO_METADATA_SCRAPER_PROVIDER_DMM_ENABLED",
        capabilities: &[
            "metadata_suggestion",
            "av_number_search",
            "dmm_direct_lookup",
            "dmm_movie_search",
            "browser_worker_rendered_html",
        ],
        field_quality: crate::engine::ProviderFieldQualityDescriptor::new(600, 500, 600, 500),
        default_field_preferences: DEFAULT_FIELD_PREFERENCES,
        secret_reference: Some(AddonSecretReferenceFieldDeclaration::new(
            DmmProviderConfig::secret_field_id(),
            "DMM Cookie",
            Some(
                "Optional Secret Reference for FANZA/DMM age confirmation or regional access. The default provider cookie only confirms age; custom values are sent only to the browser worker as a Cookie header."
                    .to_owned(),
            ),
            false,
        )),
        external_id_capabilities: DMM_EXTERNAL_ID_CAPABILITIES,
        load_config: load_config,
        proxy_configured: |_| false,
        network_policy_key: None,
        rendered_page_support: Some(ProviderRenderedPageSupport::new(rendered_page_config)),
        render_drift_case: Some(
            ProviderRenderDriftCaseDescriptor::new(
                20,
                RENDER_DRIFT_SAMPLE_DMM_AV_NUMBER_ENV_VAR,
                DEFAULT_SAMPLE_AV_NUMBER,
                render_drift_case_from_config,
            )
            .with_generic_av_sample(),
        ),
        build: build_provider,
    }
}

fn rendered_page_config(provider: &ProviderConfig) -> Option<&RenderedPageSupportConfig> {
    provider.dmm_config().map(|config| &config.rendered_pages)
}

fn render_drift_case_from_config(
    provider: &ProviderConfig,
    sample: &str,
) -> Option<BrowserWorkerRenderDriftCase> {
    provider
        .dmm_config()
        .map(|config| render_drift_case(config, sample))
}

fn load_config(input: ProviderConfigInput<'_>) -> ProviderConfig {
    let lookup = input.lookup;
    ProviderConfig::dmm(
        input.enabled,
        DmmProviderConfig::from_env_lookup(|name| lookup(name)),
    )
}

fn build_provider(config: &Config) -> ProviderBuildStatus {
    let Some(dmm_config) = config
        .provider_config(ProviderId::Dmm)
        .and_then(|provider| provider.dmm_config().cloned())
    else {
        return ProviderBuildStatus::Unavailable;
    };
    match DmmMetadataProvider::new(dmm_config) {
        Ok(provider) => ProviderBuildStatus::Ready(Box::new(provider)),
        Err(_) => ProviderBuildStatus::Unavailable,
    }
}

#[must_use]
pub(crate) fn render_drift_case(
    config: &DmmProviderConfig,
    av_number: &str,
) -> BrowserWorkerRenderDriftCase {
    let render_timeout_ms = config
        .rendered_pages
        .timeout_ms
        .max(SLOW_LIVE_RENDER_DRIFT_TIMEOUT_MS);
    let selector_timeout_ms = config
        .rendered_pages
        .timeout_ms
        .max(SLOW_LIVE_RENDER_DRIFT_SELECTOR_TIMEOUT_MS);
    BrowserWorkerRenderDriftCase::new(
        "dmm-search",
        format!(
            "{}/search/=/searchstr={}/",
            config.base_url.trim_end_matches('/'),
            client::percent_encode_query(av_number)
        ),
    )
    .with_selector("a[href*=\"cid=\"]")
    .with_selector_timeout_ms(selector_timeout_ms)
    .with_header_from_env("cookie", DMM_COOKIE_ENV_VAR)
    .with_rendered_page_defaults(&config.rendered_pages)
    .with_render_timeout_ms(render_timeout_ms)
    .with_min_text_bytes(100)
    .with_min_html_bytes(500)
}

#[derive(Clone, Debug)]
pub struct DmmMetadataProvider<T = ReqwestProviderHttpTransport>
where
    T: ProviderHttpTransport,
{
    config: DmmProviderConfig,
    rendered_pages: RenderedPageRuntime<T>,
}

#[async_trait]
impl<T> MetadataProvider for DmmMetadataProvider<T>
where
    T: ProviderHttpTransport,
{
    fn id(&self) -> ProviderId {
        ProviderId::Dmm
    }

    fn supports_av_route(&self, route: AvNumberRoute) -> bool {
        route == AvNumberRoute::Censored
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
        config::DmmProviderConfig,
        engine::{MetadataQuery, QueryExternalId},
        providers::http_runtime::{ProviderHttpRuntime, ProviderHttpRuntimeConfig},
    };

    use super::*;

    #[tokio::test]
    async fn dmm_provider_uses_browser_worker_render_contract_for_av_search_and_detail() {
        let transport = RenderedAvFixtureTransport::new(DMM_PROVIDER_ID);
        transport.push_rendered_html(
            "https://dmm.example/search/=/searchstr=SSNI-644/",
            "DMM Search",
            r#"
<!doctype html>
<html>
<body>
  <a class="tmb" href="/digital/videoa/-/detail/=/cid=ssni00644/">
    <span>SSNI-644 DMM Search Title</span>
  </a>
  <a class="tmb" href="/digital/videoa/-/detail/=/cid=abp00001/">
    <span>ABP-001 Other Title</span>
  </a>
</body>
</html>"#,
        );
        transport.push_rendered_html(
            "https://dmm.example/digital/videoa/-/detail/=/cid=ssni00644/",
            "SSNI-644 DMM Official Title",
            r#"
<!doctype html>
<html>
<head>
  <meta property="og:image" content="//pics.example/dmm-cover.jpg">
</head>
<body>
  <h1 id="title">SSNI-644 DMM Official Title</h1>
  <div class="product-info">
    <p><span>品番:</span> SSNI-644</p>
    <p><span>発売日:</span> 2024-06-07</p>
    <p><span>収録時間:</span> 119分</p>
    <p><span>メーカー:</span> Studio Alpha</p>
    <p><span>レーベル:</span> Label Beta</p>
    <p><span>シリーズ:</span> Series Gamma</p>
    <p><span>監督:</span> Director Delta</p>
  </div>
  <div class="story">Official DMM outline.</div>
  <a href="/mono/actress/-/detail/=/id=1/">Actress One</a>
  <a href="/digital/videoa/-/list/=/article=keyword/id=101/">ドラマ</a>
  <a href="/digital/videoa/-/list/=/article=genre/id=201/">制服</a>
  <span class="review-average">4.4</span>
  <div class="sample-image-block">
    <img src="//pics.example/sample1.jpg">
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
        let provider = DmmMetadataProvider::with_runtime(
            DmmProviderConfig::new(
                "https://dmm.example".to_owned(),
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

        assert_eq!(candidates.len(), 1);
        let candidate = &candidates[0];
        assert_eq!(candidate.provider, "dmm");
        assert_eq!(candidate.provider_id, "dmm:cid:ssni00644");
        assert_eq!(
            candidate.patch.title.as_deref(),
            Some("SSNI-644 DMM Official Title")
        );
        assert_eq!(
            candidate.patch.overview.as_deref(),
            Some("Official DMM outline.")
        );
        assert_eq!(candidate.patch.release_date.as_deref(), Some("2024-06-07"));
        assert_eq!(candidate.patch.runtime_minutes, Some(119));
        assert_eq!(
            candidate.patch.genres.as_ref().unwrap(),
            &vec!["ドラマ".to_owned(), "制服".to_owned()]
        );
        assert!(
            candidate
                .patch
                .tags
                .as_ref()
                .unwrap()
                .contains(&"maker:Studio Alpha".to_owned())
        );
        assert!(
            candidate.facts.external_ids.iter().any(|id| {
                id.provider == AV_NUMBER_EXTERNAL_ID_PROVIDER && id.value == "SSNI-644"
            })
        );
        assert!(candidate.facts.external_ids.iter().any(|id| {
            id.provider == "dmm_url"
                && id.value == "https://dmm.example/digital/videoa/-/detail/=/cid=ssni00644/"
        }));
        assert_eq!(candidate.facts.community_score_milli, Some(880));
        assert_eq!(
            candidate.facts.av.as_ref().unwrap().actors,
            vec!["Actress One".to_owned()]
        );
        assert_eq!(
            candidate.facts.av.as_ref().unwrap().studio.as_deref(),
            Some("Studio Alpha")
        );
        assert_eq!(
            candidate.facts.av.as_ref().unwrap().label.as_deref(),
            Some("Label Beta")
        );
        assert_eq!(
            candidate.facts.av.as_ref().unwrap().series.as_deref(),
            Some("Series Gamma")
        );
        assert_eq!(
            candidate.facts.av.as_ref().unwrap().directors,
            vec!["Director Delta".to_owned()]
        );
        assert_eq!(
            candidate.facts.av.as_ref().unwrap().extrafanart_urls,
            vec!["https://pics.example/sample1.jpg".to_owned()]
        );
        assert_eq!(candidate.artwork_candidates.len(), 2);
        assert_eq!(
            candidate.artwork_candidates[0].facts.kind,
            AddonArtworkKind::Poster
        );
        assert_eq!(
            candidate.artwork_candidates[0].facts.source_url,
            "https://pics.example/dmm-cover.jpg"
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
            "https://dmm.example/search/=/searchstr=SSNI-644/"
        );
        let detail_body = request_json_body(&requests[1]);
        assert_eq!(
            detail_body["url"],
            "https://dmm.example/digital/videoa/-/detail/=/cid=ssni00644/"
        );
        assert_eq!(search_body["headers"]["cookie"], "age_check_done=1");
        assert_eq!(detail_body["headers"]["cookie"], "age_check_done=1");
    }

    #[tokio::test]
    async fn dmm_provider_uses_explicit_dmm_id_for_direct_detail_lookup() {
        let transport = RenderedAvFixtureTransport::new(DMM_PROVIDER_ID);
        transport.push_rendered_html(
            "https://dmm.example/digital/videoa/-/detail/=/cid=ssni00644/",
            "SSNI-644 Direct DMM Title",
            r#"
<!doctype html>
<html>
<head>
  <meta property="og:image" content="https://pics.example/direct-cover.jpg">
</head>
<body>
  <h1 id="title">SSNI-644 Direct DMM Title</h1>
  <div class="product-info">
    <span>品番:</span> SSNI-644
    <span>発売日:</span> 2024-06-07
    <span>収録時間:</span> 119分
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
        let provider = DmmMetadataProvider::with_runtime(
            DmmProviderConfig::new(
                "https://dmm.example".to_owned(),
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
                    provider: "dmm".to_owned(),
                    value: "ssni00644".to_owned(),
                }],
            })
            .await
            .unwrap();

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].provider_id, "dmm:cid:ssni00644");
        assert_eq!(
            candidates[0].patch.title.as_deref(),
            Some("SSNI-644 Direct DMM Title")
        );

        let requests = transport.requests();
        assert_eq!(requests.len(), 1);
        let body = request_json_body(&requests[0]);
        assert_eq!(
            body["url"],
            "https://dmm.example/digital/videoa/-/detail/=/cid=ssni00644/"
        );
        assert_eq!(body["headers"]["cookie"], "age_check_done=1");
    }

    #[tokio::test]
    async fn dmm_provider_skips_fc2_numbers() {
        let transport = RenderedAvFixtureTransport::new(DMM_PROVIDER_ID);
        let runtime = ProviderHttpRuntime::with_transport(
            ProviderHttpRuntimeConfig {
                retry_backoff_ms: 0,
                ..ProviderHttpRuntimeConfig::default()
            },
            transport.clone(),
        );
        let provider = DmmMetadataProvider::with_runtime(
            {
                let mut config = DmmProviderConfig::new(
                    "https://dmm.example".to_owned(),
                    "http://browser-worker.example".to_owned(),
                    "/render".to_owned(),
                    10_000,
                );
                config.cookie = None;
                config
            },
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

    #[tokio::test]
    async fn dmm_provider_can_disable_default_cookie_for_tests() {
        let transport = RenderedAvFixtureTransport::new(DMM_PROVIDER_ID);
        transport.push_rendered_html(
            "https://dmm.example/digital/videoa/-/detail/=/cid=ssni00644/",
            "SSNI-644 Direct DMM Title",
            r#"
<!doctype html>
<html>
<body><h1 id="title">SSNI-644 Direct DMM Title</h1></body>
</html>"#,
        );
        let runtime = ProviderHttpRuntime::with_transport(
            ProviderHttpRuntimeConfig {
                retry_backoff_ms: 0,
                ..ProviderHttpRuntimeConfig::default()
            },
            transport.clone(),
        );
        let mut config = DmmProviderConfig::new(
            "https://dmm.example".to_owned(),
            "http://browser-worker.example".to_owned(),
            "/render".to_owned(),
            10_000,
        );
        config.cookie = None;
        let provider = DmmMetadataProvider::with_runtime(config, runtime);

        provider
            .suggest(&MetadataQuery {
                title: "SSNI-644".to_owned(),
                year: None,
                language: "zh-CN".to_owned(),
                external_ids: vec![QueryExternalId {
                    provider: "dmm".to_owned(),
                    value: "ssni00644".to_owned(),
                }],
            })
            .await
            .unwrap();

        let body = request_json_body(&transport.requests()[0]);
        assert!(body.get("headers").is_none());
    }
}
