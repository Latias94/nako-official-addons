use async_trait::async_trait;
use nako_addon_protocol::{
    AddonArtworkKind, AddonMetadataPatch, AddonSecretReferenceFieldDeclaration,
};
use serde::Deserialize;

use crate::{
    Config,
    config::{ProviderConfig, ProviderId, non_empty_trimmed},
    engine::{
        AvMetadataFacts, ExternalIdValueKind, MetadataQuery, ProviderArtworkCandidate,
        ProviderArtworkCandidateFacts, ProviderCandidateFacts, ProviderExternalId,
        ProviderExternalIdCapability, ProviderFieldQualityDescriptor, ProviderMetadataCandidate,
        ProviderOutcome,
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

pub const THEPORNDB_PROVIDER_ID: &str = "theporndb";
const THEPORNDB_URL_EXTERNAL_ID_PROVIDER: &str = "theporndb_url";
const FILE_OSHASH_EXTERNAL_ID_PROVIDER: &str = "file_oshash";
const FILE_PHASH_EXTERNAL_ID_PROVIDER: &str = "file_phash";
const THEPORNDB_EXTERNAL_ID_CAPABILITIES: &[ProviderExternalIdCapability] = &[
    ProviderExternalIdCapability::new(
        THEPORNDB_PROVIDER_ID,
        ExternalIdValueKind::Opaque,
        true,
        true,
        &["theporndb_id"],
        false,
    ),
    ProviderExternalIdCapability::new(
        THEPORNDB_URL_EXTERNAL_ID_PROVIDER,
        ExternalIdValueKind::Url,
        true,
        true,
        &["theporndb_url"],
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
    ProviderExternalIdCapability::new(
        FILE_OSHASH_EXTERNAL_ID_PROVIDER,
        ExternalIdValueKind::Opaque,
        true,
        true,
        &["file_oshash"],
        false,
    ),
    ProviderExternalIdCapability::new(
        FILE_PHASH_EXTERNAL_ID_PROVIDER,
        ExternalIdValueKind::Opaque,
        true,
        true,
        &["file_phash"],
        false,
    ),
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ThePornDbProviderConfig {
    pub(crate) api_token: Option<String>,
    pub(crate) api_base_url: String,
    pub(crate) public_base_url: String,
    pub(crate) timeout_ms: u64,
    pub(crate) proxy_url: Option<String>,
}

impl ThePornDbProviderConfig {
    pub const DEFAULT_TIMEOUT_MS: u64 = 10_000;

    #[must_use]
    pub fn from_env_lookup(mut lookup: impl FnMut(&str) -> Option<String>) -> Self {
        Self {
            api_token: lookup("NAKO_METADATA_SCRAPER_THEPORNDB_API_TOKEN")
                .and_then(non_empty_trimmed),
            api_base_url: lookup("NAKO_METADATA_SCRAPER_THEPORNDB_API_BASE_URL")
                .and_then(non_empty_trimmed)
                .unwrap_or_else(|| "https://api.theporndb.net".to_owned()),
            public_base_url: lookup("NAKO_METADATA_SCRAPER_THEPORNDB_PUBLIC_BASE_URL")
                .and_then(non_empty_trimmed)
                .unwrap_or_else(|| "https://theporndb.net".to_owned()),
            timeout_ms: lookup("NAKO_METADATA_SCRAPER_THEPORNDB_TIMEOUT_MS")
                .and_then(|value| value.trim().parse::<u64>().ok())
                .filter(|value| *value > 0)
                .unwrap_or(Self::DEFAULT_TIMEOUT_MS),
            proxy_url: lookup("NAKO_METADATA_SCRAPER_THEPORNDB_PROXY_URL")
                .and_then(non_empty_trimmed),
        }
    }

    #[must_use]
    pub const fn secret_field_id() -> &'static str {
        "theporndb_api_token"
    }

    #[must_use]
    pub fn has_api_token(&self) -> bool {
        self.api_token
            .as_ref()
            .is_some_and(|value| !value.trim().is_empty())
    }
}

#[must_use]
pub(crate) fn catalog_entry() -> ProviderCatalogEntry {
    ProviderCatalogEntry {
        id: ProviderId::ThePornDb,
        default_enabled: false,
        enabled_env_var: "NAKO_METADATA_SCRAPER_PROVIDER_THEPORNDB_ENABLED",
        capabilities: &[
            "metadata_suggestion",
            "av_number_search",
            "theporndb_scene_search",
            "theporndb_direct_lookup",
            "theporndb_scene_hash_lookup",
            "theporndb_official_api",
        ],
        field_quality: ProviderFieldQualityDescriptor::new(700, 700, 750, 700),
        secret_reference: Some(AddonSecretReferenceFieldDeclaration::new(
            ThePornDbProviderConfig::secret_field_id(),
            "ThePornDB API Token",
            Some(
                "Required Secret Reference for ThePornDB API access. The value is sent as an Authorization bearer token and is never emitted in diagnostics."
                    .to_owned(),
            ),
            true,
        )),
        external_id_capabilities: THEPORNDB_EXTERNAL_ID_CAPABILITIES,
        load_config,
        proxy_configured: theporndb_proxy_configured,
        network_policy_key: Some("theporndb_proxy_configured"),
        build: build_provider,
    }
}

fn load_config(input: ProviderConfigInput<'_>) -> ProviderConfig {
    let lookup = input.lookup;
    ProviderConfig::theporndb(
        input.enabled,
        ThePornDbProviderConfig::from_env_lookup(|name| lookup(name)),
    )
}

fn theporndb_proxy_configured(provider: &ProviderConfig) -> bool {
    provider
        .theporndb_config()
        .and_then(|config| config.proxy_url.as_ref())
        .is_some()
}

fn build_provider(config: &Config) -> ProviderBuildStatus {
    let Some(provider_config) = config
        .provider_config(ProviderId::ThePornDb)
        .and_then(|provider| provider.theporndb_config().cloned())
    else {
        return ProviderBuildStatus::Unavailable;
    };
    if !provider_config.has_api_token() {
        return ProviderBuildStatus::Unavailable;
    }
    match ThePornDbMetadataProvider::new(provider_config) {
        Ok(provider) => ProviderBuildStatus::Ready(Box::new(provider)),
        Err(_) => ProviderBuildStatus::Unavailable,
    }
}

#[derive(Clone, Debug)]
pub struct ThePornDbMetadataProvider<T = ReqwestProviderHttpTransport>
where
    T: ProviderHttpTransport,
{
    config: ThePornDbProviderConfig,
    runtime: ProviderHttpRuntime<T>,
}

impl ThePornDbMetadataProvider<ReqwestProviderHttpTransport> {
    pub fn new(config: ThePornDbProviderConfig) -> ProviderHttpResult<Self> {
        let runtime = ProviderHttpRuntime::new(runtime_config(&config))?;
        Ok(Self { config, runtime })
    }
}

impl<T> ThePornDbMetadataProvider<T>
where
    T: ProviderHttpTransport,
{
    #[must_use]
    pub fn with_runtime(config: ThePornDbProviderConfig, runtime: ProviderHttpRuntime<T>) -> Self {
        Self { config, runtime }
    }

    async fn suggest_candidates(
        &self,
        query: &MetadataQuery,
    ) -> anyhow::Result<Vec<ProviderMetadataCandidate>> {
        if let Some(hash_lookup) = hash_lookup_from_query(query) {
            return self
                .hash_candidates(hash_lookup, facts_from_query(query), query)
                .await;
        }

        if let Some(slug) =
            rendered_av::direct_external_id(query, THEPORNDB_URL_EXTERNAL_ID_PROVIDER)
                .and_then(|value| normalize_identifier(&value))
        {
            return self
                .detail_candidates(&slug, facts_from_query(query), query)
                .await;
        }

        if let Some(slug) = rendered_av::direct_external_id(query, THEPORNDB_PROVIDER_ID)
            .and_then(|value| normalize_identifier(&value))
        {
            return self
                .detail_candidates(&slug, facts_from_query(query), query)
                .await;
        }

        let query_term = facts_from_query(query)
            .map(ThePornDbSearchTerm::Av)
            .unwrap_or_else(|| ThePornDbSearchTerm::Title(query.title.clone()));
        if query_term.is_empty() {
            return Ok(Vec::new());
        }

        let search = self.search_scenes(&query_term).await?;
        Ok(search
            .into_scene_candidates(&self.config, query, query_term.av_hint())
            .into_iter()
            .take(3)
            .collect())
    }

    async fn search_scenes(
        &self,
        query_term: &ThePornDbSearchTerm,
    ) -> anyhow::Result<ThePornDbSceneSearchResponse> {
        let response = self
            .runtime
            .get_json(
                THEPORNDB_PROVIDER_ID,
                "scene_search",
                self.scene_search_url(),
                query_term.query_params(),
                self.auth_headers(),
            )
            .await?;

        Ok(serde_json::from_value(response.body)?)
    }

    async fn detail_candidates(
        &self,
        identifier: &str,
        av_hint: Option<AvQueryFacts>,
        query: &MetadataQuery,
    ) -> anyhow::Result<Vec<ProviderMetadataCandidate>> {
        let response = self
            .runtime
            .get_json(
                THEPORNDB_PROVIDER_ID,
                "scene_detail",
                self.scene_detail_url(identifier),
                Vec::new(),
                self.auth_headers(),
            )
            .await?;
        let detail: ThePornDbSceneResponse = serde_json::from_value(response.body)?;
        Ok(detail
            .data
            .and_then(|scene| scene.into_detail_facts(&self.config, av_hint))
            .map(|facts| vec![facts.into_candidate(query)])
            .unwrap_or_default())
    }

    async fn hash_candidates(
        &self,
        lookup: ThePornDbHashLookup,
        av_hint: Option<AvQueryFacts>,
        query: &MetadataQuery,
    ) -> anyhow::Result<Vec<ProviderMetadataCandidate>> {
        let response = self
            .runtime
            .get_json(
                THEPORNDB_PROVIDER_ID,
                "scene_hash_detail",
                self.scene_hash_url(&lookup.hash),
                lookup.query_params(),
                self.auth_headers(),
            )
            .await?;
        let detail: ThePornDbSceneResponse = serde_json::from_value(response.body)?;
        Ok(detail
            .data
            .and_then(|scene| scene.into_detail_facts(&self.config, av_hint))
            .map(|facts| vec![facts.into_candidate(query)])
            .unwrap_or_default())
    }

    fn scene_search_url(&self) -> String {
        format!("{}/scenes", self.config.api_base_url.trim_end_matches('/'))
    }

    fn scene_detail_url(&self, identifier: &str) -> String {
        format!(
            "{}/scenes/{}",
            self.config.api_base_url.trim_end_matches('/'),
            rendered_av::percent_encode(identifier)
        )
    }

    fn scene_hash_url(&self, hash: &str) -> String {
        format!(
            "{}/scenes/hash/{}",
            self.config.api_base_url.trim_end_matches('/'),
            rendered_av::percent_encode(hash)
        )
    }

    fn auth_headers(&self) -> Vec<(String, String)> {
        let mut headers = vec![("accept".to_owned(), "application/json".to_owned())];
        if let Some(token) = self
            .config
            .api_token
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            headers.push(("authorization".to_owned(), format!("Bearer {token}")));
        }
        headers
    }
}

#[async_trait]
impl<T> MetadataProvider for ThePornDbMetadataProvider<T>
where
    T: ProviderHttpTransport,
{
    fn id(&self) -> ProviderId {
        ProviderId::ThePornDb
    }

    fn supports_av_route(&self, route: AvNumberRoute) -> bool {
        matches!(
            route,
            AvNumberRoute::Censored
                | AvNumberRoute::Uncensored
                | AvNumberRoute::Western
                | AvNumberRoute::Unknown
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
enum ThePornDbSearchTerm {
    Av(AvQueryFacts),
    Title(String),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ThePornDbHashType {
    Oshash,
    Phash,
}

impl ThePornDbHashType {
    const fn as_api_value(self) -> &'static str {
        match self {
            Self::Oshash => "OSHASH",
            Self::Phash => "PHASH",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ThePornDbHashLookup {
    hash_type: ThePornDbHashType,
    hash: String,
}

impl ThePornDbHashLookup {
    fn query_params(&self) -> Vec<(String, String)> {
        vec![("type".to_owned(), self.hash_type.as_api_value().to_owned())]
    }
}

impl ThePornDbSearchTerm {
    fn query_params(&self) -> Vec<(String, String)> {
        match self {
            Self::Av(av) => vec![
                ("parse".to_owned(), av.number.clone()),
                ("sku".to_owned(), av.number.clone()),
                ("per_page".to_owned(), "10".to_owned()),
            ],
            Self::Title(title) => vec![
                ("parse".to_owned(), title.trim().to_owned()),
                ("q".to_owned(), title.trim().to_owned()),
                ("per_page".to_owned(), "10".to_owned()),
            ],
        }
    }

    fn av_hint(&self) -> Option<AvQueryFacts> {
        match self {
            Self::Av(av) => Some(av.clone()),
            Self::Title(title) => facts_from_text(title, AvNumberSource::ExternalId),
        }
    }

    fn is_empty(&self) -> bool {
        match self {
            Self::Av(av) => av.number.trim().is_empty(),
            Self::Title(title) => title.trim().is_empty(),
        }
    }
}

#[derive(Debug, Default, Deserialize)]
struct ThePornDbSceneSearchResponse {
    #[serde(default)]
    data: Vec<ThePornDbScene>,
}

impl ThePornDbSceneSearchResponse {
    fn into_scene_candidates(
        self,
        config: &ThePornDbProviderConfig,
        query: &MetadataQuery,
        av_hint: Option<AvQueryFacts>,
    ) -> Vec<ProviderMetadataCandidate> {
        self.data
            .into_iter()
            .filter_map(|scene| scene.into_detail_facts(config, av_hint.clone()))
            .map(|facts| facts.into_candidate(query))
            .collect()
    }
}

#[derive(Debug, Deserialize)]
struct ThePornDbSceneResponse {
    data: Option<ThePornDbScene>,
}

#[derive(Debug, Default, Deserialize)]
struct ThePornDbScene {
    id: Option<String>,
    #[serde(rename = "_id")]
    numeric_id: Option<serde_json::Value>,
    title: Option<String>,
    slug: Option<String>,
    external_id: Option<String>,
    description: Option<String>,
    rating: Option<serde_json::Value>,
    date: Option<String>,
    url: Option<String>,
    image: Option<String>,
    back_image: Option<String>,
    poster_image: Option<String>,
    poster: Option<String>,
    trailer: Option<String>,
    duration: Option<serde_json::Value>,
    sku: Option<String>,
    background: Option<ThePornDbImageSet>,
    background_back: Option<ThePornDbImageSet>,
    posters: Option<ThePornDbImageSet>,
    #[serde(default)]
    performers: Vec<ThePornDbNameResource>,
    site: Option<ThePornDbSite>,
    #[serde(default)]
    tags: Vec<ThePornDbNameResource>,
    #[serde(default)]
    hashes: Vec<ThePornDbHashResource>,
    #[serde(default)]
    directors: Vec<ThePornDbNameResource>,
    #[serde(default)]
    links: std::collections::BTreeMap<String, Option<String>>,
}

impl ThePornDbScene {
    fn into_detail_facts(
        self,
        config: &ThePornDbProviderConfig,
        av_hint: Option<AvQueryFacts>,
    ) -> Option<ThePornDbSceneFacts> {
        let title = non_empty_string(self.title.as_deref())?;
        let slug = non_empty_string(self.slug.as_deref())
            .or_else(|| slug_from_url(self.url.as_deref()?))?;
        let av = self
            .sku
            .as_deref()
            .and_then(|value| facts_from_text(value, AvNumberSource::ExternalId))
            .or_else(|| {
                self.external_id
                    .as_deref()
                    .and_then(|value| facts_from_text(value, AvNumberSource::ExternalId))
            })
            .or_else(|| facts_from_text(&title, AvNumberSource::ExternalId))
            .or(av_hint);
        let release_date = self
            .date
            .as_deref()
            .and_then(first_iso_date_prefix)
            .or_else(|| rendered_av::first_iso_date(self.date.as_deref().unwrap_or_default()));
        let release_year = release_date.as_deref().and_then(rendered_av::first_year);
        let runtime_minutes = self.duration.as_ref().and_then(duration_seconds_to_minutes);
        let performers = names_from_resources(self.performers.iter());
        let directors = names_from_resources(self.directors.iter());
        let genres = names_from_resources(self.tags.iter());
        let site_name = self.site.as_ref().and_then(ThePornDbSite::name);
        let network_name = self.site.as_ref().and_then(ThePornDbSite::network_name);
        let mut backdrop_urls = Vec::new();
        for url in [
            self.back_image.as_deref(),
            self.background
                .as_ref()
                .and_then(ThePornDbImageSet::best_url)
                .as_deref(),
            self.background_back
                .as_ref()
                .and_then(ThePornDbImageSet::best_url)
                .as_deref(),
        ]
        .into_iter()
        .flatten()
        {
            push_unique(
                &mut backdrop_urls,
                rendered_av::normalize_url(url.to_owned()),
            );
        }
        let poster_url = self
            .poster
            .as_deref()
            .or(self.poster_image.as_deref())
            .map(|url| rendered_av::normalize_url(url.to_owned()))
            .or_else(|| {
                self.posters
                    .as_ref()
                    .and_then(ThePornDbImageSet::best_url)
                    .map(|url| rendered_av::normalize_url(url))
            })
            .or_else(|| self.image.map(rendered_av::normalize_url));
        if let Some(poster_url) = &poster_url {
            backdrop_urls.retain(|url| url != poster_url);
        }
        let trailer_url = self.trailer.map(rendered_av::normalize_url);
        let mut external_links = Vec::new();
        if let Some(source_url) = self
            .url
            .as_deref()
            .and_then(|url| non_empty_string(Some(url)))
        {
            external_links.push(ProviderExternalId {
                provider: "theporndb_source_url".to_owned(),
                value: source_url,
            });
        }
        for (provider, value) in self.links {
            if let Some(value) = value.and_then(|value| non_empty_string(Some(&value))) {
                external_links.push(ProviderExternalId { provider, value });
            }
        }
        for hash in self.hashes {
            if let Some(external_id) = hash.into_external_id() {
                external_links.push(external_id);
            }
        }

        Some(ThePornDbSceneFacts {
            slug: slug.clone(),
            numeric_id: json_scalar_to_string(self.numeric_id.as_ref()),
            id: non_empty_string(self.id.as_deref()),
            external_id: non_empty_string(self.external_id.as_deref()),
            av,
            title,
            overview: non_empty_string(self.description.as_deref()),
            release_date,
            release_year,
            runtime_minutes,
            performers,
            directors,
            genres,
            site_name,
            network_name,
            rating_milli: self.rating.as_ref().and_then(rating_milli),
            poster_url,
            backdrop_urls,
            trailer_url,
            source_url: public_scene_url(config, &slug),
            external_links,
        })
    }
}

#[derive(Debug, Deserialize)]
struct ThePornDbImageSet {
    full: Option<String>,
    large: Option<String>,
    medium: Option<String>,
    small: Option<String>,
}

impl ThePornDbImageSet {
    fn best_url(&self) -> Option<String> {
        [
            self.full.as_deref(),
            self.large.as_deref(),
            self.medium.as_deref(),
            self.small.as_deref(),
        ]
        .into_iter()
        .flatten()
        .find_map(|value| non_empty_string(Some(value)))
    }
}

#[derive(Debug, Deserialize)]
struct ThePornDbNameResource {
    name: Option<String>,
    full_name: Option<String>,
    slug: Option<String>,
}

impl ThePornDbNameResource {
    fn name(&self) -> Option<String> {
        non_empty_string(self.name.as_deref())
            .or_else(|| non_empty_string(self.full_name.as_deref()))
            .or_else(|| non_empty_string(self.slug.as_deref()))
    }
}

#[derive(Debug, Deserialize)]
struct ThePornDbSite {
    name: Option<String>,
    short_name: Option<String>,
    network: Option<Box<ThePornDbSite>>,
    parent: Option<Box<ThePornDbSite>>,
}

#[derive(Debug, Deserialize)]
struct ThePornDbHashResource {
    #[serde(rename = "type")]
    hash_type: Option<String>,
    hash: Option<String>,
}

impl ThePornDbHashResource {
    fn into_external_id(self) -> Option<ProviderExternalId> {
        let hash = non_empty_string(self.hash.as_deref())?;
        let provider = match self
            .hash_type
            .as_deref()?
            .trim()
            .to_ascii_uppercase()
            .as_str()
        {
            "OSHASH" => FILE_OSHASH_EXTERNAL_ID_PROVIDER,
            "PHASH" => FILE_PHASH_EXTERNAL_ID_PROVIDER,
            _ => return None,
        };
        Some(ProviderExternalId {
            provider: provider.to_owned(),
            value: hash,
        })
    }
}

impl ThePornDbSite {
    fn name(&self) -> Option<String> {
        non_empty_string(self.name.as_deref())
            .or_else(|| non_empty_string(self.short_name.as_deref()))
    }

    fn network_name(&self) -> Option<String> {
        self.network
            .as_ref()
            .and_then(|site| site.name())
            .or_else(|| self.parent.as_ref().and_then(|site| site.name()))
    }
}

struct ThePornDbSceneFacts {
    slug: String,
    numeric_id: Option<String>,
    id: Option<String>,
    external_id: Option<String>,
    av: Option<AvQueryFacts>,
    title: String,
    overview: Option<String>,
    release_date: Option<String>,
    release_year: Option<i32>,
    runtime_minutes: Option<u32>,
    performers: Vec<String>,
    directors: Vec<String>,
    genres: Vec<String>,
    site_name: Option<String>,
    network_name: Option<String>,
    rating_milli: Option<u16>,
    poster_url: Option<String>,
    backdrop_urls: Vec<String>,
    trailer_url: Option<String>,
    source_url: String,
    external_links: Vec<ProviderExternalId>,
}

impl ThePornDbSceneFacts {
    fn into_candidate(self, query: &MetadataQuery) -> ProviderMetadataCandidate {
        let mut tags = vec![THEPORNDB_PROVIDER_ID.to_owned()];
        if let Some(av) = &self.av {
            tags.push(format!("av_number:{}", av.number));
            tags.push(format!("av_route:{:?}", av.route).to_ascii_lowercase());
        }
        tags.extend(self.performers.iter().map(|actor| format!("actor:{actor}")));
        tags.extend(
            self.directors
                .iter()
                .map(|director| format!("director:{director}")),
        );
        tags.extend(self.genres.iter().map(|tag| format!("tag:{tag}")));
        if let Some(site_name) = &self.site_name {
            tags.push(format!("site:{site_name}"));
        }
        if let Some(network_name) = &self.network_name {
            tags.push(format!("network:{network_name}"));
        }

        let mut artwork_candidates = Vec::new();
        if let Some(poster_url) = self.poster_url.clone() {
            artwork_candidates.push(theporndb_artwork_candidate(
                &self.slug,
                AddonArtworkKind::Poster,
                poster_url,
                0,
            ));
        }
        for (index, url) in self.backdrop_urls.iter().cloned().enumerate() {
            artwork_candidates.push(theporndb_artwork_candidate(
                &self.slug,
                AddonArtworkKind::Backdrop,
                url,
                index + 1,
            ));
        }

        let mut external_ids = vec![
            ProviderExternalId {
                provider: THEPORNDB_PROVIDER_ID.to_owned(),
                value: self.slug.clone(),
            },
            ProviderExternalId {
                provider: THEPORNDB_URL_EXTERNAL_ID_PROVIDER.to_owned(),
                value: self.source_url.clone(),
            },
        ];
        if let Some(id) = &self.id {
            external_ids.push(ProviderExternalId {
                provider: "theporndb_uuid".to_owned(),
                value: id.clone(),
            });
        }
        if let Some(numeric_id) = &self.numeric_id {
            external_ids.push(ProviderExternalId {
                provider: "theporndb_numeric_id".to_owned(),
                value: numeric_id.clone(),
            });
        }
        if let Some(external_id) = &self.external_id {
            external_ids.push(ProviderExternalId {
                provider: "theporndb_external_id".to_owned(),
                value: external_id.clone(),
            });
        }
        if let Some(av) = &self.av {
            external_ids.push(ProviderExternalId {
                provider: AV_NUMBER_EXTERNAL_ID_PROVIDER.to_owned(),
                value: av.number.clone(),
            });
        }
        for external_link in self.external_links {
            if !external_ids.iter().any(|existing| {
                existing
                    .provider
                    .eq_ignore_ascii_case(&external_link.provider)
                    && existing.value.eq_ignore_ascii_case(&external_link.value)
            }) {
                external_ids.push(external_link);
            }
        }

        ProviderMetadataCandidate {
            provider: THEPORNDB_PROVIDER_ID.to_owned(),
            provider_id: format!("theporndb:scene:{}", self.slug),
            patch: AddonMetadataPatch {
                title: Some(self.title.clone()),
                original_title: None,
                sort_title: Some(self.title.clone()),
                overview: self.overview.clone(),
                release_date: self.release_date.clone(),
                runtime_minutes: self.runtime_minutes,
                tagline: Some("ThePornDB scene".to_owned()),
                genres: Some(self.genres.clone()).filter(|genres| !genres.is_empty()),
                tags: Some(tags).filter(|tags| !tags.is_empty()),
                ..AddonMetadataPatch::default()
            },
            facts: ProviderCandidateFacts {
                title: Some(self.title),
                alternate_titles: self
                    .av
                    .as_ref()
                    .map(|av| vec![av.number.clone()])
                    .unwrap_or_default(),
                release_year: self.release_year,
                language: Some(query.language.clone()),
                av: AvMetadataFacts {
                    actors: self.performers.clone(),
                    all_actors: self.performers.clone(),
                    directors: self.directors.clone(),
                    series: self.network_name.clone(),
                    studio: self.site_name.clone(),
                    publisher: self.network_name.clone(),
                    maker: self.site_name.clone(),
                    label: self.site_name.clone(),
                    wanted_count: None,
                    thumb_url: self.poster_url.clone(),
                    trailer_url: self.trailer_url.clone(),
                    extrafanart_urls: self.backdrop_urls.clone(),
                }
                .non_empty(),
                community_score_milli: self.rating_milli,
                community_vote_count: None,
                external_ids,
                provider_outcomes: vec![ProviderOutcome::ThePornDbOfficialApiParsed],
                provider_note: None,
            },
            artwork_candidates,
        }
    }
}

fn runtime_config(config: &ThePornDbProviderConfig) -> ProviderHttpRuntimeConfig {
    ProviderHttpRuntimeConfig {
        timeout_ms: config.timeout_ms,
        proxy_url: config.proxy_url.clone(),
        ..ProviderHttpRuntimeConfig::default()
    }
}

fn normalize_identifier(value: &str) -> Option<String> {
    slug_from_url(value).or_else(|| {
        non_empty_string(Some(value.trim().trim_matches('/'))).map(|value| {
            value
                .strip_prefix("scenes/")
                .unwrap_or(&value)
                .trim_matches('/')
                .to_owned()
        })
    })
}

fn hash_lookup_from_query(query: &MetadataQuery) -> Option<ThePornDbHashLookup> {
    hash_lookup_for_provider(
        query,
        FILE_OSHASH_EXTERNAL_ID_PROVIDER,
        ThePornDbHashType::Oshash,
    )
    .or_else(|| {
        hash_lookup_for_provider(
            query,
            FILE_PHASH_EXTERNAL_ID_PROVIDER,
            ThePornDbHashType::Phash,
        )
    })
}

fn hash_lookup_for_provider(
    query: &MetadataQuery,
    provider: &str,
    hash_type: ThePornDbHashType,
) -> Option<ThePornDbHashLookup> {
    let hash = rendered_av::direct_external_id(query, provider)
        .and_then(|value| normalize_hash_value(&value))?;
    Some(ThePornDbHashLookup { hash_type, hash })
}

fn normalize_hash_value(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty()
        || value.chars().any(|character| {
            !(character.is_ascii_hexdigit() || character == '-' || character == '_')
        })
    {
        return None;
    }
    Some(value.to_ascii_lowercase())
}

fn slug_from_url(value: &str) -> Option<String> {
    let without_fragment = value.split('#').next().unwrap_or(value);
    let without_query = without_fragment
        .split('?')
        .next()
        .unwrap_or(without_fragment);
    let marker = "/scenes/";
    if let Some(start) = without_query.find(marker).map(|index| index + marker.len()) {
        let rest = &without_query[start..];
        let end = rest.find('/').unwrap_or(rest.len());
        return non_empty_string(Some(&rest[..end]));
    }
    None
}

fn public_scene_url(config: &ThePornDbProviderConfig, slug: &str) -> String {
    format!(
        "{}/scenes/{}",
        config.public_base_url.trim_end_matches('/'),
        rendered_av::percent_encode(slug)
    )
}

fn duration_seconds_to_minutes(value: &serde_json::Value) -> Option<u32> {
    let seconds = value.as_u64().or_else(|| {
        value
            .as_str()
            .and_then(|value| value.trim().parse::<u64>().ok())
    })?;
    u32::try_from((seconds.saturating_add(59)) / 60).ok()
}

fn rating_milli(value: &serde_json::Value) -> Option<u16> {
    let rating = value.as_f64().or_else(|| {
        value
            .as_str()
            .and_then(|value| value.trim().parse::<f64>().ok())
    })?;
    let scaled = if rating <= 5.0 {
        rating * 200.0
    } else {
        rating * 100.0
    };
    Some(scaled.round().clamp(0.0, 1000.0) as u16)
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

fn json_scalar_to_string(value: Option<&serde_json::Value>) -> Option<String> {
    let value = value?;
    if let Some(value) = value.as_str() {
        return non_empty_string(Some(value));
    }
    value
        .as_i64()
        .map(|value| value.to_string())
        .or_else(|| value.as_u64().map(|value| value.to_string()))
}

fn names_from_resources<'a>(
    values: impl IntoIterator<Item = &'a ThePornDbNameResource>,
) -> Vec<String> {
    values
        .into_iter()
        .filter_map(ThePornDbNameResource::name)
        .fold(Vec::new(), |mut result, value| {
            push_unique(&mut result, value);
            result
        })
}

fn non_empty_string(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

fn push_unique(values: &mut Vec<String>, value: String) {
    if !value.trim().is_empty() && !values.iter().any(|existing| existing == &value) {
        values.push(value);
    }
}

fn theporndb_artwork_candidate(
    slug: &str,
    kind: AddonArtworkKind,
    source_url: String,
    index: usize,
) -> ProviderArtworkCandidate {
    ProviderArtworkCandidate {
        provider: THEPORNDB_PROVIDER_ID.to_owned(),
        provider_id: format!("theporndb:scene:{slug}:artwork:{index}"),
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
    async fn theporndb_provider_searches_scenes_and_maps_metadata() {
        let transport = FakeTransport::default();
        transport.push_json(json!({
            "data": [
                {
                    "id": "uuid-001",
                    "_id": 123,
                    "title": "TPDB Scene Title",
                    "slug": "tpdb-scene-title",
                    "external_id": "TPDB-001",
                    "description": "Scene outline.",
                    "rating": 4.5,
                    "date": "2024-06-08",
                    "url": "https://studio.example/scenes/tpdb-001",
                    "poster": "https://img.example/poster.jpg",
                    "back_image": "https://img.example/back.jpg",
                    "background": {
                        "large": "https://img.example/background-large.jpg"
                    },
                    "trailer": "https://video.example/trailer.mp4",
                    "duration": 3661,
                    "sku": "TPDB-001",
                    "performers": [
                        {"name": "Performer One"},
                        {"full_name": "Performer Two"},
                        {"name": "Performer One"}
                    ],
                    "directors": [{"name": "Director One"}],
                    "tags": [{"name": "Feature"}, {"name": "Studio"}],
                    "site": {
                        "name": "Studio Site",
                        "network": {"name": "Studio Network"}
                    },
                    "hashes": [
                        {"type": "OSHASH", "hash": "d7dae9cd888c5984"},
                        {"type": "PHASH", "hash": "faceb00c"}
                    ],
                    "links": {
                        "IAFD": "https://iafd.example/title",
                        "Empty": null
                    }
                }
            ]
        }));
        let provider = provider_with_transport(transport.clone());

        let candidates = provider
            .suggest(&MetadataQuery::from_payload(
                &json!({"av_number": "TPDB-001", "language": "en-US"}),
                "en-US",
            ))
            .await
            .unwrap();

        assert_eq!(candidates.len(), 1);
        let candidate = &candidates[0];
        assert_eq!(candidate.provider, "theporndb");
        assert_eq!(candidate.provider_id, "theporndb:scene:tpdb-scene-title");
        assert_eq!(candidate.patch.title.as_deref(), Some("TPDB Scene Title"));
        assert_eq!(candidate.patch.overview.as_deref(), Some("Scene outline."));
        assert_eq!(candidate.patch.release_date.as_deref(), Some("2024-06-08"));
        assert_eq!(candidate.patch.runtime_minutes, Some(62));
        assert_eq!(
            candidate.patch.genres.as_deref(),
            Some(["Feature".to_owned(), "Studio".to_owned()].as_slice())
        );
        assert_eq!(candidate.facts.community_score_milli, Some(900));
        let av = candidate.facts.av.as_ref().unwrap();
        assert_eq!(av.actors, vec!["Performer One", "Performer Two"]);
        assert_eq!(av.directors, vec!["Director One"]);
        assert_eq!(av.studio.as_deref(), Some("Studio Site"));
        assert_eq!(av.publisher.as_deref(), Some("Studio Network"));
        assert_eq!(av.series.as_deref(), Some("Studio Network"));
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
                "https://img.example/back.jpg".to_owned(),
                "https://img.example/background-large.jpg".to_owned()
            ]
        );
        assert_eq!(candidate.artwork_candidates.len(), 3);
        assert_eq!(
            candidate.artwork_candidates[0].facts.kind,
            AddonArtworkKind::Poster
        );
        assert!(candidate.facts.external_ids.contains(&ProviderExternalId {
            provider: "theporndb".to_owned(),
            value: "tpdb-scene-title".to_owned(),
        }));
        assert!(candidate.facts.external_ids.contains(&ProviderExternalId {
            provider: "theporndb_url".to_owned(),
            value: "https://theporndb.test/scenes/tpdb-scene-title".to_owned(),
        }));
        assert!(candidate.facts.external_ids.contains(&ProviderExternalId {
            provider: "theporndb_uuid".to_owned(),
            value: "uuid-001".to_owned(),
        }));
        assert!(candidate.facts.external_ids.contains(&ProviderExternalId {
            provider: "av_number".to_owned(),
            value: "TPDB-001".to_owned(),
        }));
        assert!(candidate.facts.external_ids.contains(&ProviderExternalId {
            provider: "theporndb_source_url".to_owned(),
            value: "https://studio.example/scenes/tpdb-001".to_owned(),
        }));
        assert!(candidate.facts.external_ids.contains(&ProviderExternalId {
            provider: "file_oshash".to_owned(),
            value: "d7dae9cd888c5984".to_owned(),
        }));
        assert!(candidate.facts.external_ids.contains(&ProviderExternalId {
            provider: "file_phash".to_owned(),
            value: "faceb00c".to_owned(),
        }));
        assert!(candidate.facts.external_ids.contains(&ProviderExternalId {
            provider: "IAFD".to_owned(),
            value: "https://iafd.example/title".to_owned(),
        }));
        assert_eq!(
            candidate.facts.provider_outcomes,
            vec![ProviderOutcome::ThePornDbOfficialApiParsed]
        );

        let requests = transport.requests();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].url, "https://api.theporndb.test/scenes");
        assert!(
            requests[0]
                .query
                .contains(&("parse".to_owned(), "TPDB-001".to_owned()))
        );
        assert!(
            requests[0]
                .query
                .contains(&("sku".to_owned(), "TPDB-001".to_owned()))
        );
        assert!(
            requests[0]
                .headers
                .contains(&("authorization".to_owned(), "Bearer secret-token".to_owned()))
        );
        assert_eq!(
            transport.configs()[0].proxy_url.as_deref(),
            Some("http://proxy.example:8080")
        );
    }

    #[tokio::test]
    async fn theporndb_provider_uses_file_hash_for_direct_scene_lookup() {
        let transport = FakeTransport::default();
        transport.push_json(json!({
            "data": {
                "title": "Hash Scene",
                "slug": "hash-scene",
                "description": "Hash outline.",
                "date": "2024-03-04",
                "hashes": [{"type": "OSHASH", "hash": "abcdef0123456789"}]
            }
        }));
        let provider = provider_with_transport(transport.clone());

        let candidates = provider
            .suggest(&MetadataQuery::from_payload_with_external_id_capabilities(
                &json!({
                    "title": "Hash fallback title",
                    "file_oshash": "ABCDEF0123456789",
                    "av_number": "TPDB-001"
                }),
                "en-US",
                THEPORNDB_EXTERNAL_ID_CAPABILITIES,
            ))
            .await
            .unwrap();

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].provider_id, "theporndb:scene:hash-scene");
        assert!(
            candidates[0]
                .facts
                .external_ids
                .contains(&ProviderExternalId {
                    provider: "file_oshash".to_owned(),
                    value: "abcdef0123456789".to_owned(),
                })
        );
        let requests = transport.requests();
        assert_eq!(
            requests[0].url,
            "https://api.theporndb.test/scenes/hash/abcdef0123456789"
        );
        assert_eq!(
            requests[0].query,
            vec![("type".to_owned(), "OSHASH".to_owned())]
        );
    }

    #[tokio::test]
    async fn theporndb_provider_uses_phash_type_for_file_phash_lookup() {
        let transport = FakeTransport::default();
        transport.push_json(json!({
            "data": {
                "title": "Perceptual Hash Scene",
                "slug": "phash-scene"
            }
        }));
        let provider = provider_with_transport(transport.clone());

        let candidates = provider
            .suggest(&MetadataQuery {
                title: "Manual hash lookup".to_owned(),
                year: None,
                language: "en-US".to_owned(),
                external_ids: vec![QueryExternalId {
                    provider: "file_phash".to_owned(),
                    value: "faceB00c".to_owned(),
                }],
            })
            .await
            .unwrap();

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].provider_id, "theporndb:scene:phash-scene");
        let requests = transport.requests();
        assert_eq!(
            requests[0].url,
            "https://api.theporndb.test/scenes/hash/faceb00c"
        );
        assert_eq!(
            requests[0].query,
            vec![("type".to_owned(), "PHASH".to_owned())]
        );
    }

    #[tokio::test]
    async fn theporndb_provider_uses_explicit_slug_for_detail_lookup() {
        let transport = FakeTransport::default();
        transport.push_json(json!({
            "data": {
                "title": "Direct Scene",
                "slug": "direct-scene",
                "description": "Direct outline.",
                "date": "2024-01-02",
                "poster": "https://img.example/direct.jpg"
            }
        }));
        let provider = provider_with_transport(transport.clone());

        let candidates = provider
            .suggest(&MetadataQuery {
                title: "Manual lookup".to_owned(),
                year: None,
                language: "en-US".to_owned(),
                external_ids: vec![QueryExternalId {
                    provider: "theporndb_url".to_owned(),
                    value: "https://theporndb.net/scenes/direct-scene?foo=bar".to_owned(),
                }],
            })
            .await
            .unwrap();

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].provider_id, "theporndb:scene:direct-scene");
        assert_eq!(
            transport.requests()[0].url,
            "https://api.theporndb.test/scenes/direct-scene"
        );
    }

    #[tokio::test]
    async fn theporndb_provider_falls_back_to_title_parse_search_without_av_number() {
        let transport = FakeTransport::default();
        transport.push_json(json!({"data": []}));
        let provider = provider_with_transport(transport.clone());

        let candidates = provider
            .suggest(&MetadataQuery {
                title: "Plain Search Title".to_owned(),
                year: None,
                language: "en-US".to_owned(),
                external_ids: Vec::new(),
            })
            .await
            .unwrap();

        assert!(candidates.is_empty());
        assert!(
            transport.requests()[0]
                .query
                .contains(&("parse".to_owned(), "Plain Search Title".to_owned()))
        );
        assert!(
            transport.requests()[0]
                .query
                .contains(&("q".to_owned(), "Plain Search Title".to_owned()))
        );
    }

    #[test]
    fn theporndb_config_trims_secret_and_network_boundaries() {
        let config = ThePornDbProviderConfig::from_env_lookup(|name| match name {
            "NAKO_METADATA_SCRAPER_THEPORNDB_API_TOKEN" => Some(" secret ".to_owned()),
            "NAKO_METADATA_SCRAPER_THEPORNDB_API_BASE_URL" => {
                Some(" https://api.example ".to_owned())
            }
            "NAKO_METADATA_SCRAPER_THEPORNDB_PUBLIC_BASE_URL" => {
                Some(" https://public.example ".to_owned())
            }
            "NAKO_METADATA_SCRAPER_THEPORNDB_TIMEOUT_MS" => Some("1234".to_owned()),
            "NAKO_METADATA_SCRAPER_THEPORNDB_PROXY_URL" => {
                Some(" http://proxy.example ".to_owned())
            }
            _ => None,
        });

        assert_eq!(config.api_token.as_deref(), Some("secret"));
        assert_eq!(config.api_base_url, "https://api.example");
        assert_eq!(config.public_base_url, "https://public.example");
        assert_eq!(config.timeout_ms, 1234);
        assert_eq!(config.proxy_url.as_deref(), Some("http://proxy.example"));
    }

    fn provider_with_transport(
        transport: FakeTransport,
    ) -> ThePornDbMetadataProvider<FakeTransport> {
        let config = ThePornDbProviderConfig {
            api_token: Some("secret-token".to_owned()),
            api_base_url: "https://api.theporndb.test".to_owned(),
            public_base_url: "https://theporndb.test".to_owned(),
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
        ThePornDbMetadataProvider::with_runtime(config, runtime)
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
                        provider_id: "theporndb",
                        operation: "fake",
                        message: "fake transport response queue was empty".to_owned(),
                        attempts: 0,
                    })
                })
        }
    }
}
