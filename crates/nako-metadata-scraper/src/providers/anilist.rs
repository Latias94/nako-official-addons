use async_trait::async_trait;
use nako_addon_protocol::{
    AddonArtworkKind, AddonMetadataPatch, AddonMetadataStudio, AddonSecretReferenceFieldDeclaration,
};
use serde::{Deserialize, Serialize};

use crate::{
    Config,
    config::{ProviderConfig, ProviderId, non_empty_trimmed, parse_bool},
    engine::{
        ExternalIdValueKind, MetadataQuery, ProviderArtworkCandidate,
        ProviderArtworkCandidateFacts, ProviderCandidateFacts, ProviderExternalId,
        ProviderExternalIdCapability, ProviderMetadataCandidate, ProviderOutcome,
    },
    providers::{
        MetadataProvider, ProviderBuildStatus, ProviderConfigInput,
        http_runtime::{
            ProviderHttpResult, ProviderHttpRuntime, ProviderHttpRuntimeConfig,
            ProviderHttpTransport, ReqwestProviderHttpTransport,
        },
        registry::ProviderCatalogEntry,
    },
};

pub const ANILIST_PROVIDER_ID: &str = "anilist";
const MAL_EXTERNAL_ID_PROVIDER_ID: &str = "mal";
const ANILIST_URL_EXTERNAL_ID_PROVIDER_ID: &str = "anilist_url";
const ANILIST_EXTERNAL_ID_CAPABILITIES: &[ProviderExternalIdCapability] = &[
    ProviderExternalIdCapability::new(
        ANILIST_PROVIDER_ID,
        ExternalIdValueKind::Numeric,
        true,
        true,
        &["anilist_id"],
        true,
    ),
    ProviderExternalIdCapability::new(
        MAL_EXTERNAL_ID_PROVIDER_ID,
        ExternalIdValueKind::Numeric,
        true,
        true,
        &["mal_id"],
        true,
    ),
    ProviderExternalIdCapability::new(
        ANILIST_URL_EXTERNAL_ID_PROVIDER_ID,
        ExternalIdValueKind::Url,
        false,
        true,
        &[],
        false,
    ),
];
const ANILIST_SEARCH_LIMIT: u8 = 3;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AniListProviderConfig {
    pub access_token: Option<String>,
    pub graphql_url: String,
    pub user_agent: String,
    pub include_adult: bool,
    pub timeout_ms: u64,
    pub proxy_url: Option<String>,
}

impl AniListProviderConfig {
    pub const DEFAULT_TIMEOUT_MS: u64 = 10_000;

    #[must_use]
    pub fn from_env_lookup(mut lookup: impl FnMut(&str) -> Option<String>) -> Self {
        Self {
            access_token: lookup("NAKO_METADATA_SCRAPER_ANILIST_ACCESS_TOKEN")
                .and_then(non_empty_trimmed),
            graphql_url: lookup("NAKO_METADATA_SCRAPER_ANILIST_GRAPHQL_URL")
                .and_then(non_empty_trimmed)
                .unwrap_or_else(|| "https://graphql.anilist.co".to_owned()),
            user_agent: lookup("NAKO_METADATA_SCRAPER_ANILIST_USER_AGENT")
                .and_then(non_empty_trimmed)
                .unwrap_or_else(Self::default_user_agent),
            include_adult: lookup("NAKO_METADATA_SCRAPER_ANILIST_INCLUDE_ADULT")
                .and_then(|value| parse_bool(&value))
                .unwrap_or(false),
            timeout_ms: lookup("NAKO_METADATA_SCRAPER_ANILIST_TIMEOUT_MS")
                .and_then(|value| value.trim().parse::<u64>().ok())
                .filter(|value| *value > 0)
                .unwrap_or(Self::DEFAULT_TIMEOUT_MS),
            proxy_url: lookup("NAKO_METADATA_SCRAPER_ANILIST_PROXY_URL")
                .and_then(non_empty_trimmed),
        }
    }

    #[must_use]
    pub fn default_user_agent() -> String {
        format!(
            "Latias94/nako-official-addons/nako-metadata-scraper/{} (https://github.com/Latias94/nako-official-addons)",
            env!("CARGO_PKG_VERSION")
        )
    }

    #[must_use]
    pub const fn secret_field_id() -> &'static str {
        "anilist_access_token"
    }
}

