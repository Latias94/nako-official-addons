use async_trait::async_trait;
use nako_addon_protocol::{
    AddonArtworkKind, AddonMetadataPatch, AddonSecretReferenceFieldDeclaration,
};
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
        http_runtime::{ProviderHttpTransport, ReqwestProviderHttpTransport},
        registry::{ProviderCatalogEntry, ProviderRenderedPageSupport},
        render_drift::{
            BrowserWorkerRenderDriftAction, BrowserWorkerRenderDriftCase,
            BrowserWorkerRenderDriftWaitFor, DEFAULT_SAMPLE_AV_NUMBER,
            ProviderRenderDriftCaseDescriptor, RENDER_DRIFT_SAMPLE_JAVBUS_AV_NUMBER_ENV_VAR,
            SLOW_LIVE_RENDER_DRIFT_SELECTOR_TIMEOUT_MS, SLOW_LIVE_RENDER_DRIFT_TIMEOUT_MS,
        },
        rendered_av,
        rendered_page::{
            RenderedPageAction, RenderedPageLoadState, RenderedPageRuntime,
            RenderedPageSupportConfig, RenderedPageWaitFor,
        },
    },
};

pub const JAVBUS_PROVIDER_ID: &str = "javbus";
pub(crate) const JAVBUS_COOKIE_ENV_VAR: &str = "NAKO_METADATA_SCRAPER_JAVBUS_COOKIE";
const JAVBUS_URL_EXTERNAL_ID_PROVIDER: &str = "javbus_url";
const JAVBUS_EXTERNAL_ID_CAPABILITIES: &[ProviderExternalIdCapability] = &[
    ProviderExternalIdCapability::new(
        JAVBUS_PROVIDER_ID,
        ExternalIdValueKind::Opaque,
        true,
        true,
        &["javbus_id"],
        false,
    ),
    ProviderExternalIdCapability::new(
        JAVBUS_URL_EXTERNAL_ID_PROVIDER,
        ExternalIdValueKind::Url,
        true,
        true,
        &["javbus_url"],
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
pub struct JavbusProviderConfig {
    pub(crate) base_url: String,
    pub(crate) rendered_pages: RenderedPageSupportConfig,
    pub(crate) render_path: String,
    pub(crate) cookie: Option<String>,
}

impl JavbusProviderConfig {
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
            cookie: None,
        }
    }

    #[must_use]
    pub fn from_env_lookup(mut lookup: impl FnMut(&str) -> Option<String>) -> Self {
        let base_url = lookup("NAKO_METADATA_SCRAPER_JAVBUS_BASE_URL")
            .and_then(non_empty_trimmed)
            .unwrap_or_else(|| "https://www.javbus.com".to_owned());
        let browser_worker_base_url = lookup("NAKO_METADATA_SCRAPER_BROWSER_WORKER_BASE_URL")
            .and_then(non_empty_trimmed)
            .unwrap_or_else(|| "http://nako-browser-worker:3000".to_owned());
        let render_path = lookup("NAKO_METADATA_SCRAPER_BROWSER_WORKER_RENDER_PATH")
            .and_then(non_empty_trimmed)
            .unwrap_or_else(|| "/render".to_owned());
        let timeout_ms = lookup("NAKO_METADATA_SCRAPER_JAVBUS_TIMEOUT_MS")
            .or_else(|| lookup("NAKO_METADATA_SCRAPER_BROWSER_WORKER_TIMEOUT_MS"))
            .and_then(|value| value.trim().parse::<u64>().ok())
            .filter(|value| *value > 0)
            .unwrap_or(Self::DEFAULT_TIMEOUT_MS);

        let mut config = Self::new(base_url, browser_worker_base_url, render_path, timeout_ms);
        config.rendered_pages = config.rendered_pages.with_env_defaults(|name| lookup(name));
        config.cookie = lookup(JAVBUS_COOKIE_ENV_VAR).and_then(non_empty_trimmed);
        config
    }

    #[must_use]
    pub const fn secret_field_id() -> &'static str {
        "javbus_cookie"
    }
}

