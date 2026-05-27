use async_trait::async_trait;
use nako_addon_protocol::{AddonArtworkKind, AddonMetadataPatch};
use scraper::{Html, Selector};

#[cfg(test)]
use crate::providers::http_runtime::ProviderHttpRuntime;
use crate::{
    Config,
    config::{ProviderConfig, ProviderId, non_empty_trimmed},
    engine::{
        AvMetadataFacts, MetadataQuery, ProviderArtworkCandidate, ProviderArtworkCandidateFacts,
        ProviderCandidateFacts, ProviderExternalId, ProviderMetadataCandidate, ProviderOutcome,
        av::{
            AV_NUMBER_EXTERNAL_ID_PROVIDER, AvNumberRoute, AvNumberSource, AvQueryFacts,
            facts_from_query, facts_from_text,
        },
    },
    providers::{
        MetadataProvider, ProviderBuildStatus, ProviderConfigInput,
        http_runtime::{ProviderHttpResult, ProviderHttpTransport, ReqwestProviderHttpTransport},
        registry::ProviderCatalogEntry,
        render_drift::{
            BrowserWorkerRenderDriftCase, SLOW_LIVE_RENDER_DRIFT_SELECTOR_TIMEOUT_MS,
            SLOW_LIVE_RENDER_DRIFT_TIMEOUT_MS,
        },
        rendered_av,
        rendered_page::{RenderedHtmlPage, RenderedPageRuntime, RenderedPageSupportConfig},
    },
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OfficialUncensoredProviderConfig {
    pub(crate) base_url: String,
    pub(crate) rendered_pages: RenderedPageSupportConfig,
    pub(crate) render_path: String,
}

impl OfficialUncensoredProviderConfig {
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
    pub fn from_env_lookup(
        mut lookup: impl FnMut(&str) -> Option<String>,
        base_url_env_var: &'static str,
        timeout_env_var: &'static str,
        default_base_url: &'static str,
    ) -> Self {
        let base_url = lookup(base_url_env_var)
            .and_then(non_empty_trimmed)
            .unwrap_or_else(|| default_base_url.to_owned());
        let browser_worker_base_url = lookup("NAKO_METADATA_SCRAPER_BROWSER_WORKER_BASE_URL")
            .and_then(non_empty_trimmed)
            .unwrap_or_else(|| "http://nako-browser-worker:3000".to_owned());
        let render_path = lookup("NAKO_METADATA_SCRAPER_BROWSER_WORKER_RENDER_PATH")
            .and_then(non_empty_trimmed)
            .unwrap_or_else(|| "/render".to_owned());
        let timeout_ms = lookup(timeout_env_var)
            .or_else(|| lookup("NAKO_METADATA_SCRAPER_BROWSER_WORKER_TIMEOUT_MS"))
            .and_then(|value| value.trim().parse::<u64>().ok())
            .filter(|value| *value > 0)
            .unwrap_or(Self::DEFAULT_TIMEOUT_MS);

        let mut config = Self::new(base_url, browser_worker_base_url, render_path, timeout_ms);
        config.rendered_pages = config.rendered_pages.with_env_defaults(|name| lookup(name));
        config
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct OfficialUncensoredSite {
    pub(crate) provider_id: &'static str,
    pub(crate) url_external_id_provider: &'static str,
    pub(crate) provider_id_enum: ProviderId,
    pub(crate) default_base_url: &'static str,
    pub(crate) base_url_env_var: &'static str,
    pub(crate) timeout_env_var: &'static str,
    pub(crate) enabled_env_var: &'static str,
    pub(crate) capabilities: &'static [&'static str],
    pub(crate) field_quality: crate::engine::ProviderFieldQualityDescriptor,
    pub(crate) detail_path: OfficialUncensoredDetailPath,
    pub(crate) outcome: ProviderOutcome,
    pub(crate) tagline: &'static str,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum OfficialUncensoredDetailPath {
    CaribbeanMoviepages,
    Movies,
    MoviesDirectory,
}

impl OfficialUncensoredSite {
    pub(crate) fn detail_url(
        self,
        config: &OfficialUncensoredProviderConfig,
        av_number: &str,
    ) -> String {
        let site_number = self.site_number(av_number);
        match self.detail_path {
            OfficialUncensoredDetailPath::CaribbeanMoviepages => format!(
                "{}/moviepages/{}/index.html",
                config.base_url.trim_end_matches('/'),
                rendered_av::percent_encode(&site_number)
            ),
            OfficialUncensoredDetailPath::Movies => format!(
                "{}/movies/{}/index.html",
                config.base_url.trim_end_matches('/'),
                rendered_av::percent_encode(&site_number)
            ),
            OfficialUncensoredDetailPath::MoviesDirectory => format!(
                "{}/movies/{}/",
                config.base_url.trim_end_matches('/'),
                rendered_av::percent_encode(&site_number)
            ),
        }
    }

    pub(crate) fn site_number(self, av_number: &str) -> String {
        match self.detail_path {
            OfficialUncensoredDetailPath::CaribbeanMoviepages => av_number.replace('_', "-"),
            OfficialUncensoredDetailPath::Movies
            | OfficialUncensoredDetailPath::MoviesDirectory => av_number.replace('-', "_"),
        }
    }

    fn url_number(self, detail_url: &str) -> Option<String> {
        let marker = match self.detail_path {
            OfficialUncensoredDetailPath::CaribbeanMoviepages => "/moviepages/",
            OfficialUncensoredDetailPath::Movies
            | OfficialUncensoredDetailPath::MoviesDirectory => "/movies/",
        };
        let start = detail_url.find(marker)? + marker.len();
        let rest = &detail_url[start..];
        let end = rest.find('/').unwrap_or(rest.len());
        let value = rest[..end].trim();
        (!value.is_empty()).then(|| value.to_owned())
    }
}

#[must_use]
pub(crate) fn catalog_entry(
    site: &'static OfficialUncensoredSite,
    external_id_capabilities: &'static [crate::engine::ProviderExternalIdCapability],
    load_config: for<'a> fn(ProviderConfigInput<'a>) -> ProviderConfig,
    build: fn(&Config) -> ProviderBuildStatus,
) -> ProviderCatalogEntry {
    ProviderCatalogEntry {
        id: site.provider_id_enum,
        default_enabled: false,
        enabled_env_var: site.enabled_env_var,
        capabilities: site.capabilities,
        field_quality: site.field_quality,
        secret_reference: None,
        external_id_capabilities,
        load_config,
        proxy_configured: |_| false,
        network_policy_key: None,
        build,
    }
}

pub(crate) fn load_config(
    input: ProviderConfigInput<'_>,
    site: &'static OfficialUncensoredSite,
    make_config: fn(bool, OfficialUncensoredProviderConfig) -> ProviderConfig,
) -> ProviderConfig {
    let lookup = input.lookup;
    make_config(
        input.enabled,
        OfficialUncensoredProviderConfig::from_env_lookup(
            |name| lookup(name),
            site.base_url_env_var,
            site.timeout_env_var,
            site.default_base_url,
        ),
    )
}

pub(crate) fn build_provider(
    config: &Config,
    site: &'static OfficialUncensoredSite,
    get_config: fn(&ProviderConfig) -> Option<&OfficialUncensoredProviderConfig>,
) -> ProviderBuildStatus {
    let Some(provider_config) = config
        .provider_config(site.provider_id_enum)
        .and_then(get_config)
        .cloned()
    else {
        return ProviderBuildStatus::Unavailable;
    };
    match OfficialUncensoredMetadataProvider::new(site, provider_config) {
        Ok(provider) => ProviderBuildStatus::Ready(Box::new(provider)),
        Err(_) => ProviderBuildStatus::Unavailable,
    }
}

#[must_use]
pub(crate) fn render_drift_case(
    site: &'static OfficialUncensoredSite,
    config: &OfficialUncensoredProviderConfig,
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
        format!("{}-detail", site.provider_id),
        site.detail_url(config, av_number),
    )
    .with_selector("article, main, .movie-info, .detail, .info, h1, h2")
    .with_selector_timeout_ms(selector_timeout_ms)
    .with_rendered_page_defaults(&config.rendered_pages)
    .with_render_timeout_ms(render_timeout_ms)
    .with_min_text_bytes(100)
    .with_min_html_bytes(500)
}

