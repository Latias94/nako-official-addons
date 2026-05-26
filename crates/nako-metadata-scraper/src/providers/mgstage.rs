use async_trait::async_trait;
use nako_addon_protocol::{AddonArtworkKind, AddonMetadataPatch};
use scraper::Html;

use crate::{
    Config,
    config::{ProviderConfig, ProviderId, non_empty_trimmed},
    engine::{
        AvMetadataFacts, ExternalIdValueKind, MetadataQuery, ProviderArtworkCandidate,
        ProviderArtworkCandidateFacts, ProviderCandidateFacts, ProviderExternalId,
        ProviderExternalIdCapability, ProviderMetadataCandidate, ProviderOutcome,
        av::{
            AV_NUMBER_EXTERNAL_ID_PROVIDER, AvNumberRoute, AvNumberSource, AvQueryFacts,
            facts_from_text,
        },
    },
    providers::{
        MetadataProvider, ProviderBuildStatus, ProviderConfigInput,
        http_runtime::{
            ProviderHttpResult, ProviderHttpRuntime, ProviderHttpTransport,
            ReqwestProviderHttpTransport,
        },
        registry::ProviderCatalogEntry,
        rendered_av,
        rendered_page::{RenderedHtmlPage, RenderedPageRuntime, RenderedPageSupportConfig},
    },
};