#[must_use]
pub(crate) fn catalog_entry() -> ProviderCatalogEntry {
    ProviderCatalogEntry {
        id: ProviderId::AniList,
        default_enabled: false,
        enabled_env_var: "NAKO_METADATA_SCRAPER_PROVIDER_ANILIST_ENABLED",
        capabilities: &["metadata_suggestion", "anime_search", "graphql_api"],
        field_quality: Default::default(),
        secret_reference: Some(AddonSecretReferenceFieldDeclaration::new(
            AniListProviderConfig::secret_field_id(),
            "AniList Access Token",
            Some(
                "Optional Secret Reference for an AniList access token. Public anime metadata works without it, but authenticated requests may raise API limits."
                    .to_owned(),
            ),
            false,
        )),
        external_id_capabilities: ANILIST_EXTERNAL_ID_CAPABILITIES,
        load_config: load_config,
        proxy_configured: anilist_proxy_configured,
        network_policy_key: Some("anilist_proxy_configured"),
        build: build_provider,
    }
}

fn load_config(input: ProviderConfigInput<'_>) -> ProviderConfig {
    let lookup = input.lookup;
    ProviderConfig::anilist(
        input.enabled,
        AniListProviderConfig::from_env_lookup(|name| lookup(name)),
    )
}

fn anilist_proxy_configured(provider: &ProviderConfig) -> bool {
    provider
        .anilist_config()
        .and_then(|config| config.proxy_url.as_ref())
        .is_some()
}

fn build_provider(config: &Config) -> ProviderBuildStatus {
    let Some(anilist_config) = config
        .provider_config(ProviderId::AniList)
        .and_then(|provider| provider.anilist_config().cloned())
    else {
        return ProviderBuildStatus::Unavailable;
    };
    match AniListMetadataProvider::new(anilist_config) {
        Ok(provider) => ProviderBuildStatus::Ready(Box::new(provider)),
        Err(_) => ProviderBuildStatus::Unavailable,
    }
}

#[derive(Clone, Debug)]
pub struct AniListMetadataProvider<T = ReqwestProviderHttpTransport>
where
    T: ProviderHttpTransport,
{
    config: AniListProviderConfig,
    runtime: ProviderHttpRuntime<T>,
}

impl AniListMetadataProvider<ReqwestProviderHttpTransport> {
    pub fn new(config: AniListProviderConfig) -> ProviderHttpResult<Self> {
        let runtime = ProviderHttpRuntime::new(ProviderHttpRuntimeConfig {
            timeout_ms: config.timeout_ms,
            user_agent: config.user_agent.clone(),
            proxy_url: config.proxy_url.clone(),
            ..ProviderHttpRuntimeConfig::default()
        })?;
        Ok(Self { config, runtime })
    }
}

impl<T> AniListMetadataProvider<T>
where
    T: ProviderHttpTransport,
{
    #[cfg(test)]
    #[must_use]
    fn with_runtime(config: AniListProviderConfig, runtime: ProviderHttpRuntime<T>) -> Self {
        Self { config, runtime }
    }

    async fn fetch_media_by_id(&self, id: u64) -> anyhow::Result<Option<AniListMedia>> {
        let response = self
            .graphql(
                anilist_media_by_id_query(),
                serde_json::json!({ "id": id }),
                "media by AniList ID",
            )
            .await?;
        AniListMediaResponse::from_value(response).map(|response| response.media)
    }

    async fn fetch_media_by_mal_id(&self, id_mal: u64) -> anyhow::Result<Option<AniListMedia>> {
        let response = self
            .graphql(
                anilist_media_by_mal_id_query(),
                serde_json::json!({ "idMal": id_mal }),
                "media by MAL ID",
            )
            .await?;
        AniListMediaResponse::from_value(response).map(|response| response.media)
    }

    async fn search_media(&self, query: &MetadataQuery) -> anyhow::Result<Vec<AniListMedia>> {
        let response = self
            .graphql(
                anilist_media_search_query(),
                serde_json::json!({
                    "search": query.title,
                    "perPage": ANILIST_SEARCH_LIMIT,
                    "isAdult": self.config.include_adult,
                }),
                "search anime media",
            )
            .await?;
        AniListSearchResponse::from_value(response).map(|response| response.media)
    }

    async fn graphql(
        &self,
        query: String,
        variables: serde_json::Value,
        operation: &'static str,
    ) -> anyhow::Result<serde_json::Value> {
        let response = self
            .runtime
            .post_json(
                ANILIST_PROVIDER_ID,
                operation,
                self.config.graphql_url.clone(),
                Vec::new(),
                self.headers(),
                &GraphQlRequest { query, variables },
            )
            .await?;
        Ok(response.body)
    }

    fn headers(&self) -> Vec<(String, String)> {
        self.config
            .access_token
            .as_ref()
            .map(|token| vec![("authorization".to_owned(), format!("Bearer {token}"))])
            .unwrap_or_default()
    }
}