#[derive(Clone, Debug)]
pub(crate) struct OfficialUncensoredMetadataProvider<T = ReqwestProviderHttpTransport>
where
    T: ProviderHttpTransport,
{
    site: &'static OfficialUncensoredSite,
    config: OfficialUncensoredProviderConfig,
    rendered_pages: RenderedPageRuntime<T>,
}

impl OfficialUncensoredMetadataProvider<ReqwestProviderHttpTransport> {
    pub(crate) fn new(
        site: &'static OfficialUncensoredSite,
        config: OfficialUncensoredProviderConfig,
    ) -> ProviderHttpResult<Self> {
        let rendered_pages = RenderedPageRuntime::new(config.rendered_pages.clone())?;
        Ok(Self {
            site,
            config,
            rendered_pages,
        })
    }
}

impl<T> OfficialUncensoredMetadataProvider<T>
where
    T: ProviderHttpTransport,
{
    #[cfg(test)]
    pub(crate) fn with_runtime(
        site: &'static OfficialUncensoredSite,
        config: OfficialUncensoredProviderConfig,
        runtime: ProviderHttpRuntime<T>,
    ) -> Self {
        let rendered_pages =
            RenderedPageRuntime::with_runtime(config.rendered_pages.clone(), runtime);
        Self {
            site,
            config,
            rendered_pages,
        }
    }

    async fn suggest_candidates(
        &self,
        query: &MetadataQuery,
    ) -> anyhow::Result<Vec<ProviderMetadataCandidate>> {
        if let Some(url) =
            rendered_av::direct_external_id(query, self.site.url_external_id_provider)
        {
            return self
                .detail_candidates(
                    &rendered_av::absolute_url(&self.config.base_url, &url),
                    query,
                )
                .await;
        }

        if let Some(id) = rendered_av::direct_external_id(query, self.site.provider_id)
            .and_then(|value| normalize_official_uncensored_number(&value))
        {
            return self
                .detail_candidates(&self.site.detail_url(&self.config, &id), query)
                .await;
        }

        let Some(av) = facts_from_query(query) else {
            return Ok(Vec::new());
        };
        if av.route != AvNumberRoute::Uncensored {
            return Ok(Vec::new());
        }

        self.detail_candidates(&self.site.detail_url(&self.config, &av.number), query)
            .await
    }

    async fn detail_candidates(
        &self,
        detail_url: &str,
        query: &MetadataQuery,
    ) -> anyhow::Result<Vec<ProviderMetadataCandidate>> {
        let page = self.render(detail_url.to_owned()).await?;
        Ok(parse_detail_page(&page.html, detail_url, self.site)
            .map(|facts| vec![facts.into_candidate(query)])
            .unwrap_or_default())
    }

    async fn render(&self, url: String) -> anyhow::Result<RenderedHtmlPage> {
        let intent = self
            .config
            .rendered_pages
            .intent(&self.config.render_path, url);
        self.rendered_pages
            .render_html(self.site.provider_id, "render page", intent)
            .await
    }
}

