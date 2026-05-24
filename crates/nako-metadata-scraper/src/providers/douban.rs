mod client;
mod enrichment;
mod mapper;
mod parser;
#[cfg(test)]
mod test_support;

use async_trait::async_trait;

use crate::{
    Config,
    config::{DoubanProviderConfig, ProviderId},
    engine::{MetadataQuery, ProviderMetadataCandidate},
    providers::{
        MetadataProvider, ProviderBuildStatus,
        http_runtime::{ProviderHttpRuntime, ProviderHttpTransport, ReqwestProviderHttpTransport},
        registry::ProviderCatalogEntry,
    },
};

#[cfg(test)]
use nako_addon_protocol::AddonArtworkKind;
#[cfg(test)]
use test_support::FakeTransport;

pub const DOUBAN_PROVIDER_ID: &str = "douban";
const DOUBAN_DETAIL_ENRICHMENT_LIMIT: usize = 1;

#[must_use]
pub(crate) fn catalog_entry() -> ProviderCatalogEntry {
    ProviderCatalogEntry {
        id: ProviderId::Douban,
        capabilities: &[
            "metadata_suggestion",
            "movie_search",
            "browser_worker_rendered_html",
        ],
        secret_reference: None,
        build: build_provider,
    }
}

fn build_provider(config: &Config) -> ProviderBuildStatus {
    let Some(douban_config) = config
        .provider_config(ProviderId::Douban)
        .and_then(|provider| provider.douban.clone())
    else {
        return ProviderBuildStatus::Unavailable;
    };
    match DoubanMetadataProvider::new(douban_config) {
        Ok(provider) => ProviderBuildStatus::Ready(Box::new(provider)),
        Err(_) => ProviderBuildStatus::Unavailable,
    }
}

#[derive(Clone, Debug)]
pub struct DoubanMetadataProvider<T = ReqwestProviderHttpTransport>
where
    T: ProviderHttpTransport,
{
    config: DoubanProviderConfig,
    runtime: ProviderHttpRuntime<T>,
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
            DoubanProviderConfig {
                search_base_url: "https://movie.douban.com/subject_search".to_owned(),
                browser_worker_base_url: "http://browser-worker.example".to_owned(),
                render_path: "/render".to_owned(),
                timeout_ms: 10_000,
            },
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
