use std::collections::HashSet;

use async_trait::async_trait;
use nako_addon_protocol::{AddonArtworkKind, AddonMetadataPatch};
use serde::Deserialize;

use crate::{
    config::{ProviderId, TmdbProviderConfig},
    engine::{
        MetadataQuery, ProviderCandidateFacts, ProviderExternalId, ProviderMetadataCandidate,
        ranking,
    },
    providers::{
        MetadataProvider,
        http_runtime::{
            ProviderHttpResult, ProviderHttpRuntime, ProviderHttpRuntimeConfig,
            ProviderHttpTransport, ReqwestProviderHttpTransport,
        },
    },
};

pub const TMDB_PROVIDER_ID: &str = "tmdb";
const TMDB_IMAGE_BASE_URL: &str = "https://image.tmdb.org/t/p/original";
const TMDB_DETAIL_ENRICHMENT_LIMIT: usize = 3;
const TMDB_PARTIAL_SEARCH_NOTE: &str =
    "TMDB provider preserved candidates after partial title-variant search failure.";

#[derive(Clone, Debug)]
pub struct TmdbMetadataProvider<T = ReqwestProviderHttpTransport>
where
    T: ProviderHttpTransport,
{
    config: TmdbProviderConfig,
    runtime: ProviderHttpRuntime<T>,
}

impl TmdbMetadataProvider<ReqwestProviderHttpTransport> {
    pub fn new(config: TmdbProviderConfig) -> ProviderHttpResult<Self> {
        let runtime = ProviderHttpRuntime::new(ProviderHttpRuntimeConfig {
            proxy_url: config.proxy_url.clone(),
            ..ProviderHttpRuntimeConfig::default()
        })?;
        Ok(Self { config, runtime })
    }
}

impl<T> TmdbMetadataProvider<T>
where
    T: ProviderHttpTransport,
{
    #[must_use]
    pub fn with_runtime(config: TmdbProviderConfig, runtime: ProviderHttpRuntime<T>) -> Self {
        Self { config, runtime }
    }

    fn endpoint(&self, path: impl AsRef<str>) -> String {
        let path = path.as_ref();
        format!(
            "{}/{}",
            self.config.api_base_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }

    fn bearer_headers(&self) -> Vec<(String, String)> {
        self.config
            .read_access_token
            .as_ref()
            .map(|token| vec![("authorization".to_owned(), format!("Bearer {token}"))])
            .unwrap_or_default()
    }
}

#[async_trait]
impl<T> MetadataProvider for TmdbMetadataProvider<T>
where
    T: ProviderHttpTransport,
{
    fn id(&self) -> ProviderId {
        ProviderId::Tmdb
    }

    async fn suggest(
        &self,
        query: &MetadataQuery,
    ) -> anyhow::Result<Vec<ProviderMetadataCandidate>> {
        for movie_id in tmdb_query_movie_ids(query) {
            match self.enrich_movie_candidate_by_id(query, movie_id).await {
                Ok(candidate) => return Ok(vec![candidate]),
                Err(error) => {
                    tracing::warn!(provider = TMDB_PROVIDER_ID, %error, "TMDB direct movie lookup failed; falling back to title search");
                }
            }
        }
        for imdb_id in tmdb_query_imdb_ids(query) {
            match self.find_movie_id_by_imdb_id(&imdb_id).await {
                Ok(Some(movie_id)) => {
                    match self.enrich_movie_candidate_by_id(query, movie_id).await {
                        Ok(candidate) => return Ok(vec![candidate]),
                        Err(error) => {
                            tracing::warn!(provider = TMDB_PROVIDER_ID, %error, "TMDB IMDb find movie enrichment failed; falling back to title search");
                        }
                    }
                }
                Ok(None) => {
                    tracing::warn!(
                        provider = TMDB_PROVIDER_ID,
                        imdb_id,
                        "TMDB IMDb find returned no movie results; trying next IMDb ID"
                    );
                }
                Err(error) => {
                    tracing::warn!(provider = TMDB_PROVIDER_ID, %error, imdb_id, "TMDB IMDb find failed; trying next IMDb ID");
                }
            }
        }

        let mut search_results = Vec::new();
        let mut seen_movie_ids = HashSet::new();
        let mut last_search_error = None;

        for search_title in query.search_title_variants() {
            let search = match self.search_movies(query, &search_title).await {
                Ok(search) => search,
                Err(error) => {
                    tracing::warn!(provider = TMDB_PROVIDER_ID, %error, "TMDB title-variant search failed");
                    last_search_error = Some(error);
                    continue;
                }
            };
            for result in search.results {
                if seen_movie_ids.insert(result.id) {
                    search_results.push(result);
                }
            }
        }
        if search_results.is_empty()
            && let Some(error) = last_search_error.take()
        {
            return Err(error);
        }
        let partial_search = last_search_error.is_some();
        search_results = tmdb_ranked_enrichment_results(query, search_results);

        let mut candidates = Vec::new();
        for result in search_results {
            match self.enrich_movie_candidate(query, result.clone()).await {
                Ok(mut candidate) => {
                    if partial_search {
                        append_provider_note(
                            &mut candidate.facts.provider_note,
                            TMDB_PARTIAL_SEARCH_NOTE,
                        );
                    }
                    candidates.push(candidate);
                }
                Err(error) => {
                    tracing::warn!(provider = TMDB_PROVIDER_ID, %error, "returning degraded TMDB candidate after enrichment failure");
                    let mut candidate = result.into_degraded_candidate(query);
                    if partial_search {
                        append_provider_note(
                            &mut candidate.facts.provider_note,
                            TMDB_PARTIAL_SEARCH_NOTE,
                        );
                    }
                    candidates.push(candidate);
                }
            }
        }

        Ok(candidates)
    }
}

impl<T> TmdbMetadataProvider<T>
where
    T: ProviderHttpTransport,
{
    async fn search_movies(
        &self,
        query: &MetadataQuery,
        search_title: &str,
    ) -> anyhow::Result<TmdbSearchResponse> {
        let mut request_query = vec![
            ("query".to_owned(), search_title.to_owned()),
            ("language".to_owned(), self.config.language.clone()),
            (
                "include_adult".to_owned(),
                self.config.include_adult.to_string(),
            ),
            ("page".to_owned(), "1".to_owned()),
        ];
        if let Some(year) = query.year.filter(|year| (1..=9999).contains(year)) {
            request_query.push(("primary_release_year".to_owned(), year.to_string()));
        }

        let response = self
            .runtime
            .get_json(
                TMDB_PROVIDER_ID,
                "search movie",
                self.endpoint("search/movie"),
                request_query,
                self.bearer_headers(),
            )
            .await?;

        TmdbSearchResponse::from_value(response.body)
    }

    async fn enrich_movie_candidate(
        &self,
        query: &MetadataQuery,
        result: TmdbMovieSearchResult,
    ) -> anyhow::Result<ProviderMetadataCandidate> {
        let movie_id = result.id;
        self.enrich_movie_candidate_from_seed(query, result, movie_id)
            .await
    }

    async fn enrich_movie_candidate_by_id(
        &self,
        query: &MetadataQuery,
        movie_id: u64,
    ) -> anyhow::Result<ProviderMetadataCandidate> {
        self.enrich_movie_candidate_from_seed(
            query,
            TmdbMovieSearchResult::direct_lookup_seed(movie_id),
            movie_id,
        )
        .await
    }

    async fn enrich_movie_candidate_from_seed(
        &self,
        query: &MetadataQuery,
        result: TmdbMovieSearchResult,
        movie_id: u64,
    ) -> anyhow::Result<ProviderMetadataCandidate> {
        let detail = self.fetch_movie_detail(movie_id).await?;
        if detail.id == 0 {
            anyhow::bail!(
                "TMDB movie detail response returned zero id for requested movie {movie_id}"
            );
        }
        if detail.id != movie_id {
            anyhow::bail!(
                "TMDB movie detail response id {} did not match requested movie {movie_id}",
                detail.id
            );
        }
        let mut partial_enrichment = false;
        let external_ids = match self.fetch_movie_external_ids(movie_id).await {
            Ok(external_ids) => external_ids,
            Err(error) => {
                partial_enrichment = true;
                tracing::warn!(provider = TMDB_PROVIDER_ID, %error, "TMDB external IDs enrichment failed for detail candidate");
                TmdbMovieExternalIds::default()
            }
        };
        let alternative_titles = match self.fetch_movie_alternative_titles(movie_id).await {
            Ok(alternative_titles) => alternative_titles,
            Err(error) => {
                partial_enrichment = true;
                tracing::warn!(provider = TMDB_PROVIDER_ID, %error, "TMDB alternative titles enrichment failed for detail candidate");
                TmdbMovieAlternativeTitles::default()
            }
        };

        Ok(TmdbMovieCandidate {
            search: result,
            detail,
            external_ids,
            alternative_titles,
            partial_enrichment,
        }
        .into_candidate(query))
    }

    async fn fetch_movie_detail(&self, movie_id: u64) -> anyhow::Result<TmdbMovieDetail> {
        let response = self
            .runtime
            .get_json(
                TMDB_PROVIDER_ID,
                "movie detail",
                self.endpoint(format!("movie/{movie_id}")),
                vec![("language".to_owned(), self.config.language.clone())],
                self.bearer_headers(),
            )
            .await?;

        TmdbMovieDetail::from_value(response.body)
    }

    async fn fetch_movie_external_ids(
        &self,
        movie_id: u64,
    ) -> anyhow::Result<TmdbMovieExternalIds> {
        let response = self
            .runtime
            .get_json(
                TMDB_PROVIDER_ID,
                "movie external ids",
                self.endpoint(format!("movie/{movie_id}/external_ids")),
                Vec::new(),
                self.bearer_headers(),
            )
            .await?;

        TmdbMovieExternalIds::from_value(response.body)
    }

    async fn fetch_movie_alternative_titles(
        &self,
        movie_id: u64,
    ) -> anyhow::Result<TmdbMovieAlternativeTitles> {
        let response = self
            .runtime
            .get_json(
                TMDB_PROVIDER_ID,
                "movie alternative titles",
                self.endpoint(format!("movie/{movie_id}/alternative_titles")),
                Vec::new(),
                self.bearer_headers(),
            )
            .await?;

        TmdbMovieAlternativeTitles::from_value(response.body)
    }

    async fn find_movie_id_by_imdb_id(&self, imdb_id: &str) -> anyhow::Result<Option<u64>> {
        let response = self
            .runtime
            .get_json(
                TMDB_PROVIDER_ID,
                "find by external id",
                self.endpoint(format!("find/{imdb_id}")),
                vec![("external_source".to_owned(), "imdb_id".to_owned())],
                self.bearer_headers(),
            )
            .await?;

        Ok(TmdbFindResponse::from_value(response.body)?.first_movie_id())
    }
}

fn tmdb_ranked_enrichment_results(
    query: &MetadataQuery,
    results: Vec<TmdbMovieSearchResult>,
) -> Vec<TmdbMovieSearchResult> {
    ranking::select_ranked_provider_inputs(query, results, TMDB_DETAIL_ENRICHMENT_LIMIT, |result| {
        result.clone().into_degraded_candidate(query)
    })
}

fn tmdb_query_movie_ids(query: &MetadataQuery) -> impl Iterator<Item = u64> + '_ {
    let mut seen = HashSet::new();
    query
        .external_ids
        .iter()
        .filter(|external_id| external_id.provider.eq_ignore_ascii_case(TMDB_PROVIDER_ID))
        .filter_map(|external_id| external_id.value.trim().parse().ok())
        .filter(|movie_id| *movie_id > 0)
        .filter(move |movie_id| seen.insert(*movie_id))
}

fn tmdb_query_imdb_ids(query: &MetadataQuery) -> impl Iterator<Item = String> + '_ {
    let mut seen = HashSet::new();
    query
        .external_ids
        .iter()
        .filter(|external_id| external_id.provider.eq_ignore_ascii_case("imdb"))
        .filter_map(|external_id| normalized_imdb_id(&external_id.value))
        .filter(move |imdb_id| seen.insert(imdb_id.clone()))
}

fn normalized_imdb_id(value: &str) -> Option<String> {
    let value = value.trim();
    if value.len() > 2
        && value[..2].eq_ignore_ascii_case("tt")
        && value[2..]
            .chars()
            .all(|character| character.is_ascii_digit())
    {
        Some(format!("tt{}", &value[2..]))
    } else {
        None
    }
}