#[async_trait]
impl<T> MetadataProvider for AniListMetadataProvider<T>
where
    T: ProviderHttpTransport,
{
    fn id(&self) -> ProviderId {
        ProviderId::AniList
    }

    async fn suggest(
        &self,
        query: &MetadataQuery,
    ) -> anyhow::Result<Vec<ProviderMetadataCandidate>> {
        for media_id in query_anilist_ids(query) {
            if let Some(media) = self.fetch_media_by_id(media_id).await? {
                return Ok(media
                    .into_candidate(query, self.config.include_adult)
                    .into_iter()
                    .collect());
            }
        }
        for mal_id in query_mal_ids(query) {
            if let Some(media) = self.fetch_media_by_mal_id(mal_id).await? {
                return Ok(media
                    .into_candidate(query, self.config.include_adult)
                    .into_iter()
                    .collect());
            }
        }

        Ok(self
            .search_media(query)
            .await?
            .into_iter()
            .filter_map(|media| media.into_candidate(query, self.config.include_adult))
            .collect())
    }
}

fn query_anilist_ids(query: &MetadataQuery) -> impl Iterator<Item = u64> + '_ {
    query_external_numeric_ids(query, &[ANILIST_PROVIDER_ID, "anilist_id"])
}

fn query_mal_ids(query: &MetadataQuery) -> impl Iterator<Item = u64> + '_ {
    query_external_numeric_ids(query, &[MAL_EXTERNAL_ID_PROVIDER_ID, "mal_id"])
}

fn query_external_numeric_ids<'a>(
    query: &'a MetadataQuery,
    providers: &'static [&'static str],
) -> impl Iterator<Item = u64> + 'a {
    let mut seen = std::collections::HashSet::new();
    query
        .external_ids
        .iter()
        .filter(move |external_id| {
            providers
                .iter()
                .any(|provider| external_id.provider.eq_ignore_ascii_case(provider))
        })
        .filter_map(|external_id| external_id.value.trim().parse().ok())
        .filter(|id| *id > 0)
        .filter(move |id| seen.insert(*id))
}

