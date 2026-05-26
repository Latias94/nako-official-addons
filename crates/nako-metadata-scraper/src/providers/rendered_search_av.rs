use async_trait::async_trait;
use nako_addon_protocol::{AddonArtworkKind, AddonMetadataPatch};
use scraper::{ElementRef, Html, Selector};

#[cfg(test)]
use crate::providers::http_runtime::ProviderHttpRuntime;
use crate::{
    Config,
    config::{ProviderConfig, ProviderId, non_empty_trimmed},
    engine::{
        AvMetadataFacts, MetadataQuery, ProviderArtworkCandidate, ProviderArtworkCandidateFacts,
        ProviderCandidateFacts, ProviderExternalId, ProviderExternalIdCapability,
        ProviderFieldQualityDescriptor, ProviderMetadataCandidate, ProviderOutcome,
        av::{
            AV_NUMBER_EXTERNAL_ID_PROVIDER, AvNumberRoute, AvNumberSource, AvQueryFacts,
            facts_from_query, facts_from_text,
        },
    },
    providers::{
        MetadataProvider, ProviderBuildStatus, ProviderConfigInput,
        http_runtime::{ProviderHttpResult, ProviderHttpTransport, ReqwestProviderHttpTransport},
        registry::ProviderCatalogEntry,
        rendered_av,
        rendered_page::{RenderedHtmlPage, RenderedPageRuntime, RenderedPageSupportConfig},
    },
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RenderedSearchAvProviderConfig {
    pub(crate) base_url: String,
    pub(crate) rendered_pages: RenderedPageSupportConfig,
    pub(crate) render_path: String,
}

impl RenderedSearchAvProviderConfig {
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
pub(crate) struct RenderedSearchAvSite {
    pub(crate) provider_id: &'static str,
    pub(crate) url_external_id_provider: &'static str,
    pub(crate) provider_id_enum: ProviderId,
    pub(crate) default_base_url: &'static str,
    pub(crate) base_url_env_var: &'static str,
    pub(crate) timeout_env_var: &'static str,
    pub(crate) enabled_env_var: &'static str,
    pub(crate) capabilities: &'static [&'static str],
    pub(crate) field_quality: ProviderFieldQualityDescriptor,
    pub(crate) search_url: RenderedSearchAvSearchUrl,
    pub(crate) supported_routes: &'static [AvNumberRoute],
    pub(crate) outcome: ProviderOutcome,
    pub(crate) tagline: &'static str,
}

impl RenderedSearchAvSite {
    fn supports_route(self, route: AvNumberRoute) -> bool {
        self.supported_routes.contains(&route)
    }

    fn search_url(self, config: &RenderedSearchAvProviderConfig, av: &AvQueryFacts) -> String {
        self.search_url.build(&config.base_url, av)
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum RenderedSearchAvSearchUrl {
    Query {
        path: &'static str,
        param: &'static str,
        compact_number: bool,
    },
    Path {
        prefix: &'static str,
        compact_number: bool,
    },
}

impl RenderedSearchAvSearchUrl {
    fn build(self, base_url: &str, av: &AvQueryFacts) -> String {
        let number = if self.compact_number() {
            compact_av_number(&av.number)
        } else {
            av.number.clone()
        };
        let encoded = rendered_av::percent_encode(&number);
        match self {
            Self::Query { path, param, .. } => {
                let url = format!("{}{}", base_url.trim_end_matches('/'), path);
                let separator = if url.contains('?') { "&" } else { "?" };
                format!("{url}{separator}{param}={encoded}")
            }
            Self::Path { prefix, .. } => {
                format!("{}{}{}", base_url.trim_end_matches('/'), prefix, encoded)
            }
        }
    }

    const fn compact_number(self) -> bool {
        match self {
            Self::Query { compact_number, .. } | Self::Path { compact_number, .. } => {
                compact_number
            }
        }
    }
}

#[must_use]
pub(crate) fn catalog_entry(
    site: &'static RenderedSearchAvSite,
    external_id_capabilities: &'static [ProviderExternalIdCapability],
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
    site: &'static RenderedSearchAvSite,
    make_config: fn(bool, RenderedSearchAvProviderConfig) -> ProviderConfig,
) -> ProviderConfig {
    let lookup = input.lookup;
    make_config(
        input.enabled,
        RenderedSearchAvProviderConfig::from_env_lookup(
            |name| lookup(name),
            site.base_url_env_var,
            site.timeout_env_var,
            site.default_base_url,
        ),
    )
}

pub(crate) fn build_provider(
    config: &Config,
    site: &'static RenderedSearchAvSite,
    get_config: fn(&ProviderConfig) -> Option<&RenderedSearchAvProviderConfig>,
) -> ProviderBuildStatus {
    let Some(provider_config) = config
        .provider_config(site.provider_id_enum)
        .and_then(get_config)
        .cloned()
    else {
        return ProviderBuildStatus::Unavailable;
    };
    match RenderedSearchAvMetadataProvider::new(site, provider_config) {
        Ok(provider) => ProviderBuildStatus::Ready(Box::new(provider)),
        Err(_) => ProviderBuildStatus::Unavailable,
    }
}

#[derive(Clone, Debug)]
pub(crate) struct RenderedSearchAvMetadataProvider<T = ReqwestProviderHttpTransport>
where
    T: ProviderHttpTransport,
{
    site: &'static RenderedSearchAvSite,
    config: RenderedSearchAvProviderConfig,
    rendered_pages: RenderedPageRuntime<T>,
}

impl RenderedSearchAvMetadataProvider<ReqwestProviderHttpTransport> {
    pub(crate) fn new(
        site: &'static RenderedSearchAvSite,
        config: RenderedSearchAvProviderConfig,
    ) -> ProviderHttpResult<Self> {
        let rendered_pages = RenderedPageRuntime::new(config.rendered_pages.clone())?;
        Ok(Self {
            site,
            config,
            rendered_pages,
        })
    }
}

impl<T> RenderedSearchAvMetadataProvider<T>
where
    T: ProviderHttpTransport,
{
    #[cfg(test)]
    pub(crate) fn with_runtime(
        site: &'static RenderedSearchAvSite,
        config: RenderedSearchAvProviderConfig,
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
                    facts_from_query(query),
                    query,
                )
                .await;
        }

        if let Some(value) = rendered_av::direct_external_id(query, self.site.provider_id) {
            if looks_like_url_or_path(&value) {
                return self
                    .detail_candidates(
                        &rendered_av::absolute_url(&self.config.base_url, &value),
                        facts_from_query(query),
                        query,
                    )
                    .await;
            }

            if let Some(av) = facts_from_text(&value, AvNumberSource::ExternalId) {
                return self.search_and_detail_candidates(av, query).await;
            }
        }

        let Some(av) = facts_from_query(query) else {
            return Ok(Vec::new());
        };
        self.search_and_detail_candidates(av, query).await
    }

    async fn search_and_detail_candidates(
        &self,
        av: AvQueryFacts,
        query: &MetadataQuery,
    ) -> anyhow::Result<Vec<ProviderMetadataCandidate>> {
        if !self.site.supports_route(av.route) {
            return Ok(Vec::new());
        }

        let search = self.render(self.site.search_url(&self.config, &av)).await?;
        let Some(result) = parse_search_results(&search.html, &av, &self.config.base_url)
            .into_iter()
            .next()
        else {
            return Ok(Vec::new());
        };

        self.detail_candidates(&result.url, Some(av), query).await
    }

    async fn detail_candidates(
        &self,
        detail_url: &str,
        av: Option<AvQueryFacts>,
        query: &MetadataQuery,
    ) -> anyhow::Result<Vec<ProviderMetadataCandidate>> {
        let page = self.render(detail_url.to_owned()).await?;
        Ok(parse_detail_page(&page.html, detail_url, av, self.site)
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
impl<T> MetadataProvider for RenderedSearchAvMetadataProvider<T>
where
    T: ProviderHttpTransport,
{
    fn id(&self) -> ProviderId {
        self.site.provider_id_enum
    }

    fn supports_av_route(&self, route: AvNumberRoute) -> bool {
        self.site.supports_route(route)
    }

    async fn suggest(
        &self,
        query: &MetadataQuery,
    ) -> anyhow::Result<Vec<ProviderMetadataCandidate>> {
        self.suggest_candidates(query).await
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RenderedSearchResult {
    url: String,
}

#[derive(Clone, Debug)]
struct RenderedSearchAvDetailFacts {
    site: &'static RenderedSearchAvSite,
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
    wanted_count: Option<u32>,
    poster_url: Option<String>,
    backdrop_urls: Vec<String>,
    trailer_url: Option<String>,
}

impl RenderedSearchAvDetailFacts {
    fn into_candidate(self, query: &MetadataQuery) -> ProviderMetadataCandidate {
        let mut tags = vec![
            self.site.provider_id.to_owned(),
            format!("av_number:{}", self.av.number),
            format!("av_route:{:?}", self.av.route).to_ascii_lowercase(),
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
            artwork_candidates.push(rendered_search_av_artwork_candidate(
                self.site,
                &self.id,
                AddonArtworkKind::Poster,
                poster_url,
                0,
            ));
        }
        for (index, url) in self.backdrop_urls.iter().cloned().enumerate() {
            artwork_candidates.push(rendered_search_av_artwork_candidate(
                self.site,
                &self.id,
                AddonArtworkKind::Backdrop,
                url,
                index + 1,
            ));
        }

        ProviderMetadataCandidate {
            provider: self.site.provider_id.to_owned(),
            provider_id: format!("{}:movie:{}", self.site.provider_id, self.id),
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
                    wanted_count: self.wanted_count,
                    thumb_url: self.poster_url.clone(),
                    trailer_url: self.trailer_url.clone(),
                    extrafanart_urls: self.backdrop_urls.clone(),
                }
                .non_empty(),
                community_score_milli: self.rating_milli,
                community_vote_count: self.wanted_count,
                external_ids: vec![
                    ProviderExternalId {
                        provider: self.site.provider_id.to_owned(),
                        value: self.id,
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

fn parse_search_results(
    html: &str,
    av: &AvQueryFacts,
    base_url: &str,
) -> Vec<RenderedSearchResult> {
    let document = Html::parse_document(html);
    let Ok(selector) =
        Selector::parse("a[href], .item a[href], .video-item a[href], table a[href]")
    else {
        return Vec::new();
    };
    let mut results = Vec::new();

    for element in document.select(&selector) {
        let href = element.value().attr("href").unwrap_or_default().trim();
        if href.is_empty() || href.starts_with('#') || href.starts_with("javascript:") {
            continue;
        }
        if href.contains("/search") || href.contains("result_published") {
            continue;
        }

        let text = search_result_text(element);
        if !rendered_av::text_or_url_matches_av(&text, href, av) {
            continue;
        }
        results.push(RenderedSearchResult {
            url: rendered_av::absolute_url(base_url, href),
        });
    }

    dedupe_search_results(results)
}

fn parse_detail_page(
    html: &str,
    detail_url: &str,
    av: Option<AvQueryFacts>,
    site: &'static RenderedSearchAvSite,
) -> Option<RenderedSearchAvDetailFacts> {
    let document = Html::parse_document(html);
    let body_text = rendered_av::element_text(&document, "body").unwrap_or_default();
    let info_text = rendered_av::element_text(
        &document,
        "article, main, .movie, .video, .detail, .info, .movie-info, table, body",
    )
    .unwrap_or_else(|| body_text.clone());
    let title = rendered_av::first_non_empty(&[
        rendered_av::element_text(&document, "h1, h2, h3, .title, .movie-title, .video-title")
            .as_deref(),
        rendered_av::attr_value(&document, "meta[property=\"og:title\"]", "content").as_deref(),
        rendered_av::element_text(&document, "title").as_deref(),
        av.as_ref().map(|av| av.number.as_str()),
    ])?;
    let parsed_av = rendered_search_av_labeled_value(
        &document,
        &info_text,
        &["Number", "ID", "品番", "識別碼", "识别码", "番號", "番号"],
    )
    .and_then(|value| facts_from_text(&value, AvNumberSource::ExternalId))
    .or_else(|| facts_from_text(&title, AvNumberSource::ExternalId))
    .or_else(|| facts_from_text(detail_url, AvNumberSource::ExternalId))
    .or_else(|| facts_from_text(&body_text, AvNumberSource::ExternalId))
    .or(av)?;

    let release_date = rendered_search_av_labeled_value(
        &document,
        &info_text,
        &[
            "Release Date",
            "Released",
            "発売日",
            "發行日期",
            "发行日期",
            "公開日",
            "配信開始日",
        ],
    )
    .or_else(|| rendered_av::first_iso_date(&body_text));
    let release_year = release_date.as_deref().and_then(rendered_av::first_year);
    let runtime_minutes = rendered_search_av_labeled_value(
        &document,
        &info_text,
        &[
            "Runtime",
            "Duration",
            "Length",
            "収録時間",
            "再生時間",
            "長度",
            "长度",
        ],
    )
    .and_then(|value| rendered_av::parse_minutes(&value));
    let overview = rendered_av::first_non_empty(&[
        rendered_av::element_text(
            &document,
            ".description, .outline, .summary, .story, .introduction, .comment, .box-description",
        )
        .as_deref(),
        rendered_av::attr_value(&document, "meta[name=\"description\"]", "content").as_deref(),
    ]);
    let actors = non_empty_or_links(
        rendered_search_av_labeled_value(
            &document,
            &info_text,
            &[
                "Actor",
                "Actors",
                "Actress",
                "Cast",
                "出演",
                "出演者",
                "女優",
            ],
        ),
        rendered_av::link_texts(
            &document,
            "a[href*=\"actor\"], a[href*=\"actress\"], a[href*=\"star\"], a[href*=\"cast\"], a[href*=\"idol\"]",
        ),
    );
    let mut tags = rendered_av::link_texts(
        &document,
        "a[href*=\"genre\"], a[href*=\"tag\"], a[href*=\"category\"], .tag a, .genre a",
    );
    for tag in split_label_values(rendered_search_av_labeled_value(
        &document,
        &info_text,
        &[
            "Genre",
            "Genres",
            "Tag",
            "Category",
            "ジャンル",
            "類別",
            "类别",
        ],
    )) {
        push_unique(&mut tags, tag);
    }
    let maker = rendered_search_av_labeled_value(
        &document,
        &info_text,
        &[
            "Maker",
            "Studio",
            "Manufacturer",
            "メーカー",
            "製作商",
            "制作商",
        ],
    );
    let label = rendered_search_av_labeled_value(
        &document,
        &info_text,
        &["Label", "Publisher", "レーベル", "發行商", "发行商"],
    );
    let series =
        rendered_search_av_labeled_value(&document, &info_text, &["Series", "シリーズ", "系列"]);
    let director = rendered_search_av_labeled_value(
        &document,
        &info_text,
        &["Director", "監督", "導演", "导演"],
    );
    let rating_milli = rendered_av::element_text(&document, ".score, .rating, .review")
        .or_else(|| {
            rendered_search_av_labeled_value(&document, &info_text, &["Rating", "Score", "評価"])
        })
        .and_then(|value| rendered_av::parse_rating_milli(&value));
    let wanted_count = rendered_search_av_labeled_value(
        &document,
        &info_text,
        &["Wanted", "Want", "Favorites", "想看"],
    )
    .and_then(|value| rendered_av::first_u32(&value));
    let poster_url = rendered_av::attr_value(&document, "meta[property=\"og:image\"]", "content")
        .or_else(|| {
            rendered_av::attr_value(
                &document,
                ".poster img, .cover img, .package img, .movie-image img, .main-image img, article img, main img",
                "src",
            )
        })
        .map(|url| rendered_av::absolute_url(detail_url, &url));
    let mut backdrop_urls = rendered_av::image_urls(
        &document,
        ".sample img, .gallery img, .preview img, .sample-box img, a[href*=\"sample\"], a[href*=\"gallery\"]",
        detail_url,
    );
    if let Some(poster_url) = &poster_url {
        backdrop_urls.retain(|url| url != poster_url);
    }
    backdrop_urls.retain(|url| !looks_like_video_url(url));
    let trailer_url = first_video_url(&document, detail_url);

    Some(RenderedSearchAvDetailFacts {
        site,
        id: detail_id_from_url(detail_url).unwrap_or_else(|| parsed_av.number.clone()),
        url: detail_url.to_owned(),
        av: parsed_av,
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
        wanted_count,
        poster_url,
        backdrop_urls,
        trailer_url,
    })
}

const RENDERED_SEARCH_AV_LABELS: &[&str] = &[
    "Number",
    "ID",
    "品番",
    "識別碼",
    "识别码",
    "番號",
    "番号",
    "Release Date",
    "Released",
    "発売日",
    "發行日期",
    "发行日期",
    "公開日",
    "配信開始日",
    "Runtime",
    "Duration",
    "Length",
    "収録時間",
    "再生時間",
    "長度",
    "长度",
    "Actor",
    "Actors",
    "Actress",
    "Cast",
    "出演",
    "出演者",
    "女優",
    "Genre",
    "Genres",
    "Tag",
    "Category",
    "ジャンル",
    "類別",
    "类别",
    "Maker",
    "Studio",
    "Manufacturer",
    "メーカー",
    "製作商",
    "制作商",
    "Label",
    "Publisher",
    "レーベル",
    "發行商",
    "发行商",
    "Series",
    "シリーズ",
    "系列",
    "Director",
    "監督",
    "導演",
    "导演",
    "Rating",
    "Score",
    "評価",
    "Wanted",
    "Want",
    "Favorites",
    "想看",
];

const RENDERED_SEARCH_AV_LABEL_ROW_SELECTOR: &str = ".movie p, .movie li, .movie tr, \
         .video p, .video li, .video tr, \
         .detail p, .detail li, .detail tr, \
         .info p, .info li, .info tr, \
         .movie-info p, .movie-info li, .movie-info tr, \
         table tr, article p, article li, article tr, main p, main li, main tr";

fn rendered_search_av_labeled_value(
    document: &Html,
    info_text: &str,
    labels: &[&str],
) -> Option<String> {
    rendered_av::structured_or_labeled_value(
        document,
        RENDERED_SEARCH_AV_LABEL_ROW_SELECTOR,
        info_text,
        labels,
        RENDERED_SEARCH_AV_LABELS,
    )
}

fn rendered_search_av_artwork_candidate(
    site: &RenderedSearchAvSite,
    movie_id: &str,
    kind: AddonArtworkKind,
    source_url: String,
    index: usize,
) -> ProviderArtworkCandidate {
    ProviderArtworkCandidate {
        provider: site.provider_id.to_owned(),
        provider_id: format!("{}:movie:{movie_id}:artwork:{index}", site.provider_id),
        facts: ProviderArtworkCandidateFacts {
            kind,
            source_url,
            language: None,
            width: None,
            height: None,
        },
    }
}

fn search_result_text(element: ElementRef<'_>) -> String {
    let mut parts = vec![element.text().collect::<Vec<_>>().join(" ")];
    for attr in ["title", "alt", "data-title", "data-name"] {
        if let Some(value) = element.value().attr(attr) {
            parts.push(value.to_owned());
        }
    }
    if let Ok(selector) = Selector::parse("img") {
        for image in element.select(&selector) {
            for attr in ["alt", "title"] {
                if let Some(value) = image.value().attr(attr) {
                    parts.push(value.to_owned());
                }
            }
        }
    }
    rendered_av::normalize_whitespace(&parts.join(" "))
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

fn dedupe_search_results(results: Vec<RenderedSearchResult>) -> Vec<RenderedSearchResult> {
    results.into_iter().fold(Vec::new(), |mut values, result| {
        if !values.iter().any(|existing| existing.url == result.url) {
            values.push(result);
        }
        values
    })
}

fn compact_av_number(value: &str) -> String {
    value
        .chars()
        .filter(|character| !matches!(character, '-' | '_' | '.' | ' '))
        .collect()
}

fn detail_id_from_url(url: &str) -> Option<String> {
    if let Some(id) = rendered_av::id_query_value(url, "id") {
        return Some(id);
    }
    let without_fragment = url.split('#').next().unwrap_or(url);
    let without_query = without_fragment
        .split('?')
        .next()
        .unwrap_or(without_fragment);
    without_query
        .trim_end_matches('/')
        .rsplit('/')
        .find(|segment| !segment.is_empty() && *segment != "index.html" && *segment != "detail")
        .map(str::to_owned)
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
    let value = value.to_ascii_lowercase();
    value.ends_with(".mp4")
        || value.ends_with(".m3u8")
        || value.ends_with(".webm")
        || value.contains(".mp4?")
        || value.contains(".m3u8?")
        || value.contains(".webm?")
}

fn looks_like_url_or_path(value: &str) -> bool {
    let value = value.trim();
    value.starts_with("http://")
        || value.starts_with("https://")
        || value.starts_with('/')
        || value.contains('/')
        || value.contains('?')
}