pub const MGSTAGE_PROVIDER_ID: &str = "mgstage";
const MGSTAGE_URL_EXTERNAL_ID_PROVIDER: &str = "mgstage_url";
const MGSTAGE_EXTERNAL_ID_CAPABILITIES: &[ProviderExternalIdCapability] = &[
    ProviderExternalIdCapability::new(
        MGSTAGE_PROVIDER_ID,
        ExternalIdValueKind::Opaque,
        true,
        true,
        &["mgstage_id"],
        false,
    ),
    ProviderExternalIdCapability::new(
        MGSTAGE_URL_EXTERNAL_ID_PROVIDER,
        ExternalIdValueKind::Url,
        true,
        true,
        &["mgstage_url"],
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MgstageProviderConfig {
    pub(crate) base_url: String,
    pub(crate) rendered_pages: RenderedPageSupportConfig,
    pub(crate) render_path: String,
}

impl MgstageProviderConfig {
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
        let base_url = lookup("NAKO_METADATA_SCRAPER_MGSTAGE_BASE_URL")
            .and_then(non_empty_trimmed)
            .unwrap_or_else(|| "https://www.mgstage.com".to_owned());
        let browser_worker_base_url = lookup("NAKO_METADATA_SCRAPER_BROWSER_WORKER_BASE_URL")
            .and_then(non_empty_trimmed)
            .unwrap_or_else(|| "http://nako-browser-worker:3000".to_owned());
        let render_path = lookup("NAKO_METADATA_SCRAPER_BROWSER_WORKER_RENDER_PATH")
            .and_then(non_empty_trimmed)
            .unwrap_or_else(|| "/render".to_owned());
        let timeout_ms = lookup("NAKO_METADATA_SCRAPER_MGSTAGE_TIMEOUT_MS")
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
        id: ProviderId::Mgstage,
        default_enabled: false,
        enabled_env_var: "NAKO_METADATA_SCRAPER_PROVIDER_MGSTAGE_ENABLED",
        capabilities: &[
            "metadata_suggestion",
            "av_number_direct_lookup",
            "mgstage_direct_lookup",
            "mgstage_amateur_route",
            "browser_worker_rendered_html",
        ],
        field_quality: crate::engine::ProviderFieldQualityDescriptor::new(550, 450, 550, 600),
        secret_reference: None,
        external_id_capabilities: MGSTAGE_EXTERNAL_ID_CAPABILITIES,
        load_config: load_config,
        proxy_configured: |_| false,
        network_policy_key: None,
        build: build_provider,
    }
}

fn load_config(input: ProviderConfigInput<'_>) -> ProviderConfig {
    let lookup = input.lookup;
    ProviderConfig::mgstage(
        input.enabled,
        MgstageProviderConfig::from_env_lookup(|name| lookup(name)),
    )
}

fn build_provider(config: &Config) -> ProviderBuildStatus {
    let Some(provider_config) = config
        .provider_config(ProviderId::Mgstage)
        .and_then(|provider| provider.mgstage_config().cloned())
    else {
        return ProviderBuildStatus::Unavailable;
    };
    match MgstageMetadataProvider::new(provider_config) {
        Ok(provider) => ProviderBuildStatus::Ready(Box::new(provider)),
        Err(_) => ProviderBuildStatus::Unavailable,
    }
}

#[derive(Clone, Debug)]
pub struct MgstageMetadataProvider<T = ReqwestProviderHttpTransport>
where
    T: ProviderHttpTransport,
{
    config: MgstageProviderConfig,
    rendered_pages: RenderedPageRuntime<T>,
}

impl MgstageMetadataProvider<ReqwestProviderHttpTransport> {
    pub fn new(config: MgstageProviderConfig) -> ProviderHttpResult<Self> {
        let rendered_pages = RenderedPageRuntime::new(config.rendered_pages.clone())?;
        Ok(Self {
            config,
            rendered_pages,
        })
    }
}

impl<T> MgstageMetadataProvider<T>
where
    T: ProviderHttpTransport,
{
    #[must_use]
    pub fn with_runtime(config: MgstageProviderConfig, runtime: ProviderHttpRuntime<T>) -> Self {
        let rendered_pages =
            RenderedPageRuntime::with_runtime(config.rendered_pages.clone(), runtime);
        Self {
            config,
            rendered_pages,
        }
    }

    async fn render(&self, url: String) -> anyhow::Result<RenderedHtmlPage> {
        let intent = self
            .config
            .rendered_pages
            .intent(&self.config.render_path, url);
        self.rendered_pages
            .render_html(MGSTAGE_PROVIDER_ID, "render page", intent)
            .await
    }

    async fn suggest_candidates(
        &self,
        query: &MetadataQuery,
    ) -> anyhow::Result<Vec<ProviderMetadataCandidate>> {
        rendered_av::suggest_candidates(self, query).await
    }

    fn detail_url(&self, id: &str) -> String {
        if id.starts_with("http://") || id.starts_with("https://") {
            return id.to_owned();
        }
        format!(
            "{}/product/product_detail/{}/",
            self.config.base_url.trim_end_matches('/'),
            rendered_av::percent_encode(normalize_mgstage_id(id).as_deref().unwrap_or(id.trim()))
        )
    }

    fn absolute_url(&self, value: &str) -> String {
        rendered_av::absolute_url(&self.config.base_url, value)
    }
}

#[async_trait]
impl<T> rendered_av::RenderedAvFlow for MgstageMetadataProvider<T>
where
    T: ProviderHttpTransport,
{
    fn provider_id(&self) -> &'static str {
        MGSTAGE_PROVIDER_ID
    }

    fn url_external_id_provider(&self) -> &'static str {
        MGSTAGE_URL_EXTERNAL_ID_PROVIDER
    }

    fn supports_route(&self, route: AvNumberRoute) -> bool {
        matches!(route, AvNumberRoute::Amateur | AvNumberRoute::Censored)
    }

    async fn render_html_page(&self, url: String) -> anyhow::Result<RenderedHtmlPage> {
        self.render(url).await
    }

    fn absolute_url(&self, value: &str) -> String {
        MgstageMetadataProvider::absolute_url(self, value)
    }

    fn detail_url(&self, id: &str) -> String {
        MgstageMetadataProvider::detail_url(self, id)
    }

    fn detail_candidates(
        &self,
        html: &str,
        detail_url: &str,
        av: Option<AvQueryFacts>,
        query: &MetadataQuery,
    ) -> Vec<ProviderMetadataCandidate> {
        parse_detail_page(html, detail_url, av)
            .into_iter()
            .map(|facts| facts.into_candidate(query))
            .collect()
    }
}

