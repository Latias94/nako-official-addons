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
            ProviderHttpResult, ProviderHttpRuntime, ProviderHttpRuntimeConfig,
            ProviderHttpTransport, ReqwestProviderHttpTransport,
        },
        registry::ProviderCatalogEntry,
        rendered_av,
    },
};

pub const JAV321_PROVIDER_ID: &str = "jav321";
const JAV321_URL_EXTERNAL_ID_PROVIDER: &str = "jav321_url";
const JAV321_EXTERNAL_ID_CAPABILITIES: &[ProviderExternalIdCapability] = &[
    ProviderExternalIdCapability::new(
        JAV321_PROVIDER_ID,
        ExternalIdValueKind::Opaque,
        true,
        true,
        &["jav321_id"],
        false,
    ),
    ProviderExternalIdCapability::new(
        JAV321_URL_EXTERNAL_ID_PROVIDER,
        ExternalIdValueKind::Url,
        true,
        true,
        &["jav321_url"],
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
pub struct Jav321ProviderConfig {
    pub(crate) base_url: String,
    pub(crate) timeout_ms: u64,
    pub(crate) proxy_url: Option<String>,
}

impl Jav321ProviderConfig {
    pub const DEFAULT_TIMEOUT_MS: u64 = 10_000;

    #[must_use]
    pub const fn new(base_url: String, timeout_ms: u64, proxy_url: Option<String>) -> Self {
        Self {
            base_url,
            timeout_ms,
            proxy_url,
        }
    }

    #[must_use]
    pub fn from_env_lookup(mut lookup: impl FnMut(&str) -> Option<String>) -> Self {
        let base_url = lookup("NAKO_METADATA_SCRAPER_JAV321_BASE_URL")
            .and_then(non_empty_trimmed)
            .unwrap_or_else(|| "https://www.jav321.com".to_owned());
        let timeout_ms = lookup("NAKO_METADATA_SCRAPER_JAV321_TIMEOUT_MS")
            .and_then(|value| value.trim().parse::<u64>().ok())
            .filter(|value| *value > 0)
            .unwrap_or(Self::DEFAULT_TIMEOUT_MS);
        let proxy_url =
            lookup("NAKO_METADATA_SCRAPER_JAV321_PROXY_URL").and_then(non_empty_trimmed);

        Self::new(base_url, timeout_ms, proxy_url)
    }
}

#[must_use]
pub(crate) fn catalog_entry() -> ProviderCatalogEntry {
    ProviderCatalogEntry {
        id: ProviderId::Jav321,
        default_enabled: false,
        enabled_env_var: "NAKO_METADATA_SCRAPER_PROVIDER_JAV321_ENABLED",
        capabilities: &[
            "metadata_suggestion",
            "av_number_search",
            "jav321_direct_lookup",
            "jav321_post_form_search",
            "raw_html_parse",
        ],
        field_quality: crate::engine::ProviderFieldQualityDescriptor::new(450, 550, 350, 250),
        secret_reference: None,
        external_id_capabilities: JAV321_EXTERNAL_ID_CAPABILITIES,
        load_config: load_config,
        proxy_configured: proxy_configured,
        network_policy_key: Some("jav321_proxy_configured"),
        build: build_provider,
    }
}

fn load_config(input: ProviderConfigInput<'_>) -> ProviderConfig {
    let lookup = input.lookup;
    ProviderConfig::jav321(
        input.enabled,
        Jav321ProviderConfig::from_env_lookup(|name| lookup(name)),
    )
}

fn proxy_configured(config: &ProviderConfig) -> bool {
    config
        .jav321_config()
        .is_some_and(|config| config.proxy_url.is_some())
}

fn build_provider(config: &Config) -> ProviderBuildStatus {
    let Some(provider_config) = config
        .provider_config(ProviderId::Jav321)
        .and_then(|provider| provider.jav321_config().cloned())
    else {
        return ProviderBuildStatus::Unavailable;
    };
    match Jav321MetadataProvider::new(provider_config) {
        Ok(provider) => ProviderBuildStatus::Ready(Box::new(provider)),
        Err(_) => ProviderBuildStatus::Unavailable,
    }
}

#[derive(Clone, Debug)]
pub struct Jav321MetadataProvider<T = ReqwestProviderHttpTransport>
where
    T: ProviderHttpTransport,
{
    config: Jav321ProviderConfig,
    runtime: ProviderHttpRuntime<T>,
}

impl Jav321MetadataProvider<ReqwestProviderHttpTransport> {
    pub fn new(config: Jav321ProviderConfig) -> ProviderHttpResult<Self> {
        let runtime = ProviderHttpRuntime::new(runtime_config(&config))?;
        Ok(Self { config, runtime })
    }
}

impl<T> Jav321MetadataProvider<T>
where
    T: ProviderHttpTransport,
{
    #[must_use]
    pub const fn with_runtime(
        config: Jav321ProviderConfig,
        runtime: ProviderHttpRuntime<T>,
    ) -> Self {
        Self { config, runtime }
    }

    async fn suggest_candidates(
        &self,
        query: &MetadataQuery,
    ) -> anyhow::Result<Vec<ProviderMetadataCandidate>> {
        if let Some(url) = rendered_av::direct_external_id(query, JAV321_URL_EXTERNAL_ID_PROVIDER) {
            return self
                .detail_candidates_from_url(self.absolute_url(&url), None, query)
                .await;
        }

        if let Some(id) = rendered_av::direct_external_id(query, JAV321_PROVIDER_ID) {
            return self
                .detail_candidates_from_url(self.detail_url(&id), None, query)
                .await;
        }

        let Some(av) = facts_from_query(query) else {
            return Ok(Vec::new());
        };
        if !self.supports_av_route(av.route) {
            return Ok(Vec::new());
        }

        let search_url = self.search_url();
        let response = self
            .runtime
            .post_form_text(
                JAV321_PROVIDER_ID,
                "search",
                search_url.clone(),
                Vec::new(),
                jav321_headers(query),
                vec![("sn".to_owned(), av.number.clone())],
            )
            .await?;
        if is_not_found_page(&response.body) {
            return Ok(Vec::new());
        }

        let detail_url =
            canonical_detail_url(&response.body, &self.config.base_url).unwrap_or(search_url);
        Ok(
            parse_detail_page(&response.body, &detail_url, Some(av), query)
                .into_iter()
                .map(|facts| facts.into_candidate(query))
                .collect(),
        )
    }

    async fn detail_candidates_from_url(
        &self,
        url: String,
        av: Option<AvQueryFacts>,
        query: &MetadataQuery,
    ) -> anyhow::Result<Vec<ProviderMetadataCandidate>> {
        let response = self
            .runtime
            .get_text(
                JAV321_PROVIDER_ID,
                "detail",
                url.clone(),
                Vec::new(),
                jav321_headers(query),
            )
            .await?;
        if is_not_found_page(&response.body) {
            return Ok(Vec::new());
        }

        let detail_url = canonical_detail_url(&response.body, &self.config.base_url).unwrap_or(url);
        Ok(parse_detail_page(&response.body, &detail_url, av, query)
            .into_iter()
            .map(|facts| facts.into_candidate(query))
            .collect())
    }

    fn search_url(&self) -> String {
        format!("{}/search", self.config.base_url.trim_end_matches('/'))
    }

    fn detail_url(&self, id: &str) -> String {
        let value = id.trim();
        if value.starts_with("http://") || value.starts_with("https://") {
            return value.to_owned();
        }
        format!(
            "{}/video/{}",
            self.config.base_url.trim_end_matches('/'),
            value.trim_start_matches('/')
        )
    }

    fn absolute_url(&self, value: &str) -> String {
        rendered_av::absolute_url(&self.config.base_url, value)
    }
}

#[async_trait]
impl<T> MetadataProvider for Jav321MetadataProvider<T>
where
    T: ProviderHttpTransport,
{
    fn id(&self) -> ProviderId {
        ProviderId::Jav321
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
struct Jav321DetailFacts {
    id: String,
    url: String,
    av: AvQueryFacts,
    title: String,
    outline: Option<String>,
    release_date: Option<String>,
    release_year: Option<i32>,
    runtime_minutes: Option<u32>,
    actors: Vec<String>,
    tags: Vec<String>,
    studio: Option<String>,
    publisher: Option<String>,
    series: Option<String>,
    rating_milli: Option<u16>,
    poster_url: Option<String>,
    thumb_url: Option<String>,
    extrafanart_urls: Vec<String>,
}

impl Jav321DetailFacts {
    fn into_candidate(self, query: &MetadataQuery) -> ProviderMetadataCandidate {
        let mut tags = vec![
            JAV321_PROVIDER_ID.to_owned(),
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

        let mut artwork_candidates = Vec::new();
        if let Some(poster_url) = self.poster_url.clone() {
            artwork_candidates.push(jav321_artwork_candidate(
                &self.id,
                AddonArtworkKind::Poster,
                poster_url,
                0,
            ));
        }
        for (index, url) in self.extrafanart_urls.iter().cloned().enumerate() {
            artwork_candidates.push(jav321_artwork_candidate(
                &self.id,
                AddonArtworkKind::Backdrop,
                url,
                index + 1,
            ));
        }

        ProviderMetadataCandidate {
            provider: JAV321_PROVIDER_ID.to_owned(),
            provider_id: format!("jav321:movie:{}", self.id),
            patch: AddonMetadataPatch {
                title: Some(self.title.clone()),
                original_title: Some(self.title.clone()),
                sort_title: Some(self.title.clone()),
                overview: self.outline.clone(),
                release_date: self.release_date.clone(),
                runtime_minutes: self.runtime_minutes,
                tagline: Some("Jav321 AV title".to_owned()),
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
                    series: self.series.clone(),
                    studio: self.studio.clone(),
                    publisher: self.publisher.clone(),
                    maker: self.studio.clone(),
                    label: self.publisher.clone(),
                    thumb_url: self.thumb_url.clone(),
                    trailer_url: None,
                    extrafanart_urls: self.extrafanart_urls.clone(),
                    ..AvMetadataFacts::default()
                }
                .non_empty(),
                community_score_milli: self.rating_milli,
                community_vote_count: None,
                external_ids: vec![
                    ProviderExternalId {
                        provider: JAV321_PROVIDER_ID.to_owned(),
                        value: self.id,
                    },
                    ProviderExternalId {
                        provider: JAV321_URL_EXTERNAL_ID_PROVIDER.to_owned(),
                        value: self.url,
                    },
                    ProviderExternalId {
                        provider: AV_NUMBER_EXTERNAL_ID_PROVIDER.to_owned(),
                        value: self.av.number,
                    },
                ],
                provider_outcomes: vec![ProviderOutcome::Jav321RawHtmlParsed],
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
    query: &MetadataQuery,
) -> Option<Jav321DetailFacts> {
    let document = Html::parse_document(html);
    let base_url = site_base_url(detail_url).unwrap_or_else(|| detail_url.to_owned());
    let body_text = rendered_av::element_text(&document, "body").unwrap_or_default();
    let info_text = rendered_av::element_text(&document, ".col-md-9, main, body")
        .unwrap_or_else(|| body_text.clone());
    let normalized_info = normalize_label_markers(&info_text);

    let title = rendered_av::first_non_empty(&[
        first_heading_text(&document, "h3, h1").as_deref(),
        rendered_av::attr_value(&document, "meta[property=\"og:title\"]", "content").as_deref(),
        rendered_av::element_text(&document, "title").as_deref(),
    ])?;
    let number = jav321_labeled_value(&normalized_info, &["品番", "番號", "番号", "Number"])
        .or_else(|| facts_from_text(&title, AvNumberSource::ExternalId).map(|facts| facts.number))
        .or_else(|| {
            facts_from_text(detail_url, AvNumberSource::ExternalId).map(|facts| facts.number)
        });
    let av = number
        .as_deref()
        .and_then(|value| facts_from_text(value, AvNumberSource::ExternalId))
        .or_else(|| facts_from_text(&title, AvNumberSource::ExternalId))
        .or(av)
        .or_else(|| facts_from_text(&query.title, AvNumberSource::QueryTitle))?;
    let release_date = jav321_labeled_value(
        &normalized_info,
        &[
            "配信開始日",
            "発売日",
            "發行日期",
            "发行日期",
            "Release Date",
        ],
    )
    .or_else(|| rendered_av::first_iso_date(&body_text))
    .filter(|value| value != "0000-00-00")
    .and_then(|value| rendered_av::first_iso_date(&value).or(Some(value)));
    let release_year = release_date.as_deref().and_then(rendered_av::first_year);
    let runtime_minutes =
        jav321_labeled_value(&normalized_info, &["収録時間", "長度", "长度", "Runtime"])
            .and_then(|value| rendered_av::parse_minutes(&value));
    let actors = {
        let linked =
            rendered_av::link_texts(&document, "a[href*=\"/star/\"], a[href*=\"/heyzo_star/\"]");
        if linked.is_empty() {
            jav321_labeled_value(&normalized_info, &["出演者", "演員", "演员", "Cast"])
                .map(|value| split_people(&value))
                .unwrap_or_default()
        } else {
            linked
        }
    };
    let studio = first_link_text(&document, "a[href*=\"/company/\"]").or_else(|| {
        jav321_labeled_value(&normalized_info, &["メーカー", "片商", "製作", "Studio"])
    });
    let publisher = studio.clone();
    let series = first_link_text(&document, "a[href*=\"/series/\"]")
        .or_else(|| jav321_labeled_value(&normalized_info, &["シリーズ", "系列", "Series"]));
    let tags = rendered_av::link_texts(&document, "a[href*=\"/genre/\"]");
    let rating_milli = rating_from_score_image(&document)
        .or_else(|| jav321_labeled_value(&normalized_info, &["平均評価", "評分", "评分", "Rating"]))
        .and_then(|value| parse_jav321_rating_milli(&value));
    let outline = outline_text(&document, &title);
    let image_urls = image_src_urls(
        &document,
        ".col-md-3 img.img-responsive, img.img-responsive",
        &base_url,
    );
    let video_poster =
        rendered_av::attr_value(&document, "#vjs_sample_player, video[poster]", "poster")
            .map(|url| rendered_av::absolute_url(&base_url, &url));
    let poster_url = image_urls.first().cloned().or(video_poster.clone());
    let thumb_url = image_urls.first().cloned().or(video_poster);
    let extrafanart_urls = image_urls.into_iter().fold(Vec::new(), |mut urls, url| {
        if !urls.contains(&url) {
            urls.push(url);
        }
        urls
    });
    let url = canonical_detail_url(html, &base_url).unwrap_or_else(|| detail_url.to_owned());

    Some(Jav321DetailFacts {
        id: jav321_id_from_url(&url).unwrap_or_else(|| av.number.to_ascii_lowercase()),
        url,
        av,
        title,
        outline,
        release_date,
        release_year,
        runtime_minutes,
        actors,
        tags,
        studio,
        publisher,
        series,
        rating_milli,
        poster_url,
        thumb_url,
        extrafanart_urls,
    })
}

const JAV321_LABELS: &[&str] = &[
    "品番",
    "番號",
    "番号",
    "Number",
    "配信開始日",
    "発売日",
    "發行日期",
    "发行日期",
    "Release Date",
    "収録時間",
    "長度",
    "长度",
    "Runtime",
    "出演者",
    "演員",
    "演员",
    "Cast",
    "メーカー",
    "片商",
    "製作",
    "Studio",
    "シリーズ",
    "系列",
    "Series",
    "平均評価",
    "評分",
    "评分",
    "Rating",
];

fn runtime_config(config: &Jav321ProviderConfig) -> ProviderHttpRuntimeConfig {
    ProviderHttpRuntimeConfig {
        timeout_ms: config.timeout_ms,
        proxy_url: config.proxy_url.clone(),
        ..ProviderHttpRuntimeConfig::default()
    }
}

fn jav321_headers(query: &MetadataQuery) -> Vec<(String, String)> {
    vec![(
        "accept-language".to_owned(),
        format!("{},zh-CN;q=0.9,ja;q=0.8,en;q=0.6", query.language),
    )]
}

fn is_not_found_page(html: &str) -> bool {
    html.contains("AVが見つかりませんでした")
        || html.to_ascii_lowercase().contains("not found")
        || html.contains("没有找到")
}

fn canonical_detail_url(html: &str, base_url: &str) -> Option<String> {
    let document = Html::parse_document(html);
    let selector = Selector::parse("a[href]").ok()?;
    document.select(&selector).find_map(|link| {
        let text = rendered_av::normalize_whitespace(&link.text().collect::<Vec<_>>().join(" "));
        if !text.contains("简体中文") && !text.contains("繁體中文") && !text.contains("日本語")
        {
            return None;
        }
        link.value()
            .attr("href")
            .map(|href| rendered_av::absolute_url(base_url, href))
    })
}

fn first_heading_text(document: &Html, selector: &str) -> Option<String> {
    let selector = Selector::parse(selector).ok()?;
    document.select(&selector).find_map(|element| {
        element.text().find_map(|text| {
            Some(rendered_av::normalize_whitespace(text)).filter(|value| !value.is_empty())
        })
    })
}

fn jav321_labeled_value(info_text: &str, labels: &[&str]) -> Option<String> {
    rendered_av::labeled_value(info_text, labels, JAV321_LABELS)
}

fn normalize_label_markers(value: &str) -> String {
    JAV321_LABELS.iter().fold(value.to_owned(), |text, label| {
        text.replace(&format!("{label} :"), &format!("{label}:"))
            .replace(&format!("{label} ："), &format!("{label}:"))
    })
}

fn first_link_text(document: &Html, selector: &str) -> Option<String> {
    rendered_av::link_texts(document, selector)
        .into_iter()
        .next()
}

fn split_people(value: &str) -> Vec<String> {
    value
        .split([',', '/', '、', '，', '|'])
        .map(rendered_av::normalize_whitespace)
        .filter(|value| !value.is_empty())
        .fold(Vec::new(), |mut people, value| {
            if !people.contains(&value) {
                people.push(value);
            }
            people
        })
}

fn rating_from_score_image(document: &Html) -> Option<String> {
    let selector = Selector::parse("img[src*=\"/img/\"], img[data-original*=\"/img/\"]").ok()?;
    document.select(&selector).find_map(|image| {
        image
            .value()
            .attr("data-original")
            .or_else(|| image.value().attr("src"))
            .and_then(score_image_value)
    })
}

fn score_image_value(value: &str) -> Option<String> {
    let start = value.find("/img/")? + "/img/".len();
    let digits: String = value[start..]
        .chars()
        .take_while(char::is_ascii_digit)
        .collect();
    let score = digits.parse::<f64>().ok()? / 10.0;
    Some(score.to_string())
}

fn parse_jav321_rating_milli(value: &str) -> Option<u16> {
    rendered_av::parse_rating_milli(value)
}

fn outline_text(document: &Html, title: &str) -> Option<String> {
    rendered_av::first_non_empty(&[
        rendered_av::attr_value(document, "meta[name=\"description\"]", "content").as_deref(),
        rendered_av::element_text(
            document,
            ".summary, .outline, .description, .story, .well, [itemprop=\"description\"]",
        )
        .as_deref(),
    ])
    .filter(|value| is_outline_candidate(value, title))
    .or_else(|| outline_from_text_nodes(document, title))
}

fn outline_from_text_nodes(document: &Html, title: &str) -> Option<String> {
    let selector = Selector::parse(".col-md-9, main, body").ok()?;
    document.select(&selector).find_map(|element| {
        element.text().find_map(|text| {
            let value = rendered_av::normalize_whitespace(text);
            is_outline_candidate(&value, title).then_some(value)
        })
    })
}

fn is_outline_candidate(value: &str, title: &str) -> bool {
    value.chars().count() >= 24
        && value != title
        && !JAV321_LABELS.iter().any(|label| value.contains(label))
}

fn image_src_urls(document: &Html, selector: &str, base_url: &str) -> Vec<String> {
    let Ok(selector) = Selector::parse(selector) else {
        return Vec::new();
    };
    document
        .select(&selector)
        .filter_map(|image| image.value().attr("src"))
        .map(|value| rendered_av::absolute_url(base_url, value))
        .fold(Vec::new(), |mut urls, url| {
            if !urls.contains(&url) {
                urls.push(url);
            }
            urls
        })
}

fn jav321_artwork_candidate(
    movie_id: &str,
    kind: AddonArtworkKind,
    source_url: String,
    index: usize,
) -> ProviderArtworkCandidate {
    ProviderArtworkCandidate {
        provider: JAV321_PROVIDER_ID.to_owned(),
        provider_id: format!("jav321:movie:{movie_id}:artwork:{index}"),
        facts: ProviderArtworkCandidateFacts {
            kind,
            source_url,
            language: None,
            width: None,
            height: None,
        },
    }
}

fn jav321_id_from_url(url: &str) -> Option<String> {
    let marker = "/video/";
    let start = url.find(marker)? + marker.len();
    let rest = &url[start..];
    let end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let id = &rest[..end];
    (!id.is_empty()).then(|| id.to_owned())
}

fn site_base_url(url: &str) -> Option<String> {
    let scheme_end = url.find("://")? + 3;
    let rest = &url[scheme_end..];
    let host_end = rest.find('/').unwrap_or(rest.len());
    Some(url[..scheme_end + host_end].to_owned())
}

#[cfg(test)]
mod tests {
    use std::{
        collections::VecDeque,
        sync::{Arc, Mutex},
    };

    use crate::{
        engine::QueryExternalId,
        providers::http_runtime::{
            ProviderHttpRequest, ProviderHttpResponse, ProviderHttpRuntimeConfig,
        },
    };

    use super::*;

    #[tokio::test]
    async fn jav321_provider_posts_search_form_and_parses_detail_contract() {
        let transport = FakeTransport::default();
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: jav321_detail_html().as_bytes().to_vec(),
        }));
        let runtime = ProviderHttpRuntime::with_transport(
            ProviderHttpRuntimeConfig {
                retry_backoff_ms: 0,
                ..ProviderHttpRuntimeConfig::default()
            },
            transport.clone(),
        );
        let provider = Jav321MetadataProvider::with_runtime(
            Jav321ProviderConfig::new("https://jav321.example".to_owned(), 10_000, None),
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
        assert_eq!(candidate.provider, JAV321_PROVIDER_ID);
        assert_eq!(candidate.provider_id, "jav321:movie:ssni00644");
        assert_eq!(
            candidate.patch.title.as_deref(),
            Some("SSNI-644 Jav321 Title")
        );
        assert_eq!(
            candidate.patch.original_title.as_deref(),
            Some("SSNI-644 Jav321 Title")
        );
        assert_eq!(
            candidate.patch.overview.as_deref(),
            Some("Jav321 synthetic outline with enough detail to be treated as plot text.")
        );
        assert_eq!(candidate.patch.release_date.as_deref(), Some("2024-05-03"));
        assert_eq!(candidate.patch.runtime_minutes, Some(121));
        assert_eq!(candidate.facts.community_score_milli, Some(840));
        let av = candidate.facts.av.as_ref().unwrap();
        assert_eq!(
            av.actors,
            vec!["Actor One".to_owned(), "Actor Two".to_owned()]
        );
        assert_eq!(av.studio.as_deref(), Some("Studio Alpha"));
        assert_eq!(av.publisher.as_deref(), Some("Studio Alpha"));
        assert_eq!(av.maker.as_deref(), Some("Studio Alpha"));
        assert_eq!(av.label.as_deref(), Some("Studio Alpha"));
        assert_eq!(av.series.as_deref(), Some("Series Gamma"));
        assert_eq!(
            av.thumb_url.as_deref(),
            Some("https://img.example/cover-small.jpg")
        );
        assert_eq!(
            av.extrafanart_urls,
            vec![
                "https://img.example/cover-small.jpg".to_owned(),
                "https://img.example/sample-one.jpg".to_owned(),
            ]
        );
        assert_eq!(
            candidate.patch.genres.as_ref().unwrap(),
            &vec!["Drama".to_owned(), "Uniform".to_owned()]
        );
        assert!(candidate.facts.external_ids.iter().any(|id| {
            id.provider == JAV321_URL_EXTERNAL_ID_PROVIDER
                && id.value == "https://jav321.example/video/ssni00644"
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
        assert_eq!(
            requests[0].method,
            crate::providers::http_runtime::ProviderHttpMethod::Post
        );
        assert_eq!(requests[0].url, "https://jav321.example/search");
        assert_eq!(
            requests[0].form_body,
            vec![("sn".to_owned(), "SSNI-644".to_owned())]
        );
    }

    #[tokio::test]
    async fn jav321_provider_returns_empty_when_search_page_says_not_found() {
        let transport = FakeTransport::default();
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: "AVが見つかりませんでした".as_bytes().to_vec(),
        }));
        let runtime = ProviderHttpRuntime::with_transport(
            ProviderHttpRuntimeConfig {
                retry_backoff_ms: 0,
                ..ProviderHttpRuntimeConfig::default()
            },
            transport.clone(),
        );
        let provider = Jav321MetadataProvider::with_runtime(
            Jav321ProviderConfig::new("https://jav321.example".to_owned(), 10_000, None),
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
        assert_eq!(transport.requests().len(), 1);
    }

    #[tokio::test]
    async fn jav321_provider_uses_direct_url_lookup_with_get_text() {
        let transport = FakeTransport::default();
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: jav321_detail_html().as_bytes().to_vec(),
        }));
        let runtime = ProviderHttpRuntime::with_transport(
            ProviderHttpRuntimeConfig {
                retry_backoff_ms: 0,
                ..ProviderHttpRuntimeConfig::default()
            },
            transport.clone(),
        );
        let provider = Jav321MetadataProvider::with_runtime(
            Jav321ProviderConfig::new("https://jav321.example".to_owned(), 10_000, None),
            runtime,
        );

        let candidates = provider
            .suggest(&MetadataQuery {
                title: "Untrusted Raw Title".to_owned(),
                year: None,
                language: "zh-CN".to_owned(),
                external_ids: vec![QueryExternalId {
                    provider: JAV321_URL_EXTERNAL_ID_PROVIDER.to_owned(),
                    value: "https://jav321.example/video/ssni00644".to_owned(),
                }],
            })
            .await
            .unwrap();

        assert_eq!(candidates.len(), 1);
        assert_eq!(
            candidates[0].patch.title.as_deref(),
            Some("SSNI-644 Jav321 Title")
        );
        let requests = transport.requests();
        assert_eq!(requests.len(), 1);
        assert_eq!(
            requests[0].method,
            crate::providers::http_runtime::ProviderHttpMethod::Get
        );
        assert_eq!(requests[0].url, "https://jav321.example/video/ssni00644");
    }

    fn jav321_detail_html() -> &'static str {
        r#"
<!doctype html>
<html>
<head><title>SSNI-644 Jav321 Title</title></head>
<body>
  <a href="//jav321.example/video/ssni00644">简体中文</a>
  <h3>SSNI-644 Jav321 Title <small>HD</small></h3>
  <div class="row">
    <div class="col-md-3">
      <img class="img-responsive" src="//img.example/cover-small.jpg">
      <img class="img-responsive" src="//img.example/sample-one.jpg">
    </div>
    <div class="col-md-9">
      <b>品番</b>: SSNI-644<br>
      <b>配信開始日</b>: 2024-05-03<br>
      <b>収録時間</b>: 121 分<br>
      <b>平均評価</b>: <img data-original="/img/42.gif"><br>
      <p class="summary">Jav321 synthetic outline with enough detail to be treated as plot text.</p>
      <a href="/star/one">Actor One</a>
      <a href="/heyzo_star/two">Actor Two</a>
      <a href="/company/studio-alpha">Studio Alpha</a>
      <a href="/series/series-gamma">Series Gamma</a>
      <a href="/genre/drama">Drama</a>
      <a href="/genre/uniform">Uniform</a>
    </div>
  </div>
</body>
</html>"#
    }

    #[derive(Clone, Default)]
    struct FakeTransport {
        responses: Arc<Mutex<VecDeque<ProviderHttpResult<ProviderHttpResponse>>>>,
        requests: Arc<Mutex<Vec<ProviderHttpRequest>>>,
    }

    impl FakeTransport {
        fn push(&self, response: ProviderHttpResult<ProviderHttpResponse>) {
            self.responses.lock().unwrap().push_back(response);
        }

        fn requests(&self) -> Vec<ProviderHttpRequest> {
            self.requests.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl ProviderHttpTransport for FakeTransport {
        async fn send(
            &self,
            request: ProviderHttpRequest,
            _config: ProviderHttpRuntimeConfig,
        ) -> ProviderHttpResult<ProviderHttpResponse> {
            self.requests.lock().unwrap().push(request);
            self.responses
                .lock()
                .unwrap()
                .pop_front()
                .unwrap_or_else(|| {
                    Err(
                        crate::providers::http_runtime::ProviderHttpError::Transport {
                            provider_id: JAV321_PROVIDER_ID,
                            operation: "fake",
                            message: "fake transport response queue was empty".to_owned(),
                            attempts: 0,
                        },
                    )
                })
        }
    }
}