#[must_use]
pub(crate) fn catalog_entry() -> ProviderCatalogEntry {
    ProviderCatalogEntry {
        id: ProviderId::Javbus,
        default_enabled: false,
        enabled_env_var: "NAKO_METADATA_SCRAPER_PROVIDER_JAVBUS_ENABLED",
        capabilities: &[
            "metadata_suggestion",
            "av_number_search",
            "javbus_direct_lookup",
            "javbus_movie_search",
            "browser_worker_rendered_html",
        ],
        field_quality: crate::engine::ProviderFieldQualityDescriptor::new(400, 400, 400, 200),
        secret_reference: Some(AddonSecretReferenceFieldDeclaration::new(
            JavbusProviderConfig::secret_field_id(),
            "JavBus Cookie",
            Some(
                "Optional Secret Reference for JavBus age verification or region access. The value is sent only to the browser worker as a Cookie header and is never emitted in diagnostics."
                    .to_owned(),
            ),
            false,
        )),
        external_id_capabilities: JAVBUS_EXTERNAL_ID_CAPABILITIES,
        load_config: load_config,
        proxy_configured: |_| false,
        network_policy_key: None,
        rendered_page_support: Some(ProviderRenderedPageSupport::new(rendered_page_config)),
        render_drift_case: Some(
            ProviderRenderDriftCaseDescriptor::new(
                30,
                RENDER_DRIFT_SAMPLE_JAVBUS_AV_NUMBER_ENV_VAR,
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
        .javbus_config()
        .map(|config| &config.rendered_pages)
}

fn render_drift_case_from_config(
    provider: &ProviderConfig,
    sample: &str,
) -> Option<BrowserWorkerRenderDriftCase> {
    provider
        .javbus_config()
        .map(|config| render_drift_case(config, sample))
}

fn load_config(input: ProviderConfigInput<'_>) -> ProviderConfig {
    let lookup = input.lookup;
    ProviderConfig::javbus(
        input.enabled,
        JavbusProviderConfig::from_env_lookup(|name| lookup(name)),
    )
}

fn build_provider(config: &Config) -> ProviderBuildStatus {
    let Some(javbus_config) = config
        .provider_config(ProviderId::Javbus)
        .and_then(|provider| provider.javbus_config().cloned())
    else {
        return ProviderBuildStatus::Unavailable;
    };
    match JavbusMetadataProvider::new(javbus_config) {
        Ok(provider) => ProviderBuildStatus::Ready(Box::new(provider)),
        Err(_) => ProviderBuildStatus::Unavailable,
    }
}

#[must_use]
pub(crate) fn render_drift_case(
    config: &JavbusProviderConfig,
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
        "javbus-detail",
        format!(
            "{}/{}",
            config.base_url.trim_end_matches('/'),
            url_path_segment(av_number)
        ),
    )
    .with_selector("h3, .info, #movie, .movie")
    .with_selector_timeout_ms(selector_timeout_ms)
    .with_header_from_env("cookie", JAVBUS_COOKIE_ENV_VAR)
    .with_rendered_page_defaults(&config.rendered_pages)
    .with_render_timeout_ms(render_timeout_ms)
    .with_min_text_bytes(100)
    .with_min_html_bytes(500)
    .with_action(
        BrowserWorkerRenderDriftAction::check("#ageVerify input[type=\"checkbox\"]").optional(),
    )
    .with_action(
        BrowserWorkerRenderDriftAction::click("#ageVerify #submit")
            .optional()
            .with_wait_for(
                BrowserWorkerRenderDriftWaitFor::domcontentloaded()
                    .with_timeout_ms(selector_timeout_ms),
            ),
    )
}

#[derive(Clone, Debug)]
pub struct JavbusMetadataProvider<T = ReqwestProviderHttpTransport>
where
    T: ProviderHttpTransport,
{
    config: JavbusProviderConfig,
    rendered_pages: RenderedPageRuntime<T>,
}

impl JavbusMetadataProvider<ReqwestProviderHttpTransport> {
    pub fn new(
        config: JavbusProviderConfig,
    ) -> crate::providers::http_runtime::ProviderHttpResult<Self> {
        let rendered_pages = RenderedPageRuntime::new(config.rendered_pages.clone())?;
        Ok(Self {
            config,
            rendered_pages,
        })
    }
}

impl<T> JavbusMetadataProvider<T>
where
    T: ProviderHttpTransport,
{
    #[must_use]
    pub fn with_runtime(
        config: JavbusProviderConfig,
        runtime: crate::providers::http_runtime::ProviderHttpRuntime<T>,
    ) -> Self {
        let rendered_pages =
            RenderedPageRuntime::with_runtime(config.rendered_pages.clone(), runtime);
        Self {
            config,
            rendered_pages,
        }
    }

    async fn render(
        &self,
        url: String,
    ) -> anyhow::Result<crate::providers::rendered_page::RenderedHtmlPage> {
        let mut intent = self
            .config
            .rendered_pages
            .intent(&self.config.render_path, url)
            .with_action(
                RenderedPageAction::check("#ageVerify input[type=\"checkbox\"]").optional(),
            )
            .with_action(
                RenderedPageAction::click("#ageVerify #submit")
                    .optional()
                    .with_wait_for(
                        RenderedPageWaitFor::new(RenderedPageLoadState::DomContentLoaded)
                            .with_timeout_ms(self.config.rendered_pages.timeout_ms),
                    ),
            );
        if let Some(cookie) = self.config.cookie.as_ref() {
            intent = intent.with_header("cookie", cookie);
        }
        self.rendered_pages
            .render_html(JAVBUS_PROVIDER_ID, "render page", intent)
            .await
    }

    async fn suggest_candidates(
        &self,
        query: &MetadataQuery,
    ) -> anyhow::Result<Vec<ProviderMetadataCandidate>> {
        rendered_av::suggest_candidates(self, query).await
    }

    fn search_url(&self, number: &str) -> String {
        format!(
            "{}/search/{}",
            self.config.base_url.trim_end_matches('/'),
            url_path_segment(number)
        )
    }

    fn detail_url(&self, id: &str) -> String {
        format!(
            "{}/{}",
            self.config.base_url.trim_end_matches('/'),
            id.trim().trim_start_matches('/')
        )
    }
}

#[async_trait]
impl<T> rendered_av::RenderedAvFlow for JavbusMetadataProvider<T>
where
    T: ProviderHttpTransport,
{
    fn provider_id(&self) -> &'static str {
        JAVBUS_PROVIDER_ID
    }

    fn url_external_id_provider(&self) -> &'static str {
        JAVBUS_URL_EXTERNAL_ID_PROVIDER
    }

    fn supports_route(&self, route: AvNumberRoute) -> bool {
        matches!(route, AvNumberRoute::Censored | AvNumberRoute::Uncensored)
    }

    async fn render_html_page(
        &self,
        url: String,
    ) -> anyhow::Result<crate::providers::rendered_page::RenderedHtmlPage> {
        self.render(url).await
    }

    fn absolute_url(&self, value: &str) -> String {
        rendered_av::absolute_url(&self.config.base_url, value)
    }

    fn detail_url(&self, id: &str) -> String {
        JavbusMetadataProvider::detail_url(self, id)
    }

    fn direct_lookup_av(&self, _query: &MetadataQuery) -> Option<AvQueryFacts> {
        None
    }

    fn prefer_direct_detail_for_av(&self) -> bool {
        true
    }

    fn search_url(&self, av: &AvQueryFacts) -> Option<String> {
        Some(JavbusMetadataProvider::search_url(self, &av.number))
    }

    fn search_results(
        &self,
        html: &str,
        av: &AvQueryFacts,
    ) -> Vec<rendered_av::RenderedAvSearchResult> {
        parse_search_results(html, av, &self.config.base_url)
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
impl<T> MetadataProvider for JavbusMetadataProvider<T>
where
    T: ProviderHttpTransport,
{
    fn id(&self) -> ProviderId {
        ProviderId::Javbus
    }

    fn supports_av_route(&self, route: AvNumberRoute) -> bool {
        matches!(route, AvNumberRoute::Censored | AvNumberRoute::Uncensored)
    }

    async fn suggest(
        &self,
        query: &MetadataQuery,
    ) -> anyhow::Result<Vec<ProviderMetadataCandidate>> {
        self.suggest_candidates(query).await
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct JavbusSearchResult {
    url: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct JavbusDetailFacts {
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
    poster_url: Option<String>,
    extrafanart_urls: Vec<String>,
}

impl JavbusDetailFacts {
    fn into_candidate(self, query: &MetadataQuery) -> ProviderMetadataCandidate {
        let mut tags = vec![
            JAVBUS_PROVIDER_ID.to_owned(),
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

        let mut artwork_candidates = Vec::new();
        if let Some(poster_url) = self.poster_url.clone() {
            artwork_candidates.push(javbus_artwork_candidate(
                &self.id,
                AddonArtworkKind::Poster,
                poster_url,
                0,
            ));
        }
        for (index, url) in self.extrafanart_urls.iter().cloned().enumerate() {
            artwork_candidates.push(javbus_artwork_candidate(
                &self.id,
                AddonArtworkKind::Backdrop,
                url,
                index + 1,
            ));
        }

        ProviderMetadataCandidate {
            provider: JAVBUS_PROVIDER_ID.to_owned(),
            provider_id: format!("javbus:movie:{}", self.id),
            patch: AddonMetadataPatch {
                title: Some(self.title.clone()),
                original_title: None,
                sort_title: Some(self.title.clone()),
                overview: None,
                release_date: self.release_date.clone(),
                runtime_minutes: self.runtime_minutes,
                tagline: Some("JavBus AV title".to_owned()),
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
                    thumb_url: self.poster_url.clone(),
                    extrafanart_urls: self.extrafanart_urls.clone(),
                    ..AvMetadataFacts::default()
                }
                .non_empty(),
                community_score_milli: None,
                community_vote_count: None,
                external_ids: vec![
                    ProviderExternalId {
                        provider: JAVBUS_PROVIDER_ID.to_owned(),
                        value: self.id,
                    },
                    ProviderExternalId {
                        provider: JAVBUS_URL_EXTERNAL_ID_PROVIDER.to_owned(),
                        value: self.url,
                    },
                    ProviderExternalId {
                        provider: AV_NUMBER_EXTERNAL_ID_PROVIDER.to_owned(),
                        value: self.av.number,
                    },
                ],
                provider_outcomes: vec![ProviderOutcome::JavbusRenderedHtmlParsed],
                provider_note: None,
            },
            artwork_candidates,
        }
    }
}

fn parse_search_results(html: &str, av: &AvQueryFacts, base_url: &str) -> Vec<JavbusSearchResult> {
    let document = Html::parse_document(html);
    let Ok(selector) = Selector::parse(".movie-box, .photo-frame a, .item a, a[href*=\"/\"]")
    else {
        return Vec::new();
    };
    let mut results = Vec::new();

    for element in document.select(&selector) {
        let href = element
            .value()
            .attr("href")
            .or_else(|| {
                element
                    .select(&Selector::parse("a[href]").ok()?)
                    .next()
                    .and_then(|link| link.value().attr("href"))
            })
            .unwrap_or_default();
        if href.is_empty() || href.contains("/search/") {
            continue;
        }
        let text = normalize_whitespace(&element.text().collect::<Vec<_>>().join(" "));
        if !text_or_url_matches_av(&text, href, av) {
            continue;
        }
        results.push(JavbusSearchResult {
            url: absolute_url(base_url, href),
        });
    }

    dedupe_search_results(results)
}

fn parse_detail_page(
    html: &str,
    detail_url: &str,
    av: Option<AvQueryFacts>,
) -> Option<JavbusDetailFacts> {
    let document = Html::parse_document(html);
    let body_text = element_text(&document, "body").unwrap_or_default();
    let info_text = element_text(&document, ".movie, .info, .container, body")
        .unwrap_or_else(|| body_text.clone());
    let base_url = site_base_url(detail_url).unwrap_or_else(|| detail_url.to_owned());
    let title = first_non_empty(&[
        element_text(&document, "h3, h1").as_deref(),
        attr_value(&document, "meta[property=\"og:title\"]", "content").as_deref(),
        element_text(&document, "title").as_deref(),
    ])?;
    if is_age_verification_page(&document, &title, &body_text) {
        return None;
    }
    let number = javbus_labeled_value(
        &document,
        &info_text,
        &["識別碼", "识别码", "品番", "番號", "Number"],
    )
    .or_else(|| facts_from_text(&title, AvNumberSource::ExternalId).map(|facts| facts.number))
    .or_else(|| facts_from_text(detail_url, AvNumberSource::ExternalId).map(|facts| facts.number));
    let av = number
        .as_deref()
        .and_then(|value| facts_from_text(value, AvNumberSource::ExternalId))
        .or_else(|| facts_from_text(&title, AvNumberSource::ExternalId))
        .or(av)?;
    let release_date = javbus_labeled_value(
        &document,
        &info_text,
        &["發行日期", "发行日期", "発売日", "Release Date"],
    )
    .or_else(|| first_iso_date(&body_text));
    let release_year = release_date.as_deref().and_then(first_year);
    let runtime_minutes = javbus_labeled_value(
        &document,
        &info_text,
        &["長度", "长度", "収録時間", "Runtime"],
    )
    .and_then(|value| parse_minutes(&value));
    let actors = actor_names(&document);
    let tags = link_texts(&document, "a[href*=\"/genre/\"], a[href*=\"/tag/\"]");
    let studio = first_link_text(&document, "a[href*=\"/studio/\"]").or_else(|| {
        javbus_labeled_value(
            &document,
            &info_text,
            &["製作商", "制作商", "メーカー", "Studio"],
        )
    });
    let publisher = first_link_text(&document, "a[href*=\"/label/\"]").or_else(|| {
        javbus_labeled_value(
            &document,
            &info_text,
            &["發行商", "发行商", "Label", "Publisher"],
        )
    });
    let series = first_link_text(&document, "a[href*=\"/series/\"]")
        .or_else(|| javbus_labeled_value(&document, &info_text, &["系列", "Series"]));
    let director = first_link_text(&document, "a[href*=\"/director/\"]")
        .or_else(|| javbus_labeled_value(&document, &info_text, &["導演", "导演", "Director"]));
    let poster_url = first_attr_url(
        &document,
        &base_url,
        &[
            ("a.bigImage", "href"),
            (".bigImage", "href"),
            ("a.bigImage img, .bigImage img", "src"),
            ("meta[property=\"og:image\"]", "content"),
        ],
    );
    let extrafanart_urls = image_urls(
        &document,
        "#sample-waterfall a, .sample-box, a.sample-box, .samples a, .sample img",
        &base_url,
    );

    Some(JavbusDetailFacts {
        id: movie_id_from_url(detail_url).unwrap_or_else(|| av.number.clone()),
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
        poster_url,
        extrafanart_urls,
    })
}

fn javbus_artwork_candidate(
    movie_id: &str,
    kind: AddonArtworkKind,
    source_url: String,
    index: usize,
) -> ProviderArtworkCandidate {
    ProviderArtworkCandidate {
        provider: JAVBUS_PROVIDER_ID.to_owned(),
        provider_id: format!("javbus:movie:{movie_id}:artwork:{index}"),
        facts: ProviderArtworkCandidateFacts {
            kind,
            source_url,
            language: None,
            width: None,
            height: None,
        },
    }
}

fn dedupe_search_results(results: Vec<JavbusSearchResult>) -> Vec<JavbusSearchResult> {
    results.into_iter().fold(Vec::new(), |mut values, result| {
        if !values.iter().any(|existing| existing.url == result.url) {
            values.push(result);
        }
        values
    })
}

fn text_or_url_matches_av(text: &str, url: &str, av: &AvQueryFacts) -> bool {
    [text, url]
        .into_iter()
        .filter_map(|value| facts_from_text(value, AvNumberSource::ExternalId))
        .any(|facts| facts.number.eq_ignore_ascii_case(&av.number))
        || compact(text).contains(&compact(&av.number))
        || compact(url).contains(&compact(&av.number))
}

fn element_text(document: &Html, selector: &str) -> Option<String> {
    let selector = Selector::parse(selector).ok()?;
    document
        .select(&selector)
        .next()
        .map(|element| normalize_whitespace(&element.text().collect::<Vec<_>>().join(" ")))
        .filter(|value| !value.is_empty())
}

fn first_link_text(document: &Html, selector: &str) -> Option<String> {
    link_texts(document, selector).into_iter().next()
}

fn actor_names(document: &Html) -> Vec<String> {
    let mut values = link_texts(document, "a[href*=\"/star/\"], a[href*=\"/actor/\"]");
    push_attr_values(
        document,
        ".star-name img, a[href*=\"/star/\"] img, a[href*=\"/actor/\"] img",
        "title",
        &mut values,
    );
    push_attr_values(
        document,
        ".star-name img, a[href*=\"/star/\"] img, a[href*=\"/actor/\"] img",
        "alt",
        &mut values,
    );
    values
}

fn link_texts(document: &Html, selector: &str) -> Vec<String> {
    let Ok(selector) = Selector::parse(selector) else {
        return Vec::new();
    };
    document
        .select(&selector)
        .map(|element| normalize_whitespace(&element.text().collect::<Vec<_>>().join(" ")))
        .filter(|value| !value.is_empty())
        .fold(Vec::new(), |mut values, value| {
            if !values.contains(&value) {
                values.push(value);
            }
            values
        })
}

fn push_attr_values(document: &Html, selector: &str, attr: &str, values: &mut Vec<String>) {
    let Ok(selector) = Selector::parse(selector) else {
        return;
    };
    for value in document
        .select(&selector)
        .filter_map(|element| element.value().attr(attr))
        .map(normalize_whitespace)
        .filter(|value| !value.is_empty())
    {
        if !values.contains(&value) {
            values.push(value);
        }
    }
}

fn image_urls(document: &Html, selector: &str, base_url: &str) -> Vec<String> {
    let Ok(selector) = Selector::parse(selector) else {
        return Vec::new();
    };
    document
        .select(&selector)
        .filter_map(|element| {
            element
                .value()
                .attr("href")
                .or_else(|| element.value().attr("src"))
                .or_else(|| {
                    element
                        .select(&Selector::parse("img").ok()?)
                        .next()
                        .and_then(|image| image.value().attr("src"))
                })
        })
        .map(|value| absolute_url(base_url, value))
        .map(normalize_url)
        .filter(|value| !value.trim().is_empty())
        .fold(Vec::new(), |mut values, value| {
            if !values.contains(&value) {
                values.push(value);
            }
            values
        })
}

fn attr_value(document: &Html, selector: &str, attr: &str) -> Option<String> {
    let selector = Selector::parse(selector).ok()?;
    document
        .select(&selector)
        .find_map(|element| element.value().attr(attr))
        .map(str::to_owned)
        .filter(|value| !value.trim().is_empty())
}

fn is_age_verification_page(document: &Html, title: &str, body_text: &str) -> bool {
    selector_exists(document, "#ageVerify")
        || title
            .to_ascii_lowercase()
            .contains("age verification javbus")
        || body_text.contains("你是否已經成年")
        || body_text.contains("所在地區年齡檢測")
}

fn selector_exists(document: &Html, selector: &str) -> bool {
    Selector::parse(selector)
        .ok()
        .is_some_and(|selector| document.select(&selector).next().is_some())
}

fn first_attr_url(document: &Html, base_url: &str, selectors: &[(&str, &str)]) -> Option<String> {
    selectors
        .iter()
        .find_map(|(selector, attr)| attr_value(document, selector, attr))
        .map(|value| normalize_url(absolute_url(base_url, &value)))
}

const JAVBUS_LABELS: &[&str] = &[
    "識別碼",
    "识别码",
    "品番",
    "番號",
    "Number",
    "發行日期",
    "发行日期",
    "発売日",
    "Release Date",
    "長度",
    "长度",
    "収録時間",
    "Runtime",
    "製作商",
    "制作商",
    "メーカー",
    "Studio",
    "發行商",
    "发行商",
    "Label",
    "Publisher",
    "系列",
    "Series",
    "導演",
    "导演",
    "Director",
];

const JAVBUS_LABEL_ROW_SELECTOR: &str = ".movie p, .movie li, .movie tr, \
         .info p, .info li, .info tr, \
         .container p, .container li, .container tr, \
         table tr";

fn javbus_labeled_value(document: &Html, info_text: &str, labels: &[&str]) -> Option<String> {
    rendered_av::structured_or_labeled_value(
        document,
        JAVBUS_LABEL_ROW_SELECTOR,
        info_text,
        labels,
        JAVBUS_LABELS,
    )
}

fn first_non_empty(values: &[Option<&str>]) -> Option<String> {
    values
        .iter()
        .flatten()
        .map(|value| normalize_whitespace(value))
        .find(|value| !value.is_empty())
}

fn first_iso_date(text: &str) -> Option<String> {
    for token in text.split_whitespace() {
        if token.len() >= 10 && token.as_bytes().get(4) == Some(&b'-') {
            let value = &token[..10];
            if value
                .chars()
                .enumerate()
                .all(|(index, character)| matches!(index, 4 | 7) || character.is_ascii_digit())
            {
                return Some(value.to_owned());
            }
        }
    }
    None
}

fn first_year(text: &str) -> Option<i32> {
    for token in text.split(|character: char| !character.is_ascii_digit()) {
        if token.len() == 4 {
            let year = token.parse::<i32>().ok()?;
            if (1888..=2100).contains(&year) {
                return Some(year);
            }
        }
    }
    None
}

fn parse_minutes(value: &str) -> Option<u32> {
    value
        .split(|character: char| !character.is_ascii_digit())
        .find(|value| !value.is_empty())
        .and_then(|value| value.parse::<u32>().ok())
}

fn movie_id_from_url(url: &str) -> Option<String> {
    let value = url.trim_end_matches('/').rsplit('/').next()?.trim();
    (!value.is_empty()).then(|| value.to_owned())
}

fn site_base_url(url: &str) -> Option<String> {
    let scheme_end = url.find("://")? + 3;
    let rest = &url[scheme_end..];
    let host_end = rest.find('/').unwrap_or(rest.len());
    (host_end > 0).then(|| url[..scheme_end + host_end].to_owned())
}

fn absolute_url(base_url: &str, value: &str) -> String {
    let value = value.trim();
    if value.starts_with("http://") || value.starts_with("https://") {
        return value.to_owned();
    }
    if let Some(value) = value.strip_prefix("//") {
        return format!("https://{value}");
    }
    format!(
        "{}/{}",
        base_url.trim_end_matches('/'),
        value.trim_start_matches('/')
    )
}

fn normalize_url(value: String) -> String {
    if let Some(value) = value.strip_prefix("//") {
        return format!("https://{value}");
    }
    value
}

fn normalize_whitespace(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn compact(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .map(|character| character.to_ascii_uppercase())
        .collect()
}

fn url_path_segment(value: &str) -> String {
    value
        .bytes()
        .flat_map(|byte| {
            if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.') {
                vec![char::from(byte)]
            } else {
                format!("%{byte:02X}").chars().collect()
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::providers::{
        http_runtime::{ProviderHttpRuntime, ProviderHttpRuntimeConfig},
        rendered_av_fixture::{RenderedAvFixtureTransport, request_json_body},
    };

    use super::*;

    #[test]
    fn javbus_config_trims_cookie_secret() {
        let config = JavbusProviderConfig::from_env_lookup(|name| match name {
            "NAKO_METADATA_SCRAPER_JAVBUS_COOKIE" => Some(" age=verified ".to_owned()),
            _ => None,
        });

        assert_eq!(config.cookie.as_deref(), Some("age=verified"));
        assert_eq!(JavbusProviderConfig::secret_field_id(), "javbus_cookie");
    }

    #[tokio::test]
    async fn javbus_provider_sends_cookie_to_browser_worker_render_request() {
        let transport = RenderedAvFixtureTransport::new(JAVBUS_PROVIDER_ID);
        transport.push_rendered_html(
            "https://javbus.example/SSNI-644",
            "SSNI-644 Cookie Title",
            r#"
<!doctype html>
<html>
<body>
  <h3>SSNI-644 Cookie Title</h3>
  <div class="movie"><p>識別碼: SSNI-644</p></div>
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
        let mut config = JavbusProviderConfig::new(
            "https://javbus.example".to_owned(),
            "http://browser-worker.example".to_owned(),
            "/render".to_owned(),
            10_000,
        );
        config.cookie = Some("age=verified".to_owned());
        let provider = JavbusMetadataProvider::with_runtime(config, runtime);

        let candidates = provider
            .suggest(&MetadataQuery::from_payload(
                &serde_json::json!({"file_name": "SSNI-00644.mp4"}),
                "zh-CN",
            ))
            .await
            .unwrap();

        assert_eq!(candidates.len(), 1);
        let requests = transport.requests();
        let body = request_json_body(&requests[0]);
        assert_eq!(body["headers"]["cookie"], "age=verified");
    }

    #[tokio::test]
    async fn javbus_provider_uses_browser_worker_render_contract_for_av_search_and_detail() {
        let transport = RenderedAvFixtureTransport::new(JAVBUS_PROVIDER_ID);
        transport.push_rendered_html(
            "https://javbus.example/SSNI-644",
            "No direct detail",
            r#"
<!doctype html>
<html>
<body>not found</body>
</html>"#,
        );
        transport.push_rendered_html(
            "https://javbus.example/search/SSNI-644",
            "JavBus Search",
            r#"
<!doctype html>
<html>
<body>
  <a class="movie-box" href="/SSNI-644">
    <span class="photo-info">SSNI-644 JavBus Synthetic Title</span>
  </a>
  <a class="movie-box" href="/ABP-001">ABP-001 Other Title</a>
</body>
</html>"#,
        );
        transport.push_rendered_html(
            "https://javbus.example/SSNI-644",
            "SSNI-644 JavBus Synthetic Title",
            r#"
<!doctype html>
<html>
<head>
  <meta property="og:image" content="//img.example/javbus-cover.jpg">
</head>
<body>
  <h3>SSNI-644 JavBus Synthetic Title</h3>
  <div class="movie">
    <p>識別碼: SSNI-644</p>
    <p>發行日期: 2024-05-02</p>
    <p>長度: 121分鐘</p>
  </div>
  <a href="/star/actor-one">Actor One</a>
  <a href="/star/actor-two">Actor Two</a>
  <a href="/genre/drama">剧情</a>
  <a href="/genre/uniform">制服</a>
  <a href="/studio/studio-alpha">Studio Alpha</a>
  <a href="/label/publisher-beta">Publisher Beta</a>
  <a href="/series/series-gamma">Series Gamma</a>
  <a href="/director/director-delta">Director Delta</a>
  <a class="bigImage"><img src="//img.example/poster.jpg"></a>
  <div id="sample-waterfall">
    <a href="//img.example/sample1.jpg"><img src="//img.example/sample1-thumb.jpg"></a>
    <a href="https://img.example/sample2.jpg">sample</a>
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
        let provider = JavbusMetadataProvider::with_runtime(
            JavbusProviderConfig::new(
                "https://javbus.example".to_owned(),
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
        assert_eq!(candidate.provider, "javbus");
        assert_eq!(candidate.provider_id, "javbus:movie:SSNI-644");
        assert_eq!(
            candidate.patch.title.as_deref(),
            Some("SSNI-644 JavBus Synthetic Title")
        );
        assert_eq!(candidate.patch.release_date.as_deref(), Some("2024-05-02"));
        assert_eq!(candidate.patch.runtime_minutes, Some(121));
        assert_eq!(
            candidate.patch.genres.as_ref().unwrap(),
            &vec!["剧情".to_owned(), "制服".to_owned()]
        );
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
        assert_eq!(
            candidate.facts.av.as_ref().unwrap().extrafanart_urls,
            vec![
                "https://img.example/sample1.jpg".to_owned(),
                "https://img.example/sample2.jpg".to_owned()
            ]
        );
        assert!(
            candidate.facts.external_ids.iter().any(|id| {
                id.provider == AV_NUMBER_EXTERNAL_ID_PROVIDER && id.value == "SSNI-644"
            })
        );
        assert!(candidate.facts.external_ids.iter().any(|id| {
            id.provider == JAVBUS_URL_EXTERNAL_ID_PROVIDER
                && id.value == "https://javbus.example/SSNI-644"
        }));
        assert_eq!(candidate.artwork_candidates.len(), 3);
        assert_eq!(
            candidate.artwork_candidates[0].facts.kind,
            AddonArtworkKind::Poster
        );

        let requests = transport.requests();
        assert_eq!(requests.len(), 3);
        assert_eq!(requests[0].url, "http://browser-worker.example/render");
        assert_eq!(requests[1].url, "http://browser-worker.example/render");
        assert_eq!(requests[2].url, "http://browser-worker.example/render");
        let direct_body = request_json_body(&requests[0]);
        assert_eq!(direct_body["url"], "https://javbus.example/SSNI-644");
        assert_eq!(direct_body["actions"][0]["type"], "check");
        assert_eq!(
            direct_body["actions"][0]["selector"],
            "#ageVerify input[type=\"checkbox\"]"
        );
        assert_eq!(direct_body["actions"][0]["optional"], true);
        assert_eq!(direct_body["actions"][1]["type"], "click");
        assert_eq!(direct_body["actions"][1]["selector"], "#ageVerify #submit");
        assert_eq!(
            direct_body["actions"][1]["wait_for"]["state"],
            "domcontentloaded"
        );
        let search_body = request_json_body(&requests[1]);
        assert_eq!(search_body["url"], "https://javbus.example/search/SSNI-644");
        let detail_body = request_json_body(&requests[2]);
        assert_eq!(detail_body["url"], "https://javbus.example/SSNI-644");
    }

    #[tokio::test]
    async fn javbus_provider_prefers_direct_detail_for_inferred_av_number() {
        let transport = RenderedAvFixtureTransport::new(JAVBUS_PROVIDER_ID);
        transport.push_rendered_html(
            "https://javbus.example/SSNI-644",
            "SSNI-644 Direct Field-Rich Title",
            r#"
<!doctype html>
<html>
<body>
  <h3>SSNI-644 Direct Field-Rich Title</h3>
  <div class="container">
    <p><span class="header">識別碼:</span> SSNI-644</p>
    <p><span class="header">發行日期:</span> 2024-05-02</p>
    <p><span class="header">長度:</span> 121分鐘</p>
  </div>
  <div class="star-name"><a href="/star/actor-one">Actor One</a></div>
  <a href="/genre/drama"><label>剧情</label></a>
  <a href="/studio/studio-alpha">Studio Alpha</a>
  <a href="/label/publisher-beta">Publisher Beta</a>
  <a href="/series/series-gamma">Series Gamma</a>
  <a href="/director/director-delta">Director Delta</a>
  <a class="bigImage" href="/pics/cover/ssni644_b.jpg">
    <img src="/pics/thumb/ssni644.jpg">
  </a>
  <div id="sample-waterfall">
    <a href="/pics/sample/sample1.jpg"><img src="/pics/sample/sample1-thumb.jpg"></a>
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
        let provider = JavbusMetadataProvider::with_runtime(
            JavbusProviderConfig::new(
                "https://javbus.example".to_owned(),
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
        let av = candidate.facts.av.as_ref().unwrap();
        assert_eq!(
            candidate.patch.title.as_deref(),
            Some("SSNI-644 Direct Field-Rich Title")
        );
        assert_eq!(candidate.patch.release_date.as_deref(), Some("2024-05-02"));
        assert_eq!(candidate.patch.runtime_minutes, Some(121));
        assert_eq!(av.actors, vec!["Actor One".to_owned()]);
        assert_eq!(av.studio.as_deref(), Some("Studio Alpha"));
        assert_eq!(av.publisher.as_deref(), Some("Publisher Beta"));
        assert_eq!(av.series.as_deref(), Some("Series Gamma"));
        assert_eq!(av.directors, vec!["Director Delta".to_owned()]);
        assert_eq!(
            av.thumb_url.as_deref(),
            Some("https://javbus.example/pics/cover/ssni644_b.jpg")
        );
        assert_eq!(
            av.extrafanart_urls,
            vec!["https://javbus.example/pics/sample/sample1.jpg".to_owned()]
        );
        assert_eq!(candidate.artwork_candidates.len(), 2);
        assert_eq!(
            candidate.artwork_candidates[0].facts.source_url,
            "https://javbus.example/pics/cover/ssni644_b.jpg"
        );
        assert_eq!(
            transport.rendered_request_urls(),
            vec!["https://javbus.example/SSNI-644".to_owned()]
        );
    }

    #[tokio::test]
    async fn javbus_provider_rejects_age_verification_pages_as_non_candidates() {
        let transport = RenderedAvFixtureTransport::new(JAVBUS_PROVIDER_ID);
        transport.push_rendered_html(
            "https://javbus.example/SSNI-644",
            "Age Verification JavBus - JavBus",
            r#"
<!doctype html>
<html>
<body>
  <title>Age Verification JavBus - JavBus</title>
  <div id="ageVerify">
    <h4>你是否已經成年?</h4>
    <form id="form1"><input type="checkbox"><input id="submit" type="submit"></form>
  </div>
</body>
</html>"#,
        );
        transport.push_rendered_html(
            "https://javbus.example/search/SSNI-644",
            "Age Verification JavBus - JavBus",
            r#"
<!doctype html>
<html>
<body>
  <title>Age Verification JavBus - JavBus</title>
  <div id="ageVerify"><h4>所在地區年齡檢測</h4></div>
</body>
</html>"#,
        );
        let runtime = ProviderHttpRuntime::with_transport(
            ProviderHttpRuntimeConfig {
                retry_backoff_ms: 0,
                ..ProviderHttpRuntimeConfig::default()
            },
            transport,
        );
        let provider = JavbusMetadataProvider::with_runtime(
            JavbusProviderConfig::new(
                "https://javbus.example".to_owned(),
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
    }

    #[tokio::test]
    async fn javbus_provider_parses_search_page_when_it_is_already_detail_page() {
        let transport = RenderedAvFixtureTransport::new(JAVBUS_PROVIDER_ID);
        transport.push_rendered_html(
            "https://javbus.example/search/SSNI-644",
            "SSNI-644 Detail Title",
            r#"
<!doctype html>
<html>
<head>
  <title>SSNI-644 Detail Title</title>
</head>
<body>
  <div class="container">
    <label>識別碼:</label><span>SSNI-644</span>
    <label>發行日期:</label><span>2024-05-02</span>
    <label>長度:</label><span>121分鐘</span>
  </div>
  <a href="/star/actor-one">Actor One</a>
  <a href="/genre/drama">剧情</a>
  <a href="/studio/studio-alpha">Studio Alpha</a>
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
        let provider = JavbusMetadataProvider::with_runtime(
            JavbusProviderConfig::new(
                "https://javbus.example".to_owned(),
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
        assert_eq!(
            candidates[0].patch.title.as_deref(),
            Some("SSNI-644 Detail Title")
        );
        assert_eq!(
            candidates[0].patch.release_date.as_deref(),
            Some("2024-05-02")
        );
        assert_eq!(candidates[0].patch.runtime_minutes, Some(121));
        assert_eq!(transport.requests().len(), 1);
    }

    #[tokio::test]
    async fn javbus_provider_uses_explicit_javbus_id_for_direct_detail_lookup() {
        let transport = RenderedAvFixtureTransport::new(JAVBUS_PROVIDER_ID);
        transport.push_rendered_html(
            "https://javbus.example/SSNI-644",
            "SSNI-644 Direct JavBus Title",
            r#"
<!doctype html>
<html>
<body>
  <h3>SSNI-644 Direct JavBus Title</h3>
  <div class="movie">
    <p>識別碼: SSNI-644</p>
    <p>發行日期: 2024-05-02</p>
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
        let provider = JavbusMetadataProvider::with_runtime(
            JavbusProviderConfig::new(
                "https://javbus.example".to_owned(),
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
                external_ids: vec![crate::engine::QueryExternalId {
                    provider: "javbus".to_owned(),
                    value: "SSNI-644".to_owned(),
                }],
            })
            .await
            .unwrap();

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].provider_id, "javbus:movie:SSNI-644");
        assert_eq!(
            candidates[0].patch.title.as_deref(),
            Some("SSNI-644 Direct JavBus Title")
        );

        let requests = transport.requests();
        assert_eq!(requests.len(), 1);
        let body = request_json_body(&requests[0]);
        assert_eq!(body["url"], "https://javbus.example/SSNI-644");
    }

    #[tokio::test]
    async fn javbus_provider_skips_fc2_numbers() {
        let transport = RenderedAvFixtureTransport::new(JAVBUS_PROVIDER_ID);
        let runtime = ProviderHttpRuntime::with_transport(
            ProviderHttpRuntimeConfig {
                retry_backoff_ms: 0,
                ..ProviderHttpRuntimeConfig::default()
            },
            transport.clone(),
        );
        let provider = JavbusMetadataProvider::with_runtime(
            JavbusProviderConfig::new(
                "https://javbus.example".to_owned(),
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