#[async_trait]
impl<T> MetadataProvider for OfficialUncensoredMetadataProvider<T>
where
    T: ProviderHttpTransport,
{
    fn id(&self) -> ProviderId {
        self.site.provider_id_enum
    }

    fn supports_av_route(&self, route: AvNumberRoute) -> bool {
        route == AvNumberRoute::Uncensored
    }

    async fn suggest(
        &self,
        query: &MetadataQuery,
    ) -> anyhow::Result<Vec<ProviderMetadataCandidate>> {
        self.suggest_candidates(query).await
    }
}

#[derive(Clone, Debug)]
struct OfficialUncensoredDetailFacts {
    site: &'static OfficialUncensoredSite,
    site_number: String,
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
    poster_url: Option<String>,
    backdrop_urls: Vec<String>,
    trailer_url: Option<String>,
}

impl OfficialUncensoredDetailFacts {
    fn into_candidate(self, query: &MetadataQuery) -> ProviderMetadataCandidate {
        let mut tags = vec![
            self.site.provider_id.to_owned(),
            format!("av_number:{}", self.av.number),
            "av_route:uncensored".to_owned(),
        ];
        tags.extend(self.actors.iter().map(|actor| format!("actor:{actor}")));
        tags.extend(self.tags.iter().map(|tag| format!("tag:{tag}")));
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
            artwork_candidates.push(official_uncensored_artwork_candidate(
                self.site,
                &self.site_number,
                AddonArtworkKind::Poster,
                poster_url,
                0,
            ));
        }
        for (index, url) in self.backdrop_urls.iter().cloned().enumerate() {
            artwork_candidates.push(official_uncensored_artwork_candidate(
                self.site,
                &self.site_number,
                AddonArtworkKind::Backdrop,
                url,
                index + 1,
            ));
        }