#[derive(Debug, Serialize)]
struct GraphQlRequest {
    query: String,
    variables: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct GraphQlResponse<T> {
    data: Option<T>,
    #[serde(default)]
    errors: Vec<GraphQlError>,
}

#[derive(Debug, Deserialize)]
struct GraphQlError {
    message: Option<String>,
}

impl<T> GraphQlResponse<T> {
    fn into_data(self, label: &'static str) -> anyhow::Result<T> {
        if !self.errors.is_empty() {
            let messages = self
                .errors
                .into_iter()
                .filter_map(|error| error.message)
                .collect::<Vec<_>>()
                .join("; ");
            anyhow::bail!("AniList GraphQL {label} returned errors: {messages}");
        }
        self.data
            .ok_or_else(|| anyhow::anyhow!("AniList GraphQL {label} response missing data"))
    }
}

#[derive(Debug, Deserialize)]
struct AniListMediaResponse {
    #[serde(rename = "Media")]
    media: Option<AniListMedia>,
}

impl AniListMediaResponse {
    fn from_value(value: serde_json::Value) -> anyhow::Result<Self> {
        serde_json::from_value::<GraphQlResponse<Self>>(value)
            .map_err(|error| anyhow::anyhow!("failed to parse AniList media response: {error}"))?
            .into_data("media")
    }
}

#[derive(Debug, Deserialize)]
struct AniListSearchData {
    #[serde(rename = "Page")]
    page: AniListMediaPage,
}

#[derive(Debug, Deserialize)]
struct AniListMediaPage {
    #[serde(default)]
    media: Vec<AniListMedia>,
}

struct AniListSearchResponse {
    media: Vec<AniListMedia>,
}

impl AniListSearchResponse {
    fn from_value(value: serde_json::Value) -> anyhow::Result<Self> {
        let data = serde_json::from_value::<GraphQlResponse<AniListSearchData>>(value)
            .map_err(|error| anyhow::anyhow!("failed to parse AniList search response: {error}"))?
            .into_data("search")?;
        Ok(Self {
            media: data
                .page
                .media
                .into_iter()
                .filter(|media| media.id > 0)
                .collect(),
        })
    }
}

#[derive(Clone, Debug, Deserialize)]
struct AniListMedia {
    id: u64,
    #[serde(rename = "idMal")]
    id_mal: Option<u64>,
    title: Option<AniListTitle>,
    description: Option<String>,
    #[serde(rename = "siteUrl")]
    site_url: Option<String>,
    #[serde(rename = "startDate")]
    start_date: Option<AniListDate>,
    episodes: Option<u32>,
    duration: Option<u32>,
    #[serde(default)]
    genres: Vec<String>,
    #[serde(default)]
    tags: Vec<AniListTag>,
    #[serde(rename = "averageScore")]
    average_score: Option<u16>,
    #[serde(rename = "meanScore")]
    mean_score: Option<u16>,
    popularity: Option<u32>,
    favourites: Option<u32>,
    #[serde(rename = "coverImage")]
    cover_image: Option<AniListCoverImage>,
    #[serde(rename = "bannerImage")]
    banner_image: Option<String>,
    format: Option<String>,
    status: Option<String>,
    source: Option<String>,
    #[serde(rename = "countryOfOrigin")]
    country_of_origin: Option<String>,
    #[serde(rename = "isAdult")]
    is_adult: Option<bool>,
    studios: Option<AniListStudioConnection>,
}

impl AniListMedia {
    fn into_candidate(
        self,
        query: &MetadataQuery,
        include_adult: bool,
    ) -> Option<ProviderMetadataCandidate> {
        if self.id == 0 || (!include_adult && self.is_adult == Some(true)) {
            return None;
        }
        let artwork_candidates = self.artwork_candidates();
        let external_ids = self.external_ids();
        let title =
            selected_title(query, self.title.as_ref()).unwrap_or_else(|| query.title.clone());
        let original_title = self
            .title
            .as_ref()
            .and_then(|title| first_non_empty(&[title.romaji.as_deref(), title.native.as_deref()]));
        let release_date = self.start_date.as_ref().and_then(AniListDate::iso_date);
        let release_year = self
            .start_date
            .as_ref()
            .and_then(|date| date.year)
            .map(i32::from);
        let score = self.average_score.or(self.mean_score);
        let mut tags = vec!["anilist".to_owned()];
        push_prefixed_tag(&mut tags, "format", self.format.as_deref());
        push_prefixed_tag(&mut tags, "status", self.status.as_deref());
        push_prefixed_tag(&mut tags, "source", self.source.as_deref());
        push_prefixed_tag(&mut tags, "country", self.country_of_origin.as_deref());
        push_prefixed_number_tag(&mut tags, "episodes", self.episodes);
        push_prefixed_number_tag(&mut tags, "popularity", self.popularity);
        push_prefixed_number_tag(&mut tags, "favourites", self.favourites);
        for tag in self
            .tags
            .iter()
            .filter(|tag| {
                tag.is_media_spoiler != Some(true) && tag.is_general_spoiler != Some(true)
            })
            .filter(|tag| tag.rank.unwrap_or_default() >= 50)
            .take(8)
        {
            push_unique_normalized(&mut tags, tag.name.as_deref());
        }
        let genres = normalized_values(self.genres);
        let studios = self
            .studios
            .map(|studios| studios.into_studios())
            .filter(|studios| !studios.is_empty());

        Some(ProviderMetadataCandidate {
            provider: ANILIST_PROVIDER_ID.to_owned(),
            provider_id: format!("anilist:media:{}", self.id),
            patch: AddonMetadataPatch {
                title: Some(title.clone()),
                original_title: original_title.clone().filter(|original| original != &title),
                sort_title: Some(title.clone()),
                overview: self
                    .description
                    .and_then(|description| clean_description(&description)),
                release_date,
                runtime_minutes: self.duration,
                tagline: self.format.and_then(|format| normalize_non_empty(&format)),
                genres: Some(genres).filter(|genres| !genres.is_empty()),
                tags: Some(tags).filter(|tags| !tags.is_empty()),
                studios,
                ..AddonMetadataPatch::default()
            },
            facts: ProviderCandidateFacts {
                title: Some(title),
                alternate_titles: self
                    .title
                    .as_ref()
                    .map(|titles| titles.alternate_titles(original_title.as_deref()))
                    .unwrap_or_default(),
                release_year,
                language: Some(query.language.clone()),
                av: None,
                community_score_milli: score.map(|score| score.saturating_mul(10).min(1000)),
                community_vote_count: None,
                external_ids,
                provider_outcomes: vec![ProviderOutcome::AniListMediaMapped],
                provider_note: None,
            },
            artwork_candidates,
        })
    }