fn append_provider_note(note: &mut Option<String>, fragment: &str) {
    match note {
        Some(value) => {
            if !value.ends_with(' ') {
                value.push(' ');
            }
            value.push_str(fragment);
        }
        None => *note = Some(fragment.to_owned()),
    }
}

#[derive(Debug, Deserialize)]
struct TmdbSearchResponse {
    #[serde(default)]
    results: Vec<TmdbMovieSearchResult>,
}

impl TmdbSearchResponse {
    fn from_value(value: serde_json::Value) -> anyhow::Result<Self> {
        let items = value
            .get("results")
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "failed to parse TMDB search movie response: missing field `results`"
                )
            })?
            .as_array()
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "failed to parse TMDB search movie response: `results` must be an array"
                )
            })?;
        let mut skipped_count = 0usize;
        let results = items
            .iter()
            .filter_map(|item| match serde_json::from_value::<TmdbMovieSearchResult>(item.clone()) {
                Ok(result) if result.id > 0 => Some(result),
                Ok(_) => {
                    skipped_count += 1;
                    tracing::warn!(
                        provider = TMDB_PROVIDER_ID,
                        "skipping TMDB search result item with zero id"
                    );
                    None
                }
                Err(error) => {
                    skipped_count += 1;
                    tracing::warn!(provider = TMDB_PROVIDER_ID, %error, "skipping malformed TMDB search result item");
                    None
                }
            })
            .collect();
        if !items.is_empty() && skipped_count == items.len() {
            anyhow::bail!("all TMDB search result items were malformed");
        }
        Ok(Self { results })
    }
}

#[derive(Debug, Deserialize)]
struct TmdbFindResponse {
    #[serde(default)]
    movie_results: Vec<TmdbFindMovieResult>,
}

impl TmdbFindResponse {
    fn from_value(value: serde_json::Value) -> anyhow::Result<Self> {
        serde_json::from_value(value)
            .map_err(|error| anyhow::anyhow!("failed to parse TMDB find response: {error}"))
    }

    fn first_movie_id(&self) -> Option<u64> {
        self.movie_results
            .iter()
            .map(|movie| movie.id)
            .find(|movie_id| *movie_id > 0)
    }
}

#[derive(Debug, Deserialize)]
struct TmdbFindMovieResult {
    id: u64,
}

#[derive(Clone, Debug, Deserialize)]
struct TmdbMovieSearchResult {
    id: u64,
    title: Option<String>,
    original_title: Option<String>,
    overview: Option<String>,
    release_date: Option<String>,
    poster_path: Option<String>,
    backdrop_path: Option<String>,
    #[serde(default)]
    genre_ids: Vec<u64>,
    vote_average: Option<f64>,
    vote_count: Option<u32>,
}

impl TmdbMovieSearchResult {
    fn direct_lookup_seed(id: u64) -> Self {
        Self {
            id,
            title: None,
            original_title: None,
            overview: None,
            release_date: None,
            poster_path: None,
            backdrop_path: None,
            genre_ids: Vec::new(),
            vote_average: None,
            vote_count: None,
        }
    }

    fn into_degraded_candidate(self, query: &MetadataQuery) -> ProviderMetadataCandidate {
        let TmdbMovieSearchResult {
            id,
            title: search_title,
            original_title,
            overview,
            release_date,
            poster_path,
            backdrop_path,
            genre_ids,
            vote_average,
            vote_count,
        } = self;
        let title = first_non_empty(&[search_title.as_deref(), original_title.as_deref()])
            .unwrap_or_else(|| query.title.clone());
        let original_title = first_non_empty(&[original_title.as_deref()]);
        let overview = non_empty(overview);
        let release_date = non_empty(release_date);
        let release_year = release_year(release_date.as_deref());
        let genres = Some(
            genre_ids
                .into_iter()
                .filter_map(tmdb_genre_name)
                .map(str::to_owned)
                .collect(),
        )
        .filter(|genres: &Vec<String>| !genres.is_empty());
        let alternate_titles = tmdb_alternate_titles(
            &title,
            [original_title.as_deref(), search_title.as_deref()],
            TmdbMovieAlternativeTitles::default(),
        );
        let mut tags = vec!["tmdb".to_owned(), "tmdb_degraded".to_owned()];
        if let Some(vote_average) = vote_average {
            tags.push(format!("tmdb_vote_average:{vote_average:.1}"));
        }
        if let Some(vote_count) = vote_count {
            tags.push(format!("tmdb_vote_count:{vote_count}"));
        }
        let mut artwork_candidates = Vec::new();
        if let Some(poster_path) = non_empty(poster_path) {
            artwork_candidates.push(tmdb_artwork_candidate(
                TMDB_PROVIDER_ID,
                id,
                AddonArtworkKind::Poster,
                tmdb_image_url(&poster_path),
            ));
        }
        if let Some(backdrop_path) = non_empty(backdrop_path) {
            artwork_candidates.push(tmdb_artwork_candidate(
                TMDB_PROVIDER_ID,
                id,
                AddonArtworkKind::Backdrop,
                tmdb_image_url(&backdrop_path),
            ));
        }

        ProviderMetadataCandidate {
            provider: TMDB_PROVIDER_ID.to_owned(),
            provider_id: format!("tmdb:movie:{id}"),
            patch: AddonMetadataPatch {
                title: Some(title.clone()),
                original_title: original_title.clone().filter(|value| value != &title),
                sort_title: Some(title.clone()),
                overview,
                release_date,
                runtime_minutes: None,
                tagline: None,
                genres,
                tags: Some(tags),
            },
            facts: ProviderCandidateFacts {
                title: Some(title),
                alternate_titles,
                release_year: release_year.map(i32::from),
                language: Some(query.language.clone()),
                community_score_milli: vote_average
                    .map(|value| (value * 100.0).round().clamp(0.0, 1000.0) as u16),
                community_vote_count: vote_count,
                external_ids: vec![ProviderExternalId {
                    provider: TMDB_PROVIDER_ID.to_owned(),
                    value: id.to_string(),
                }],
                provider_note: Some(
                    "TMDB movie candidate degraded from search response after enrichment failure."
                        .to_owned(),
                ),
            },
            artwork_candidates,
        }
    }
}

#[derive(Debug, Deserialize)]
struct TmdbMovieDetail {
    id: u64,
    title: Option<String>,
    original_title: Option<String>,
    overview: Option<String>,
    release_date: Option<String>,
    runtime: Option<u32>,
    tagline: Option<String>,
    original_language: Option<String>,
    poster_path: Option<String>,
    backdrop_path: Option<String>,
    #[serde(default)]
    genres: Vec<TmdbGenre>,
    vote_average: Option<f64>,
    vote_count: Option<u32>,
}

impl TmdbMovieDetail {
    fn from_value(value: serde_json::Value) -> anyhow::Result<Self> {
        serde_json::from_value(value)
            .map_err(|error| anyhow::anyhow!("failed to parse TMDB movie detail response: {error}"))
    }
}

#[derive(Debug, Deserialize)]
struct TmdbGenre {
    id: u64,
    name: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct TmdbMovieExternalIds {
    imdb_id: Option<String>,
    wikidata_id: Option<String>,
    facebook_id: Option<String>,
    instagram_id: Option<String>,
    twitter_id: Option<String>,
}

impl TmdbMovieExternalIds {
    fn from_value(value: serde_json::Value) -> anyhow::Result<Self> {
        serde_json::from_value(value).map_err(|error| {
            anyhow::anyhow!("failed to parse TMDB movie external IDs response: {error}")
        })
    }

    fn into_external_ids(self, tmdb_id: u64) -> Vec<ProviderExternalId> {
        let mut ids = vec![ProviderExternalId {
            provider: TMDB_PROVIDER_ID.to_owned(),
            value: tmdb_id.to_string(),
        }];
        push_external_id(&mut ids, "imdb", self.imdb_id);
        push_external_id(&mut ids, "wikidata", self.wikidata_id);
        push_external_id(&mut ids, "facebook", self.facebook_id);
        push_external_id(&mut ids, "instagram", self.instagram_id);
        push_external_id(&mut ids, "twitter", self.twitter_id);
        ids
    }
}

#[derive(Debug, Default, Deserialize)]
struct TmdbMovieAlternativeTitles {
    #[serde(default)]
    titles: Vec<TmdbAlternativeTitle>,
}

impl TmdbMovieAlternativeTitles {
    fn from_value(value: serde_json::Value) -> anyhow::Result<Self> {
        serde_json::from_value(value).map_err(|error| {
            anyhow::anyhow!("failed to parse TMDB movie alternative titles response: {error}")
        })
    }
}

#[derive(Debug, Deserialize)]
struct TmdbAlternativeTitle {
    title: Option<String>,
}

struct TmdbMovieCandidate {
    search: TmdbMovieSearchResult,
    detail: TmdbMovieDetail,
    external_ids: TmdbMovieExternalIds,
    alternative_titles: TmdbMovieAlternativeTitles,
    partial_enrichment: bool,
}

impl TmdbMovieCandidate {
    fn into_candidate(self, query: &MetadataQuery) -> ProviderMetadataCandidate {
        let title = first_non_empty(&[
            self.detail.title.as_deref(),
            self.search.title.as_deref(),
            self.detail.original_title.as_deref(),
            self.search.original_title.as_deref(),
        ])
        .unwrap_or_else(|| query.title.clone());
        let original_title = first_non_empty(&[
            self.detail.original_title.as_deref(),
            self.search.original_title.as_deref(),
        ]);
        let overview = non_empty(self.detail.overview).or_else(|| non_empty(self.search.overview));
        let release_date =
            non_empty(self.detail.release_date).or_else(|| non_empty(self.search.release_date));
        let release_year = release_year(release_date.as_deref());
        let genres = detail_genre_names(&self.detail.genres).or_else(|| {
            Some(
                self.search
                    .genre_ids
                    .into_iter()
                    .filter_map(tmdb_genre_name)
                    .map(str::to_owned)
                    .collect(),
            )
            .filter(|genres: &Vec<String>| !genres.is_empty())
        });
        let vote_average = self.detail.vote_average.or(self.search.vote_average);
        let vote_count = self.detail.vote_count.or(self.search.vote_count);
        let external_ids = self.external_ids.into_external_ids(self.detail.id);
        let alternate_titles = tmdb_alternate_titles(
            &title,
            [
                original_title.as_deref(),
                self.detail.title.as_deref(),
                self.search.title.as_deref(),
                self.detail.original_title.as_deref(),
                self.search.original_title.as_deref(),
            ],
            self.alternative_titles,
        );
        let mut tags = vec!["tmdb".to_owned()];
        if let Some(vote_average) = vote_average {
            tags.push(format!("tmdb_vote_average:{vote_average:.1}"));
        }
        if let Some(vote_count) = vote_count {
            tags.push(format!("tmdb_vote_count:{vote_count}"));
        }
        let mut artwork_candidates = Vec::new();
        if let Some(poster_path) = non_empty(self.detail.poster_path) {
            artwork_candidates.push(tmdb_artwork_candidate(
                TMDB_PROVIDER_ID,
                self.detail.id,
                AddonArtworkKind::Poster,
                tmdb_image_url(&poster_path),
            ));
        }
        if let Some(backdrop_path) = non_empty(self.detail.backdrop_path) {
            artwork_candidates.push(tmdb_artwork_candidate(
                TMDB_PROVIDER_ID,
                self.detail.id,
                AddonArtworkKind::Backdrop,
                tmdb_image_url(&backdrop_path),
            ));
        }

        ProviderMetadataCandidate {
            provider: TMDB_PROVIDER_ID.to_owned(),
            provider_id: format!("tmdb:movie:{}", self.detail.id),
            patch: AddonMetadataPatch {
                title: Some(title.clone()),
                original_title: original_title.clone().filter(|value| value != &title),
                sort_title: Some(title.clone()),
                overview,
                release_date,
                runtime_minutes: self.detail.runtime,
                tagline: non_empty(self.detail.tagline),
                genres,
                tags: Some(tags).filter(|tags| !tags.is_empty()),
            },
            facts: ProviderCandidateFacts {
                title: Some(title),
                alternate_titles,
                release_year: release_year.map(i32::from),
                language: non_empty(self.detail.original_language)
                    .or_else(|| Some(query.language.clone())),
                community_score_milli: vote_average
                    .map(|value| (value * 100.0).round().clamp(0.0, 1000.0) as u16),
                community_vote_count: vote_count,
                external_ids,
                provider_note: Some(
                    if self.partial_enrichment {
                        "TMDB movie candidate partially enriched with search and detail responses after secondary enrichment failure."
                    } else {
                        "TMDB movie candidate enriched with search, detail, and external ID responses."
                    }
                    .to_owned(),
                ),
            },
            artwork_candidates,
        }
    }
}

fn first_non_empty(values: &[Option<&str>]) -> Option<String> {
    values
        .iter()
        .flatten()
        .find_map(|value| normalize_non_empty(value))
}

fn non_empty(value: Option<String>) -> Option<String> {
    value.and_then(|value| normalize_non_empty(&value))
}

fn tmdb_alternate_titles<const N: usize>(
    selected_title: &str,
    known_titles: [Option<&str>; N],
    alternative_titles: TmdbMovieAlternativeTitles,
) -> Vec<String> {
    let mut titles = Vec::new();
    for title in known_titles.into_iter().flatten() {
        push_unique_title(&mut titles, selected_title, title);
    }
    for title in alternative_titles
        .titles
        .into_iter()
        .filter_map(|title| title.title)
    {
        push_unique_title(&mut titles, selected_title, &title);
    }
    titles
}

fn push_unique_title(values: &mut Vec<String>, selected_title: &str, title: &str) {
    let title = title.trim();
    if title.is_empty() || title == selected_title || values.iter().any(|value| value == title) {
        return;
    }
    values.push(title.to_owned());
}

fn tmdb_artwork_candidate(
    provider: &str,
    movie_id: u64,
    kind: AddonArtworkKind,
    source_url: String,
) -> crate::engine::ProviderArtworkCandidate {
    crate::engine::ProviderArtworkCandidate {
        provider: provider.to_owned(),
        provider_id: format!("tmdb:movie:{movie_id}"),
        facts: crate::engine::ProviderArtworkCandidateFacts {
            kind,
            source_url,
            language: None,
            width: None,
            height: None,
        },
    }
}

fn tmdb_image_url(path: &str) -> String {
    format!(
        "{}/{}",
        TMDB_IMAGE_BASE_URL.trim_end_matches('/'),
        path.trim_start_matches('/')
    )
}

fn push_external_id(ids: &mut Vec<ProviderExternalId>, provider: &str, value: Option<String>) {
    if let Some(value) = non_empty(value) {
        ids.push(ProviderExternalId {
            provider: provider.to_owned(),
            value,
        });
    }
}

fn normalize_non_empty(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_owned())
}

