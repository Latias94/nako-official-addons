use async_trait::async_trait;
use nako_addon_protocol::{AddonArtworkKind, AddonMetadataPatch};
use serde::{Deserialize, Deserializer};

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

pub const PRESTIGE_PROVIDER_ID: &str = "prestige";
const PRESTIGE_URL_EXTERNAL_ID_PROVIDER: &str = "prestige_url";
const PRESTIGE_EXTERNAL_ID_CAPABILITIES: &[ProviderExternalIdCapability] = &[
    ProviderExternalIdCapability::new(
        PRESTIGE_PROVIDER_ID,
        ExternalIdValueKind::Opaque,
        true,
        true,
        &["prestige_id"],
        false,
    ),
    ProviderExternalIdCapability::new(
        PRESTIGE_URL_EXTERNAL_ID_PROVIDER,
        ExternalIdValueKind::Url,
        true,
        true,
        &["prestige_url"],
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
pub struct PrestigeProviderConfig {
    pub(crate) base_url: String,
    pub(crate) timeout_ms: u64,
    pub(crate) proxy_url: Option<String>,
}

impl PrestigeProviderConfig {
    pub const DEFAULT_TIMEOUT_MS: u64 = 10_000;

    #[must_use]
    pub fn from_env_lookup(mut lookup: impl FnMut(&str) -> Option<String>) -> Self {
        Self {
            base_url: lookup("NAKO_METADATA_SCRAPER_PRESTIGE_BASE_URL")
                .and_then(non_empty_trimmed)
                .unwrap_or_else(|| "https://www.prestige-av.com".to_owned()),
            timeout_ms: lookup("NAKO_METADATA_SCRAPER_PRESTIGE_TIMEOUT_MS")
                .and_then(|value| value.trim().parse::<u64>().ok())
                .filter(|value| *value > 0)
                .unwrap_or(Self::DEFAULT_TIMEOUT_MS),
            proxy_url: lookup("NAKO_METADATA_SCRAPER_PRESTIGE_PROXY_URL")
                .and_then(non_empty_trimmed),
        }
    }
}

#[must_use]
pub(crate) fn catalog_entry() -> ProviderCatalogEntry {
    ProviderCatalogEntry {
        id: ProviderId::Prestige,
        default_enabled: false,
        enabled_env_var: "NAKO_METADATA_SCRAPER_PROVIDER_PRESTIGE_ENABLED",
        capabilities: &[
            "metadata_suggestion",
            "av_number_search",
            "prestige_direct_lookup",
            "prestige_movie_search",
            "prestige_official_api",
        ],
        field_quality: crate::engine::ProviderFieldQualityDescriptor::new(650, 450, 650, 650),
        secret_reference: None,
        external_id_capabilities: PRESTIGE_EXTERNAL_ID_CAPABILITIES,
        load_config: load_config,
        proxy_configured: prestige_proxy_configured,
        network_policy_key: Some("prestige_proxy_configured"),
        rendered_page_support: None,
        render_drift_case: None,
        build: build_provider,
    }
}

fn load_config(input: ProviderConfigInput<'_>) -> ProviderConfig {
    let lookup = input.lookup;
    ProviderConfig::prestige(
        input.enabled,
        PrestigeProviderConfig::from_env_lookup(|name| lookup(name)),
    )
}

fn prestige_proxy_configured(provider: &ProviderConfig) -> bool {
    provider
        .prestige_config()
        .and_then(|config| config.proxy_url.as_ref())
        .is_some()
}

fn build_provider(config: &Config) -> ProviderBuildStatus {
    let Some(provider_config) = config
        .provider_config(ProviderId::Prestige)
        .and_then(|provider| provider.prestige_config().cloned())
    else {
        return ProviderBuildStatus::Unavailable;
    };
    match PrestigeMetadataProvider::new(provider_config) {
        Ok(provider) => ProviderBuildStatus::Ready(Box::new(provider)),
        Err(_) => ProviderBuildStatus::Unavailable,
    }
}

#[derive(Clone, Debug)]
pub struct PrestigeMetadataProvider<T = ReqwestProviderHttpTransport>
where
    T: ProviderHttpTransport,
{
    config: PrestigeProviderConfig,
    runtime: ProviderHttpRuntime<T>,
}

impl PrestigeMetadataProvider<ReqwestProviderHttpTransport> {
    pub fn new(config: PrestigeProviderConfig) -> ProviderHttpResult<Self> {
        let runtime = ProviderHttpRuntime::new(runtime_config(&config))?;
        Ok(Self { config, runtime })
    }
}

impl<T> PrestigeMetadataProvider<T>
where
    T: ProviderHttpTransport,
{
    #[must_use]
    pub fn with_runtime(config: PrestigeProviderConfig, runtime: ProviderHttpRuntime<T>) -> Self {
        Self { config, runtime }
    }

    async fn suggest_candidates(
        &self,
        query: &MetadataQuery,
    ) -> anyhow::Result<Vec<ProviderMetadataCandidate>> {
        if let Some(id) = rendered_av::direct_external_id(query, PRESTIGE_URL_EXTERNAL_ID_PROVIDER)
            .and_then(|value| normalize_prestige_id(&value))
        {
            return self
                .detail_candidates(&id, facts_from_query(query), query)
                .await;
        }

        if let Some(id) = rendered_av::direct_external_id(query, PRESTIGE_PROVIDER_ID)
            .and_then(|value| normalize_prestige_id(&value))
        {
            return self
                .detail_candidates(&id, facts_from_query(query), query)
                .await;
        }

        let Some(av) = facts_from_query(query) else {
            return Ok(Vec::new());
        };
        if !self.supports_av_route(av.route) {
            return Ok(Vec::new());
        }

        let search = self.search_products(&av).await?;
        let Some(product_uuid) = search.matching_product_uuid(&av) else {
            return Ok(Vec::new());
        };

        self.detail_candidates(&product_uuid, Some(av), query).await
    }

    async fn search_products(&self, av: &AvQueryFacts) -> anyhow::Result<PrestigeSearchResponse> {
        let response = self
            .runtime
            .get_json(
                PRESTIGE_PROVIDER_ID,
                "search",
                self.search_url(),
                vec![
                    ("isEnabledQuery".to_owned(), "true".to_owned()),
                    ("searchText".to_owned(), av.number.clone()),
                    ("isEnableAggregation".to_owned(), "false".to_owned()),
                    ("release".to_owned(), "false".to_owned()),
                    ("reservation".to_owned(), "false".to_owned()),
                    ("soldOut".to_owned(), "false".to_owned()),
                    ("from".to_owned(), "0".to_owned()),
                    ("aggregationTermsSize".to_owned(), "0".to_owned()),
                    ("size".to_owned(), "20".to_owned()),
                ],
                json_headers(),
            )
            .await?;

        Ok(serde_json::from_value(response.body)?)
    }

    async fn detail_candidates(
        &self,
        product_uuid: &str,
        av: Option<AvQueryFacts>,
        query: &MetadataQuery,
    ) -> anyhow::Result<Vec<ProviderMetadataCandidate>> {
        let detail = self.product_detail(product_uuid).await?;
        Ok(detail
            .into_detail_facts(self, product_uuid, av)
            .map(|facts| vec![facts.into_candidate(query)])
            .unwrap_or_default())
    }

    async fn product_detail(&self, product_uuid: &str) -> anyhow::Result<PrestigeProductDetail> {
        let response = self
            .runtime
            .get_json(
                PRESTIGE_PROVIDER_ID,
                "product_detail",
                self.product_api_url(product_uuid),
                Vec::new(),
                json_headers(),
            )
            .await?;

        Ok(serde_json::from_value(response.body)?)
    }

    fn search_url(&self) -> String {
        format!("{}/api/search", self.config.base_url.trim_end_matches('/'))
    }

    fn product_api_url(&self, product_uuid: &str) -> String {
        format!(
            "{}/api/product/{}",
            self.config.base_url.trim_end_matches('/'),
            rendered_av::percent_encode(product_uuid)
        )
    }

    fn goods_url(&self, product_uuid: &str) -> String {
        format!(
            "{}/goods/{}",
            self.config.base_url.trim_end_matches('/'),
            rendered_av::percent_encode(product_uuid)
        )
    }

    fn media_url(&self, path: &str) -> String {
        let path = path.trim();
        if path.starts_with("http://") || path.starts_with("https://") || path.starts_with("//") {
            return rendered_av::normalize_url(path.to_owned());
        }

        let relative = path.trim_start_matches('/');
        if relative.starts_with("api/media/") {
            return format!(
                "{}/{}",
                self.config.base_url.trim_end_matches('/'),
                relative
            );
        }

        format!(
            "{}/api/media/{}",
            self.config.base_url.trim_end_matches('/'),
            relative
        )
    }
}

#[async_trait]
impl<T> MetadataProvider for PrestigeMetadataProvider<T>
where
    T: ProviderHttpTransport,
{
    fn id(&self) -> ProviderId {
        ProviderId::Prestige
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

#[derive(Debug, Deserialize)]
struct PrestigeSearchResponse {
    #[serde(default)]
    hits: PrestigeSearchHits,
}

impl PrestigeSearchResponse {
    fn matching_product_uuid(&self, av: &AvQueryFacts) -> Option<String> {
        self.hits.hits.iter().find_map(|hit| {
            hit.source
                .matches_av(av)
                .then(|| non_empty_string(hit.source.product_uuid.as_deref()))
                .flatten()
        })
    }
}

#[derive(Debug, Default, Deserialize)]
struct PrestigeSearchHits {
    #[serde(default)]
    hits: Vec<PrestigeSearchHit>,
}

#[derive(Debug, Deserialize)]
struct PrestigeSearchHit {
    #[serde(rename = "_source")]
    source: PrestigeSearchHitSource,
}

#[derive(Debug, Deserialize)]
struct PrestigeSearchHitSource {
    #[serde(rename = "productUuid")]
    product_uuid: Option<String>,
    #[serde(rename = "deliveryItemId")]
    delivery_item_id: Option<String>,
    title: Option<String>,
}

impl PrestigeSearchHitSource {
    fn matches_av(&self, av: &AvQueryFacts) -> bool {
        [self.delivery_item_id.as_deref(), self.title.as_deref()]
            .into_iter()
            .flatten()
            .any(|value| text_matches_av(value, av))
    }
}

#[derive(Debug, Deserialize)]
struct PrestigeProductDetail {
    #[serde(rename = "productUuid")]
    product_uuid: Option<String>,
    #[serde(rename = "deliveryItemId")]
    delivery_item_id: Option<String>,
    title: Option<String>,
    body: Option<String>,
    #[serde(default, deserialize_with = "deserialize_vec_or_default")]
    actress: Vec<PrestigeNamedValue>,
    thumbnail: Option<PrestigePathValue>,
    #[serde(rename = "packageImage")]
    package_image: Option<PrestigePathValue>,
    #[serde(default, deserialize_with = "deserialize_vec_or_default")]
    sku: Vec<PrestigeSku>,
    #[serde(rename = "playTime")]
    play_time: Option<serde_json::Value>,
    series: Option<PrestigeNamedValue>,
    #[serde(default, deserialize_with = "deserialize_vec_or_default")]
    genre: Vec<PrestigeNamedValue>,
    #[serde(default, deserialize_with = "deserialize_vec_or_default")]
    directors: Vec<PrestigeNamedValue>,
    maker: Option<PrestigeNamedValue>,
    label: Option<PrestigeNamedValue>,
    #[serde(default, deserialize_with = "deserialize_vec_or_default")]
    media: Vec<PrestigePathValue>,
    movie: Option<PrestigePathValue>,
}

impl PrestigeProductDetail {
    fn into_detail_facts<T>(
        self,
        provider: &PrestigeMetadataProvider<T>,
        product_uuid_hint: &str,
        av_hint: Option<AvQueryFacts>,
    ) -> Option<PrestigeDetailFacts>
    where
        T: ProviderHttpTransport,
    {
        let product_uuid = non_empty_string(self.product_uuid.as_deref())
            .unwrap_or_else(|| product_uuid_hint.trim().to_owned());
        let title = non_empty_string(self.title.as_deref())?;
        let sku_id = self
            .sku
            .iter()
            .find_map(|sku| non_empty_string(sku.sku_id.as_deref()));
        let av = [
            self.delivery_item_id.as_deref(),
            sku_id.as_deref(),
            Some(title.as_str()),
        ]
        .into_iter()
        .flatten()
        .find_map(|value| facts_from_text(value, AvNumberSource::ExternalId))
        .or(av_hint)?;
        if av.route != AvNumberRoute::Censored {
            return None;
        }

        let release_date = self.sku.iter().find_map(|sku| {
            sku.sales_start_at
                .as_deref()
                .and_then(first_iso_date_prefix)
        });
        let release_year = release_date.as_deref().and_then(rendered_av::first_year);
        let runtime_minutes = self.play_time.as_ref().and_then(json_u32);
        let actors = named_values(self.actress.iter());
        let genres = named_values(self.genre.iter());
        let directors = named_values(self.directors.iter());
        let series = self.series.as_ref().and_then(PrestigeNamedValue::value);
        let maker = self.maker.as_ref().and_then(PrestigeNamedValue::value);
        let label = self.label.as_ref().and_then(PrestigeNamedValue::value);
        let thumb_url = self
            .thumbnail
            .as_ref()
            .and_then(PrestigePathValue::path)
            .map(|path| provider.media_url(&path));
        let poster_url = self
            .package_image
            .as_ref()
            .and_then(PrestigePathValue::path)
            .map(|path| provider.media_url(&path))
            .or_else(|| thumb_url.clone());
        let mut backdrop_urls = self
            .media
            .iter()
            .filter_map(PrestigePathValue::path)
            .map(|path| provider.media_url(&path))
            .collect::<Vec<_>>();
        if let Some(thumb_url) = &thumb_url
            && poster_url.as_ref() != Some(thumb_url)
        {
            push_unique(&mut backdrop_urls, thumb_url.clone());
        }
        let trailer_url = self
            .movie
            .as_ref()
            .and_then(PrestigePathValue::path)
            .map(|path| provider.media_url(&path));

        Some(PrestigeDetailFacts {
            url: provider.goods_url(&product_uuid),
            product_uuid,
            av,
            title,
            overview: non_empty_string(self.body.as_deref()),
            release_date,
            release_year,
            runtime_minutes,
            actors,
            genres,
            directors,
            series,
            maker,
            label,
            thumb_url,
            poster_url,
            backdrop_urls,
            trailer_url,
        })
    }
}

#[derive(Debug, Deserialize)]
struct PrestigeNamedValue {
    name: Option<String>,
    value: Option<String>,
    #[serde(rename = "displayName")]
    display_name: Option<String>,
}

impl PrestigeNamedValue {
    fn value(&self) -> Option<String> {
        non_empty_string(self.name.as_deref())
            .or_else(|| non_empty_string(self.value.as_deref()))
            .or_else(|| non_empty_string(self.display_name.as_deref()))
    }
}

#[derive(Debug, Deserialize)]
struct PrestigePathValue {
    path: Option<String>,
}

impl PrestigePathValue {
    fn path(&self) -> Option<String> {
        non_empty_string(self.path.as_deref())
    }
}

#[derive(Debug, Deserialize)]
struct PrestigeSku {
    #[serde(rename = "skuId")]
    sku_id: Option<String>,
    #[serde(rename = "salesStartAt")]
    sales_start_at: Option<String>,
}

struct PrestigeDetailFacts {
    product_uuid: String,
    url: String,
    av: AvQueryFacts,
    title: String,
    overview: Option<String>,
    release_date: Option<String>,
    release_year: Option<i32>,
    runtime_minutes: Option<u32>,
    actors: Vec<String>,
    genres: Vec<String>,
    directors: Vec<String>,
    series: Option<String>,
    maker: Option<String>,
    label: Option<String>,
    thumb_url: Option<String>,
    poster_url: Option<String>,
    backdrop_urls: Vec<String>,
    trailer_url: Option<String>,
}

impl PrestigeDetailFacts {
    fn into_candidate(self, query: &MetadataQuery) -> ProviderMetadataCandidate {
        let mut tags = vec![
            PRESTIGE_PROVIDER_ID.to_owned(),
            format!("av_number:{}", self.av.number),
            format!("av_route:{:?}", self.av.route).to_ascii_lowercase(),
        ];
        tags.extend(self.actors.iter().map(|actor| format!("actor:{actor}")));
        for director in &self.directors {
            tags.push(format!("director:{director}"));
        }
        if let Some(maker) = &self.maker {
            tags.push(format!("maker:{maker}"));
        }
        if let Some(label) = &self.label {
            tags.push(format!("label:{label}"));
        }
        if let Some(series) = &self.series {
            tags.push(format!("series:{series}"));
        }

        let mut artwork_candidates = Vec::new();
        if let Some(poster_url) = self.poster_url.clone() {
            artwork_candidates.push(prestige_artwork_candidate(
                &self.product_uuid,
                AddonArtworkKind::Poster,
                poster_url,
                0,
            ));
        }
        for (index, url) in self.backdrop_urls.iter().cloned().enumerate() {
            artwork_candidates.push(prestige_artwork_candidate(
                &self.product_uuid,
                AddonArtworkKind::Backdrop,
                url,
                index + 1,
            ));
        }

        ProviderMetadataCandidate {
            provider: PRESTIGE_PROVIDER_ID.to_owned(),
            provider_id: format!("prestige:product:{}", self.product_uuid),
            patch: AddonMetadataPatch {
                title: Some(self.title.clone()),
                original_title: None,
                sort_title: Some(self.title.clone()),
                overview: self.overview.clone(),
                release_date: self.release_date.clone(),
                runtime_minutes: self.runtime_minutes,
                tagline: Some("Prestige AV title".to_owned()),
                genres: Some(self.genres.clone()).filter(|genres| !genres.is_empty()),
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
                    directors: self.directors.clone(),
                    series: self.series.clone(),
                    studio: self.maker.clone(),
                    publisher: self.label.clone(),
                    maker: self.maker.clone(),
                    label: self.label.clone(),
                    wanted_count: None,
                    thumb_url: self.thumb_url.clone().or_else(|| self.poster_url.clone()),
                    trailer_url: self.trailer_url.clone(),
                    extrafanart_urls: self.backdrop_urls.clone(),
                }
                .non_empty(),
                community_score_milli: None,
                community_vote_count: None,
                external_ids: vec![
                    ProviderExternalId {
                        provider: PRESTIGE_PROVIDER_ID.to_owned(),
                        value: self.product_uuid,
                    },
                    ProviderExternalId {
                        provider: PRESTIGE_URL_EXTERNAL_ID_PROVIDER.to_owned(),
                        value: self.url,
                    },
                    ProviderExternalId {
                        provider: AV_NUMBER_EXTERNAL_ID_PROVIDER.to_owned(),
                        value: self.av.number,
                    },
                ],
                provider_outcomes: vec![ProviderOutcome::PrestigeOfficialApiParsed],
                provider_note: None,
            },
            artwork_candidates,
        }
    }
}

fn runtime_config(config: &PrestigeProviderConfig) -> ProviderHttpRuntimeConfig {
    ProviderHttpRuntimeConfig {
        timeout_ms: config.timeout_ms,
        proxy_url: config.proxy_url.clone(),
        ..ProviderHttpRuntimeConfig::default()
    }
}

fn json_headers() -> Vec<(String, String)> {
    vec![("accept".to_owned(), "application/json".to_owned())]
}

fn normalize_prestige_id(value: &str) -> Option<String> {
    product_id_from_url(value).or_else(|| non_empty_string(Some(value.trim().trim_matches('/'))))
}

fn product_id_from_url(value: &str) -> Option<String> {
    ["/api/product/", "/goods/"].into_iter().find_map(|marker| {
        let start = value.find(marker)? + marker.len();
        let rest = &value[start..];
        let end = rest.find(['/', '?', '#', '&']).unwrap_or(rest.len());
        non_empty_string(Some(&rest[..end]))
    })
}

fn text_matches_av(value: &str, av: &AvQueryFacts) -> bool {
    facts_from_text(value, AvNumberSource::ExternalId)
        .is_some_and(|facts| facts.number.eq_ignore_ascii_case(&av.number))
        || compact(value).contains(&compact(&av.number))
}

fn compact(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .map(|character| character.to_ascii_uppercase())
        .collect()
}

fn named_values<'a>(values: impl IntoIterator<Item = &'a PrestigeNamedValue>) -> Vec<String> {
    values
        .into_iter()
        .filter_map(PrestigeNamedValue::value)
        .fold(Vec::new(), |mut result, value| {
            push_unique(&mut result, value);
            result
        })
}

fn push_unique(values: &mut Vec<String>, value: String) {
    if !value.trim().is_empty() && !values.iter().any(|existing| existing == &value) {
        values.push(value);
    }
}

fn first_iso_date_prefix(value: &str) -> Option<String> {
    let value = value.trim();
    let Some(prefix) = value.get(..10) else {
        return rendered_av::first_iso_date(value);
    };
    if prefix.as_bytes().get(4) == Some(&b'-')
        && prefix.as_bytes().get(7) == Some(&b'-')
        && prefix
            .chars()
            .enumerate()
            .all(|(index, character)| matches!(index, 4 | 7) || character.is_ascii_digit())
    {
        return Some(prefix.to_owned());
    }
    rendered_av::first_iso_date(value)
}

fn json_u32(value: &serde_json::Value) -> Option<u32> {
    value
        .as_u64()
        .and_then(|value| u32::try_from(value).ok())
        .or_else(|| {
            value
                .as_str()
                .and_then(|value| value.trim().parse::<u32>().ok())
        })
}

fn non_empty_string(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

fn prestige_artwork_candidate(
    product_uuid: &str,
    kind: AddonArtworkKind,
    source_url: String,
    index: usize,
) -> ProviderArtworkCandidate {
    ProviderArtworkCandidate {
        provider: PRESTIGE_PROVIDER_ID.to_owned(),
        provider_id: format!("prestige:product:{product_uuid}:artwork:{index}"),
        facts: ProviderArtworkCandidateFacts {
            kind,
            source_url,
            language: None,
            width: None,
            height: None,
        },
    }
}

fn deserialize_vec_or_default<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Ok(Option::<Vec<T>>::deserialize(deserializer)?.unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use std::{
        collections::VecDeque,
        sync::{Arc, Mutex},
    };

    use async_trait::async_trait;
    use serde_json::json;

    use crate::{
        engine::{MetadataQuery, QueryExternalId},
        providers::http_runtime::{
            ProviderHttpError, ProviderHttpRequest, ProviderHttpResponse, ProviderHttpResult,
            ProviderHttpRuntimeConfig,
        },
    };

    use super::*;

    #[tokio::test]
    async fn prestige_provider_searches_api_and_maps_product_detail() {
        let transport = FakeTransport::default();
        transport.push_json(json!({
            "hits": {
                "hits": [
                    {
                        "_source": {
                            "productUuid": "uuid-other",
                            "deliveryItemId": "ABW-999",
                            "title": "Other Product"
                        }
                    },
                    {
                        "_source": {
                            "productUuid": "uuid-001",
                            "deliveryItemId": "ABW-001",
                            "title": "ABW-001 Search Title"
                        }
                    }
                ]
            }
        }));
        transport.push_json(product_detail_json());
        let provider = provider_with_transport(transport.clone());

        let candidates = provider
            .suggest(&MetadataQuery {
                title: "ABW-001.mp4".to_owned(),
                year: None,
                language: "ja-JP".to_owned(),
                external_ids: Vec::new(),
            })
            .await
            .unwrap();

        assert_eq!(candidates.len(), 1);
        let candidate = &candidates[0];
        assert_eq!(candidate.provider, "prestige");
        assert_eq!(candidate.provider_id, "prestige:product:uuid-001");
        assert_eq!(
            candidate.patch.title.as_deref(),
            Some("ABW-001 Prestige Official Title")
        );
        assert_eq!(
            candidate.patch.overview.as_deref(),
            Some("Prestige official outline.")
        );
        assert_eq!(candidate.patch.release_date.as_deref(), Some("2024-06-09"));
        assert_eq!(candidate.patch.runtime_minutes, Some(121));
        assert_eq!(
            candidate.patch.genres.as_deref(),
            Some(["Drama".to_owned(), "Uniform".to_owned()].as_slice())
        );
        let av = candidate.facts.av.as_ref().unwrap();
        assert_eq!(av.actors, vec!["Actress One", "Actress Two"]);
        assert_eq!(av.directors, vec!["Director One"]);
        assert_eq!(av.series.as_deref(), Some("Prestige Series"));
        assert_eq!(av.studio.as_deref(), Some("Prestige Maker"));
        assert_eq!(av.publisher.as_deref(), Some("Prestige Label"));
        assert_eq!(
            av.thumb_url.as_deref(),
            Some("https://prestige.example/api/media/thumb/abw001.jpg")
        );
        assert_eq!(
            av.trailer_url.as_deref(),
            Some("https://prestige.example/api/media/movie/abw001.mp4")
        );
        assert_eq!(
            av.extrafanart_urls,
            vec![
                "https://prestige.example/api/media/sample/abw001-1.jpg".to_owned(),
                "https://image.example/sample2.jpg".to_owned(),
                "https://prestige.example/api/media/thumb/abw001.jpg".to_owned(),
            ]
        );
        assert_eq!(candidate.artwork_candidates.len(), 4);
        assert_eq!(
            candidate.artwork_candidates[0].facts.kind,
            AddonArtworkKind::Poster
        );
        assert_eq!(
            candidate.artwork_candidates[0].facts.source_url,
            "https://prestige.example/api/media/package/abw001.jpg"
        );
        assert!(candidate.facts.external_ids.contains(&ProviderExternalId {
            provider: "prestige".to_owned(),
            value: "uuid-001".to_owned(),
        }));
        assert!(candidate.facts.external_ids.contains(&ProviderExternalId {
            provider: "prestige_url".to_owned(),
            value: "https://prestige.example/goods/uuid-001".to_owned(),
        }));
        assert!(candidate.facts.external_ids.contains(&ProviderExternalId {
            provider: "av_number".to_owned(),
            value: "ABW-001".to_owned(),
        }));
        assert_eq!(
            candidate.facts.provider_outcomes,
            vec![ProviderOutcome::PrestigeOfficialApiParsed]
        );

        let requests = transport.requests();
        assert_eq!(requests.len(), 2);
        assert_eq!(requests[0].url, "https://prestige.example/api/search");
        assert!(
            requests[0]
                .query
                .contains(&("searchText".to_owned(), "ABW-001".to_owned()))
        );
        assert_eq!(
            requests[1].url,
            "https://prestige.example/api/product/uuid-001"
        );
        assert_eq!(
            transport.configs()[0].proxy_url.as_deref(),
            Some("http://proxy.example:8080")
        );
    }

    #[tokio::test]
    async fn prestige_provider_uses_explicit_id_for_direct_detail_lookup() {
        let transport = FakeTransport::default();
        transport.push_json(product_detail_json());
        let provider = provider_with_transport(transport.clone());

        let candidates = provider
            .suggest(&MetadataQuery {
                title: "Manual lookup".to_owned(),
                year: None,
                language: "ja-JP".to_owned(),
                external_ids: vec![QueryExternalId {
                    provider: "prestige".to_owned(),
                    value: "uuid-001".to_owned(),
                }],
            })
            .await
            .unwrap();

        assert_eq!(candidates.len(), 1);
        let requests = transport.requests();
        assert_eq!(requests.len(), 1);
        assert_eq!(
            requests[0].url,
            "https://prestige.example/api/product/uuid-001"
        );
    }

    #[tokio::test]
    async fn prestige_provider_parses_explicit_url_for_direct_detail_lookup() {
        let transport = FakeTransport::default();
        transport.push_json(product_detail_json());
        let provider = provider_with_transport(transport.clone());

        let candidates = provider
            .suggest(&MetadataQuery {
                title: "Manual lookup".to_owned(),
                year: None,
                language: "ja-JP".to_owned(),
                external_ids: vec![QueryExternalId {
                    provider: "prestige_url".to_owned(),
                    value: "https://www.prestige-av.com/goods/uuid-001?locale=ja".to_owned(),
                }],
            })
            .await
            .unwrap();

        assert_eq!(candidates.len(), 1);
        assert_eq!(
            transport.requests()[0].url,
            "https://prestige.example/api/product/uuid-001"
        );
    }

    #[tokio::test]
    async fn prestige_provider_skips_non_censored_av_routes() {
        let transport = FakeTransport::default();
        let provider = provider_with_transport(transport.clone());

        let candidates = provider
            .suggest(&MetadataQuery {
                title: "FC2PPV-1723984.mp4".to_owned(),
                year: None,
                language: "ja-JP".to_owned(),
                external_ids: Vec::new(),
            })
            .await
            .unwrap();

        assert!(candidates.is_empty());
        assert!(transport.requests().is_empty());
        assert!(!provider.supports_av_route(AvNumberRoute::Fc2));
        assert!(provider.supports_av_route(AvNumberRoute::Censored));
    }

    fn product_detail_json() -> serde_json::Value {
        json!({
            "productUuid": "uuid-001",
            "deliveryItemId": "ABW-001",
            "title": "ABW-001 Prestige Official Title",
            "body": "Prestige official outline.",
            "actress": [
                { "name": "Actress One" },
                { "name": "Actress Two" },
                { "name": "Actress One" }
            ],
            "thumbnail": { "path": "thumb/abw001.jpg" },
            "packageImage": { "path": "package/abw001.jpg" },
            "sku": [
                {
                    "skuId": "ABW-001",
                    "salesStartAt": "2024-06-09T00:00:00+09:00"
                }
            ],
            "playTime": 121,
            "series": { "name": "Prestige Series" },
            "genre": [{ "name": "Drama" }, { "name": "Uniform" }],
            "directors": [{ "name": "Director One" }],
            "maker": { "name": "Prestige Maker" },
            "label": { "name": "Prestige Label" },
            "media": [
                { "path": "sample/abw001-1.jpg" },
                { "path": "https://image.example/sample2.jpg" }
            ],
            "movie": { "path": "movie/abw001.mp4" }
        })
    }

    fn provider_with_transport(
        transport: FakeTransport,
    ) -> PrestigeMetadataProvider<FakeTransport> {
        let config = PrestigeProviderConfig {
            base_url: "https://prestige.example".to_owned(),
            timeout_ms: 1234,
            proxy_url: Some("http://proxy.example:8080".to_owned()),
        };
        let runtime = ProviderHttpRuntime::with_transport(
            ProviderHttpRuntimeConfig {
                retry_backoff_ms: 0,
                ..runtime_config(&config)
            },
            transport,
        );
        PrestigeMetadataProvider::with_runtime(config, runtime)
    }

    #[derive(Clone, Default)]
    struct FakeTransport {
        responses: Arc<Mutex<VecDeque<ProviderHttpResult<ProviderHttpResponse>>>>,
        requests: Arc<Mutex<Vec<ProviderHttpRequest>>>,
        configs: Arc<Mutex<Vec<ProviderHttpRuntimeConfig>>>,
    }

    impl FakeTransport {
        fn push_json(&self, body: serde_json::Value) {
            self.responses
                .lock()
                .unwrap()
                .push_back(Ok(ProviderHttpResponse {
                    status: 200,
                    body: serde_json::to_vec(&body).unwrap(),
                }));
        }

        fn requests(&self) -> Vec<ProviderHttpRequest> {
            self.requests.lock().unwrap().clone()
        }

        fn configs(&self) -> Vec<ProviderHttpRuntimeConfig> {
            self.configs.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl ProviderHttpTransport for FakeTransport {
        async fn send(
            &self,
            request: ProviderHttpRequest,
            config: ProviderHttpRuntimeConfig,
        ) -> ProviderHttpResult<ProviderHttpResponse> {
            self.requests.lock().unwrap().push(request);
            self.configs.lock().unwrap().push(config);
            self.responses
                .lock()
                .unwrap()
                .pop_front()
                .unwrap_or_else(|| {
                    Err(ProviderHttpError::Transport {
                        provider_id: "prestige",
                        operation: "fake",
                        message: "fake transport response queue was empty".to_owned(),
                        attempts: 0,
                    })
                })
        }
    }
}
