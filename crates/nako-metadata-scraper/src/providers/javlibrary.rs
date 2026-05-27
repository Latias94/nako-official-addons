use async_trait::async_trait;
use nako_addon_protocol::{AddonArtworkKind, AddonMetadataPatch};
use scraper::{Html, Selector};

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
        registry::{
            ProviderCatalogEntry, ProviderDefaultFieldPreference, ProviderRenderedPageSupport,
        },
        render_drift::{
            BrowserWorkerRenderDriftCase, DEFAULT_SAMPLE_AV_NUMBER,
            ProviderRenderDriftCaseDescriptor, RENDER_DRIFT_SAMPLE_JAVLIBRARY_AV_NUMBER_ENV_VAR,
        },
        rendered_av,
        rendered_page::{RenderedHtmlPage, RenderedPageRuntime, RenderedPageSupportConfig},
    },
};

pub const JAVLIBRARY_PROVIDER_ID: &str = "javlibrary";
const JAVLIBRARY_URL_EXTERNAL_ID_PROVIDER: &str = "javlibrary_url";
const JAVLIBRARY_EXTERNAL_ID_CAPABILITIES: &[ProviderExternalIdCapability] = &[
    ProviderExternalIdCapability::new(
        JAVLIBRARY_PROVIDER_ID,
        ExternalIdValueKind::Opaque,
        true,
        true,
        &["javlibrary_id"],
        false,
    ),
    ProviderExternalIdCapability::new(
        JAVLIBRARY_URL_EXTERNAL_ID_PROVIDER,
        ExternalIdValueKind::Url,
        true,
        true,
        &["javlibrary_url"],
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
    ProviderDefaultFieldPreference::title(60),
    ProviderDefaultFieldPreference::actors(30),
    ProviderDefaultFieldPreference::wanted(10),
    ProviderDefaultFieldPreference::score(20),
    ProviderDefaultFieldPreference::score_votes(10),
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JavlibraryProviderConfig {
    pub(crate) base_url: String,
    pub(crate) language_path: String,
    pub(crate) rendered_pages: RenderedPageSupportConfig,
    pub(crate) render_path: String,
}

impl JavlibraryProviderConfig {
    pub const DEFAULT_TIMEOUT_MS: u64 = 10_000;

    #[must_use]
    pub(crate) fn new(
        base_url: String,
        language_path: String,
        browser_worker_base_url: String,
        render_path: String,
        timeout_ms: u64,
    ) -> Self {
        Self {
            base_url,
            language_path,
            rendered_pages: RenderedPageSupportConfig::new(browser_worker_base_url, timeout_ms),
            render_path,
        }
    }

    #[must_use]
    pub fn from_env_lookup(mut lookup: impl FnMut(&str) -> Option<String>) -> Self {
        let base_url = lookup("NAKO_METADATA_SCRAPER_JAVLIBRARY_BASE_URL")
            .and_then(non_empty_trimmed)
            .unwrap_or_else(|| "https://www.javlibrary.com".to_owned());
        let language_path = lookup("NAKO_METADATA_SCRAPER_JAVLIBRARY_LANGUAGE")
            .and_then(non_empty_trimmed)
            .unwrap_or_else(|| "cn".to_owned());
        let browser_worker_base_url = lookup("NAKO_METADATA_SCRAPER_BROWSER_WORKER_BASE_URL")
            .and_then(non_empty_trimmed)
            .unwrap_or_else(|| "http://nako-browser-worker:3000".to_owned());
        let render_path = lookup("NAKO_METADATA_SCRAPER_BROWSER_WORKER_RENDER_PATH")
            .and_then(non_empty_trimmed)
            .unwrap_or_else(|| "/render".to_owned());
        let timeout_ms = lookup("NAKO_METADATA_SCRAPER_JAVLIBRARY_TIMEOUT_MS")
            .or_else(|| lookup("NAKO_METADATA_SCRAPER_BROWSER_WORKER_TIMEOUT_MS"))
            .and_then(|value| value.trim().parse::<u64>().ok())
            .filter(|value| *value > 0)
            .unwrap_or(Self::DEFAULT_TIMEOUT_MS);

        let mut config = Self::new(
            base_url,
            language_path,
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
        id: ProviderId::Javlibrary,
        default_enabled: false,
        enabled_env_var: "NAKO_METADATA_SCRAPER_PROVIDER_JAVLIBRARY_ENABLED",
        capabilities: &[
            "metadata_suggestion",
            "av_number_search",
            "javlibrary_direct_lookup",
            "javlibrary_movie_search",
            "browser_worker_rendered_html",
        ],
        field_quality: crate::engine::ProviderFieldQualityDescriptor::new(350, 600, 0, 0),
        default_field_preferences: DEFAULT_FIELD_PREFERENCES,
        secret_reference: None,
        external_id_capabilities: JAVLIBRARY_EXTERNAL_ID_CAPABILITIES,
        load_config: load_config,
        proxy_configured: |_| false,
        network_policy_key: None,
        rendered_page_support: Some(ProviderRenderedPageSupport::new(rendered_page_config)),
        render_drift_case: Some(
            ProviderRenderDriftCaseDescriptor::new(
                40,
                RENDER_DRIFT_SAMPLE_JAVLIBRARY_AV_NUMBER_ENV_VAR,
                DEFAULT_SAMPLE_AV_NUMBER,
                render_drift_case_from_config,
            )
            .with_generic_av_sample(),
        ),
        build: build_provider,
    }
}

fn rendered_page_config(provider: &ProviderConfig) -> Option<&RenderedPageSupportConfig> {
    provider
        .javlibrary_config()
        .map(|config| &config.rendered_pages)
}

fn render_drift_case_from_config(
    provider: &ProviderConfig,
    sample: &str,
) -> Option<BrowserWorkerRenderDriftCase> {
    provider
        .javlibrary_config()
        .map(|config| render_drift_case(config, sample))
}

fn load_config(input: ProviderConfigInput<'_>) -> ProviderConfig {
    let lookup = input.lookup;
    ProviderConfig::javlibrary(
        input.enabled,
        JavlibraryProviderConfig::from_env_lookup(|name| lookup(name)),
    )
}

fn build_provider(config: &Config) -> ProviderBuildStatus {
    let Some(provider_config) = config
        .provider_config(ProviderId::Javlibrary)
        .and_then(|provider| provider.javlibrary_config().cloned())
    else {
        return ProviderBuildStatus::Unavailable;
    };
    match JavlibraryMetadataProvider::new(provider_config) {
        Ok(provider) => ProviderBuildStatus::Ready(Box::new(provider)),
        Err(_) => ProviderBuildStatus::Unavailable,
    }
}

#[must_use]
pub(crate) fn render_drift_case(
    config: &JavlibraryProviderConfig,
    av_number: &str,
) -> BrowserWorkerRenderDriftCase {
    BrowserWorkerRenderDriftCase::new(
        "javlibrary-search",
        format!(
            "{}/{}/vl_searchbyid.php?keyword={}",
            config.base_url.trim_end_matches('/'),
            config
                .language_path
                .trim()
                .trim_matches('/')
                .trim_start_matches('.'),
            rendered_av::percent_encode(av_number)
        ),
    )
    .with_selector("a[href*=\"?v=\"], .video a[href], .videothumblist a[href]")
    .with_rendered_page_defaults(&config.rendered_pages)
    .with_render_timeout_ms(config.rendered_pages.timeout_ms)
    .with_min_text_bytes(100)
    .with_min_html_bytes(500)
}

#[derive(Clone, Debug)]
pub struct JavlibraryMetadataProvider<T = ReqwestProviderHttpTransport>
where
    T: ProviderHttpTransport,
{
    config: JavlibraryProviderConfig,
    rendered_pages: RenderedPageRuntime<T>,
}

impl JavlibraryMetadataProvider<ReqwestProviderHttpTransport> {
    pub fn new(config: JavlibraryProviderConfig) -> ProviderHttpResult<Self> {
        let rendered_pages = RenderedPageRuntime::new(config.rendered_pages.clone())?;
        Ok(Self {
            config,
            rendered_pages,
        })
    }
}

impl<T> JavlibraryMetadataProvider<T>
where
    T: ProviderHttpTransport,
{
    #[must_use]
    pub fn with_runtime(config: JavlibraryProviderConfig, runtime: ProviderHttpRuntime<T>) -> Self {
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
            .render_html(JAVLIBRARY_PROVIDER_ID, "render page", intent)
            .await
    }

    async fn suggest_candidates(
        &self,
        query: &MetadataQuery,
    ) -> anyhow::Result<Vec<ProviderMetadataCandidate>> {
        rendered_av::suggest_candidates(self, query).await
    }

    fn search_url(&self, av_number: &str) -> String {
        format!(
            "{}/vl_searchbyid.php?keyword={}",
            self.localized_base_url(),
            rendered_av::percent_encode(av_number)
        )
    }

    fn detail_url(&self, id: &str) -> String {
        if id.starts_with("http://") || id.starts_with("https://") {
            return id.to_owned();
        }
        let id = javlibrary_id_from_url(id).unwrap_or_else(|| id.trim().to_owned());
        format!(
            "{}/?v={}",
            self.localized_base_url(),
            rendered_av::percent_encode(&id)
        )
    }

    fn absolute_url(&self, value: &str) -> String {
        rendered_av::absolute_url(&self.localized_base_url(), value)
    }

    fn localized_base_url(&self) -> String {
        format!(
            "{}/{}",
            self.config.base_url.trim_end_matches('/'),
            self.config
                .language_path
                .trim()
                .trim_matches('/')
                .trim_start_matches('.')
        )
    }
}

#[async_trait]
impl<T> rendered_av::RenderedAvFlow for JavlibraryMetadataProvider<T>
where
    T: ProviderHttpTransport,
{
    fn provider_id(&self) -> &'static str {
        JAVLIBRARY_PROVIDER_ID
    }

    fn url_external_id_provider(&self) -> &'static str {
        JAVLIBRARY_URL_EXTERNAL_ID_PROVIDER
    }

    fn supports_route(&self, route: AvNumberRoute) -> bool {
        matches!(
            route,
            AvNumberRoute::Censored | AvNumberRoute::Uncensored | AvNumberRoute::Amateur
        )
    }

    async fn render_html_page(&self, url: String) -> anyhow::Result<RenderedHtmlPage> {
        self.render(url).await
    }

    fn absolute_url(&self, value: &str) -> String {
        JavlibraryMetadataProvider::absolute_url(self, value)
    }

    fn detail_url(&self, id: &str) -> String {
        JavlibraryMetadataProvider::detail_url(self, id)
    }

    fn search_url(&self, av: &AvQueryFacts) -> Option<String> {
        Some(JavlibraryMetadataProvider::search_url(self, &av.number))
    }

    fn search_results(
        &self,
        html: &str,
        av: &AvQueryFacts,
    ) -> Vec<rendered_av::RenderedAvSearchResult> {
        parse_search_results(html, av, &self.localized_base_url())
            .into_iter()
            .map(|result| rendered_av::RenderedAvSearchResult::new(result.url))
            .collect()
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
impl<T> MetadataProvider for JavlibraryMetadataProvider<T>
where
    T: ProviderHttpTransport,
{
    fn id(&self) -> ProviderId {
        ProviderId::Javlibrary
    }

    fn supports_av_route(&self, route: AvNumberRoute) -> bool {
        matches!(
            route,
            AvNumberRoute::Censored | AvNumberRoute::Uncensored | AvNumberRoute::Amateur
        )
    }

    async fn suggest(
        &self,
        query: &MetadataQuery,
    ) -> anyhow::Result<Vec<ProviderMetadataCandidate>> {
        self.suggest_candidates(query).await
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct JavlibrarySearchResult {
    url: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct JavlibraryDetailFacts {
    id: String,
    url: String,
    av: AvQueryFacts,
    title: String,
    release_date: Option<String>,
    release_year: Option<i32>,
    runtime_minutes: Option<u32>,
    actors: Vec<String>,
    tags: Vec<String>,
    studio: Option<String>,
    publisher: Option<String>,
    series: Option<String>,
    director: Option<String>,
    rating_milli: Option<u16>,
    wanted_count: Option<u32>,
    poster_url: Option<String>,
}

impl JavlibraryDetailFacts {
    fn into_candidate(self, query: &MetadataQuery) -> ProviderMetadataCandidate {
        let mut tags = vec![
            JAVLIBRARY_PROVIDER_ID.to_owned(),
            format!("av_number:{}", self.av.number),
            format!("av_route:{:?}", self.av.route).to_ascii_lowercase(),
        ];
        tags.extend(self.actors.iter().map(|actor| format!("actor:{actor}")));
        if let Some(studio) = &self.studio {
            tags.push(format!("studio:{studio}"));
        }
        if let Some(publisher) = &self.publisher {
            tags.push(format!("publisher:{publisher}"));
        }
        if let Some(series) = &self.series {
            tags.push(format!("series:{series}"));
        }
        if let Some(director) = &self.director {
            tags.push(format!("director:{director}"));
        }
        if let Some(wanted) = self.wanted_count {
            tags.push(format!("wanted:{wanted}"));
        }

        let artwork_candidates = self
            .poster_url
            .clone()
            .into_iter()
            .map(|poster_url| javlibrary_artwork_candidate(&self.id, poster_url))
            .collect();

        ProviderMetadataCandidate {
            provider: JAVLIBRARY_PROVIDER_ID.to_owned(),
            provider_id: format!("javlibrary:movie:{}", self.id),
            patch: AddonMetadataPatch {
                title: Some(self.title.clone()),
                original_title: None,
                sort_title: Some(self.title.clone()),
                overview: None,
                release_date: self.release_date.clone(),
                runtime_minutes: self.runtime_minutes,
                tagline: Some("JavLibrary AV community record".to_owned()),
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
                    studio: self.studio.clone(),
                    publisher: self.publisher.clone(),
                    maker: self.studio.clone(),
                    label: self.publisher.clone(),
                    wanted_count: self.wanted_count,
                    thumb_url: self.poster_url.clone(),
                    trailer_url: None,
                    extrafanart_urls: Vec::new(),
                }
                .non_empty(),
                community_score_milli: self.rating_milli,
                community_vote_count: self.wanted_count,
                external_ids: vec![
                    ProviderExternalId {
                        provider: JAVLIBRARY_PROVIDER_ID.to_owned(),
                        value: self.id,
                    },
                    ProviderExternalId {
                        provider: JAVLIBRARY_URL_EXTERNAL_ID_PROVIDER.to_owned(),
                        value: self.url,
                    },
                    ProviderExternalId {
                        provider: AV_NUMBER_EXTERNAL_ID_PROVIDER.to_owned(),
                        value: self.av.number,
                    },
                ],
                provider_outcomes: vec![ProviderOutcome::JavlibraryRenderedHtmlParsed],
                provider_note: None,
            },
            artwork_candidates,
        }
    }
}

fn parse_search_results(
    html: &str,
    av: &AvQueryFacts,
    localized_base_url: &str,
) -> Vec<JavlibrarySearchResult> {
    let document = Html::parse_document(html);
    let Ok(selector) = Selector::parse("a[href*=\"?v=\"], .video a[href], .videothumblist a[href]")
    else {
        return Vec::new();
    };

    document
        .select(&selector)
        .filter_map(|link| {
            let href = link.value().attr("href")?;
            let text =
                rendered_av::normalize_whitespace(&link.text().collect::<Vec<_>>().join(" "));
            if !rendered_av::text_or_url_matches_av(&text, href, av) {
                return None;
            }
            Some(JavlibrarySearchResult {
                url: rendered_av::absolute_url(localized_base_url, href),
            })
        })
        .fold(Vec::new(), |mut results, result| {
            if !results.iter().any(|existing| existing.url == result.url) {
                results.push(result);
            }
            results
        })
}

fn parse_detail_page(
    html: &str,
    detail_url: &str,
    av: Option<AvQueryFacts>,
) -> Option<JavlibraryDetailFacts> {
    let document = Html::parse_document(html);
    let body_text = rendered_av::element_text(&document, "body").unwrap_or_default();
    let info_text = rendered_av::element_text(&document, "#video_info, .video_info, #video, body")
        .unwrap_or_else(|| body_text.clone());
    let title = rendered_av::first_non_empty(&[
        rendered_av::element_text(&document, "#video_title h3, h3, h1").as_deref(),
        rendered_av::attr_value(&document, "meta[property=\"og:title\"]", "content").as_deref(),
    ])?;
    let number = javlibrary_labeled_value(
        &document,
        &info_text,
        &["品番", "識別碼", "识别码", "Number"],
    )
    .or_else(|| facts_from_text(&title, AvNumberSource::ExternalId).map(|facts| facts.number))
    .or_else(|| facts_from_text(detail_url, AvNumberSource::ExternalId).map(|facts| facts.number));
    let av = number
        .as_deref()
        .and_then(|value| facts_from_text(value, AvNumberSource::ExternalId))
        .or_else(|| facts_from_text(&title, AvNumberSource::ExternalId))
        .or(av)?;
    let release_date = javlibrary_labeled_value(
        &document,
        &info_text,
        &["発売日", "發行日期", "发行日期", "Release Date"],
    )
    .or_else(|| rendered_av::first_iso_date(&body_text));
    let release_year = release_date.as_deref().and_then(rendered_av::first_year);
    let runtime_minutes = javlibrary_labeled_value(
        &document,
        &info_text,
        &["収録時間", "長度", "长度", "Runtime"],
    )
    .and_then(|value| rendered_av::parse_minutes(&value));
    let studio =
        first_link_text(&document, "a[href*=\"maker\"], a[href*=\"studio\"]").or_else(|| {
            javlibrary_labeled_value(&document, &info_text, &["メーカー", "片商", "Studio"])
        });
    let publisher = first_link_text(&document, "a[href*=\"label\"], a[href*=\"publisher\"]")
        .or_else(|| {
            javlibrary_labeled_value(
                &document,
                &info_text,
                &["レーベル", "發行商", "发行商", "Label", "Publisher"],
            )
        });
    let series = first_link_text(&document, "a[href*=\"series\"]")
        .or_else(|| javlibrary_labeled_value(&document, &info_text, &["系列", "Series"]));
    let director = first_link_text(&document, "a[href*=\"director\"]").or_else(|| {
        javlibrary_labeled_value(&document, &info_text, &["監督", "導演", "导演", "Director"])
    });
    let actors = rendered_av::link_texts(
        &document,
        "a[href*=\"star.php\"], a[href*=\"/star/\"], a[href*=\"star=\"]",
    );
    let tags = rendered_av::link_texts(
        &document,
        "a[href*=\"genre\"], a[href*=\"category\"], a[href*=\"tag\"]",
    );
    let rating_milli = rendered_av::element_text(
        &document,
        "#video_review .score, .score, .rating, [class*=\"score\"]",
    )
    .or_else(|| javlibrary_labeled_value(&document, &info_text, &["评分", "評分", "Rating"]))
    .and_then(|value| rendered_av::parse_rating_milli(&value));
    let wanted_count = rendered_av::element_text(
        &document,
        ".wanted, .userswanted, #video_favorite_edit, [class*=\"wanted\"]",
    )
    .or_else(|| javlibrary_labeled_value(&document, &info_text, &["想看", "Wanted"]))
    .and_then(|value| rendered_av::first_u32(&value));
    let poster_url =
        rendered_av::attr_value(&document, "#video_jacket_img, .video_jacket_img", "src")
            .or_else(|| {
                rendered_av::attr_value(&document, "meta[property=\"og:image\"]", "content")
            })
            .map(rendered_av::normalize_url);

    Some(JavlibraryDetailFacts {
        id: javlibrary_id_from_url(detail_url).unwrap_or_else(|| av.number.clone()),
        url: detail_url.to_owned(),
        av,
        title,
        release_date,
        release_year,
        runtime_minutes,
        actors,
        tags,
        studio,
        publisher,
        series,
        director,
        rating_milli,
        wanted_count,
        poster_url,
    })
}

const JAVLIBRARY_LABELS: &[&str] = &[
    "品番",
    "識別碼",
    "识别码",
    "Number",
    "発売日",
    "發行日期",
    "发行日期",
    "Release Date",
    "収録時間",
    "長度",
    "长度",
    "Runtime",
    "メーカー",
    "片商",
    "Studio",
    "レーベル",
    "發行商",
    "发行商",
    "Label",
    "Publisher",
    "系列",
    "Series",
    "監督",
    "導演",
    "导演",
    "Director",
    "评分",
    "評分",
    "Rating",
    "想看",
    "Wanted",
];

const JAVLIBRARY_LABEL_ROW_SELECTOR: &str = "#video_info p, #video_info li, #video_info tr, \
         .video_info p, .video_info li, .video_info tr, \
         #video p, #video li, #video tr, table tr";

fn javlibrary_labeled_value(document: &Html, info_text: &str, labels: &[&str]) -> Option<String> {
    rendered_av::structured_or_labeled_value(
        document,
        JAVLIBRARY_LABEL_ROW_SELECTOR,
        info_text,
        labels,
        JAVLIBRARY_LABELS,
    )
}

fn first_link_text(document: &Html, selector: &str) -> Option<String> {
    rendered_av::link_texts(document, selector)
        .into_iter()
        .next()
}

fn javlibrary_artwork_candidate(movie_id: &str, source_url: String) -> ProviderArtworkCandidate {
    ProviderArtworkCandidate {
        provider: JAVLIBRARY_PROVIDER_ID.to_owned(),
        provider_id: format!("javlibrary:movie:{movie_id}:artwork:0"),
        facts: ProviderArtworkCandidateFacts {
            kind: AddonArtworkKind::Poster,
            source_url,
            language: None,
            width: None,
            height: None,
        },
    }
}

fn javlibrary_id_from_url(url: &str) -> Option<String> {
    rendered_av::id_query_value(url, "v")
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
    async fn javlibrary_provider_uses_browser_worker_render_contract_for_av_search_and_detail() {
        let transport = RenderedAvFixtureTransport::new(JAVLIBRARY_PROVIDER_ID);
        transport.push_rendered_html(
            "https://javlibrary.example/cn/vl_searchbyid.php?keyword=SSNI-644",
            "JavLibrary Search",
            r#"
<!doctype html>
<html>
<body>
  <div class="video"><a href="?v=javli123"><span>SSNI-644 JavLibrary Title</span></a></div>
  <div class="video"><a href="?v=other999"><span>ABP-001 Other Title</span></a></div>
</body>
</html>"#,
        );
        transport.push_rendered_html(
            "https://javlibrary.example/cn/?v=javli123",
            "SSNI-644 JavLibrary Title",
            r#"
<!doctype html>
<html>
<head>
  <meta property="og:image" content="https://img.example/javlibrary-cover.jpg">
</head>
<body>
  <h3 id="video_title">SSNI-644 JavLibrary Title</h3>
  <div id="video_info">
    <p>品番: SSNI-644</p>
    <p>發行日期: 2024-05-03</p>
    <p>長度: 120分鐘</p>
    <p>片商: Studio Alpha</p>
    <p>發行商: Publisher Beta</p>
    <p>系列: Series Gamma</p>
    <p>導演: Director Delta</p>
    <p>想看: 234 users wanted</p>
  </div>
  <a href="star.php?star=one">Actor One</a>
  <a href="star.php?star=two">Actor Two</a>
  <a href="vl_genre.php?g=drama">剧情</a>
  <a href="vl_genre.php?g=uniform">制服</a>
  <span class="score">4.3</span>
  <img id="video_jacket_img" src="https://img.example/jacket.jpg">
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
        let provider = JavlibraryMetadataProvider::with_runtime(
            JavlibraryProviderConfig::new(
                "https://javlibrary.example".to_owned(),
                "cn".to_owned(),
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
        assert_eq!(candidate.provider, "javlibrary");
        assert_eq!(candidate.provider_id, "javlibrary:movie:javli123");
        assert_eq!(
            candidate.patch.title.as_deref(),
            Some("SSNI-644 JavLibrary Title")
        );
        assert_eq!(candidate.patch.release_date.as_deref(), Some("2024-05-03"));
        assert_eq!(candidate.patch.runtime_minutes, Some(120));
        assert_eq!(
            candidate.facts.av.as_ref().unwrap().actors,
            vec!["Actor One".to_owned(), "Actor Two".to_owned()]
        );
        assert_eq!(
            candidate.facts.av.as_ref().unwrap().studio.as_deref(),
            Some("Studio Alpha")
        );
        assert_eq!(
            candidate.facts.av.as_ref().unwrap().publisher.as_deref(),
            Some("Publisher Beta")
        );
        assert_eq!(
            candidate.facts.av.as_ref().unwrap().series.as_deref(),
            Some("Series Gamma")
        );
        assert_eq!(
            candidate.facts.av.as_ref().unwrap().directors,
            vec!["Director Delta".to_owned()]
        );
        assert_eq!(candidate.facts.av.as_ref().unwrap().wanted_count, Some(234));
        assert_eq!(candidate.facts.community_score_milli, Some(860));
        assert!(candidate.facts.external_ids.iter().any(|id| {
            id.provider == JAVLIBRARY_URL_EXTERNAL_ID_PROVIDER
                && id.value == "https://javlibrary.example/cn/?v=javli123"
        }));
        assert_eq!(candidate.artwork_candidates.len(), 1);
        assert_eq!(
            candidate.artwork_candidates[0].facts.kind,
            AddonArtworkKind::Poster
        );

        let requests = transport.requests();
        assert_eq!(requests.len(), 2);
        let search_body = request_json_body(&requests[0]);
        assert_eq!(
            search_body["url"],
            "https://javlibrary.example/cn/vl_searchbyid.php?keyword=SSNI-644"
        );
        let detail_body = request_json_body(&requests[1]);
        assert_eq!(
            detail_body["url"],
            "https://javlibrary.example/cn/?v=javli123"
        );
    }

    #[tokio::test]
    async fn javlibrary_provider_uses_explicit_id_for_direct_detail_lookup() {
        let transport = RenderedAvFixtureTransport::new(JAVLIBRARY_PROVIDER_ID);
        transport.push_rendered_html(
            "https://javlibrary.example/cn/?v=javli123",
            "SSNI-644 Direct JavLibrary Title",
            r#"
<!doctype html>
<html>
<body>
  <h3 id="video_title">SSNI-644 Direct JavLibrary Title</h3>
  <div id="video_info">
    <p>品番: SSNI-644</p>
    <p>發行日期: 2024-05-03</p>
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
        let provider = JavlibraryMetadataProvider::with_runtime(
            JavlibraryProviderConfig::new(
                "https://javlibrary.example".to_owned(),
                "cn".to_owned(),
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
                    provider: "javlibrary".to_owned(),
                    value: "javli123".to_owned(),
                }],
            })
            .await
            .unwrap();

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].provider_id, "javlibrary:movie:javli123");
        assert_eq!(
            candidates[0].patch.title.as_deref(),
            Some("SSNI-644 Direct JavLibrary Title")
        );

        let requests = transport.requests();
        assert_eq!(requests.len(), 1);
        let body = request_json_body(&requests[0]);
        assert_eq!(body["url"], "https://javlibrary.example/cn/?v=javli123");
    }

    #[tokio::test]
    async fn javlibrary_provider_skips_fc2_numbers() {
        let transport = RenderedAvFixtureTransport::new(JAVLIBRARY_PROVIDER_ID);
        let runtime = ProviderHttpRuntime::with_transport(
            ProviderHttpRuntimeConfig {
                retry_backoff_ms: 0,
                ..ProviderHttpRuntimeConfig::default()
            },
            transport.clone(),
        );
        let provider = JavlibraryMetadataProvider::with_runtime(
            JavlibraryProviderConfig::new(
                "https://javlibrary.example".to_owned(),
                "cn".to_owned(),
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
