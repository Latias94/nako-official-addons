mod client;
mod enrichment;
mod mapper;
mod parser;
#[cfg(test)]
mod test_support;

use async_trait::async_trait;

use crate::{
    Config,
    config::{ProviderConfig, ProviderId},
    engine::{
        ExternalIdValueKind, MetadataQuery, ProviderExternalIdCapability, ProviderMetadataCandidate,
    },
    providers::{
        MetadataProvider, ProviderBuildStatus, ProviderConfigInput,
        http_runtime::{ProviderHttpTransport, ReqwestProviderHttpTransport},
        registry::ProviderCatalogEntry,
        render_drift::BrowserWorkerRenderDriftCase,
        rendered_page::{RenderedPageRuntime, RenderedPageSupportConfig},
    },
};

#[cfg(test)]
use nako_addon_protocol::AddonArtworkKind;
#[cfg(test)]
use test_support::FakeTransport;

pub const DOUBAN_PROVIDER_ID: &str = "douban";
const DOUBAN_DETAIL_ENRICHMENT_LIMIT: usize = 1;
const DOUBAN_EXTERNAL_ID_CAPABILITIES: &[ProviderExternalIdCapability] =
    &[ProviderExternalIdCapability::new(
        DOUBAN_PROVIDER_ID,
        ExternalIdValueKind::Numeric,
        false,
        true,
        &[],
        true,
    )];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DoubanProviderConfig {
    pub(crate) search_base_url: String,
    pub(crate) rendered_pages: RenderedPageSupportConfig,
    pub(crate) render_path: String,
}

impl DoubanProviderConfig {
    pub const DEFAULT_TIMEOUT_MS: u64 = 10_000;

    #[must_use]
    pub(crate) fn new(
        search_base_url: String,
        browser_worker_base_url: String,
        render_path: String,
        timeout_ms: u64,
    ) -> Self {
        Self {
            search_base_url,
            rendered_pages: RenderedPageSupportConfig::new(browser_worker_base_url, timeout_ms),
            render_path,
        }
    }

    #[must_use]
    pub fn from_env_lookup(mut lookup: impl FnMut(&str) -> Option<String>) -> Self {
        let search_base_url = lookup("NAKO_METADATA_SCRAPER_DOUBAN_SEARCH_BASE_URL")
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "https://movie.douban.com/subject_search".to_owned());
        let browser_worker_base_url = lookup("NAKO_METADATA_SCRAPER_BROWSER_WORKER_BASE_URL")
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "http://nako-browser-worker:3000".to_owned());
        let render_path = lookup("NAKO_METADATA_SCRAPER_BROWSER_WORKER_RENDER_PATH")
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "/render".to_owned());
        let timeout_ms = lookup("NAKO_METADATA_SCRAPER_DOUBAN_TIMEOUT_MS")
            .or_else(|| lookup("NAKO_METADATA_SCRAPER_BROWSER_WORKER_TIMEOUT_MS"))
            .and_then(|value| value.trim().parse::<u64>().ok())
            .filter(|value| *value > 0)
            .unwrap_or(Self::DEFAULT_TIMEOUT_MS);

        let mut config = Self::new(
            search_base_url,
            browser_worker_base_url,
            render_path,
            timeout_ms,
        );
        config.rendered_pages = config.rendered_pages.with_env_defaults(|name| lookup(name));
        config
    }
}

#[must_use]
pub(crate) fn catalog_entry() -> ProviderCatalogEntry {
    ProviderCatalogEntry {
        id: ProviderId::Douban,
        default_enabled: false,
        enabled_env_var: "NAKO_METADATA_SCRAPER_PROVIDER_DOUBAN_ENABLED",
        capabilities: &[
            "metadata_suggestion",
            "movie_search",
            "browser_worker_rendered_html",
        ],
        field_quality: Default::default(),
        secret_reference: None,
        external_id_capabilities: DOUBAN_EXTERNAL_ID_CAPABILITIES,
        load_config: load_config,
        proxy_configured: |_| false,
        network_policy_key: None,
        build: build_provider,
    }
}

fn load_config(input: ProviderConfigInput<'_>) -> ProviderConfig {
    let lookup = input.lookup;
    ProviderConfig::douban(
        input.enabled,
        DoubanProviderConfig::from_env_lookup(|name| lookup(name)),
    )
}

fn build_provider(config: &Config) -> ProviderBuildStatus {
    let Some(douban_config) = config
        .provider_config(ProviderId::Douban)
        .and_then(|provider| provider.douban_config().cloned())
    else {
        return ProviderBuildStatus::Unavailable;
    };
    match DoubanMetadataProvider::new(douban_config) {
        Ok(provider) => ProviderBuildStatus::Ready(Box::new(provider)),
        Err(_) => ProviderBuildStatus::Unavailable,
    }
}

#[must_use]
pub(crate) fn render_drift_case(
    config: &DoubanProviderConfig,
    title: &str,
) -> BrowserWorkerRenderDriftCase {
    BrowserWorkerRenderDriftCase::new(
        "douban-search",
        format!(
            "{}?search_text={}",
            config.search_base_url.trim_end_matches('?'),
            client::percent_encode_query(title)
        ),
    )
    .with_selector("a[href*=\"/subject/\"]")
    .with_rendered_page_defaults(&config.rendered_pages)
    .with_render_timeout_ms(config.rendered_pages.timeout_ms)
    .with_min_text_bytes(100)
    .with_min_html_bytes(500)
}