        ProviderMetadataCandidate {
            provider: self.site.provider_id.to_owned(),
            provider_id: format!("{}:movie:{}", self.site.provider_id, self.site_number),
            patch: AddonMetadataPatch {
                title: Some(self.title.clone()),
                original_title: None,
                sort_title: Some(self.title.clone()),
                overview: self.overview.clone(),
                release_date: self.release_date.clone(),
                runtime_minutes: self.runtime_minutes,
                tagline: Some(self.site.tagline.to_owned()),
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
                    thumb_url: self.poster_url.clone(),
                    trailer_url: self.trailer_url.clone(),
                    extrafanart_urls: self.backdrop_urls.clone(),
                    ..AvMetadataFacts::default()
                }
                .non_empty(),
                community_score_milli: None,
                community_vote_count: None,
                external_ids: vec![
                    ProviderExternalId {
                        provider: self.site.provider_id.to_owned(),
                        value: self.site_number,
                    },
                    ProviderExternalId {
                        provider: self.site.url_external_id_provider.to_owned(),
                        value: self.url,
                    },
                    ProviderExternalId {
                        provider: AV_NUMBER_EXTERNAL_ID_PROVIDER.to_owned(),
                        value: self.av.number,
                    },
                ],
                provider_outcomes: vec![self.site.outcome],
                provider_note: None,
            },
            artwork_candidates,
        }
    }
}