    fn external_ids(&self) -> Vec<ProviderExternalId> {
        let mut ids = vec![ProviderExternalId {
            provider: ANILIST_PROVIDER_ID.to_owned(),
            value: self.id.to_string(),
        }];
        if let Some(id_mal) = self.id_mal.filter(|id| *id > 0) {
            ids.push(ProviderExternalId {
                provider: MAL_EXTERNAL_ID_PROVIDER_ID.to_owned(),
                value: id_mal.to_string(),
            });
        }
        if let Some(site_url) = self.site_url.as_deref().and_then(normalize_non_empty) {
            ids.push(ProviderExternalId {
                provider: ANILIST_URL_EXTERNAL_ID_PROVIDER_ID.to_owned(),
                value: site_url,
            });
        }
        ids
    }

    fn artwork_candidates(&self) -> Vec<ProviderArtworkCandidate> {
        let provider_id = format!("anilist:media:{}", self.id);
        let mut candidates = Vec::new();
        if let Some(cover_image) = &self.cover_image {
            for value in [
                cover_image.extra_large.as_deref(),
                cover_image.large.as_deref(),
                cover_image.medium.as_deref(),
            ] {
                push_artwork_candidate(
                    &mut candidates,
                    &provider_id,
                    AddonArtworkKind::Poster,
                    value,
                );
            }
        }
        push_artwork_candidate(
            &mut candidates,
            &provider_id,
            AddonArtworkKind::Backdrop,
            self.banner_image.as_deref(),
        );
        candidates
    }
}

#[derive(Clone, Debug, Deserialize)]
struct AniListTitle {
    romaji: Option<String>,
    english: Option<String>,
    native: Option<String>,
    #[serde(rename = "userPreferred")]
    user_preferred: Option<String>,
}

impl AniListTitle {
    fn alternate_titles(&self, selected: Option<&str>) -> Vec<String> {
        let mut titles = Vec::new();
        for title in [
            self.romaji.as_deref(),
            self.english.as_deref(),
            self.native.as_deref(),
            self.user_preferred.as_deref(),
        ] {
            push_unique_title(&mut titles, selected, title);
        }
        titles
    }
}

#[derive(Clone, Debug, Deserialize)]
struct AniListDate {
    year: Option<u16>,
    month: Option<u8>,
    day: Option<u8>,
}

impl AniListDate {
    fn iso_date(&self) -> Option<String> {
        let year = self.year?;
        let month = self.month.filter(|month| (1..=12).contains(month))?;
        let day = self.day.filter(|day| (1..=31).contains(day))?;
        Some(format!("{year:04}-{month:02}-{day:02}"))
    }
}

#[derive(Clone, Debug, Deserialize)]
struct AniListTag {
    name: Option<String>,
    rank: Option<u8>,
    #[serde(rename = "isMediaSpoiler")]
    is_media_spoiler: Option<bool>,
    #[serde(rename = "isGeneralSpoiler")]
    is_general_spoiler: Option<bool>,
}

#[derive(Clone, Debug, Deserialize)]
struct AniListCoverImage {
    #[serde(rename = "extraLarge")]
    extra_large: Option<String>,
    large: Option<String>,
    medium: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
struct AniListStudioConnection {
    #[serde(default)]
    nodes: Vec<AniListStudio>,
}

impl AniListStudioConnection {
    fn into_studios(self) -> Vec<AddonMetadataStudio> {
        self.nodes
            .into_iter()
            .filter_map(|studio| studio.name.and_then(|name| normalize_non_empty(&name)))
            .fold(Vec::new(), |mut values, name| {
                if !values
                    .iter()
                    .any(|studio: &AddonMetadataStudio| studio.name == name)
                {
                    values.push(AddonMetadataStudio {
                        name,
                        external_ids: Vec::new(),
                    });
                }
                values
            })
    }
}

#[derive(Clone, Debug, Deserialize)]
struct AniListStudio {
    name: Option<String>,
}

fn selected_title(query: &MetadataQuery, title: Option<&AniListTitle>) -> Option<String> {
    let title = title?;
    for candidate in [
        title.user_preferred.as_deref(),
        title.english.as_deref(),
        title.romaji.as_deref(),
        title.native.as_deref(),
    ] {
        if title_matches(&query.title, candidate) {
            return candidate.and_then(normalize_non_empty);
        }
    }
    if query.language.to_ascii_lowercase().starts_with("en") {
        first_non_empty(&[
            title.english.as_deref(),
            title.user_preferred.as_deref(),
            title.romaji.as_deref(),
            title.native.as_deref(),
        ])
    } else {
        first_non_empty(&[
            title.user_preferred.as_deref(),
            title.romaji.as_deref(),
            title.english.as_deref(),
            title.native.as_deref(),
        ])
    }
}

fn title_matches(query_title: &str, candidate_title: Option<&str>) -> bool {
    let Some(candidate_title) = candidate_title.and_then(normalize_non_empty) else {
        return false;
    };
    normalize_title(query_title) == normalize_title(&candidate_title)
}

fn normalize_title(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn first_non_empty(values: &[Option<&str>]) -> Option<String> {
    values
        .iter()
        .flatten()
        .find_map(|value| normalize_non_empty(value))
}

fn normalized_values(values: Vec<String>) -> Vec<String> {
    values.into_iter().fold(Vec::new(), |mut values, value| {
        push_unique_normalized(&mut values, Some(&value));
        values
    })
}

fn push_unique_title(values: &mut Vec<String>, selected: Option<&str>, value: Option<&str>) {
    let Some(value) = value.and_then(normalize_non_empty) else {
        return;
    };
    if selected.is_some_and(|selected| selected == value)
        || values.iter().any(|item| item == &value)
    {
        return;
    }
    values.push(value);
}

fn push_prefixed_tag(tags: &mut Vec<String>, key: &str, value: Option<&str>) {
    let Some(value) = value.and_then(normalize_non_empty) else {
        return;
    };
    push_unique_normalized(tags, Some(&format!("anilist_{key}:{value}")));
}

fn push_prefixed_number_tag(tags: &mut Vec<String>, key: &str, value: Option<u32>) {
    if let Some(value) = value.filter(|value| *value > 0) {
        push_unique_normalized(tags, Some(&format!("anilist_{key}:{value}")));
    }
}

fn push_unique_normalized(values: &mut Vec<String>, value: Option<&str>) {
    let Some(value) = value.and_then(normalize_non_empty) else {
        return;
    };
    if !values.iter().any(|existing| existing == &value) {
        values.push(value);
    }
}

fn clean_description(value: &str) -> Option<String> {
    let mut out = String::new();
    let mut in_tag = false;
    for character in value.chars() {
        match character {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                out.push(' ');
            }
            _ if !in_tag => out.push(character),
            _ => {}
        }
    }
    normalize_non_empty(&out.split_whitespace().collect::<Vec<_>>().join(" "))
}

fn normalize_non_empty(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_owned())
}

fn push_artwork_candidate(
    candidates: &mut Vec<ProviderArtworkCandidate>,
    provider_id: &str,
    kind: AddonArtworkKind,
    source_url: Option<&str>,
) {
    let Some(source_url) = source_url.and_then(normalize_non_empty) else {
        return;
    };
    if candidates
        .iter()
        .any(|candidate| candidate.facts.source_url == source_url)
    {
        return;
    }
    candidates.push(ProviderArtworkCandidate {
        provider: ANILIST_PROVIDER_ID.to_owned(),
        provider_id: provider_id.to_owned(),
        facts: ProviderArtworkCandidateFacts {
            kind,
            source_url,
            language: None,
            width: None,
            height: None,
        },
    });
}

const ANILIST_MEDIA_FIELDS: &str = r#"
id
idMal
siteUrl
title { romaji english native userPreferred }
description(asHtml: false)
startDate { year month day }
episodes
duration
genres
tags { name rank isMediaSpoiler isGeneralSpoiler }
averageScore
meanScore
popularity
favourites
coverImage { extraLarge large medium }
bannerImage
format
status
source
countryOfOrigin
isAdult
studios(isMain: true) { nodes { name } }
"#;

fn anilist_media_by_id_query() -> String {
    format!(
        "query AniListMediaById($id: Int!) {{ Media(id: $id, type: ANIME) {{ {ANILIST_MEDIA_FIELDS} }} }}"
    )
}

fn anilist_media_by_mal_id_query() -> String {
    format!(
        "query AniListMediaByMalId($idMal: Int!) {{ Media(idMal: $idMal, type: ANIME) {{ {ANILIST_MEDIA_FIELDS} }} }}"
    )
}

fn anilist_media_search_query() -> String {
    format!(
        "query AniListMediaSearch($search: String!, $perPage: Int!, $isAdult: Boolean) {{ Page(page: 1, perPage: $perPage) {{ media(search: $search, type: ANIME, isAdult: $isAdult, sort: SEARCH_MATCH) {{ {ANILIST_MEDIA_FIELDS} }} }} }}"
    )
}

#[cfg(test)]
mod tests {
    use std::{
        collections::VecDeque,
        sync::{Arc, Mutex},
    };