#[derive(Clone, Debug)]
pub struct DoubanMetadataProvider<T = ReqwestProviderHttpTransport>
where
    T: ProviderHttpTransport,
{
    config: DoubanProviderConfig,
    rendered_pages: RenderedPageRuntime<T>,
}

#[async_trait]
impl<T> MetadataProvider for DoubanMetadataProvider<T>
where
    T: ProviderHttpTransport,
{
    fn id(&self) -> ProviderId {
        ProviderId::Douban
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
        config::DoubanProviderConfig,
        providers::http_runtime::{ProviderHttpRuntime, ProviderHttpRuntimeConfig},
    };

    use super::*;

    #[tokio::test]
    async fn douban_provider_uses_browser_worker_render_contract_for_search_and_detail() {
        let transport = FakeTransport::default();
        transport.push_rendered_html(
            "https://movie.douban.com/subject_search?search_text=%E5%8D%83%E4%B8%8E%E5%8D%83%E5%AF%BB",
            "Douban Search",
            r#"
<!doctype html>
<html>
<body>
  <div class="result">
    <a class="title" href="https://movie.douban.com/subject/1291561/">千与千寻</a>
    <span class="year">2001</span>
  </div>
</body>
</html>"#,
        );
        transport.push_rendered_html(
            "https://movie.douban.com/subject/1291561/",
            "千与千寻 (豆瓣)",
            r#"
<!doctype html>
<html>
<head>
  <meta property="og:image" content="https://img1.doubanio.com/view/photo/s_ratio_poster/public/p123.jpg">
</head>
<body>
  <h1>
    <span property="v:itemreviewed">千与千寻</span>
    <span class="year">(2001)</span>
  </h1>
  <div id="info">
    <span class="pl">又名:</span> 神隐少女 / Spirited Away
    <span class="pl">片长:</span> 125分钟
    <span class="pl">类型:</span> 剧情 / 动画 / 奇幻
  </div>
  <span property="v:initialReleaseDate" content="2001-07-20">2001-07-20</span>
  <strong class="ll rating_num" property="v:average">9.4</strong>
  <span property="v:votes">2345678</span>
  <span class="short">少女误入神灵世界。</span>
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
        let provider = DoubanMetadataProvider::with_runtime(
            DoubanProviderConfig::new(
                "https://movie.douban.com/subject_search".to_owned(),
                "http://browser-worker.example".to_owned(),
                "/render".to_owned(),
                10_000,
            ),
            runtime,
        );

        let candidates = provider
            .suggest(&MetadataQuery {
                title: "千与千寻".to_owned(),
                year: Some(2001),
                language: "zh-CN".to_owned(),
                external_ids: Vec::new(),
            })
            .await
            .unwrap();

        assert_eq!(candidates.len(), 1);
        let candidate = &candidates[0];
        assert_eq!(candidate.provider, "douban");
        assert_eq!(candidate.provider_id, "douban:subject:1291561");
        assert_eq!(candidate.patch.title.as_deref(), Some("千与千寻"));
        assert_eq!(
            candidate.patch.original_title.as_deref(),
            Some("神隐少女 / Spirited Away")
        );
        assert_eq!(candidate.patch.release_date.as_deref(), Some("2001-07-20"));
        assert_eq!(candidate.patch.runtime_minutes, Some(125));
        assert_eq!(
            candidate.patch.overview.as_deref(),
            Some("少女误入神灵世界。")
        );
        assert_eq!(
            candidate.patch.genres.as_ref().unwrap(),
            &vec!["剧情".to_owned(), "动画".to_owned(), "奇幻".to_owned()]
        );
        assert_eq!(candidate.facts.title.as_deref(), Some("千与千寻"));
        assert_eq!(candidate.facts.release_year, Some(2001));
        assert_eq!(candidate.facts.language.as_deref(), Some("zh-CN"));
        assert_eq!(candidate.facts.community_score_milli, Some(940));
        assert_eq!(candidate.facts.community_vote_count, Some(2_345_678));
        assert!(
            candidate
                .facts
                .external_ids
                .iter()
                .any(|id| id.provider == "douban" && id.value == "1291561")
        );
        assert_eq!(candidate.artwork_candidates.len(), 1);
        assert_eq!(
            candidate.artwork_candidates[0].facts.kind,
            AddonArtworkKind::Poster
        );
        assert_eq!(
            candidate.artwork_candidates[0].facts.source_url,
            "https://img1.doubanio.com/view/photo/s_ratio_poster/public/p123.jpg"
        );

        let requests = transport.requests();
        assert_eq!(requests.len(), 2);
        assert_eq!(requests[0].url, "http://browser-worker.example/render");
        assert_eq!(requests[1].url, "http://browser-worker.example/render");
        let search_body: serde_json::Value =
            serde_json::from_slice(requests[0].json_body.as_ref().unwrap()).unwrap();
        assert_eq!(
            search_body["url"],
            "https://movie.douban.com/subject_search?search_text=%E5%8D%83%E4%B8%8E%E5%8D%83%E5%AF%BB"
        );
        let detail_body: serde_json::Value =
            serde_json::from_slice(requests[1].json_body.as_ref().unwrap()).unwrap();
        assert_eq!(
            detail_body["url"],
            "https://movie.douban.com/subject/1291561/"
        );
    }
}