fn parse_detail_page(
    html: &str,
    detail_url: &str,
    site: &'static OfficialUncensoredSite,
) -> Option<OfficialUncensoredDetailFacts> {
    let document = Html::parse_document(html);
    let body_text = rendered_av::element_text(&document, "body").unwrap_or_default();
    let site_number = site
        .url_number(detail_url)
        .or_else(|| normalize_official_uncensored_number(&body_text))?;
    let av = facts_from_text(&site_number, AvNumberSource::ExternalId)?;
    if av.route != AvNumberRoute::Uncensored {
        return None;
    }
    let title = rendered_av::first_non_empty(&[
        rendered_av::element_text(&document, "h1, h2, .movie-title, .title").as_deref(),
        rendered_av::attr_value(&document, "meta[property=\"og:title\"]", "content").as_deref(),
        Some(av.number.as_str()),
    ])
    .map(|title| strip_number_prefix(&title, &site_number))?;
    let info_text = rendered_av::element_text(
        &document,
        "article, main, .movie-info, .detail, .info, body",
    )
    .unwrap_or_else(|| body_text.clone());
    let overview = rendered_av::first_non_empty(&[
        rendered_av::element_text(&document, ".description, .outline, .summary, .comment")
            .as_deref(),
        rendered_av::attr_value(&document, "meta[name=\"description\"]", "content").as_deref(),
    ]);
    let release_date = official_labeled_value(
        &document,
        &info_text,
        &["配信開始日", "発売日", "Release Date", "公開日"],
    )
    .or_else(|| rendered_av::first_iso_date(&body_text));
    let release_year = release_date.as_deref().and_then(rendered_av::first_year);
    let runtime_minutes = official_labeled_value(
        &document,
        &info_text,
        &["収録時間", "再生時間", "Runtime", "Duration"],
    )
    .and_then(|value| rendered_av::parse_minutes(&value));
    let actors = non_empty_or_links(
        official_labeled_value(
            &document,
            &info_text,
            &["出演", "出演者", "女優", "Actor", "Actress"],
        ),
        rendered_av::link_texts(
            &document,
            "a[href*=\"actor\"], a[href*=\"actress\"], a[href*=\"star\"], a[href*=\"model\"]",
        ),
    );
    let mut tags = rendered_av::link_texts(
        &document,
        "a[href*=\"tag\"], a[href*=\"genre\"], a[href*=\"category\"]",
    );
    for tag in split_label_values(official_labeled_value(
        &document,
        &info_text,
        &["ジャンル", "Genre", "Tag"],
    )) {
        push_unique(&mut tags, tag);
    }
    let maker = official_labeled_value(&document, &info_text, &["メーカー", "Maker", "Studio"]);
    let label = official_labeled_value(&document, &info_text, &["レーベル", "Label", "Publisher"]);
    let series = official_labeled_value(&document, &info_text, &["シリーズ", "Series"]);
    let director = official_labeled_value(&document, &info_text, &["監督", "Director"]);
    let poster_url = rendered_av::attr_value(&document, "meta[property=\"og:image\"]", "content")
        .or_else(|| {
            rendered_av::attr_value(
                &document,
                ".poster img, .package img, .movie-image img, article img",
                "src",
            )
        })
        .map(|url| rendered_av::absolute_url(detail_url, &url));
    let mut backdrop_urls = rendered_av::image_urls(
        &document,
        ".sample img, .gallery img, .sample a[href], .gallery a[href]",
        detail_url,
    );
    if let Some(poster_url) = &poster_url {
        backdrop_urls.retain(|url| url != poster_url);
    }
    backdrop_urls.retain(|url| !looks_like_video_url(url));
    let trailer_url = first_video_url(&document, detail_url);

    Some(OfficialUncensoredDetailFacts {
        site,
        site_number: site.site_number(&av.number),
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
        poster_url,
        backdrop_urls,
        trailer_url,
    })
}

const OFFICIAL_UNCENSORED_LABELS: &[&str] = &[
    "配信開始日",
    "発売日",
    "Release Date",
    "公開日",
    "収録時間",
    "再生時間",
    "Runtime",
    "Duration",
    "出演",
    "出演者",
    "女優",
    "Actor",
    "Actress",
    "ジャンル",
    "Genre",
    "Tag",
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
];

fn official_labeled_value(document: &Html, info_text: &str, labels: &[&str]) -> Option<String> {
    rendered_av::structured_or_labeled_value(
        document,
        OFFICIAL_UNCENSORED_LABEL_ROW_SELECTOR,
        info_text,
        labels,
        OFFICIAL_UNCENSORED_LABELS,
    )
}

const OFFICIAL_UNCENSORED_LABEL_ROW_SELECTOR: &str = ".movie-info p, .movie-info li, .movie-info tr, \
         .detail p, .detail li, .detail tr, \
         .info p, .info li, .info tr, \
         article p, article li, article tr, main p, main li, main tr";

fn normalize_official_uncensored_number(value: &str) -> Option<String> {
    facts_from_text(value, AvNumberSource::ExternalId)
        .filter(|facts| facts.route == AvNumberRoute::Uncensored)
        .map(|facts| facts.number)
}