fn release_year(value: Option<&str>) -> Option<u16> {
    let value = value?.trim();
    if value
        .as_bytes()
        .get(4)
        .is_some_and(|value| value.is_ascii_digit())
    {
        return None;
    }
    let year = value.get(0..4)?;
    year.parse::<u16>().ok().filter(|year| *year > 0)
}

fn detail_genre_names(genres: &[TmdbGenre]) -> Option<Vec<String>> {
    Some(
        genres
            .iter()
            .filter(|genre| genre.id != 0)
            .filter_map(|genre| genre.name.as_ref())
            .filter_map(|name| normalize_non_empty(name))
            .collect(),
    )
    .filter(|genres: &Vec<String>| !genres.is_empty())
}

fn tmdb_genre_name(id: u64) -> Option<&'static str> {
    match id {
        12 => Some("Adventure"),
        16 => Some("Animation"),
        18 => Some("Drama"),
        28 => Some("Action"),
        35 => Some("Comedy"),
        878 => Some("Science Fiction"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::VecDeque,
        sync::{Arc, Mutex},
    };

    use crate::providers::http_runtime::{
        ProviderHttpRequest, ProviderHttpResponse, ProviderHttpRuntimeConfig, ProviderHttpTransport,
    };

    use super::*;

    #[test]
    fn tmdb_query_movie_ids_ignores_zero_and_invalid_values() {
        let query = MetadataQuery {
            title: "The Matrix".to_owned(),
            year: Some(1999),
            language: "en-US".to_owned(),
            external_ids: vec![
                crate::engine::QueryExternalId {
                    provider: "tmdb".to_owned(),
                    value: "0".to_owned(),
                },
                crate::engine::QueryExternalId {
                    provider: "TMDB".to_owned(),
                    value: "603".to_owned(),
                },
                crate::engine::QueryExternalId {
                    provider: "tmdb".to_owned(),
                    value: "603".to_owned(),
                },
                crate::engine::QueryExternalId {
                    provider: "tmdb".to_owned(),
                    value: "not-a-number".to_owned(),
                },
            ],
        };

        let movie_ids = tmdb_query_movie_ids(&query).collect::<Vec<_>>();

        assert_eq!(movie_ids, vec![603]);
    }

    #[test]
    fn tmdb_find_response_ignores_zero_movie_ids() {
        let response = TmdbFindResponse {
            movie_results: vec![
                TmdbFindMovieResult { id: 0 },
                TmdbFindMovieResult { id: 603 },
            ],
        };

        assert_eq!(response.first_movie_id(), Some(603));
    }

    #[test]
    fn tmdb_release_year_ignores_zero_year_values() {
        assert_eq!(release_year(Some("0000-03-31")), None);
        assert_eq!(release_year(Some("1999-03-31")), Some(1999));
        assert_eq!(release_year(Some(" 1999-03-31 ")), Some(1999));
        assert_eq!(release_year(Some("10000-03-31")), None);
    }

    #[tokio::test]
    async fn tmdb_search_omits_primary_release_year_when_query_year_is_invalid() {
        let transport = FakeTransport::default();
        for _ in 0..2 {
            transport.push(Ok(ProviderHttpResponse {
                status: 200,
                body: br#"{"results": []}"#.to_vec(),
            }));
        }
        let runtime = ProviderHttpRuntime::with_transport(
            ProviderHttpRuntimeConfig {
                retry_backoff_ms: 0,
                ..ProviderHttpRuntimeConfig::default()
            },
            transport.clone(),
        );
        let provider = TmdbMetadataProvider::with_runtime(
            TmdbProviderConfig {
                read_access_token: None,
                api_base_url: "https://tmdb.example/3".to_owned(),
                language: "en-US".to_owned(),
                include_adult: false,
                proxy_url: None,
            },
            runtime,
        );

        for year in [0, 10000] {
            provider
                .search_movies(
                    &MetadataQuery {
                        title: "The Matrix".to_owned(),
                        year: Some(year),
                        language: "en-US".to_owned(),
                        external_ids: Vec::new(),
                    },
                    "The Matrix",
                )
                .await
                .unwrap();
        }

        let requests = transport.requests();
        assert_eq!(requests.len(), 2);
        for request in requests {
            assert!(
                request
                    .query
                    .iter()
                    .all(|(key, _)| key != "primary_release_year")
            );
        }
    }

    #[test]
    fn tmdb_search_response_skips_zero_id_items() {
        let response = TmdbSearchResponse::from_value(serde_json::json!({
            "results": [
                {"id": 0},
                {"id": 603}
            ]
        }))
        .unwrap();

        assert_eq!(response.results.len(), 1);
        assert_eq!(response.results[0].id, 603);
    }

    #[tokio::test]
    async fn tmdb_provider_rejects_mismatched_detail_id_for_direct_lookup() {
        let transport = FakeTransport::default();
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{"id": 999}"#.to_vec(),
        }));
        let runtime = ProviderHttpRuntime::with_transport(
            ProviderHttpRuntimeConfig {
                retry_backoff_ms: 0,
                ..ProviderHttpRuntimeConfig::default()
            },
            transport.clone(),
        );
        let provider = TmdbMetadataProvider::with_runtime(
            TmdbProviderConfig {
                read_access_token: None,
                api_base_url: "https://tmdb.example/3".to_owned(),
                language: "en-US".to_owned(),
                include_adult: false,
                proxy_url: None,
            },
            runtime,
        );

        let error = provider
            .enrich_movie_candidate_from_seed(
                &MetadataQuery {
                    title: "The Matrix".to_owned(),
                    year: Some(1999),
                    language: "en-US".to_owned(),
                    external_ids: Vec::new(),
                },
                TmdbMovieSearchResult::direct_lookup_seed(603),
                603,
            )
            .await
            .unwrap_err();

        assert!(
            error
                .to_string()
                .contains("did not match requested movie 603")
        );
        assert_eq!(transport.requests().len(), 1);
    }

    #[tokio::test]
    async fn tmdb_provider_uses_http_runtime_and_maps_movie_candidates() {
        let transport = FakeTransport::default();
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{
                "results": [{
                    "id": 603,
                    "title": "The Matrix",
                    "original_title": "The Matrix",
                    "overview": "A synthetic test overview.",
                    "release_date": "1999-03-31",
                    "genre_ids": [28, 878],
                    "vote_average": 8.2,
                    "vote_count": 12345
                }]
            }"#
            .to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{
                "id": 603,
                "title": "The Matrix",
                "original_title": "The Matrix",
                "overview": "A detail overview.",
                "release_date": "1999-03-31",
                "runtime": 136,
                "tagline": "Welcome to the Real World.",
                "original_language": "en",
                "poster_path": "/poster.jpg",
                "backdrop_path": "/backdrop.jpg",
                "genres": [
                    {"id": 28, "name": "Action"},
                    {"id": 878, "name": "Science Fiction"}
                ],
                "vote_average": 8.7,
                "vote_count": 23456
            }"#
            .to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{
                "imdb_id": "tt0133093",
                "wikidata_id": "Q83495"
            }"#
            .to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: r#"{
                "id": 603,
                "titles": [
                    {"iso_3166_1": "CN", "title": "黑客帝国", "type": "localized"},
                    {"iso_3166_1": "US", "title": "The Matrix", "type": "original"}
                ]
            }"#
            .as_bytes()
            .to_vec(),
        }));
        let runtime = ProviderHttpRuntime::with_transport(
            ProviderHttpRuntimeConfig {
                retry_backoff_ms: 0,
                ..ProviderHttpRuntimeConfig::default()
            },
            transport.clone(),
        );
        let provider = TmdbMetadataProvider::with_runtime(
            TmdbProviderConfig {
                read_access_token: Some("tmdb-token".to_owned()),
                api_base_url: "https://tmdb.example/3".to_owned(),
                language: "en-US".to_owned(),
                include_adult: false,
                proxy_url: None,
            },
            runtime,
        );

        let candidates = provider
            .suggest(&MetadataQuery {
                title: "The Matrix".to_owned(),
                year: Some(1999),
                language: "zh-CN".to_owned(),
                external_ids: Vec::new(),
            })
            .await
            .unwrap();

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].provider, "tmdb");
        assert_eq!(candidates[0].provider_id, "tmdb:movie:603");
        assert_eq!(candidates[0].patch.title.as_deref(), Some("The Matrix"));
        assert_eq!(
            candidates[0].patch.overview.as_deref(),
            Some("A detail overview.")
        );
        assert_eq!(candidates[0].patch.runtime_minutes, Some(136));
        assert_eq!(
            candidates[0].patch.tagline.as_deref(),
            Some("Welcome to the Real World.")
        );
        assert_eq!(candidates[0].facts.title.as_deref(), Some("The Matrix"));
        assert!(
            candidates[0]
                .facts
                .alternate_titles
                .iter()
                .any(|title| title == "黑客帝国")
        );
        assert_eq!(candidates[0].facts.release_year, Some(1999));
        assert_eq!(candidates[0].facts.community_score_milli, Some(870));
        assert_eq!(candidates[0].facts.community_vote_count, Some(23456));
        assert_eq!(
            candidates[0].patch.genres.as_ref().unwrap(),
            &vec!["Action".to_owned(), "Science Fiction".to_owned()]
        );
        assert_eq!(candidates[0].facts.external_ids[0].value, "603");
        assert!(
            candidates[0]
                .facts
                .external_ids
                .iter()
                .any(|id| id.provider == "imdb" && id.value == "tt0133093")
        );
        assert!(
            candidates[0]
                .artwork_candidates
                .iter()
                .any(|candidate| candidate.facts.kind == AddonArtworkKind::Poster
                    && candidate.facts.source_url
                        == "https://image.tmdb.org/t/p/original/poster.jpg")
        );
        assert!(
            candidates[0]
                .artwork_candidates
                .iter()
                .any(
                    |candidate| candidate.facts.kind == AddonArtworkKind::Backdrop
                        && candidate.facts.source_url
                            == "https://image.tmdb.org/t/p/original/backdrop.jpg"
                )
        );

        let requests = transport.requests();
        assert_eq!(requests[0].url, "https://tmdb.example/3/search/movie");
        assert_eq!(
            requests[0].headers,
            vec![("authorization".to_owned(), "Bearer tmdb-token".to_owned())]
        );
        assert!(
            requests[0]
                .query
                .contains(&("primary_release_year".to_owned(), "1999".to_owned()))
        );
        assert_eq!(requests[1].url, "https://tmdb.example/3/movie/603");
        assert_eq!(
            requests[1].query,
            vec![("language".to_owned(), "en-US".to_owned())]
        );
        assert_eq!(
            requests[2].url,
            "https://tmdb.example/3/movie/603/external_ids"
        );
        assert_eq!(
            requests[3].url,
            "https://tmdb.example/3/movie/603/alternative_titles"
        );
        assert!(transport.configs()[0].proxy_url.is_none());
    }

    #[tokio::test]
    async fn tmdb_provider_new_uses_proxy_url_from_config() {
        let provider = TmdbMetadataProvider::new(TmdbProviderConfig {
            read_access_token: Some("tmdb-token".to_owned()),
            api_base_url: "https://tmdb.example/3".to_owned(),
            language: "en-US".to_owned(),
            include_adult: false,
            proxy_url: Some("http://proxy.example:8080".to_owned()),
        })
        .unwrap();

        assert_eq!(
            provider.runtime.config().proxy_url.as_deref(),
            Some("http://proxy.example:8080")
        );
    }

    #[test]
    fn tmdb_candidate_mapping_trims_provider_text_boundaries() {
        let candidate = TmdbMovieCandidate {
            search: TmdbMovieSearchResult {
                id: 603,
                title: Some(" Search Title ".to_owned()),
                original_title: Some(" Search Original ".to_owned()),
                overview: Some(" Search overview. ".to_owned()),
                release_date: Some(" 1999-03-30 ".to_owned()),
                poster_path: Some(" /search-poster.jpg ".to_owned()),
                backdrop_path: None,
                genre_ids: vec![28],
                vote_average: Some(8.0),
                vote_count: Some(1000),
            },
            detail: TmdbMovieDetail {
                id: 603,
                title: Some(" The Matrix ".to_owned()),
                original_title: Some(" The Matrix Original ".to_owned()),
                overview: Some(" A detail overview. ".to_owned()),
                release_date: Some(" 1999-03-31 ".to_owned()),
                runtime: Some(136),
                tagline: Some(" Welcome to the Real World. ".to_owned()),
                original_language: Some(" en ".to_owned()),
                poster_path: Some(" /poster.jpg ".to_owned()),
                backdrop_path: Some(" /backdrop.jpg ".to_owned()),
                genres: vec![
                    TmdbGenre {
                        id: 28,
                        name: Some(" Action ".to_owned()),
                    },
                    TmdbGenre {
                        id: 878,
                        name: Some("   ".to_owned()),
                    },
                ],
                vote_average: Some(8.7),
                vote_count: Some(23456),
            },
            external_ids: TmdbMovieExternalIds {
                imdb_id: Some(" tt0133093 ".to_owned()),
                wikidata_id: Some(" Q83495 ".to_owned()),
                facebook_id: Some("   ".to_owned()),
                instagram_id: None,
                twitter_id: None,
            },
            alternative_titles: TmdbMovieAlternativeTitles {
                titles: vec![
                    TmdbAlternativeTitle {
                        title: Some(" 黑客帝国 ".to_owned()),
                    },
                    TmdbAlternativeTitle {
                        title: Some("   ".to_owned()),
                    },
                ],
            },
            partial_enrichment: false,
        }
        .into_candidate(&MetadataQuery {
            title: "The Matrix".to_owned(),
            year: Some(1999),
            language: "zh-CN".to_owned(),
            external_ids: Vec::new(),
        });

        assert_eq!(candidate.patch.title.as_deref(), Some("The Matrix"));
        assert_eq!(
            candidate.patch.original_title.as_deref(),
            Some("The Matrix Original")
        );
        assert_eq!(
            candidate.patch.overview.as_deref(),
            Some("A detail overview.")
        );
        assert_eq!(candidate.patch.release_date.as_deref(), Some("1999-03-31"));
        assert_eq!(
            candidate.patch.tagline.as_deref(),
            Some("Welcome to the Real World.")
        );
        assert_eq!(candidate.facts.release_year, Some(1999));
        assert_eq!(candidate.facts.language.as_deref(), Some("en"));
        assert_eq!(
            candidate.patch.genres.as_ref().unwrap(),
            &vec!["Action".to_owned()]
        );
        assert!(
            candidate
                .facts
                .external_ids
                .iter()
                .any(|id| id.provider == "imdb" && id.value == "tt0133093")
        );
        assert!(
            candidate
                .facts
                .external_ids
                .iter()
                .any(|id| id.provider == "wikidata" && id.value == "Q83495")
        );
        assert!(
            candidate
                .facts
                .alternate_titles
                .iter()
                .any(|title| title == "黑客帝国")
        );
        assert!(
            candidate
                .artwork_candidates
                .iter()
                .any(|candidate| candidate.facts.source_url
                    == "https://image.tmdb.org/t/p/original/poster.jpg")
        );
        assert!(
            candidate
                .artwork_candidates
                .iter()
                .any(|candidate| candidate.facts.source_url
                    == "https://image.tmdb.org/t/p/original/backdrop.jpg")
        );
    }

    #[tokio::test]
    async fn tmdb_provider_uses_query_external_id_for_direct_movie_lookup() {
        let transport = FakeTransport::default();
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{
                "id": 603,
                "title": "The Matrix",
                "original_title": "The Matrix",
                "overview": "A direct detail overview.",
                "release_date": "1999-03-31",
                "runtime": 136,
                "tagline": "Welcome to the Real World.",
                "original_language": "en",
                "poster_path": "/poster.jpg",
                "backdrop_path": null,
                "genres": [{"id": 28, "name": "Action"}],
                "vote_average": 8.7,
                "vote_count": 23456
            }"#
            .to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{"imdb_id": "tt0133093"}"#.to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: r#"{"titles": [{"title": "黑客帝国"}]}"#.as_bytes().to_vec(),
        }));
        let runtime = ProviderHttpRuntime::with_transport(
            ProviderHttpRuntimeConfig {
                retry_backoff_ms: 0,
                ..ProviderHttpRuntimeConfig::default()
            },
            transport.clone(),
        );
        let provider = TmdbMetadataProvider::with_runtime(
            TmdbProviderConfig {
                read_access_token: None,
                api_base_url: "https://tmdb.example/3".to_owned(),
                language: "en-US".to_owned(),
                include_adult: false,
                proxy_url: None,
            },
            runtime,
        );

        let candidates = provider
            .suggest(&MetadataQuery {
                title: "Wrong Local Title".to_owned(),
                year: Some(1999),
                language: "zh-CN".to_owned(),
                external_ids: vec![crate::engine::QueryExternalId {
                    provider: "tmdb".to_owned(),
                    value: "603".to_owned(),
                }],
            })
            .await
            .unwrap();

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].provider_id, "tmdb:movie:603");
        assert_eq!(candidates[0].patch.title.as_deref(), Some("The Matrix"));
        assert_eq!(
            candidates[0].patch.overview.as_deref(),
            Some("A direct detail overview.")
        );
        assert!(
            candidates[0]
                .facts
                .external_ids
                .iter()
                .any(|id| id.provider == "tmdb" && id.value == "603")
        );
        assert!(
            candidates[0]
                .facts
                .external_ids
                .iter()
                .any(|id| id.provider == "imdb" && id.value == "tt0133093")
        );
        assert!(
            candidates[0]
                .facts
                .alternate_titles
                .iter()
                .any(|title| title == "黑客帝国")
        );

        let requests = transport.requests();
        assert_eq!(requests.len(), 3);
        assert_eq!(requests[0].url, "https://tmdb.example/3/movie/603");
        assert_eq!(
            requests[1].url,
            "https://tmdb.example/3/movie/603/external_ids"
        );
        assert_eq!(
            requests[2].url,
            "https://tmdb.example/3/movie/603/alternative_titles"
        );
        assert!(
            requests
                .iter()
                .all(|request| request.url != "https://tmdb.example/3/search/movie")
        );
    }

    #[tokio::test]
    async fn tmdb_provider_uses_query_imdb_external_id_for_find_movie_lookup() {
        let transport = FakeTransport::default();
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{
                "movie_results": [{
                    "id": 603,
                    "title": "The Matrix",
                    "original_title": "The Matrix",
                    "overview": "Find result.",
                    "release_date": "1999-03-31",
                    "genre_ids": [28],
                    "vote_average": 8.7,
                    "vote_count": 23456
                }],
                "tv_results": [],
                "person_results": []
            }"#
            .to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{
                "id": 603,
                "title": "The Matrix",
                "original_title": "The Matrix",
                "overview": "Detail from IMDb find.",
                "release_date": "1999-03-31",
                "runtime": 136,
                "tagline": "Welcome to the Real World.",
                "original_language": "en",
                "poster_path": null,
                "backdrop_path": null,
                "genres": [{"id": 28, "name": "Action"}],
                "vote_average": 8.7,
                "vote_count": 23456
            }"#
            .to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{"imdb_id": "tt0133093"}"#.to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: r#"{"titles": [{"title": "黑客帝国"}]}"#.as_bytes().to_vec(),
        }));
        let runtime = ProviderHttpRuntime::with_transport(
            ProviderHttpRuntimeConfig {
                retry_backoff_ms: 0,
                ..ProviderHttpRuntimeConfig::default()
            },
            transport.clone(),
        );
        let provider = TmdbMetadataProvider::with_runtime(
            TmdbProviderConfig {
                read_access_token: None,
                api_base_url: "https://tmdb.example/3".to_owned(),
                language: "en-US".to_owned(),
                include_adult: false,
                proxy_url: None,
            },
            runtime,
        );

        let candidates = provider
            .suggest(&MetadataQuery {
                title: "Wrong Local Title".to_owned(),
                year: Some(1999),
                language: "zh-CN".to_owned(),
                external_ids: vec![crate::engine::QueryExternalId {
                    provider: "imdb".to_owned(),
                    value: "tt0133093".to_owned(),
                }],
            })
            .await
            .unwrap();

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].provider_id, "tmdb:movie:603");
        assert_eq!(
            candidates[0].patch.overview.as_deref(),
            Some("Detail from IMDb find.")
        );
        assert!(
            candidates[0]
                .facts
                .external_ids
                .iter()
                .any(|id| id.provider == "imdb" && id.value == "tt0133093")
        );

        let requests = transport.requests();
        assert_eq!(requests.len(), 4);
        assert_eq!(requests[0].url, "https://tmdb.example/3/find/tt0133093");
        assert!(
            requests[0]
                .query
                .contains(&("external_source".to_owned(), "imdb_id".to_owned()))
        );
        assert_eq!(requests[1].url, "https://tmdb.example/3/movie/603");
        assert!(
            requests
                .iter()
                .all(|request| request.url != "https://tmdb.example/3/search/movie")
        );
    }

    #[tokio::test]
    async fn tmdb_provider_normalizes_query_imdb_external_id_case_for_find_lookup() {
        let transport = FakeTransport::default();
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{
                "movie_results": [{
                    "id": 603,
                    "title": "The Matrix",
                    "original_title": "The Matrix",
                    "overview": "Find result.",
                    "release_date": "1999-03-31",
                    "genre_ids": [28],
                    "vote_average": 8.7,
                    "vote_count": 23456
                }],
                "tv_results": [],
                "person_results": []
            }"#
            .to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{
                "id": 603,
                "title": "The Matrix",
                "original_title": "The Matrix",
                "overview": "Detail from normalized IMDb find.",
                "release_date": "1999-03-31",
                "runtime": 136,
                "tagline": null,
                "original_language": "en",
                "poster_path": null,
                "backdrop_path": null,
                "genres": [{"id": 28, "name": "Action"}],
                "vote_average": 8.7,
                "vote_count": 23456
            }"#
            .to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{"imdb_id": "tt0133093"}"#.to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{"titles": []}"#.to_vec(),
        }));
        let runtime = ProviderHttpRuntime::with_transport(
            ProviderHttpRuntimeConfig {
                retry_backoff_ms: 0,
                ..ProviderHttpRuntimeConfig::default()
            },
            transport.clone(),
        );
        let provider = TmdbMetadataProvider::with_runtime(
            TmdbProviderConfig {
                read_access_token: None,
                api_base_url: "https://tmdb.example/3".to_owned(),
                language: "en-US".to_owned(),
                include_adult: false,
                proxy_url: None,
            },
            runtime,
        );

        let candidates = provider
            .suggest(&MetadataQuery {
                title: "Wrong Local Title".to_owned(),
                year: Some(1999),
                language: "en-US".to_owned(),
                external_ids: vec![crate::engine::QueryExternalId {
                    provider: "IMDB".to_owned(),
                    value: "TT0133093".to_owned(),
                }],
            })
            .await
            .unwrap();

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].provider_id, "tmdb:movie:603");
        assert_eq!(
            candidates[0].patch.overview.as_deref(),
            Some("Detail from normalized IMDb find.")
        );
        let requests = transport.requests();
        assert_eq!(requests[0].url, "https://tmdb.example/3/find/tt0133093");
        assert!(
            requests
                .iter()
                .all(|request| request.url != "https://tmdb.example/3/search/movie")
        );
    }

    #[tokio::test]
    async fn tmdb_provider_falls_back_to_search_when_query_imdb_find_is_empty() {
        let transport = FakeTransport::default();
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{"movie_results": [], "tv_results": [], "person_results": []}"#.to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{
                "results": [{
                    "id": 603,
                    "title": "The Matrix",
                    "original_title": "The Matrix",
                    "overview": "Search summary.",
                    "release_date": "1999-03-31",
                    "genre_ids": [28],
                    "vote_average": 8.2,
                    "vote_count": 12345
                }]
            }"#
            .to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{
                "id": 603,
                "title": "The Matrix",
                "original_title": "The Matrix",
                "overview": "Detail from search fallback.",
                "release_date": "1999-03-31",
                "runtime": 136,
                "tagline": null,
                "original_language": "en",
                "poster_path": null,
                "backdrop_path": null,
                "genres": [{"id": 28, "name": "Action"}],
                "vote_average": 8.7,
                "vote_count": 23456
            }"#
            .to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{}"#.to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{"titles": []}"#.to_vec(),
        }));
        let runtime = ProviderHttpRuntime::with_transport(
            ProviderHttpRuntimeConfig {
                retry_backoff_ms: 0,
                ..ProviderHttpRuntimeConfig::default()
            },
            transport.clone(),
        );
        let provider = TmdbMetadataProvider::with_runtime(
            TmdbProviderConfig {
                read_access_token: None,
                api_base_url: "https://tmdb.example/3".to_owned(),
                language: "en-US".to_owned(),
                include_adult: false,
                proxy_url: None,
            },
            runtime,
        );

        let candidates = provider
            .suggest(&MetadataQuery {
                title: "The Matrix".to_owned(),
                year: Some(1999),
                language: "en-US".to_owned(),
                external_ids: vec![crate::engine::QueryExternalId {
                    provider: "imdb".to_owned(),
                    value: "tt0000000".to_owned(),
                }],
            })
            .await
            .unwrap();

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].provider_id, "tmdb:movie:603");
        assert_eq!(
            candidates[0].patch.overview.as_deref(),
            Some("Detail from search fallback.")
        );

        let requests = transport.requests();
        assert_eq!(requests[0].url, "https://tmdb.example/3/find/tt0000000");
        assert_eq!(requests[1].url, "https://tmdb.example/3/search/movie");
        assert_eq!(requests[2].url, "https://tmdb.example/3/movie/603");
    }

    #[tokio::test]
    async fn tmdb_provider_uses_later_imdb_external_id_when_first_find_is_empty() {
        let transport = FakeTransport::default();
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{"movie_results": [], "tv_results": [], "person_results": []}"#.to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{
                "movie_results": [{
                    "id": 603,
                    "title": "The Matrix",
                    "original_title": "The Matrix",
                    "overview": "Find result.",
                    "release_date": "1999-03-31",
                    "genre_ids": [28],
                    "vote_average": 8.7,
                    "vote_count": 23456
                }],
                "tv_results": [],
                "person_results": []
            }"#
            .to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{
                "id": 603,
                "title": "The Matrix",
                "original_title": "The Matrix",
                "overview": "Detail from later IMDb find.",
                "release_date": "1999-03-31",
                "runtime": 136,
                "tagline": null,
                "original_language": "en",
                "poster_path": null,
                "backdrop_path": null,
                "genres": [{"id": 28, "name": "Action"}],
                "vote_average": 8.7,
                "vote_count": 23456
            }"#
            .to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{"imdb_id": "tt0133093"}"#.to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{"titles": []}"#.to_vec(),
        }));
        let runtime = ProviderHttpRuntime::with_transport(
            ProviderHttpRuntimeConfig {
                retry_backoff_ms: 0,
                ..ProviderHttpRuntimeConfig::default()
            },
            transport.clone(),
        );
        let provider = TmdbMetadataProvider::with_runtime(
            TmdbProviderConfig {
                read_access_token: None,
                api_base_url: "https://tmdb.example/3".to_owned(),
                language: "en-US".to_owned(),
                include_adult: false,
                proxy_url: None,
            },
            runtime,
        );

        let candidates = provider
            .suggest(&MetadataQuery {
                title: "Wrong Local Title".to_owned(),
                year: Some(1999),
                language: "en-US".to_owned(),
                external_ids: vec![
                    crate::engine::QueryExternalId {
                        provider: "imdb".to_owned(),
                        value: "tt0000000".to_owned(),
                    },
                    crate::engine::QueryExternalId {
                        provider: "imdb".to_owned(),
                        value: "tt0133093".to_owned(),
                    },
                ],
            })
            .await
            .unwrap();

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].provider_id, "tmdb:movie:603");
        assert_eq!(
            candidates[0].patch.overview.as_deref(),
            Some("Detail from later IMDb find.")
        );
        let requests = transport.requests();
        assert_eq!(requests[0].url, "https://tmdb.example/3/find/tt0000000");
        assert_eq!(requests[1].url, "https://tmdb.example/3/find/tt0133093");
        assert!(
            requests
                .iter()
                .all(|request| request.url != "https://tmdb.example/3/search/movie")
        );
    }

    #[tokio::test]
    async fn tmdb_provider_uses_later_imdb_external_id_when_first_find_fails() {
        let transport = FakeTransport::default();
        transport.push(Ok(ProviderHttpResponse {
            status: 503,
            body: br#"temporarily unavailable"#.to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{
                "movie_results": [{
                    "id": 603,
                    "title": "The Matrix",
                    "original_title": "The Matrix",
                    "overview": "Find result.",
                    "release_date": "1999-03-31",
                    "genre_ids": [28],
                    "vote_average": 8.7,
                    "vote_count": 23456
                }],
                "tv_results": [],
                "person_results": []
            }"#
            .to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{
                "id": 603,
                "title": "The Matrix",
                "original_title": "The Matrix",
                "overview": "Detail after failed IMDb find.",
                "release_date": "1999-03-31",
                "runtime": 136,
                "tagline": null,
                "original_language": "en",
                "poster_path": null,
                "backdrop_path": null,
                "genres": [{"id": 28, "name": "Action"}],
                "vote_average": 8.7,
                "vote_count": 23456
            }"#
            .to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{"imdb_id": "tt0133093"}"#.to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{"titles": []}"#.to_vec(),
        }));
        let runtime = ProviderHttpRuntime::with_transport(
            ProviderHttpRuntimeConfig {
                max_attempts: 1,
                retry_backoff_ms: 0,
                ..ProviderHttpRuntimeConfig::default()
            },
            transport.clone(),
        );
        let provider = TmdbMetadataProvider::with_runtime(
            TmdbProviderConfig {
                read_access_token: None,
                api_base_url: "https://tmdb.example/3".to_owned(),
                language: "en-US".to_owned(),
                include_adult: false,
                proxy_url: None,
            },
            runtime,
        );

        let candidates = provider
            .suggest(&MetadataQuery {
                title: "Wrong Local Title".to_owned(),
                year: Some(1999),
                language: "en-US".to_owned(),
                external_ids: vec![
                    crate::engine::QueryExternalId {
                        provider: "imdb".to_owned(),
                        value: "tt0000000".to_owned(),
                    },
                    crate::engine::QueryExternalId {
                        provider: "imdb".to_owned(),
                        value: "tt0133093".to_owned(),
                    },
                ],
            })
            .await
            .unwrap();

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].provider_id, "tmdb:movie:603");
        assert_eq!(
            candidates[0].patch.overview.as_deref(),
            Some("Detail after failed IMDb find.")
        );
        let requests = transport.requests();
        assert_eq!(requests[0].url, "https://tmdb.example/3/find/tt0000000");
        assert_eq!(requests[1].url, "https://tmdb.example/3/find/tt0133093");
        assert!(
            requests
                .iter()
                .all(|request| request.url != "https://tmdb.example/3/search/movie")
        );
    }

    #[tokio::test]
    async fn tmdb_provider_falls_back_to_search_when_query_external_id_is_invalid() {
        let transport = FakeTransport::default();
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{
                "results": [{
                    "id": 603,
                    "title": "The Matrix",
                    "original_title": "The Matrix",
                    "overview": "Search summary.",
                    "release_date": "1999-03-31",
                    "genre_ids": [28],
                    "vote_average": 8.2,
                    "vote_count": 12345
                }]
            }"#
            .to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{
                "id": 603,
                "title": "The Matrix",
                "original_title": "The Matrix",
                "overview": "Detail summary.",
                "release_date": "1999-03-31",
                "runtime": 136,
                "tagline": null,
                "original_language": "en",
                "poster_path": null,
                "backdrop_path": null,
                "genres": [{"id": 28, "name": "Action"}],
                "vote_average": 8.7,
                "vote_count": 23456
            }"#
            .to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{}"#.to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{"titles": []}"#.to_vec(),
        }));
        let runtime = ProviderHttpRuntime::with_transport(
            ProviderHttpRuntimeConfig {
                retry_backoff_ms: 0,
                ..ProviderHttpRuntimeConfig::default()
            },
            transport.clone(),
        );
        let provider = TmdbMetadataProvider::with_runtime(
            TmdbProviderConfig {
                read_access_token: None,
                api_base_url: "https://tmdb.example/3".to_owned(),
                language: "en-US".to_owned(),
                include_adult: false,
                proxy_url: None,
            },
            runtime,
        );

        let candidates = provider
            .suggest(&MetadataQuery {
                title: "The Matrix".to_owned(),
                year: Some(1999),
                language: "en-US".to_owned(),
                external_ids: vec![crate::engine::QueryExternalId {
                    provider: "tmdb".to_owned(),
                    value: "not-a-number".to_owned(),
                }],
            })
            .await
            .unwrap();

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].provider_id, "tmdb:movie:603");
        let requests = transport.requests();
        assert_eq!(requests[0].url, "https://tmdb.example/3/search/movie");
        assert_eq!(requests[1].url, "https://tmdb.example/3/movie/603");
    }

    #[tokio::test]
    async fn tmdb_provider_uses_later_valid_query_external_id_when_first_is_invalid() {
        let transport = FakeTransport::default();
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{
                "id": 603,
                "title": "The Matrix",
                "original_title": "The Matrix",
                "overview": "Direct detail summary.",
                "release_date": "1999-03-31",
                "runtime": 136,
                "tagline": null,
                "original_language": "en",
                "poster_path": null,
                "backdrop_path": null,
                "genres": [{"id": 28, "name": "Action"}],
                "vote_average": 8.7,
                "vote_count": 23456
            }"#
            .to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{}"#.to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{"titles": []}"#.to_vec(),
        }));
        let runtime = ProviderHttpRuntime::with_transport(
            ProviderHttpRuntimeConfig {
                retry_backoff_ms: 0,
                ..ProviderHttpRuntimeConfig::default()
            },
            transport.clone(),
        );
        let provider = TmdbMetadataProvider::with_runtime(
            TmdbProviderConfig {
                read_access_token: None,
                api_base_url: "https://tmdb.example/3".to_owned(),
                language: "en-US".to_owned(),
                include_adult: false,
                proxy_url: None,
            },
            runtime,
        );

        let candidates = provider
            .suggest(&MetadataQuery {
                title: "Wrong Local Title".to_owned(),
                year: Some(1999),
                language: "en-US".to_owned(),
                external_ids: vec![
                    crate::engine::QueryExternalId {
                        provider: "tmdb".to_owned(),
                        value: "not-a-number".to_owned(),
                    },
                    crate::engine::QueryExternalId {
                        provider: "TMDB".to_owned(),
                        value: "603".to_owned(),
                    },
                ],
            })
            .await
            .unwrap();

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].provider_id, "tmdb:movie:603");
        let requests = transport.requests();
        assert_eq!(requests[0].url, "https://tmdb.example/3/movie/603");
        assert!(
            requests
                .iter()
                .all(|request| request.url != "https://tmdb.example/3/search/movie")
        );
    }

    #[tokio::test]
    async fn tmdb_provider_falls_back_to_search_when_direct_movie_lookup_fails() {
        let transport = FakeTransport::default();
        transport.push(Ok(ProviderHttpResponse {
            status: 404,
            body: br#"not found"#.to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{
                "results": [{
                    "id": 603,
                    "title": "The Matrix",
                    "original_title": "The Matrix",
                    "overview": "Search summary.",
                    "release_date": "1999-03-31",
                    "genre_ids": [28],
                    "vote_average": 8.2,
                    "vote_count": 12345
                }]
            }"#
            .to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{
                "id": 603,
                "title": "The Matrix",
                "original_title": "The Matrix",
                "overview": "Recovered detail summary.",
                "release_date": "1999-03-31",
                "runtime": 136,
                "tagline": null,
                "original_language": "en",
                "poster_path": null,
                "backdrop_path": null,
                "genres": [{"id": 28, "name": "Action"}],
                "vote_average": 8.7,
                "vote_count": 23456
            }"#
            .to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{}"#.to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{"titles": []}"#.to_vec(),
        }));
        let runtime = ProviderHttpRuntime::with_transport(
            ProviderHttpRuntimeConfig {
                max_attempts: 1,
                retry_backoff_ms: 0,
                ..ProviderHttpRuntimeConfig::default()
            },
            transport.clone(),
        );
        let provider = TmdbMetadataProvider::with_runtime(
            TmdbProviderConfig {
                read_access_token: None,
                api_base_url: "https://tmdb.example/3".to_owned(),
                language: "en-US".to_owned(),
                include_adult: false,
                proxy_url: None,
            },
            runtime,
        );

        let candidates = provider
            .suggest(&MetadataQuery {
                title: "The Matrix".to_owned(),
                year: Some(1999),
                language: "en-US".to_owned(),
                external_ids: vec![crate::engine::QueryExternalId {
                    provider: "tmdb".to_owned(),
                    value: "999999".to_owned(),
                }],
            })
            .await
            .unwrap();

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].provider_id, "tmdb:movie:603");
        assert_eq!(
            candidates[0].patch.overview.as_deref(),
            Some("Recovered detail summary.")
        );
        let requests = transport.requests();
        assert_eq!(requests[0].url, "https://tmdb.example/3/movie/999999");
        assert_eq!(requests[1].url, "https://tmdb.example/3/search/movie");
        assert_eq!(requests[2].url, "https://tmdb.example/3/movie/603");
    }

    #[tokio::test]
    async fn tmdb_provider_uses_later_valid_query_external_id_when_first_lookup_fails() {
        let transport = FakeTransport::default();
        transport.push(Ok(ProviderHttpResponse {
            status: 404,
            body: br#"not found"#.to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{
                "id": 603,
                "title": "The Matrix",
                "original_title": "The Matrix",
                "overview": "Direct detail overview.",
                "release_date": "1999-03-31",
                "runtime": 136,
                "tagline": null,
                "original_language": "en",
                "poster_path": null,
                "backdrop_path": null,
                "genres": [{"id": 28, "name": "Action"}],
                "vote_average": 8.7,
                "vote_count": 23456
            }"#
            .to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{}"#.to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{"titles": []}"#.to_vec(),
        }));
        let runtime = ProviderHttpRuntime::with_transport(
            ProviderHttpRuntimeConfig {
                max_attempts: 1,
                retry_backoff_ms: 0,
                ..ProviderHttpRuntimeConfig::default()
            },
            transport.clone(),
        );
        let provider = TmdbMetadataProvider::with_runtime(
            TmdbProviderConfig {
                read_access_token: None,
                api_base_url: "https://tmdb.example/3".to_owned(),
                language: "en-US".to_owned(),
                include_adult: false,
                proxy_url: None,
            },
            runtime,
        );

        let candidates = provider
            .suggest(&MetadataQuery {
                title: "Wrong Local Title".to_owned(),
                year: Some(1999),
                language: "en-US".to_owned(),
                external_ids: vec![
                    crate::engine::QueryExternalId {
                        provider: "tmdb".to_owned(),
                        value: "999999".to_owned(),
                    },
                    crate::engine::QueryExternalId {
                        provider: "tmdb".to_owned(),
                        value: "603".to_owned(),
                    },
                ],
            })
            .await
            .unwrap();

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].provider_id, "tmdb:movie:603");
        let requests = transport.requests();
        assert_eq!(requests[0].url, "https://tmdb.example/3/movie/999999");
        assert_eq!(requests[1].url, "https://tmdb.example/3/movie/603");
        assert!(
            requests
                .iter()
                .all(|request| request.url != "https://tmdb.example/3/search/movie")
        );
    }

    #[tokio::test]
    async fn tmdb_provider_deduplicates_query_external_ids_before_direct_lookup() {
        let transport = FakeTransport::default();
        transport.push(Ok(ProviderHttpResponse {
            status: 404,
            body: br#"not found"#.to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{
                "id": 603,
                "title": "The Matrix",
                "original_title": "The Matrix",
                "overview": "Direct detail summary.",
                "release_date": "1999-03-31",
                "runtime": 136,
                "tagline": null,
                "original_language": "en",
                "poster_path": null,
                "backdrop_path": null,
                "genres": [{"id": 28, "name": "Action"}],
                "vote_average": 8.7,
                "vote_count": 23456
            }"#
            .to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{}"#.to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{"titles": []}"#.to_vec(),
        }));
        let runtime = ProviderHttpRuntime::with_transport(
            ProviderHttpRuntimeConfig {
                max_attempts: 1,
                retry_backoff_ms: 0,
                ..ProviderHttpRuntimeConfig::default()
            },
            transport.clone(),
        );
        let provider = TmdbMetadataProvider::with_runtime(
            TmdbProviderConfig {
                read_access_token: None,
                api_base_url: "https://tmdb.example/3".to_owned(),
                language: "en-US".to_owned(),
                include_adult: false,
                proxy_url: None,
            },
            runtime,
        );

        let candidates = provider
            .suggest(&MetadataQuery {
                title: "Wrong Local Title".to_owned(),
                year: Some(1999),
                language: "en-US".to_owned(),
                external_ids: vec![
                    crate::engine::QueryExternalId {
                        provider: "tmdb".to_owned(),
                        value: "999999".to_owned(),
                    },
                    crate::engine::QueryExternalId {
                        provider: "TMDB".to_owned(),
                        value: "999999".to_owned(),
                    },
                    crate::engine::QueryExternalId {
                        provider: "tmdb".to_owned(),
                        value: "603".to_owned(),
                    },
                ],
            })
            .await
            .unwrap();

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].provider_id, "tmdb:movie:603");
        let requests = transport.requests();
        let failed_lookup_count = requests
            .iter()
            .filter(|request| request.url == "https://tmdb.example/3/movie/999999")
            .count();
        assert_eq!(failed_lookup_count, 1);
        assert_eq!(requests[1].url, "https://tmdb.example/3/movie/603");
    }

    #[tokio::test]
    async fn tmdb_provider_deduplicates_query_imdb_external_ids_before_find_lookup() {
        let transport = FakeTransport::default();
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{"movie_results": [], "tv_results": [], "person_results": []}"#.to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{
                "movie_results": [{
                    "id": 603,
                    "title": "The Matrix",
                    "original_title": "The Matrix",
                    "overview": "Find result.",
                    "release_date": "1999-03-31",
                    "genre_ids": [28],
                    "vote_average": 8.7,
                    "vote_count": 23456
                }],
                "tv_results": [],
                "person_results": []
            }"#
            .to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{
                "id": 603,
                "title": "The Matrix",
                "original_title": "The Matrix",
                "overview": "Detail from deduped IMDb find.",
                "release_date": "1999-03-31",
                "runtime": 136,
                "tagline": null,
                "original_language": "en",
                "poster_path": null,
                "backdrop_path": null,
                "genres": [{"id": 28, "name": "Action"}],
                "vote_average": 8.7,
                "vote_count": 23456
            }"#
            .to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{"imdb_id": "tt0133093"}"#.to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{"titles": []}"#.to_vec(),
        }));
        let runtime = ProviderHttpRuntime::with_transport(
            ProviderHttpRuntimeConfig {
                retry_backoff_ms: 0,
                ..ProviderHttpRuntimeConfig::default()
            },
            transport.clone(),
        );
        let provider = TmdbMetadataProvider::with_runtime(
            TmdbProviderConfig {
                read_access_token: None,
                api_base_url: "https://tmdb.example/3".to_owned(),
                language: "en-US".to_owned(),
                include_adult: false,
                proxy_url: None,
            },
            runtime,
        );

        let candidates = provider
            .suggest(&MetadataQuery {
                title: "Wrong Local Title".to_owned(),
                year: Some(1999),
                language: "en-US".to_owned(),
                external_ids: vec![
                    crate::engine::QueryExternalId {
                        provider: "imdb".to_owned(),
                        value: "TT0000000".to_owned(),
                    },
                    crate::engine::QueryExternalId {
                        provider: "IMDB".to_owned(),
                        value: "tt0000000".to_owned(),
                    },
                    crate::engine::QueryExternalId {
                        provider: "imdb".to_owned(),
                        value: "tt0133093".to_owned(),
                    },
                ],
            })
            .await
            .unwrap();

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].provider_id, "tmdb:movie:603");
        let requests = transport.requests();
        let empty_find_count = requests
            .iter()
            .filter(|request| request.url == "https://tmdb.example/3/find/tt0000000")
            .count();
        assert_eq!(empty_find_count, 1);
        assert_eq!(requests[1].url, "https://tmdb.example/3/find/tt0133093");
    }

    #[tokio::test]
    async fn tmdb_provider_skips_malformed_search_result_items() {
        let transport = FakeTransport::default();
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{
                "results": [
                    {
                        "title": "Malformed Result",
                        "original_title": "Malformed Result",
                        "overview": "Missing ID should not poison the response.",
                        "release_date": "1999-03-31",
                        "genre_ids": [28]
                    },
                    {
                        "id": 603,
                        "title": "The Matrix",
                        "original_title": "The Matrix",
                        "overview": "Search summary.",
                        "release_date": "1999-03-31",
                        "genre_ids": [28, 878],
                        "vote_average": 8.2,
                        "vote_count": 12345
                    }
                ]
            }"#
            .to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{
                "id": 603,
                "title": "The Matrix",
                "original_title": "The Matrix",
                "overview": "Detail summary.",
                "release_date": "1999-03-31",
                "runtime": 136,
                "tagline": null,
                "original_language": "en",
                "poster_path": null,
                "backdrop_path": null,
                "genres": [{"id": 28, "name": "Action"}],
                "vote_average": 8.7,
                "vote_count": 23456
            }"#
            .to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{}"#.to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{"titles": []}"#.to_vec(),
        }));
        let runtime = ProviderHttpRuntime::with_transport(
            ProviderHttpRuntimeConfig {
                retry_backoff_ms: 0,
                ..ProviderHttpRuntimeConfig::default()
            },
            transport.clone(),
        );
        let provider = TmdbMetadataProvider::with_runtime(
            TmdbProviderConfig {
                read_access_token: None,
                api_base_url: "https://tmdb.example/3".to_owned(),
                language: "en-US".to_owned(),
                include_adult: false,
                proxy_url: None,
            },
            runtime,
        );

        let candidates = provider
            .suggest(&MetadataQuery {
                title: "The Matrix".to_owned(),
                year: Some(1999),
                language: "en-US".to_owned(),
                external_ids: Vec::new(),
            })
            .await
            .unwrap();

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].provider_id, "tmdb:movie:603");
        assert_eq!(candidates[0].patch.title.as_deref(), Some("The Matrix"));
        let requests = transport.requests();
        assert_eq!(requests[0].url, "https://tmdb.example/3/search/movie");
        assert_eq!(requests[1].url, "https://tmdb.example/3/movie/603");
    }

    #[tokio::test]
    async fn tmdb_provider_reports_error_when_all_search_result_items_are_malformed() {
        let transport = FakeTransport::default();
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{
                "results": [
                    {
                        "title": "Malformed Result One",
                        "release_date": "1999-03-31"
                    },
                    {
                        "title": "Malformed Result Two",
                        "release_date": "1999-03-31"
                    }
                ]
            }"#
            .to_vec(),
        }));
        let runtime = ProviderHttpRuntime::with_transport(
            ProviderHttpRuntimeConfig {
                retry_backoff_ms: 0,
                ..ProviderHttpRuntimeConfig::default()
            },
            transport,
        );
        let provider = TmdbMetadataProvider::with_runtime(
            TmdbProviderConfig {
                read_access_token: None,
                api_base_url: "https://tmdb.example/3".to_owned(),
                language: "en-US".to_owned(),
                include_adult: false,
                proxy_url: None,
            },
            runtime,
        );

        let error = provider
            .suggest(&MetadataQuery {
                title: "The Matrix".to_owned(),
                year: Some(1999),
                language: "en-US".to_owned(),
                external_ids: Vec::new(),
            })
            .await
            .unwrap_err();

        assert!(
            error
                .to_string()
                .contains("all TMDB search result items were malformed")
        );
    }

    #[tokio::test]
    async fn tmdb_provider_falls_back_to_normalized_search_title_when_raw_search_is_empty() {
        let transport = FakeTransport::default();
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{"results": []}"#.to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{
                "results": [{
                    "id": 634649,
                    "title": "Spider-Man: No Way Home",
                    "original_title": "Spider-Man: No Way Home",
                    "overview": "Search summary.",
                    "release_date": "2021-12-15",
                    "genre_ids": [28],
                    "vote_average": 8.0,
                    "vote_count": 1000
                }]
            }"#
            .to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{
                "id": 634649,
                "title": "Spider-Man: No Way Home",
                "original_title": "Spider-Man: No Way Home",
                "overview": "Detail summary.",
                "release_date": "2021-12-15",
                "runtime": 148,
                "tagline": null,
                "original_language": "en",
                "poster_path": null,
                "backdrop_path": null,
                "genres": [{"id": 28, "name": "Action"}],
                "vote_average": 8.0,
                "vote_count": 1000
            }"#
            .to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{}"#.to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{"titles": []}"#.to_vec(),
        }));
        let runtime = ProviderHttpRuntime::with_transport(
            ProviderHttpRuntimeConfig {
                retry_backoff_ms: 0,
                ..ProviderHttpRuntimeConfig::default()
            },
            transport.clone(),
        );
        let provider = TmdbMetadataProvider::with_runtime(
            TmdbProviderConfig {
                read_access_token: None,
                api_base_url: "https://tmdb.example/3".to_owned(),
                language: "en-US".to_owned(),
                include_adult: false,
                proxy_url: None,
            },
            runtime,
        );

        let candidates = provider
            .suggest(&MetadataQuery {
                title: "Spider-Man: No Way Home".to_owned(),
                year: Some(2021),
                language: "en-US".to_owned(),
                external_ids: Vec::new(),
            })
            .await
            .unwrap();

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].provider_id, "tmdb:movie:634649");

        let requests = transport.requests();
        assert_eq!(requests[0].url, "https://tmdb.example/3/search/movie");
        assert!(
            requests[0]
                .query
                .contains(&("query".to_owned(), "Spider-Man: No Way Home".to_owned()))
        );
        assert_eq!(requests[1].url, "https://tmdb.example/3/search/movie");
        assert!(
            requests[1]
                .query
                .contains(&("query".to_owned(), "spider man no way home".to_owned()))
        );
        assert_eq!(requests[2].url, "https://tmdb.example/3/movie/634649");
    }

    #[tokio::test]
    async fn tmdb_provider_merges_search_title_variants_with_deduped_enrichment_budget() {
        let transport = FakeTransport::default();
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{
                "results": [
                    {
                        "id": 10,
                        "title": "Spider-Man: No Way Home",
                        "original_title": "Spider-Man: No Way Home",
                        "overview": "Raw result one.",
                        "release_date": "2021-12-15",
                        "genre_ids": [28],
                        "vote_average": 8.0,
                        "vote_count": 1000
                    },
                    {
                        "id": 20,
                        "title": "Spider Man No Way Home",
                        "original_title": "Spider Man No Way Home",
                        "overview": "Raw result two.",
                        "release_date": "2021-12-15",
                        "genre_ids": [28],
                        "vote_average": 7.9,
                        "vote_count": 900
                    }
                ]
            }"#
            .to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{
                "results": [
                    {
                        "id": 20,
                        "title": "Spider Man No Way Home",
                        "original_title": "Spider Man No Way Home",
                        "overview": "Duplicate normalized result.",
                        "release_date": "2021-12-15",
                        "genre_ids": [28],
                        "vote_average": 7.9,
                        "vote_count": 900
                    },
                    {
                        "id": 30,
                        "title": "No Way Home",
                        "original_title": "No Way Home",
                        "overview": "Normalized-only result.",
                        "release_date": "2021-12-16",
                        "genre_ids": [28],
                        "vote_average": 7.5,
                        "vote_count": 700
                    }
                ]
            }"#
            .to_vec(),
        }));
        for movie_id in [10, 20, 30] {
            transport.push(Ok(ProviderHttpResponse {
                status: 200,
                body: format!(
                    r#"{{
                        "id": {movie_id},
                        "title": "Candidate {movie_id}",
                        "original_title": "Candidate {movie_id}",
                        "overview": "Detail {movie_id}.",
                        "release_date": "2021-12-15",
                        "runtime": 148,
                        "tagline": null,
                        "original_language": "en",
                        "poster_path": null,
                        "backdrop_path": null,
                        "genres": [{{"id": 28, "name": "Action"}}],
                        "vote_average": 8.0,
                        "vote_count": 1000
                    }}"#
                )
                .into_bytes(),
            }));
            transport.push(Ok(ProviderHttpResponse {
                status: 200,
                body: br#"{}"#.to_vec(),
            }));
            transport.push(Ok(ProviderHttpResponse {
                status: 200,
                body: br#"{"titles": []}"#.to_vec(),
            }));
        }
        let runtime = ProviderHttpRuntime::with_transport(
            ProviderHttpRuntimeConfig {
                retry_backoff_ms: 0,
                ..ProviderHttpRuntimeConfig::default()
            },
            transport.clone(),
        );
        let provider = TmdbMetadataProvider::with_runtime(
            TmdbProviderConfig {
                read_access_token: None,
                api_base_url: "https://tmdb.example/3".to_owned(),
                language: "en-US".to_owned(),
                include_adult: false,
                proxy_url: None,
            },
            runtime,
        );

        let candidates = provider
            .suggest(&MetadataQuery {
                title: "Spider-Man: No Way Home".to_owned(),
                year: Some(2021),
                language: "en-US".to_owned(),
                external_ids: Vec::new(),
            })
            .await
            .unwrap();

        assert_eq!(
            candidates
                .iter()
                .map(|candidate| candidate.provider_id.as_str())
                .collect::<Vec<_>>(),
            vec!["tmdb:movie:10", "tmdb:movie:20", "tmdb:movie:30"]
        );

        let requests = transport.requests();
        assert_eq!(requests[0].url, "https://tmdb.example/3/search/movie");
        assert_eq!(requests[1].url, "https://tmdb.example/3/search/movie");
        assert!(
            requests[1]
                .query
                .contains(&("query".to_owned(), "spider man no way home".to_owned()))
        );
        let detail_urls = requests
            .iter()
            .filter(|request| {
                request.url.starts_with("https://tmdb.example/3/movie/")
                    && !request.url.ends_with("/external_ids")
                    && !request.url.ends_with("/alternative_titles")
            })
            .map(|request| request.url.as_str())
            .collect::<Vec<_>>();
        assert_eq!(
            detail_urls,
            vec![
                "https://tmdb.example/3/movie/10",
                "https://tmdb.example/3/movie/20",
                "https://tmdb.example/3/movie/30"
            ]
        );
    }

    #[tokio::test]
    async fn tmdb_provider_prioritizes_more_relevant_merged_search_results_for_enrichment() {
        let transport = FakeTransport::default();
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{
                "results": [
                    {
                        "id": 10,
                        "title": "Spider Adjacent One",
                        "original_title": "Spider Adjacent One",
                        "overview": "Weak raw result one.",
                        "release_date": "2021-12-15",
                        "genre_ids": [28],
                        "vote_average": 7.0,
                        "vote_count": 500
                    },
                    {
                        "id": 20,
                        "title": "Spider Adjacent Two",
                        "original_title": "Spider Adjacent Two",
                        "overview": "Weak raw result two.",
                        "release_date": "2021-12-15",
                        "genre_ids": [28],
                        "vote_average": 7.0,
                        "vote_count": 500
                    }
                ]
            }"#
            .to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{
                "results": [
                    {
                        "id": 40,
                        "title": "Spider-Man: No Way Home",
                        "original_title": "Spider-Man: No Way Home",
                        "overview": "Strong normalized result.",
                        "release_date": "2021-12-15",
                        "genre_ids": [28],
                        "vote_average": 8.5,
                        "vote_count": 1500
                    },
                    {
                        "id": 50,
                        "title": "Spider Man No Way Home",
                        "original_title": "Spider Man No Way Home",
                        "overview": "Second strong normalized result.",
                        "release_date": "2021-12-15",
                        "genre_ids": [28],
                        "vote_average": 8.4,
                        "vote_count": 1400
                    }
                ]
            }"#
            .to_vec(),
        }));
        for movie_id in [40, 50, 10] {
            transport.push(Ok(ProviderHttpResponse {
                status: 200,
                body: format!(
                    r#"{{
                        "id": {movie_id},
                        "title": "Candidate {movie_id}",
                        "original_title": "Candidate {movie_id}",
                        "overview": "Detail {movie_id}.",
                        "release_date": "2021-12-15",
                        "runtime": 148,
                        "tagline": null,
                        "original_language": "en",
                        "poster_path": null,
                        "backdrop_path": null,
                        "genres": [{{"id": 28, "name": "Action"}}],
                        "vote_average": 8.0,
                        "vote_count": 1000
                    }}"#
                )
                .into_bytes(),
            }));
            transport.push(Ok(ProviderHttpResponse {
                status: 200,
                body: br#"{}"#.to_vec(),
            }));
            transport.push(Ok(ProviderHttpResponse {
                status: 200,
                body: br#"{"titles": []}"#.to_vec(),
            }));
        }
        let runtime = ProviderHttpRuntime::with_transport(
            ProviderHttpRuntimeConfig {
                retry_backoff_ms: 0,
                ..ProviderHttpRuntimeConfig::default()
            },
            transport.clone(),
        );
        let provider = TmdbMetadataProvider::with_runtime(
            TmdbProviderConfig {
                read_access_token: None,
                api_base_url: "https://tmdb.example/3".to_owned(),
                language: "en-US".to_owned(),
                include_adult: false,
                proxy_url: None,
            },
            runtime,
        );

        let candidates = provider
            .suggest(&MetadataQuery {
                title: "Spider-Man: No Way Home".to_owned(),
                year: Some(2021),
                language: "en-US".to_owned(),
                external_ids: Vec::new(),
            })
            .await
            .unwrap();

        assert_eq!(
            candidates
                .iter()
                .map(|candidate| candidate.provider_id.as_str())
                .collect::<Vec<_>>(),
            vec!["tmdb:movie:40", "tmdb:movie:50", "tmdb:movie:10"]
        );

        let requests = transport.requests();
        let detail_urls = requests
            .iter()
            .filter(|request| {
                request.url.starts_with("https://tmdb.example/3/movie/")
                    && !request.url.ends_with("/external_ids")
                    && !request.url.ends_with("/alternative_titles")
            })
            .map(|request| request.url.as_str())
            .collect::<Vec<_>>();
        assert_eq!(
            detail_urls,
            vec![
                "https://tmdb.example/3/movie/40",
                "https://tmdb.example/3/movie/50",
                "https://tmdb.example/3/movie/10"
            ]
        );
    }

    #[tokio::test]
    async fn tmdb_provider_preserves_search_results_when_later_title_variant_search_fails() {
        let transport = FakeTransport::default();
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{
                "results": [{
                    "id": 10,
                    "title": "Spider-Man: No Way Home",
                    "original_title": "Spider-Man: No Way Home",
                    "overview": "Raw search result.",
                    "release_date": "2021-12-15",
                    "genre_ids": [28],
                    "vote_average": 8.0,
                    "vote_count": 1000
                }]
            }"#
            .to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 503,
            body: br#"temporarily unavailable"#.to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{
                "id": 10,
                "title": "Spider-Man: No Way Home",
                "original_title": "Spider-Man: No Way Home",
                "overview": "Detail result.",
                "release_date": "2021-12-15",
                "runtime": 148,
                "tagline": null,
                "original_language": "en",
                "poster_path": null,
                "backdrop_path": null,
                "genres": [{"id": 28, "name": "Action"}],
                "vote_average": 8.0,
                "vote_count": 1000
            }"#
            .to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{}"#.to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{"titles": []}"#.to_vec(),
        }));
        let runtime = ProviderHttpRuntime::with_transport(
            ProviderHttpRuntimeConfig {
                max_attempts: 1,
                retry_backoff_ms: 0,
                ..ProviderHttpRuntimeConfig::default()
            },
            transport.clone(),
        );
        let provider = TmdbMetadataProvider::with_runtime(
            TmdbProviderConfig {
                read_access_token: None,
                api_base_url: "https://tmdb.example/3".to_owned(),
                language: "en-US".to_owned(),
                include_adult: false,
                proxy_url: None,
            },
            runtime,
        );

        let candidates = provider
            .suggest(&MetadataQuery {
                title: "Spider-Man: No Way Home".to_owned(),
                year: Some(2021),
                language: "en-US".to_owned(),
                external_ids: Vec::new(),
            })
            .await
            .unwrap();

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].provider_id, "tmdb:movie:10");
        assert_eq!(
            candidates[0].patch.overview.as_deref(),
            Some("Detail result.")
        );
        let provider_note = candidates[0].facts.provider_note.as_deref().unwrap();
        assert!(provider_note.contains("partial title-variant search failure"));
        assert!(!provider_note.contains("503"));
        assert!(!provider_note.contains("temporarily unavailable"));
        assert!(!provider_note.contains("https://"));

        let requests = transport.requests();
        assert_eq!(requests[0].url, "https://tmdb.example/3/search/movie");
        assert_eq!(requests[1].url, "https://tmdb.example/3/search/movie");
        assert_eq!(requests[2].url, "https://tmdb.example/3/movie/10");
    }

    #[tokio::test]
    async fn tmdb_provider_propagates_error_when_all_title_variant_searches_fail() {
        let transport = FakeTransport::default();
        transport.push(Ok(ProviderHttpResponse {
            status: 503,
            body: br#"raw unavailable"#.to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 503,
            body: br#"normalized unavailable"#.to_vec(),
        }));
        let runtime = ProviderHttpRuntime::with_transport(
            ProviderHttpRuntimeConfig {
                max_attempts: 1,
                retry_backoff_ms: 0,
                ..ProviderHttpRuntimeConfig::default()
            },
            transport.clone(),
        );
        let provider = TmdbMetadataProvider::with_runtime(
            TmdbProviderConfig {
                read_access_token: None,
                api_base_url: "https://tmdb.example/3".to_owned(),
                language: "en-US".to_owned(),
                include_adult: false,
                proxy_url: None,
            },
            runtime,
        );

        let error = provider
            .suggest(&MetadataQuery {
                title: "Spider-Man: No Way Home".to_owned(),
                year: Some(2021),
                language: "en-US".to_owned(),
                external_ids: Vec::new(),
            })
            .await
            .unwrap_err();

        assert!(error.to_string().contains("HTTP 503"));
        let requests = transport.requests();
        assert_eq!(requests.len(), 2);
        assert_eq!(requests[0].url, "https://tmdb.example/3/search/movie");
        assert_eq!(requests[1].url, "https://tmdb.example/3/search/movie");
    }

    #[tokio::test]
    async fn tmdb_provider_returns_degraded_candidate_after_failed_enrichment() {
        let transport = FakeTransport::default();
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{
                "results": [
                    {
                        "id": 10,
                        "title": "Broken Candidate",
                        "original_title": "Broken Candidate",
                        "overview": "Search result one.",
                        "release_date": "2021-01-01",
                        "poster_path": "/broken-poster.jpg",
                        "backdrop_path": "/broken-backdrop.jpg",
                        "genre_ids": [28],
                        "vote_average": 8.0,
                        "vote_count": 1000
                    },
                    {
                        "id": 20,
                        "title": "Usable Candidate",
                        "original_title": "Usable Candidate",
                        "overview": "Search result two.",
                        "release_date": "2021-01-02",
                        "genre_ids": [28],
                        "vote_average": 7.8,
                        "vote_count": 900
                    }
                ]
            }"#
            .to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 503,
            body: br#"temporarily unavailable"#.to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{
                "id": 20,
                "title": "Usable Candidate",
                "original_title": "Usable Candidate",
                "overview": "Detail two.",
                "release_date": "2021-01-02",
                "runtime": 120,
                "tagline": null,
                "original_language": "en",
                "poster_path": null,
                "backdrop_path": null,
                "genres": [{"id": 28, "name": "Action"}],
                "vote_average": 7.8,
                "vote_count": 900
            }"#
            .to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{}"#.to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{"titles": []}"#.to_vec(),
        }));
        let runtime = ProviderHttpRuntime::with_transport(
            ProviderHttpRuntimeConfig {
                max_attempts: 1,
                retry_backoff_ms: 0,
                ..ProviderHttpRuntimeConfig::default()
            },
            transport.clone(),
        );
        let provider = TmdbMetadataProvider::with_runtime(
            TmdbProviderConfig {
                read_access_token: None,
                api_base_url: "https://tmdb.example/3".to_owned(),
                language: "en-US".to_owned(),
                include_adult: false,
                proxy_url: None,
            },
            runtime,
        );

        let candidates = provider
            .suggest(&MetadataQuery {
                title: "Candidate".to_owned(),
                year: Some(2021),
                language: "en-US".to_owned(),
                external_ids: Vec::new(),
            })
            .await
            .unwrap();

        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[0].provider_id, "tmdb:movie:10");
        assert_eq!(
            candidates[0].patch.title.as_deref(),
            Some("Broken Candidate")
        );
        assert_eq!(
            candidates[0].patch.overview.as_deref(),
            Some("Search result one.")
        );
        assert_eq!(
            candidates[0].patch.release_date.as_deref(),
            Some("2021-01-01")
        );
        assert_eq!(candidates[0].patch.runtime_minutes, None);
        assert_eq!(
            candidates[0].patch.genres.as_ref().unwrap(),
            &vec!["Action".to_owned()]
        );
        assert_eq!(candidates[0].facts.release_year, Some(2021));
        assert_eq!(candidates[0].facts.community_score_milli, Some(800));
        assert_eq!(candidates[0].facts.community_vote_count, Some(1000));
        assert!(
            candidates[0]
                .facts
                .external_ids
                .iter()
                .any(|id| id.provider == "tmdb" && id.value == "10")
        );
        assert!(
            candidates[0]
                .facts
                .provider_note
                .as_deref()
                .is_some_and(|note| note.contains("degraded"))
        );
        assert!(
            candidates[0]
                .artwork_candidates
                .iter()
                .any(|candidate| candidate.facts.kind == AddonArtworkKind::Poster
                    && candidate.facts.source_url
                        == "https://image.tmdb.org/t/p/original/broken-poster.jpg")
        );
        assert!(
            candidates[0]
                .artwork_candidates
                .iter()
                .any(
                    |candidate| candidate.facts.kind == AddonArtworkKind::Backdrop
                        && candidate.facts.source_url
                            == "https://image.tmdb.org/t/p/original/broken-backdrop.jpg"
                )
        );
        assert_eq!(candidates[1].provider_id, "tmdb:movie:20");
        let requests = transport.requests();
        assert_eq!(requests[1].url, "https://tmdb.example/3/movie/10");
        assert_eq!(requests[2].url, "https://tmdb.example/3/movie/20");
    }

    #[tokio::test]
    async fn tmdb_provider_keeps_detail_candidate_when_secondary_enrichment_fails() {
        let transport = FakeTransport::default();
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{
                "results": [{
                    "id": 10,
                    "title": "Search Candidate",
                    "original_title": "Search Candidate",
                    "overview": "Search result.",
                    "release_date": "2021-01-01",
                    "genre_ids": [28],
                    "vote_average": 7.0,
                    "vote_count": 700
                }]
            }"#
            .to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{
                "id": 10,
                "title": "Detail Candidate",
                "original_title": "Detail Original",
                "overview": "Detail result.",
                "release_date": "2021-01-02",
                "runtime": 121,
                "tagline": "Detail tagline.",
                "original_language": "en",
                "poster_path": "/detail-poster.jpg",
                "backdrop_path": null,
                "genres": [{"id": 28, "name": "Action"}],
                "vote_average": 8.0,
                "vote_count": 1000
            }"#
            .to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 503,
            body: br#"temporarily unavailable"#.to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{
                "titles": [
                    {"title": "Detail Alias"}
                ]
            }"#
            .to_vec(),
        }));
        let runtime = ProviderHttpRuntime::with_transport(
            ProviderHttpRuntimeConfig {
                max_attempts: 1,
                retry_backoff_ms: 0,
                ..ProviderHttpRuntimeConfig::default()
            },
            transport.clone(),
        );
        let provider = TmdbMetadataProvider::with_runtime(
            TmdbProviderConfig {
                read_access_token: None,
                api_base_url: "https://tmdb.example/3".to_owned(),
                language: "en-US".to_owned(),
                include_adult: false,
                proxy_url: None,
            },
            runtime,
        );

        let candidates = provider
            .suggest(&MetadataQuery {
                title: "Detail Candidate".to_owned(),
                year: Some(2021),
                language: "en-US".to_owned(),
                external_ids: Vec::new(),
            })
            .await
            .unwrap();

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].provider_id, "tmdb:movie:10");
        assert_eq!(
            candidates[0].patch.title.as_deref(),
            Some("Detail Candidate")
        );
        assert_eq!(
            candidates[0].patch.overview.as_deref(),
            Some("Detail result.")
        );
        assert_eq!(candidates[0].patch.runtime_minutes, Some(121));
        assert!(
            candidates[0]
                .facts
                .alternate_titles
                .iter()
                .any(|title| title == "Detail Alias")
        );
        assert!(
            candidates[0]
                .facts
                .provider_note
                .as_deref()
                .is_some_and(|note| note.contains("partial"))
        );
        assert!(
            candidates[0]
                .artwork_candidates
                .iter()
                .any(|candidate| candidate.facts.source_url
                    == "https://image.tmdb.org/t/p/original/detail-poster.jpg")
        );

        let requests = transport.requests();
        assert_eq!(requests[1].url, "https://tmdb.example/3/movie/10");
        assert_eq!(
            requests[2].url,
            "https://tmdb.example/3/movie/10/external_ids"
        );
        assert_eq!(
            requests[3].url,
            "https://tmdb.example/3/movie/10/alternative_titles"
        );
    }

    #[derive(Clone, Default)]
    struct FakeTransport {
        responses: Arc<Mutex<VecDeque<ProviderHttpResult<ProviderHttpResponse>>>>,
        requests: Arc<Mutex<Vec<ProviderHttpRequest>>>,
        configs: Arc<Mutex<Vec<ProviderHttpRuntimeConfig>>>,
    }

    impl FakeTransport {
        fn push(&self, response: ProviderHttpResult<ProviderHttpResponse>) {
            self.responses.lock().unwrap().push_back(response);
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
                    Err(
                        crate::providers::http_runtime::ProviderHttpError::Transport {
                            provider_id: TMDB_PROVIDER_ID,
                            operation: "fake",
                            message: "fake transport response queue was empty".to_owned(),
                            attempts: 0,
                        },
                    )
                })
        }
    }
}