    use crate::providers::http_runtime::{
        ProviderHttpRequest, ProviderHttpResponse, ProviderHttpRuntimeConfig,
    };

    use super::*;

    #[test]
    fn anilist_query_ids_accept_aliases_and_skip_invalid_values() {
        let query = MetadataQuery {
            title: "Cowboy Bebop".to_owned(),
            year: Some(1998),
            language: "en-US".to_owned(),
            external_ids: vec![
                crate::engine::QueryExternalId {
                    provider: "anilist_id".to_owned(),
                    value: "0".to_owned(),
                },
                crate::engine::QueryExternalId {
                    provider: "ANILIST".to_owned(),
                    value: "1".to_owned(),
                },
                crate::engine::QueryExternalId {
                    provider: "mal_id".to_owned(),
                    value: "1".to_owned(),
                },
                crate::engine::QueryExternalId {
                    provider: "mal".to_owned(),
                    value: "not-a-number".to_owned(),
                },
            ],
        };

        assert_eq!(query_anilist_ids(&query).collect::<Vec<_>>(), vec![1]);
        assert_eq!(query_mal_ids(&query).collect::<Vec<_>>(), vec![1]);
    }

    #[tokio::test]
    async fn anilist_provider_uses_query_external_id_for_direct_lookup() {
        let transport = FakeTransport::default();
        transport.push(media_response(1, 1));
        let provider = provider_with_transport(transport.clone());

        let candidates = provider
            .suggest(&MetadataQuery {
                title: "Cowboy Bebop".to_owned(),
                year: Some(1998),
                language: "en-US".to_owned(),
                external_ids: vec![crate::engine::QueryExternalId {
                    provider: "anilist".to_owned(),
                    value: "1".to_owned(),
                }],
            })
            .await
            .unwrap();

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].provider_id, "anilist:media:1");
        assert_eq!(candidates[0].patch.title.as_deref(), Some("Cowboy Bebop"));
        assert_eq!(
            candidates[0].patch.overview.as_deref(),
            Some("In the year 2071, bounty hunters travel through space.")
        );
        assert_eq!(
            candidates[0].patch.release_date.as_deref(),
            Some("1998-04-03")
        );
        assert_eq!(candidates[0].patch.runtime_minutes, Some(24));
        assert_eq!(candidates[0].facts.release_year, Some(1998));
        assert_eq!(candidates[0].facts.community_score_milli, Some(860));
        assert!(
            candidates[0]
                .facts
                .external_ids
                .iter()
                .any(|id| id.provider == "mal" && id.value == "1")
        );
        assert!(
            candidates[0]
                .patch
                .studios
                .as_ref()
                .unwrap()
                .iter()
                .any(|studio| studio.name == "Sunrise")
        );
        assert!(
            candidates[0]
                .artwork_candidates
                .iter()
                .any(|candidate| candidate.facts.kind == AddonArtworkKind::Poster
                    && candidate.facts.source_url == "https://img.example/cover.jpg")
        );