#[async_trait]
impl<T> MetadataProvider for MgstageMetadataProvider<T>
where
    T: ProviderHttpTransport,
{
    fn id(&self) -> ProviderId {
        ProviderId::Mgstage
    }

    fn supports_av_route(&self, route: AvNumberRoute) -> bool {
        matches!(route, AvNumberRoute::Amateur | AvNumberRoute::Censored)
    }

    async fn suggest(
        &self,
        query: &MetadataQuery,
    ) -> anyhow::Result<Vec<ProviderMetadataCandidate>> {
        self.suggest_candidates(query).await
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct MgstageDetailFacts {
    id: String,
    url: String,
    av: AvQueryFacts,
    title: String,
    overview: Option<String>,
    release_date: Option<String>,
    release_year: Option<i32>,
    runtime_minutes: Option<u32>,
    actors: Vec<String>,
    tags: Vec<String>,
    maker: Option<String>,
    label: Option<String>,
    series: Option<String>,
    director: Option<String>,
    rating_milli: Option<u16>,
    poster_url: Option<String>,
    backdrop_urls: Vec<String>,
    trailer_url: Option<String>,
}

impl MgstageDetailFacts {
    fn into_candidate(self, query: &MetadataQuery) -> ProviderMetadataCandidate {
        let mut tags = vec![
            MGSTAGE_PROVIDER_ID.to_owned(),
            format!("av_number:{}", self.av.number),
            format!("av_route:{:?}", self.av.route).to_ascii_lowercase(),
        ];
        tags.extend(self.actors.iter().map(|actor| format!("actor:{actor}")));
        if let Some(maker) = &self.maker {
            tags.push(format!("maker:{maker}"));
        }
        if let Some(label) = &self.label {
            tags.push(format!("label:{label}"));
        }
        if let Some(series) = &self.series {
            tags.push(format!("series:{series}"));
        }
        if let Some(director) = &self.director {
            tags.push(format!("director:{director}"));
        }

        let mut artwork_candidates = Vec::new();
        if let Some(poster_url) = self.poster_url.clone() {
            artwork_candidates.push(mgstage_artwork_candidate(
                &self.id,
                AddonArtworkKind::Poster,
                poster_url,
                0,
            ));
        }
        for (index, url) in self.backdrop_urls.iter().cloned().enumerate() {
            artwork_candidates.push(mgstage_artwork_candidate(
                &self.id,
                AddonArtworkKind::Backdrop,
                url,
                index + 1,
            ));
        }

        ProviderMetadataCandidate {
            provider: MGSTAGE_PROVIDER_ID.to_owned(),
            provider_id: format!("mgstage:product:{}", self.id),
            patch: AddonMetadataPatch {
                title: Some(self.title.clone()),
                original_title: None,
                sort_title: Some(self.title.clone()),
                overview: self.overview.clone(),
                release_date: self.release_date.clone(),
                runtime_minutes: self.runtime_minutes,
                tagline: Some("MGStage AV title".to_owned()),
                genres: Some(self.tags.clone()).filter(|genres| !genres.is_empty()),
                tags: Some(tags).filter(|tags| !tags.is_empty()),
                ..AddonMetadataPatch::default()
            },
            facts: ProviderCandidateFacts {
                title: Some(self.title),
                alternate_titles: vec![self.av.number.clone()],
                release_year: self.release_year,
                language: Some(query.language.clone()),
                av: AvMetadataFacts {
                    actors: self.actors.clone(),
                    all_actors: self.actors.clone(),
                    directors: self.director.clone().into_iter().collect(),
                    series: self.series.clone(),
                    studio: self.maker.clone(),
                    publisher: self.label.clone(),
                    maker: self.maker.clone(),
                    label: self.label.clone(),
                    wanted_count: None,
                    thumb_url: self.poster_url.clone(),
                    trailer_url: self.trailer_url.clone(),
                    extrafanart_urls: self.backdrop_urls.clone(),
                }
                .non_empty(),
                community_score_milli: self.rating_milli,
                community_vote_count: None,
                external_ids: vec![
                    ProviderExternalId {
                        provider: MGSTAGE_PROVIDER_ID.to_owned(),
                        value: self.id,
                    },
                    ProviderExternalId {
                        provider: MGSTAGE_URL_EXTERNAL_ID_PROVIDER.to_owned(),
                        value: self.url,
                    },
                    ProviderExternalId {
                        provider: AV_NUMBER_EXTERNAL_ID_PROVIDER.to_owned(),
                        value: self.av.number,
                    },
                ],
                provider_outcomes: vec![ProviderOutcome::MgstageRenderedHtmlParsed],
                provider_note: None,
            },
            artwork_candidates,
        }
    }
}

fn parse_detail_page(
    html: &str,
    detail_url: &str,
    av: Option<AvQueryFacts>,
) -> Option<MgstageDetailFacts> {
    let document = Html::parse_document(html);
    let body_text = rendered_av::element_text(&document, "body").unwrap_or_default();
    let info_text = rendered_av::element_text(&document, ".detail, .product_detail, table, body")
        .unwrap_or_else(|| body_text.clone());
    let title = rendered_av::first_non_empty(&[
        rendered_av::element_text(&document, "h1, .product_title, .detail_title").as_deref(),
        rendered_av::attr_value(&document, "meta[property=\"og:title\"]", "content").as_deref(),
    ])?;
    let number = mgstage_labeled_value(
        &document,
        &info_text,
        &["品番", "商品番号", "Product ID", "品名"],
    )
    .or_else(|| facts_from_text(&title, AvNumberSource::ExternalId).map(|facts| facts.number))
    .or_else(|| facts_from_text(detail_url, AvNumberSource::ExternalId).map(|facts| facts.number));
    let av = number
        .as_deref()
        .and_then(|value| facts_from_text(value, AvNumberSource::ExternalId))
        .or_else(|| facts_from_text(&title, AvNumberSource::ExternalId))
        .or(av)?;
    let release_date = mgstage_labeled_value(
        &document,
        &info_text,
        &["発売日", "配信開始日", "Release Date"],
    )
    .or_else(|| rendered_av::first_iso_date(&body_text));
    let release_year = release_date.as_deref().and_then(rendered_av::first_year);
    let runtime_minutes = mgstage_labeled_value(
        &document,
        &info_text,
        &["収録時間", "再生時間", "Duration", "Runtime"],
    )
    .and_then(|value| rendered_av::parse_minutes(&value));
    let overview = rendered_av::first_non_empty(&[
        rendered_av::element_text(&document, ".introduction, .story, .comment, #introduction")
            .as_deref(),
        rendered_av::attr_value(&document, "meta[name=\"description\"]", "content").as_deref(),
    ]);
    let maker = mgstage_labeled_value(&document, &info_text, &["メーカー", "Maker", "Studio"]);
    let label = mgstage_labeled_value(&document, &info_text, &["レーベル", "Label", "Publisher"]);
    let series = mgstage_labeled_value(&document, &info_text, &["シリーズ", "Series"]);
    let director = mgstage_labeled_value(&document, &info_text, &["監督", "Director"]);
    let actors = rendered_av::link_texts(
        &document,
        "a[href*=\"actor\"], a[href*=\"actress\"], a[href*=\"performer\"]",
    );
    let tags = rendered_av::link_texts(
        &document,
        "a[href*=\"genre\"], a[href*=\"category\"], a[href*=\"tag\"], .tag a",
    );
    let rating_milli =
        rendered_av::element_text(&document, ".review, .score, .rating, [class*=\"review\"]")
            .or_else(|| mgstage_labeled_value(&document, &info_text, &["評価", "Rating"]))
            .and_then(|value| rendered_av::parse_rating_milli(&value));
    let poster_url = rendered_av::attr_value(&document, "meta[property=\"og:image\"]", "content")
        .or_else(|| {
            rendered_av::attr_value(
                &document,
                ".package img, #package img, .main_image img, .jacket img",
                "src",
            )
        })
        .map(rendered_av::normalize_url);
    let backdrop_urls = rendered_av::image_urls(
        &document,
        ".sample_image img, .sample-photo img, .sample_images img, a[href*=\"sample\"]",
        detail_url,
    );
    let trailer_url =
        rendered_av::attr_value(&document, "video source, source[type*=\"video\"]", "src")
            .or_else(|| {
                rendered_av::attr_value(
                    &document,
                    "a[href*=\"sampleplayer\"], a[href*=\"sample_movie\"]",
                    "href",
                )
            })
            .map(|url| rendered_av::absolute_url(detail_url, &url));

    Some(MgstageDetailFacts {
        id: product_id_from_url(detail_url).unwrap_or_else(|| av.number.clone()),
        url: detail_url.to_owned(),
        av,
        title,
        overview,
        release_date,
        release_year,
        runtime_minutes,
        actors,
        tags,
        maker,
        label,
        series,
        director,
        rating_milli,
        poster_url,
        backdrop_urls,
        trailer_url,
    })
}

const MGSTAGE_LABELS: &[&str] = &[
    "品番",
    "商品番号",
    "Product ID",
    "品名",
    "発売日",
    "配信開始日",
    "Release Date",
    "収録時間",
    "再生時間",
    "Duration",
    "Runtime",
    "メーカー",
    "Maker",
    "Studio",
    "レーベル",
    "Label",
    "Publisher",
    "シリーズ",
    "Series",
    "監督",
    "Director",
    "評価",
    "Rating",
];

const MGSTAGE_LABEL_ROW_SELECTOR: &str = ".detail p, .detail li, .detail tr, \
         .product_detail p, .product_detail li, .product_detail tr, \
         table tr";

fn mgstage_labeled_value(document: &Html, info_text: &str, labels: &[&str]) -> Option<String> {
    rendered_av::structured_or_labeled_value(
        document,
        MGSTAGE_LABEL_ROW_SELECTOR,
        info_text,
        labels,
        MGSTAGE_LABELS,
    )
}

fn mgstage_artwork_candidate(
    product_id: &str,
    kind: AddonArtworkKind,
    source_url: String,
    index: usize,
) -> ProviderArtworkCandidate {
    ProviderArtworkCandidate {
        provider: MGSTAGE_PROVIDER_ID.to_owned(),
        provider_id: format!("mgstage:product:{product_id}:artwork:{index}"),
        facts: ProviderArtworkCandidateFacts {
            kind,
            source_url,
            language: None,
            width: None,
            height: None,
        },
    }
}

fn normalize_mgstage_id(value: &str) -> Option<String> {
    if let Some(id) = product_id_from_url(value) {
        return Some(id);
    }
    let value = value.trim().trim_matches(['/', '?', '#', '&']);
    (!value.is_empty()).then(|| value.to_owned())
}

fn product_id_from_url(url: &str) -> Option<String> {
    let marker = "/product/product_detail/";
    let start = url.find(marker)? + marker.len();
    let rest = &url[start..];
    let end = rest.find(['/', '?', '#', '&']).unwrap_or(rest.len());
    let id = &rest[..end];
    (!id.is_empty()).then(|| id.to_owned())
}

#[cfg(test)]
mod tests {
    use crate::{
        engine::{MetadataQuery, QueryExternalId},
        providers::{
            http_runtime::ProviderHttpRuntimeConfig,
            rendered_av_fixture::{RenderedAvFixtureTransport, request_json_body},
        },
    };

    use super::*;

    #[tokio::test]
    async fn mgstage_provider_uses_browser_worker_render_contract_for_amateur_av_detail() {
        let transport = RenderedAvFixtureTransport::new(MGSTAGE_PROVIDER_ID);
        transport.push_rendered_html(
            "https://mgstage.example/product/product_detail/300MIUM-382/",
            "300MIUM-382 MGStage Title",
            r#"
<!doctype html>
<html>
<head>
  <meta property="og:image" content="//image.example/mgstage-cover.jpg">
  <meta name="description" content="MGStage synthetic outline.">
</head>
<body>
  <h1>300MIUM-382 MGStage Title</h1>
  <div class="product_detail">
    <p>品番: 300MIUM-382</p>
    <p>配信開始日: 2024-06-08</p>
    <p>収録時間: 93分</p>
    <p>メーカー: MGS Maker</p>
    <p>レーベル: MGS Label</p>
    <p>シリーズ: MGS Series</p>
  </div>
  <a href="/search/cSearch.php?actor=1">Actor One</a>
  <a href="/genre/drama">ドラマ</a>
  <a href="/genre/amateur">素人</a>
  <span class="review">4.1</span>
  <div class="sample_image">
    <img src="//image.example/sample1.jpg">
    <img src="https://image.example/sample2.jpg">
  </div>
  <video><source src="//movie.example/sample.mp4" type="video/mp4"></video>
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
        let provider = MgstageMetadataProvider::with_runtime(
            MgstageProviderConfig::new(
                "https://mgstage.example".to_owned(),
                "http://browser-worker.example".to_owned(),
                "/render".to_owned(),
                10_000,
            ),
            runtime,
        );

        let candidates = provider
            .suggest(&MetadataQuery::from_payload(
                &serde_json::json!({"file_name": "300MIUM-382.mp4"}),
                "zh-CN",
            ))
            .await
            .unwrap();

        assert_eq!(candidates.len(), 1);
        let candidate = &candidates[0];
        assert_eq!(candidate.provider, "mgstage");
        assert_eq!(candidate.provider_id, "mgstage:product:300MIUM-382");
        assert_eq!(
            candidate.patch.title.as_deref(),
            Some("300MIUM-382 MGStage Title")
        );
        assert_eq!(candidate.patch.release_date.as_deref(), Some("2024-06-08"));
        assert_eq!(candidate.patch.runtime_minutes, Some(93));
        assert_eq!(
            candidate.facts.av.as_ref().unwrap().actors,
            vec!["Actor One".to_owned()]
        );
        assert_eq!(
            candidate.facts.av.as_ref().unwrap().trailer_url.as_deref(),
            Some("https://movie.example/sample.mp4")
        );
        assert_eq!(
            candidate.facts.av.as_ref().unwrap().studio.as_deref(),
            Some("MGS Maker")
        );
        assert_eq!(
            candidate.facts.av.as_ref().unwrap().publisher.as_deref(),
            Some("MGS Label")
        );
        assert_eq!(
            candidate.facts.av.as_ref().unwrap().series.as_deref(),
            Some("MGS Series")
        );
        assert_eq!(candidate.facts.community_score_milli, Some(820));
        assert!(candidate.facts.external_ids.iter().any(|id| {
            id.provider == AV_NUMBER_EXTERNAL_ID_PROVIDER && id.value == "300MIUM-382"
        }));
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
        assert_eq!(requests.len(), 1);
        let body = request_json_body(&requests[0]);
        assert_eq!(
            body["url"],
            "https://mgstage.example/product/product_detail/300MIUM-382/"
        );
    }

    #[tokio::test]
    async fn mgstage_provider_uses_explicit_id_for_direct_detail_lookup() {
        let transport = RenderedAvFixtureTransport::new(MGSTAGE_PROVIDER_ID);
        transport.push_rendered_html(
            "https://mgstage.example/product/product_detail/300MIUM-382/",
            "300MIUM-382 Direct MGStage Title",
            r#"
<!doctype html>
<html>
<body>
  <h1>300MIUM-382 Direct MGStage Title</h1>
  <div class="product_detail">
    <p>品番: 300MIUM-382</p>
    <p>配信開始日: 2024-06-08</p>
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
        let provider = MgstageMetadataProvider::with_runtime(
            MgstageProviderConfig::new(
                "https://mgstage.example".to_owned(),
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
                    provider: "mgstage".to_owned(),
                    value: "300MIUM-382".to_owned(),
                }],
            })
            .await
            .unwrap();

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].provider_id, "mgstage:product:300MIUM-382");
        assert_eq!(
            candidates[0].patch.title.as_deref(),
            Some("300MIUM-382 Direct MGStage Title")
        );
    }

    #[tokio::test]
    async fn mgstage_provider_skips_fc2_numbers() {
        let transport = RenderedAvFixtureTransport::new(MGSTAGE_PROVIDER_ID);
        let runtime = ProviderHttpRuntime::with_transport(
            ProviderHttpRuntimeConfig {
                retry_backoff_ms: 0,
                ..ProviderHttpRuntimeConfig::default()
            },
            transport.clone(),
        );
        let provider = MgstageMetadataProvider::with_runtime(
            MgstageProviderConfig::new(
                "https://mgstage.example".to_owned(),
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
}
