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
            facts_from_query, facts_from_text,
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

pub const FC2PPVDB_PROVIDER_ID: &str = "fc2ppvdb";
const FC2PPVDB_URL_EXTERNAL_ID_PROVIDER: &str = "fc2ppvdb_url";
const FC2PPVDB_EXTERNAL_ID_CAPABILITIES: &[ProviderExternalIdCapability] = &[
    ProviderExternalIdCapability::new(
        FC2PPVDB_PROVIDER_ID,
        ExternalIdValueKind::Opaque,
        true,
        true,
        &["fc2ppvdb_id"],
        false,
    ),
    ProviderExternalIdCapability::new(
        FC2PPVDB_URL_EXTERNAL_ID_PROVIDER,
        ExternalIdValueKind::Url,
        true,
        true,
        &["fc2ppvdb_url"],
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
pub struct Fc2ppvdbProviderConfig {
    pub(crate) base_url: String,
    pub(crate) rendered_pages: RenderedPageSupportConfig,
    pub(crate) render_path: String,
}

impl Fc2ppvdbProviderConfig {
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
        let base_url = lookup("NAKO_METADATA_SCRAPER_FC2PPVDB_BASE_URL")
            .and_then(non_empty_trimmed)
            .unwrap_or_else(|| "https://fc2ppvdb.com".to_owned());
        let browser_worker_base_url = lookup("NAKO_METADATA_SCRAPER_BROWSER_WORKER_BASE_URL")
            .and_then(non_empty_trimmed)
            .unwrap_or_else(|| "http://nako-browser-worker:3000".to_owned());
        let render_path = lookup("NAKO_METADATA_SCRAPER_BROWSER_WORKER_RENDER_PATH")
            .and_then(non_empty_trimmed)
            .unwrap_or_else(|| "/render".to_owned());
        let timeout_ms = lookup("NAKO_METADATA_SCRAPER_FC2PPVDB_TIMEOUT_MS")
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
        id: ProviderId::Fc2ppvdb,
        default_enabled: false,
        enabled_env_var: "NAKO_METADATA_SCRAPER_PROVIDER_FC2PPVDB_ENABLED",
        capabilities: &[
            "metadata_suggestion",
            "av_number_search",
            "fc2ppvdb_direct_lookup",
            "fc2_long_tail",
            "browser_worker_rendered_html",
        ],
        field_quality: crate::engine::ProviderFieldQualityDescriptor::new(430, 450, 450, 350),
        secret_reference: None,
        external_id_capabilities: FC2PPVDB_EXTERNAL_ID_CAPABILITIES,
        load_config: load_config,
        proxy_configured: |_| false,
        network_policy_key: None,
        build: build_provider,
    }
}

fn load_config(input: ProviderConfigInput<'_>) -> ProviderConfig {
    let lookup = input.lookup;
    ProviderConfig::fc2ppvdb(
        input.enabled,
        Fc2ppvdbProviderConfig::from_env_lookup(|name| lookup(name)),
    )
}

fn build_provider(config: &Config) -> ProviderBuildStatus {
    let Some(provider_config) = config
        .provider_config(ProviderId::Fc2ppvdb)
        .and_then(|provider| provider.fc2ppvdb_config().cloned())
    else {
        return ProviderBuildStatus::Unavailable;
    };
    match Fc2ppvdbMetadataProvider::new(provider_config) {
        Ok(provider) => ProviderBuildStatus::Ready(Box::new(provider)),
        Err(_) => ProviderBuildStatus::Unavailable,
    }
}

#[derive(Clone, Debug)]
pub struct Fc2ppvdbMetadataProvider<T = ReqwestProviderHttpTransport>
where
    T: ProviderHttpTransport,
{
    config: Fc2ppvdbProviderConfig,
    rendered_pages: RenderedPageRuntime<T>,
}

impl Fc2ppvdbMetadataProvider<ReqwestProviderHttpTransport> {
    pub fn new(config: Fc2ppvdbProviderConfig) -> ProviderHttpResult<Self> {
        let rendered_pages = RenderedPageRuntime::new(config.rendered_pages.clone())?;
        Ok(Self {
            config,
            rendered_pages,
        })
    }
}

impl<T> Fc2ppvdbMetadataProvider<T>
where
    T: ProviderHttpTransport,
{
    #[must_use]
    pub fn with_runtime(config: Fc2ppvdbProviderConfig, runtime: ProviderHttpRuntime<T>) -> Self {
        let rendered_pages =
            RenderedPageRuntime::with_runtime(config.rendered_pages.clone(), runtime);
        Self {
            config,
            rendered_pages,
        }
    }

    async fn suggest_candidates(
        &self,
        query: &MetadataQuery,
    ) -> anyhow::Result<Vec<ProviderMetadataCandidate>> {
        if let Some(url) = rendered_av::direct_external_id(query, FC2PPVDB_URL_EXTERNAL_ID_PROVIDER)
        {
            return self
                .detail_candidates(
                    &rendered_av::absolute_url(&self.config.base_url, &url),
                    query,
                )
                .await;
        }

        if let Some(article_id) = rendered_av::direct_external_id(query, FC2PPVDB_PROVIDER_ID)
            .and_then(|value| normalize_fc2_article_id(&value))
        {
            return self
                .detail_candidates(&self.detail_url(&article_id), query)
                .await;
        }

        let Some(av) = facts_from_query(query) else {
            return Ok(Vec::new());
        };
        if av.route != AvNumberRoute::Fc2 {
            return Ok(Vec::new());
        }
        let Some(article_id) = article_id_from_av_number(&av.number) else {
            return Ok(Vec::new());
        };

        self.detail_candidates(&self.detail_url(&article_id), query)
            .await
    }

    async fn detail_candidates(
        &self,
        detail_url: &str,
        query: &MetadataQuery,
    ) -> anyhow::Result<Vec<ProviderMetadataCandidate>> {
        let page = self.render(detail_url.to_owned()).await?;
        Ok(parse_detail_page(&page.html, detail_url)
            .map(|facts| vec![facts.into_candidate(query)])
            .unwrap_or_default())
    }

    async fn render(&self, url: String) -> anyhow::Result<RenderedHtmlPage> {
        let intent = self
            .config
            .rendered_pages
            .intent(&self.config.render_path, url);
        self.rendered_pages
            .render_html(FC2PPVDB_PROVIDER_ID, "render page", intent)
            .await
    }

    fn detail_url(&self, article_id: &str) -> String {
        format!(
            "{}/articles/{}",
            self.config.base_url.trim_end_matches('/'),
            article_id
        )
    }
}

#[async_trait]
impl<T> MetadataProvider for Fc2ppvdbMetadataProvider<T>
where
    T: ProviderHttpTransport,
{
    fn id(&self) -> ProviderId {
        ProviderId::Fc2ppvdb
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

#[derive(Clone, Debug, Eq, PartialEq)]
struct Fc2ppvdbDetailFacts {
    article_id: String,
    url: String,
    av: AvQueryFacts,
    title: String,
    overview: Option<String>,
    release_date: Option<String>,
    release_year: Option<i32>,
    runtime_minutes: Option<u32>,
    actors: Vec<String>,
    tags: Vec<String>,
    seller: Option<String>,
    mosaic: Option<String>,
    poster_url: Option<String>,
    backdrop_urls: Vec<String>,
    trailer_url: Option<String>,
}

impl Fc2ppvdbDetailFacts {
    fn into_candidate(self, query: &MetadataQuery) -> ProviderMetadataCandidate {
        let mut tags = vec![
            FC2PPVDB_PROVIDER_ID.to_owned(),
            format!("av_number:{}", self.av.number),
            "av_route:fc2".to_owned(),
        ];
        tags.extend(self.actors.iter().map(|actor| format!("actor:{actor}")));
        tags.extend(self.tags.iter().map(|tag| format!("tag:{tag}")));
        if let Some(seller) = &self.seller {
            tags.push(format!("seller:{seller}"));
        }
        if let Some(mosaic) = &self.mosaic {
            tags.push(format!("mosaic:{mosaic}"));
        }

        let mut artwork_candidates = Vec::new();
        if let Some(poster_url) = self.poster_url.clone() {
            artwork_candidates.push(fc2ppvdb_artwork_candidate(
                &self.article_id,
                AddonArtworkKind::Poster,
                poster_url,
                0,
            ));
        }
        for (index, url) in self.backdrop_urls.iter().cloned().enumerate() {
            artwork_candidates.push(fc2ppvdb_artwork_candidate(
                &self.article_id,
                AddonArtworkKind::Backdrop,
                url,
                index + 1,
            ));
        }

        ProviderMetadataCandidate {
            provider: FC2PPVDB_PROVIDER_ID.to_owned(),
            provider_id: format!("fc2ppvdb:article:{}", self.article_id),
            patch: AddonMetadataPatch {
                title: Some(self.title.clone()),
                original_title: None,
                sort_title: Some(self.title.clone()),
                overview: self.overview.clone(),
                release_date: self.release_date.clone(),
                runtime_minutes: self.runtime_minutes,
                tagline: Some("FC2PPVDB AV article".to_owned()),
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
                    series: Some("FC2".to_owned()),
                    studio: self.seller.clone(),
                    publisher: self.seller.clone(),
                    maker: self.seller.clone(),
                    label: self.seller.clone(),
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
                        provider: FC2PPVDB_PROVIDER_ID.to_owned(),
                        value: self.article_id,
                    },
                    ProviderExternalId {
                        provider: FC2PPVDB_URL_EXTERNAL_ID_PROVIDER.to_owned(),
                        value: self.url,
                    },
                    ProviderExternalId {
                        provider: AV_NUMBER_EXTERNAL_ID_PROVIDER.to_owned(),
                        value: self.av.number,
                    },
                ],
                provider_outcomes: vec![ProviderOutcome::Fc2ppvdbRenderedHtmlParsed],
                provider_note: None,
            },
            artwork_candidates,
        }
    }
}

fn parse_detail_page(html: &str, detail_url: &str) -> Option<Fc2ppvdbDetailFacts> {
    let document = Html::parse_document(html);
    let body_text = rendered_av::element_text(&document, "body").unwrap_or_default();
    let article_id = article_id_from_url(detail_url).or_else(|| {
        facts_from_text(&body_text, AvNumberSource::ExternalId)
            .and_then(|facts| article_id_from_av_number(&facts.number))
    })?;
    let av = facts_from_text(&format!("FC2-{article_id}"), AvNumberSource::ExternalId)?;
    let title = rendered_av::first_non_empty(&[
        rendered_av::element_text(&document, "h1, h2, h3, article header").as_deref(),
        rendered_av::attr_value(&document, "meta[property=\"og:title\"]", "content").as_deref(),
        Some(av.number.as_str()),
    ])
    .map(|title| strip_fc2_prefix(&title, &article_id))?;
    let info_text =
        rendered_av::element_text(&document, "article, main, .article, .container, body")
            .unwrap_or_else(|| body_text.clone());
    let release_date = rendered_av::labeled_value(
        &info_text,
        &["販売日", "配信開始日", "Release Date"],
        FC2PPVDB_LABELS,
    )
    .or_else(|| rendered_av::first_iso_date(&body_text));
    let release_year = release_date.as_deref().and_then(rendered_av::first_year);
    let runtime_minutes = rendered_av::labeled_value(
        &info_text,
        &["収録時間", "再生時間", "Runtime"],
        FC2PPVDB_LABELS,
    )
    .and_then(|value| rendered_av::parse_minutes(&value));
    let seller = rendered_av::labeled_value(
        &info_text,
        &["販売者", "Seller", "メーカー", "Maker"],
        FC2PPVDB_LABELS,
    )
    .or_else(|| {
        rendered_av::link_texts(
            &document,
            "a[href*=\"seller\"], a[href*=\"maker\"], a[href*=\"studio\"]",
        )
        .into_iter()
        .next()
    });
    let actors = non_empty_or_links(
        rendered_av::labeled_value(&info_text, &["女優", "Actress", "Actor"], FC2PPVDB_LABELS),
        rendered_av::link_texts(
            &document,
            "a[href*=\"actress\"], a[href*=\"actor\"], a[href*=\"star\"]",
        ),
    );
    let mut tags = rendered_av::link_texts(
        &document,
        "a[href*=\"tag\"], a[href*=\"genre\"], a[href*=\"category\"]",
    );
    for tag in split_label_values(rendered_av::labeled_value(
        &info_text,
        &["タグ", "Tag", "Genre"],
        FC2PPVDB_LABELS,
    )) {
        push_unique(&mut tags, tag);
    }
    tags.retain(|tag| !tag.contains("無修正"));
    let overview = rendered_av::first_non_empty(&[
        rendered_av::element_text(
            &document,
            ".summary, .outline, .description, .content .text",
        )
        .as_deref(),
        rendered_av::attr_value(&document, "meta[name=\"description\"]", "content").as_deref(),
    ]);
    let mosaic = rendered_av::labeled_value(
        &info_text,
        &["モザイク", "Mosaic", "资源参数"],
        FC2PPVDB_LABELS,
    );
    let poster_url = rendered_av::attr_value(&document, "meta[property=\"og:image\"]", "content")
        .or_else(|| {
            rendered_av::attr_value(
                &document,
                "img[alt*=\"FC2\"], .cover img, .article-cover img, article img",
                "src",
            )
        })
        .map(|url| rendered_av::absolute_url(detail_url, &url));
    let mut backdrop_urls = rendered_av::image_urls(
        &document,
        ".sample img, .gallery img, a[href*=\"sample\"], a[href*=\"gallery\"]",
        detail_url,
    );
    if let Some(poster_url) = &poster_url {
        backdrop_urls.retain(|url| url != poster_url);
    }
    backdrop_urls.retain(|url| !looks_like_video_url(url));
    let trailer_url = first_video_url(&document, detail_url);

    Some(Fc2ppvdbDetailFacts {
        article_id,
        url: detail_url.to_owned(),
        av,
        title,
        overview,
        release_date,
        release_year,
        runtime_minutes,
        actors,
        tags,
        seller,
        mosaic,
        poster_url,
        backdrop_urls,
        trailer_url,
    })
}

const FC2PPVDB_LABELS: &[&str] = &[
    "販売日",
    "配信開始日",
    "Release Date",
    "女優",
    "Actress",
    "Actor",
    "タグ",
    "Tag",
    "Genre",
    "販売者",
    "Seller",
    "メーカー",
    "Maker",
    "モザイク",
    "Mosaic",
    "资源参数",
    "収録時間",
    "再生時間",
    "Runtime",
];

fn normalize_fc2_article_id(value: &str) -> Option<String> {
    article_id_from_url(value)
        .or_else(|| article_id_from_av_number(value))
        .or_else(|| {
            let trimmed = value.trim();
            trimmed
                .chars()
                .all(|character| character.is_ascii_digit())
                .then(|| trimmed.to_owned())
        })
}

fn article_id_from_url(value: &str) -> Option<String> {
    ["/articles/", "/article/"].into_iter().find_map(|marker| {
        let start = value.find(marker)? + marker.len();
        let rest = &value[start..];
        let end = rest.find(['/', '?', '#', '&']).unwrap_or(rest.len());
        normalize_fc2_article_id(&rest[..end])
    })
}

fn article_id_from_av_number(value: &str) -> Option<String> {
    facts_from_text(value, AvNumberSource::ExternalId).and_then(|facts| {
        (facts.route == AvNumberRoute::Fc2)
            .then(|| facts.number.strip_prefix("FC2-").map(str::to_owned))
            .flatten()
    })
}

fn strip_fc2_prefix(title: &str, article_id: &str) -> String {
    let title = rendered_av::normalize_whitespace(title);
    for prefix in [
        format!("FC2-{article_id}"),
        format!("FC2PPV-{article_id}"),
        article_id.to_owned(),
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

fn fc2ppvdb_artwork_candidate(
    article_id: &str,
    kind: AddonArtworkKind,
    source_url: String,
    index: usize,
) -> ProviderArtworkCandidate {
    ProviderArtworkCandidate {
        provider: FC2PPVDB_PROVIDER_ID.to_owned(),
        provider_id: format!("fc2ppvdb:article:{article_id}:artwork:{index}"),
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
            http_runtime::{ProviderHttpRuntime, ProviderHttpRuntimeConfig},
            rendered_av_fixture::{RenderedAvFixtureTransport, request_json_body},
        },
    };

    use super::*;

    #[tokio::test]
    async fn fc2ppvdb_provider_uses_browser_worker_render_contract_for_fc2_detail() {
        let transport = RenderedAvFixtureTransport::new(FC2PPVDB_PROVIDER_ID);
        transport.push_rendered_html(
            "https://fc2ppvdb.example/articles/1723984",
            "FC2-1723984 Long Tail Title",
            detail_html("FC2-1723984 Long Tail Title"),
        );
        let provider = provider_with_transport(transport.clone());

        let candidates = provider
            .suggest(&MetadataQuery::from_payload(
                &serde_json::json!({"file_name": "FC2PPV-1723984.mp4"}),
                "zh-CN",
            ))
            .await
            .unwrap();

        assert_eq!(candidates.len(), 1);
        let candidate = &candidates[0];
        assert_eq!(candidate.provider, "fc2ppvdb");
        assert_eq!(candidate.provider_id, "fc2ppvdb:article:1723984");
        assert_eq!(candidate.patch.title.as_deref(), Some("Long Tail Title"));
        assert_eq!(
            candidate.patch.overview.as_deref(),
            Some("Long-tail fallback outline.")
        );
        assert_eq!(candidate.patch.release_date.as_deref(), Some("2024-04-21"));
        assert_eq!(candidate.patch.runtime_minutes, Some(91));
        assert_eq!(
            candidate.patch.genres.as_deref(),
            Some(["Amateur".to_owned(), "POV".to_owned()].as_slice())
        );
        let av = candidate.facts.av.as_ref().unwrap();
        assert_eq!(av.actors, vec!["Actress One", "Actress Two"]);
        assert_eq!(av.studio.as_deref(), Some("FC2 Seller"));
        assert_eq!(av.series.as_deref(), Some("FC2"));
        assert_eq!(
            av.thumb_url.as_deref(),
            Some("https://img.example/fc2ppvdb-cover.jpg")
        );
        assert_eq!(
            av.trailer_url.as_deref(),
            Some("https://video.example/fc2ppvdb-sample.mp4")
        );
        assert_eq!(
            av.extrafanart_urls,
            vec![
                "https://img.example/fc2ppvdb-sample1.jpg".to_owned(),
                "https://img.example/fc2ppvdb-sample2.jpg".to_owned(),
            ]
        );
        assert_eq!(candidate.artwork_candidates.len(), 3);
        assert_eq!(
            candidate.artwork_candidates[0].facts.kind,
            AddonArtworkKind::Poster
        );
        assert!(candidate.facts.external_ids.contains(&ProviderExternalId {
            provider: "fc2ppvdb".to_owned(),
            value: "1723984".to_owned(),
        }));
        assert!(candidate.facts.external_ids.contains(&ProviderExternalId {
            provider: "fc2ppvdb_url".to_owned(),
            value: "https://fc2ppvdb.example/articles/1723984".to_owned(),
        }));
        assert!(candidate.facts.external_ids.contains(&ProviderExternalId {
            provider: "av_number".to_owned(),
            value: "FC2-1723984".to_owned(),
        }));
        assert_eq!(
            candidate.facts.provider_outcomes,
            vec![ProviderOutcome::Fc2ppvdbRenderedHtmlParsed]
        );

        let requests = transport.requests();
        assert_eq!(requests.len(), 1);
        let body = request_json_body(&requests[0]);
        assert_eq!(body["url"], "https://fc2ppvdb.example/articles/1723984");
    }

    #[tokio::test]
    async fn fc2ppvdb_provider_uses_explicit_id_for_direct_detail_lookup() {
        let transport = RenderedAvFixtureTransport::new(FC2PPVDB_PROVIDER_ID);
        transport.push_rendered_html(
            "https://fc2ppvdb.example/articles/1723984",
            "FC2-1723984 Direct Title",
            detail_html("FC2-1723984 Direct Title"),
        );
        let provider = provider_with_transport(transport.clone());

        let candidates = provider
            .suggest(&MetadataQuery {
                title: "Manual lookup".to_owned(),
                year: None,
                language: "zh-CN".to_owned(),
                external_ids: vec![QueryExternalId {
                    provider: "fc2ppvdb".to_owned(),
                    value: "FC2-1723984".to_owned(),
                }],
            })
            .await
            .unwrap();

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].provider_id, "fc2ppvdb:article:1723984");
        assert_eq!(
            request_json_body(&transport.requests()[0])["url"],
            "https://fc2ppvdb.example/articles/1723984"
        );
    }

    #[tokio::test]
    async fn fc2ppvdb_provider_uses_explicit_url_for_direct_detail_lookup() {
        let transport = RenderedAvFixtureTransport::new(FC2PPVDB_PROVIDER_ID);
        transport.push_rendered_html(
            "https://mirror.example/articles/1723984?from=user",
            "FC2-1723984 URL Title",
            detail_html("FC2-1723984 URL Title"),
        );
        let provider = provider_with_transport(transport.clone());

        let candidates = provider
            .suggest(&MetadataQuery {
                title: "Manual lookup".to_owned(),
                year: None,
                language: "zh-CN".to_owned(),
                external_ids: vec![QueryExternalId {
                    provider: "fc2ppvdb_url".to_owned(),
                    value: "https://mirror.example/articles/1723984?from=user".to_owned(),
                }],
            })
            .await
            .unwrap();

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].provider_id, "fc2ppvdb:article:1723984");
        assert_eq!(
            request_json_body(&transport.requests()[0])["url"],
            "https://mirror.example/articles/1723984?from=user"
        );
    }

    #[tokio::test]
    async fn fc2ppvdb_provider_skips_non_fc2_numbers() {
        let transport = RenderedAvFixtureTransport::new(FC2PPVDB_PROVIDER_ID);
        let provider = provider_with_transport(transport.clone());

        let candidates = provider
            .suggest(&MetadataQuery::from_payload(
                &serde_json::json!({"file_name": "SSNI-00644.mp4"}),
                "zh-CN",
            ))
            .await
            .unwrap();

        assert!(candidates.is_empty());
        assert!(transport.requests().is_empty());
        assert!(provider.supports_av_route(AvNumberRoute::Fc2));
        assert!(!provider.supports_av_route(AvNumberRoute::Censored));
    }

    fn provider_with_transport(
        transport: RenderedAvFixtureTransport,
    ) -> Fc2ppvdbMetadataProvider<RenderedAvFixtureTransport> {
        let runtime = ProviderHttpRuntime::with_transport(
            ProviderHttpRuntimeConfig {
                retry_backoff_ms: 0,
                ..ProviderHttpRuntimeConfig::default()
            },
            transport,
        );
        Fc2ppvdbMetadataProvider::with_runtime(
            Fc2ppvdbProviderConfig::new(
                "https://fc2ppvdb.example".to_owned(),
                "http://browser-worker.example".to_owned(),
                "/render".to_owned(),
                10_000,
            ),
            runtime,
        )
    }

    fn detail_html(title: &str) -> &'static str {
        match title {
            "FC2-1723984 Long Tail Title" => {
                r#"
<!doctype html>
<html>
<head>
  <meta property="og:image" content="https://img.example/fc2ppvdb-cover.jpg">
  <meta name="description" content="Long-tail fallback outline.">
</head>
<body>
  <article>
    <h2><a href="/articles/1723984">FC2-1723984 Long Tail Title</a></h2>
    <div class="details">
      <div>販売日：<span>2024-04-21</span></div>
      <div>女優：<span><a href="/actress/one">Actress One</a>, <a href="/actress/two">Actress Two</a></span></div>
      <div>タグ：<span><a href="/tag/amateur">Amateur</a>, <a href="/tag/pov">POV</a>, <a href="/tag/uncensored">無修正</a></span></div>
      <div>販売者：<span><a href="/seller/fc2-seller">FC2 Seller</a></span></div>
      <div>モザイク：<span>有</span></div>
      <div>収録時間：<span>91分</span></div>
    </div>
    <section class="summary">Long-tail fallback outline.</section>
    <div class="sample">
      <img src="https://img.example/fc2ppvdb-sample1.jpg">
      <a href="https://img.example/fc2ppvdb-sample2.jpg">sample</a>
    </div>
    <a href="https://video.example/fc2ppvdb-sample.mp4">Sample video</a>
  </article>
</body>
</html>"#
            }
            "FC2-1723984 Direct Title" => {
                r#"
<!doctype html>
<html><body><main>
  <h2>FC2-1723984 Direct Title</h2>
  <div>販売日：<span>2024-04-21</span></div>
  <div>販売者：<span>FC2 Seller</span></div>
</main></body></html>"#
            }
            _ => {
                r#"
<!doctype html>
<html><body><main>
  <h2>FC2-1723984 URL Title</h2>
  <div>販売日：<span>2024-04-21</span></div>
  <div>販売者：<span>FC2 Seller</span></div>
</main></body></html>"#
            }
        }
    }
}
