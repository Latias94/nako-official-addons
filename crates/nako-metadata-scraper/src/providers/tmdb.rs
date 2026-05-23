use async_trait::async_trait;
use nako_addon_protocol::AddonMetadataPatch;
use serde::Deserialize;

use crate::{
    config::{ProviderId, TmdbProviderConfig},
    engine::{
        MetadataQuery, ProviderCandidateFacts, ProviderExternalId, ProviderMetadataCandidate,
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
const TMDB_DETAIL_ENRICHMENT_LIMIT: usize = 3;

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
        let runtime = ProviderHttpRuntime::new(ProviderHttpRuntimeConfig::default())?;
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
        let mut request_query = vec![
            ("query".to_owned(), query.title.clone()),
            ("language".to_owned(), self.config.language.clone()),
            (
                "include_adult".to_owned(),
                self.config.include_adult.to_string(),
            ),
            ("page".to_owned(), "1".to_owned()),
        ];
        if let Some(year) = query.year {
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
        let search = TmdbSearchResponse::from_value(response.body)?;
        let mut candidates = Vec::new();

        for result in search
            .results
            .into_iter()
            .take(TMDB_DETAIL_ENRICHMENT_LIMIT)
        {
            candidates.push(self.enrich_movie_candidate(query, result).await?);
        }

        Ok(candidates)
    }
}

impl<T> TmdbMetadataProvider<T>
where
    T: ProviderHttpTransport,
{
    async fn enrich_movie_candidate(
        &self,
        query: &MetadataQuery,
        result: TmdbMovieSearchResult,
    ) -> anyhow::Result<ProviderMetadataCandidate> {
        let movie_id = result.id;
        let detail = self.fetch_movie_detail(movie_id).await?;
        let external_ids = self.fetch_movie_external_ids(movie_id).await?;

        Ok(TmdbMovieCandidate {
            search: result,
            detail,
            external_ids,
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
}

#[derive(Debug, Deserialize)]
struct TmdbSearchResponse {
    #[serde(default)]
    results: Vec<TmdbMovieSearchResult>,
}

impl TmdbSearchResponse {
    fn from_value(value: serde_json::Value) -> anyhow::Result<Self> {
        serde_json::from_value(value)
            .map_err(|error| anyhow::anyhow!("failed to parse TMDB search movie response: {error}"))
    }
}

#[derive(Debug, Deserialize)]
struct TmdbMovieSearchResult {
    id: u64,
    title: Option<String>,
    original_title: Option<String>,
    overview: Option<String>,
    release_date: Option<String>,
    #[serde(default)]
    genre_ids: Vec<u64>,
    vote_average: Option<f64>,
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

struct TmdbMovieCandidate {
    search: TmdbMovieSearchResult,
    detail: TmdbMovieDetail,
    external_ids: TmdbMovieExternalIds,
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
        let release_date = non_empty(self.detail.release_date).or_else(|| {
            self.search
                .release_date
                .filter(|value| !value.trim().is_empty())
        });
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
        let external_ids = self.external_ids.into_external_ids(self.detail.id);
        let mut tags = vec!["tmdb".to_owned()];
        if let Some(vote_average) = vote_average {
            tags.push(format!("tmdb_vote_average:{vote_average:.1}"));
        }
        if let Some(poster_path) = non_empty(self.detail.poster_path) {
            tags.push(format!("tmdb_poster_path:{poster_path}"));
        }
        if let Some(backdrop_path) = non_empty(self.detail.backdrop_path) {
            tags.push(format!("tmdb_backdrop_path:{backdrop_path}"));
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
                release_year: release_year.map(i32::from),
                language: self
                    .detail
                    .original_language
                    .or_else(|| Some(query.language.clone())),
                external_ids,
                provider_note: Some(
                    "TMDB movie candidate enriched with search, detail, and external ID responses."
                        .to_owned(),
                ),
            },
        }
    }
}

fn first_non_empty(values: &[Option<&str>]) -> Option<String> {
    values
        .iter()
        .flatten()
        .find(|value| !value.trim().is_empty())
        .map(|value| (*value).to_owned())
}

fn non_empty(value: Option<String>) -> Option<String> {
    value.filter(|value| !value.trim().is_empty())
}

fn push_external_id(ids: &mut Vec<ProviderExternalId>, provider: &str, value: Option<String>) {
    if let Some(value) = non_empty(value) {
        ids.push(ProviderExternalId {
            provider: provider.to_owned(),
            value,
        });
    }
}

fn release_year(value: Option<&str>) -> Option<u16> {
    let year = value?.get(0..4)?;
    year.parse().ok()
}

fn detail_genre_names(genres: &[TmdbGenre]) -> Option<Vec<String>> {
    Some(
        genres
            .iter()
            .filter(|genre| genre.id != 0)
            .filter_map(|genre| genre.name.as_ref())
            .filter(|name| !name.trim().is_empty())
            .cloned()
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
        ProviderHttpRequest, ProviderHttpResponse, ProviderHttpRuntimeConfig,
    };

    use super::*;

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
                    "vote_average": 8.2
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
                "vote_average": 8.7
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
        assert_eq!(candidates[0].facts.release_year, Some(1999));
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
                .patch
                .tags
                .as_ref()
                .unwrap()
                .contains(&"tmdb_poster_path:/poster.jpg".to_owned())
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