fn strip_number_prefix(title: &str, site_number: &str) -> String {
    let title = rendered_av::normalize_whitespace(title);
    for prefix in [
        site_number.to_owned(),
        site_number.replace('-', "_"),
        site_number.replace('_', "-"),
    ] {
        let stripped = title
            .strip_prefix(&prefix)
            .map(str::trim)
            .map(|value| value.trim_start_matches(['-', ':', '：']).trim());
        if let Some(stripped) = stripped.filter(|value| !value.is_empty()) {
            return stripped.to_owned();
        }
    }
    title
}

fn non_empty_or_links(label_value: Option<String>, links: Vec<String>) -> Vec<String> {
    let mut values = split_label_values(label_value);
    for link in links {
        push_unique(&mut values, link);
    }
    values
}

fn split_label_values(value: Option<String>) -> Vec<String> {
    value
        .into_iter()
        .flat_map(|value| {
            value
                .split([',', '/', '、', '，'])
                .map(rendered_av::normalize_whitespace)
                .filter(|value| !value.is_empty())
                .collect::<Vec<_>>()
        })
        .collect()
}

fn push_unique(values: &mut Vec<String>, value: String) {
    if !value.trim().is_empty() && !values.iter().any(|existing| existing == &value) {
        values.push(value);
    }
}

fn first_video_url(document: &Html, detail_url: &str) -> Option<String> {
    let selector = Selector::parse("video source, source[type*=\"video\"], a[href]").ok()?;
    document.select(&selector).find_map(|element| {
        let value = element
            .value()
            .attr("src")
            .or_else(|| element.value().attr("href"))?
            .trim();
        looks_like_video_url(value).then(|| rendered_av::absolute_url(detail_url, value))
    })
}

fn looks_like_video_url(value: &str) -> bool {
    let normalized = value.to_ascii_lowercase();
    normalized.contains("trailer")
        || normalized.contains("movie")
        || normalized.contains("video")
        || normalized.ends_with(".mp4")
        || normalized.ends_with(".m3u8")
        || normalized.ends_with(".webm")
}