        let requests = transport.requests();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].url, "https://graphql.anilist.example");
        let body: serde_json::Value =
            serde_json::from_slice(requests[0].json_body.as_ref().unwrap()).unwrap();
        assert_eq!(body["variables"]["id"], 1);
    }

    #[tokio::test]
    async fn anilist_provider_searches_anime_media() {
        let transport = FakeTransport::default();
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: serde_json::json!({
                "data": {
                    "Page": {
                        "media": [media_json(1, 1)]
                    }
                }
            })
            .to_string()
            .into_bytes(),
        }));
        let provider = provider_with_transport(transport.clone());

        let candidates = provider
            .suggest(&MetadataQuery {
                title: "Bebop".to_owned(),
                year: Some(1998),
                language: "en-US".to_owned(),
                external_ids: Vec::new(),
            })
            .await
            .unwrap();

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].patch.title.as_deref(), Some("Cowboy Bebop"));

        let requests = transport.requests();
        let body: serde_json::Value =
            serde_json::from_slice(requests[0].json_body.as_ref().unwrap()).unwrap();
        assert_eq!(body["variables"]["search"], "Bebop");
        assert_eq!(body["variables"]["perPage"], ANILIST_SEARCH_LIMIT);
        assert_eq!(body["variables"]["isAdult"], false);
    }

    fn provider_with_transport(transport: FakeTransport) -> AniListMetadataProvider<FakeTransport> {
        let runtime = ProviderHttpRuntime::with_transport(
            ProviderHttpRuntimeConfig {
                retry_backoff_ms: 0,
                ..ProviderHttpRuntimeConfig::default()
            },
            transport,
        );
        AniListMetadataProvider::with_runtime(
            AniListProviderConfig {
                access_token: None,
                graphql_url: "https://graphql.anilist.example".to_owned(),
                user_agent: "Latias94/test-addon/0.1.0".to_owned(),
                include_adult: false,
                timeout_ms: 10_000,
                proxy_url: None,
            },
            runtime,
        )
    }

    fn media_response(id: u64, id_mal: u64) -> ProviderHttpResult<ProviderHttpResponse> {
        Ok(ProviderHttpResponse {
            status: 200,
            body: serde_json::json!({
                "data": {
                    "Media": media_json(id, id_mal)
                }
            })
            .to_string()
            .into_bytes(),
        })
    }

    fn media_json(id: u64, id_mal: u64) -> serde_json::Value {
        serde_json::json!({
            "id": id,
            "idMal": id_mal,
            "siteUrl": "https://anilist.co/anime/1/Cowboy-Bebop/",
            "title": {
                "romaji": "Cowboy Bebop",
                "english": "Cowboy Bebop",
                "native": "カウボーイビバップ",
                "userPreferred": "Cowboy Bebop"
            },
            "description": "In the year 2071, <br>bounty hunters travel through space.",
            "startDate": {"year": 1998, "month": 4, "day": 3},
            "episodes": 26,
            "duration": 24,
            "genres": ["Action", "Sci-Fi"],
            "tags": [
                {"name": "Space", "rank": 90, "isMediaSpoiler": false, "isGeneralSpoiler": false},
                {"name": "Spoiler", "rank": 90, "isMediaSpoiler": true, "isGeneralSpoiler": false}
            ],
            "averageScore": 86,
            "meanScore": 84,
            "popularity": 500000,
            "favourites": 30000,
            "coverImage": {
                "extraLarge": "https://img.example/cover.jpg",
                "large": "https://img.example/cover.jpg",
                "medium": "https://img.example/cover-small.jpg"
            },
            "bannerImage": "https://img.example/banner.jpg",
            "format": "TV",
            "status": "FINISHED",
            "source": "ORIGINAL",
            "countryOfOrigin": "JP",
            "isAdult": false,
            "studios": {
                "nodes": [{"name": "Sunrise"}]
            }
        })
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
                .unwrap_or_else(|| panic!("missing fake response"))
        }
    }
}