fn official_uncensored_artwork_candidate(
    site: &OfficialUncensoredSite,
    site_number: &str,
    kind: AddonArtworkKind,
    source_url: String,
    index: usize,
) -> ProviderArtworkCandidate {
    ProviderArtworkCandidate {
        provider: site.provider_id.to_owned(),
        provider_id: format!("{}:movie:{site_number}:artwork:{index}", site.provider_id),
        facts: ProviderArtworkCandidateFacts {
            kind,
            source_url,
            language: None,
            width: None,
            height: None,
        },
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        engine::{MetadataQuery, QueryExternalId},
        providers::{
            caribbean::CARIBBEAN_SITE,
            http_runtime::{ProviderHttpRuntime, ProviderHttpRuntimeConfig},
            onepondo::ONEPONDO_SITE,
            rendered_av_fixture::{RenderedAvFixtureTransport, request_json_body},
            tenmusume::TENMUSUME_SITE,
        },
    };

    use super::*;

    #[tokio::test]
    async fn caribbean_provider_uses_browser_worker_render_contract_for_uncensored_detail() {
        let transport = RenderedAvFixtureTransport::new(CARIBBEAN_SITE.provider_id);
        transport.push_rendered_html(
            "https://caribbean.example/moviepages/010116-001/index.html",
            "010116-001 Official Uncensored Title",
            &detail_html("010116-001 Official Uncensored Title"),
        );
        let provider = provider_with_transport(
            &CARIBBEAN_SITE,
            "https://caribbean.example",
            transport.clone(),
        );

        let candidates = provider
            .suggest(&MetadataQuery::from_payload(
                &serde_json::json!({"file_name": "010116-001.mp4"}),
                "ja-JP",
            ))
            .await
            .unwrap();

        assert_official_candidate(
            &candidates[0],
            "caribbean",
            "caribbean:movie:010116-001",
            "010116_001",
            "https://caribbean.example/moviepages/010116-001/index.html",
            ProviderOutcome::CaribbeanRenderedHtmlParsed,
        );
        let body = request_json_body(&transport.requests()[0]);
        assert_eq!(
            body["url"],
            "https://caribbean.example/moviepages/010116-001/index.html"
        );
    }

    #[tokio::test]
    async fn official_1pondo_provider_uses_browser_worker_render_contract_for_uncensored_detail() {
        let transport = RenderedAvFixtureTransport::new(ONEPONDO_SITE.provider_id);
        transport.push_rendered_html(
            "https://1pondo.example/movies/010116_001/",
            "010116_001 Official Uncensored Title",
            &detail_html("010116_001 Official Uncensored Title"),
        );
        let provider =
            provider_with_transport(&ONEPONDO_SITE, "https://1pondo.example", transport.clone());

        let candidates = provider
            .suggest(&MetadataQuery::from_payload(
                &serde_json::json!({"file_name": "010116_001.mp4"}),
                "ja-JP",
            ))
            .await
            .unwrap();

        assert_official_candidate(
            &candidates[0],
            "1pondo",
            "1pondo:movie:010116_001",
            "010116_001",
            "https://1pondo.example/movies/010116_001/",
            ProviderOutcome::OnePondoRenderedHtmlParsed,
        );
        let body = request_json_body(&transport.requests()[0]);
        assert_eq!(body["url"], "https://1pondo.example/movies/010116_001/");
    }

    #[tokio::test]
    async fn official_10musume_provider_supports_explicit_url_lookup() {
        let transport = RenderedAvFixtureTransport::new(TENMUSUME_SITE.provider_id);
        transport.push_rendered_html(
            "https://mirror.example/movies/010116_01/index.html",
            "010116_01 Official Uncensored Title",
            &detail_html("010116_01 Official Uncensored Title"),
        );
        let provider = provider_with_transport(
            &TENMUSUME_SITE,
            "https://10musume.example",
            transport.clone(),
        );

        let candidates = provider
            .suggest(&MetadataQuery {
                title: "Manual lookup".to_owned(),
                year: None,
                language: "ja-JP".to_owned(),
                external_ids: vec![QueryExternalId {
                    provider: "10musume_url".to_owned(),
                    value: "https://mirror.example/movies/010116_01/index.html".to_owned(),
                }],
            })
            .await
            .unwrap();

        assert_official_candidate(
            &candidates[0],
            "10musume",
            "10musume:movie:010116_01",
            "010116_01",
            "https://mirror.example/movies/010116_01/index.html",
            ProviderOutcome::TenMusumeRenderedHtmlParsed,
        );
        let body = request_json_body(&transport.requests()[0]);
        assert_eq!(
            body["url"],
            "https://mirror.example/movies/010116_01/index.html"
        );
    }

    #[tokio::test]
    async fn official_uncensored_providers_skip_censored_and_fc2_routes() {
        let transport = RenderedAvFixtureTransport::new(ONEPONDO_SITE.provider_id);
        let provider =
            provider_with_transport(&ONEPONDO_SITE, "https://1pondo.example", transport.clone());

        let candidates = provider
            .suggest(&MetadataQuery::from_payload(
                &serde_json::json!({"file_name": "SSNI-00644.mp4"}),
                "ja-JP",
            ))
            .await
            .unwrap();

        assert!(candidates.is_empty());
        assert!(transport.requests().is_empty());
        assert!(provider.supports_av_route(AvNumberRoute::Uncensored));
        assert!(!provider.supports_av_route(AvNumberRoute::Censored));
        assert!(!provider.supports_av_route(AvNumberRoute::Fc2));
    }

    fn provider_with_transport(
        site: &'static OfficialUncensoredSite,
        base_url: &str,
        transport: RenderedAvFixtureTransport,
    ) -> OfficialUncensoredMetadataProvider<RenderedAvFixtureTransport> {
        let runtime = ProviderHttpRuntime::with_transport(
            ProviderHttpRuntimeConfig {
                retry_backoff_ms: 0,
                ..ProviderHttpRuntimeConfig::default()
            },
            transport,
        );
        OfficialUncensoredMetadataProvider::with_runtime(
            site,
            OfficialUncensoredProviderConfig::new(
                base_url.to_owned(),
                "http://browser-worker.example".to_owned(),
                "/render".to_owned(),
                10_000,
            ),
            runtime,
        )
    }

    fn assert_official_candidate(
        candidate: &ProviderMetadataCandidate,
        provider_id: &str,
        provider_candidate_id: &str,
        av_number: &str,
        detail_url: &str,
        outcome: ProviderOutcome,
    ) {
        assert_eq!(candidate.provider, provider_id);
        assert_eq!(candidate.provider_id, provider_candidate_id);
        assert_eq!(
            candidate.patch.title.as_deref(),
            Some("Official Uncensored Title")
        );
        assert_eq!(
            candidate.patch.overview.as_deref(),
            Some("Official uncensored outline.")
        );
        assert_eq!(candidate.patch.release_date.as_deref(), Some("2024-07-01"));
        assert_eq!(candidate.patch.runtime_minutes, Some(98));
        assert_eq!(
            candidate.patch.genres.as_deref(),
            Some(["Drama".to_owned(), "HD".to_owned()].as_slice())
        );
        let av = candidate.facts.av.as_ref().unwrap();
        assert_eq!(av.actors, vec!["Actress One", "Actress Two"]);
        assert_eq!(av.directors, vec!["Director One"]);
        assert_eq!(av.series.as_deref(), Some("Series One"));
        assert_eq!(av.studio.as_deref(), Some("Official Maker"));
        assert_eq!(av.publisher.as_deref(), Some("Official Label"));
        assert_eq!(
            av.thumb_url.as_deref(),
            Some("https://img.example/poster.jpg")
        );
        assert_eq!(
            av.trailer_url.as_deref(),
            Some("https://video.example/trailer.mp4")
        );
        assert_eq!(
            av.extrafanart_urls,
            vec![
                "https://img.example/sample1.jpg".to_owned(),
                "https://img.example/sample2.jpg".to_owned(),
            ]
        );
        assert_eq!(candidate.artwork_candidates.len(), 3);
        assert!(
            candidate.facts.external_ids.contains(&ProviderExternalId {
                provider: provider_id.to_owned(),
                value: provider_candidate_id
                    .strip_prefix(&format!("{provider_id}:movie:"))
                    .unwrap()
                    .to_owned(),
            })
        );
        assert!(candidate.facts.external_ids.contains(&ProviderExternalId {
            provider: format!("{provider_id}_url"),
            value: detail_url.to_owned(),
        }));
        assert!(candidate.facts.external_ids.contains(&ProviderExternalId {
            provider: "av_number".to_owned(),
            value: av_number.to_owned(),
        }));
        assert_eq!(candidate.facts.provider_outcomes, vec![outcome]);
    }

    fn detail_html(title: &str) -> String {
        format!(
            r#"
<!doctype html>
<html>
<head>
  <meta property="og:image" content="https://img.example/poster.jpg">
  <meta name="description" content="Official uncensored outline.">
</head>
<body>
  <main>
    <h1>{title}</h1>
    <div class="movie-info">
      <p>配信開始日：<span>2024-07-01</span></p>
      <p>収録時間：<span>98分</span></p>
      <p>出演：<span><a href="/actor/one">Actress One</a>, <a href="/actor/two">Actress Two</a></span></p>
      <p>ジャンル：<span><a href="/genre/drama">Drama</a>, <a href="/genre/hd">HD</a></span></p>
      <p>メーカー：<span>Official Maker</span></p>
      <p>レーベル：<span>Official Label</span></p>
      <p>シリーズ：<span>Series One</span></p>
      <p>監督：<span>Director One</span></p>
    </div>
    <section class="description">Official uncensored outline.</section>
    <div class="sample">
      <img src="https://img.example/sample1.jpg">
      <a href="https://img.example/sample2.jpg">sample</a>
    </div>
    <a href="https://video.example/trailer.mp4">Trailer</a>
  </main>
</body>
</html>"#
        )
    }
}
