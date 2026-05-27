mod client;
mod enrichment;
mod mapper;
mod parser;
mod search;
#[cfg(test)]
mod test_support;

use async_trait::async_trait;
use nako_addon_protocol::AddonSecretReferenceFieldDeclaration;

use crate::{
    Config,
    config::{ProviderConfig, ProviderId, non_empty_trimmed, parse_bool},
    engine::{
        ExternalIdValueKind, MetadataQuery, ProviderExternalIdCapability, ProviderMetadataCandidate,
    },
    providers::{
        MetadataProvider, ProviderBuildStatus, ProviderConfigInput,
        http_runtime::{ProviderHttpRuntime, ProviderHttpTransport, ReqwestProviderHttpTransport},
        registry::ProviderCatalogEntry,
    },
};

#[cfg(test)]
use mapper::{TmdbMovieCandidate, TmdbMovieSearchResult, release_year};
#[cfg(test)]
use nako_addon_protocol::AddonArtworkKind;
#[cfg(test)]
use parser::{
    TmdbAlternativeTitle, TmdbFindMovieResult, TmdbFindResponse, TmdbFindTvResult, TmdbGenre,
    TmdbMovieAlternativeTitles, TmdbMovieDetail, TmdbMovieExternalIds, TmdbSearchResponse,
    TmdbTvSearchResponse,
};
#[cfg(test)]
use search::{tmdb_query_movie_ids, tmdb_query_tv_ids};

#[cfg(test)]
use test_support::FakeTransport;

pub const TMDB_PROVIDER_ID: &str = "tmdb";
pub const TMDB_TV_EXTERNAL_ID_PROVIDER_ID: &str = "tmdb_tv";
const TMDB_EXTERNAL_ID_CAPABILITIES: &[ProviderExternalIdCapability] = &[
    ProviderExternalIdCapability::new(
        "tmdb",
        ExternalIdValueKind::Numeric,
        true,
        true,
        &["tmdb_id"],
        true,
    ),
    ProviderExternalIdCapability::new(
        "tmdb_tv",
        ExternalIdValueKind::Numeric,
        true,
        true,
        &["tmdb_tv_id"],
        true,
    ),
    ProviderExternalIdCapability::new(
        "imdb",
        ExternalIdValueKind::Opaque,
        true,
        true,
        &["imdb_id"],
        true,
    ),
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TmdbProviderConfig {
    pub read_access_token: Option<String>,
    pub api_base_url: String,
    pub language: String,
    pub include_adult: bool,
    pub proxy_url: Option<String>,
}

impl TmdbProviderConfig {
    #[must_use]
    pub fn from_env_lookup(mut lookup: impl FnMut(&str) -> Option<String>) -> Self {
        Self {
            read_access_token: lookup("NAKO_METADATA_SCRAPER_TMDB_READ_ACCESS_TOKEN")
                .and_then(non_empty_trimmed),
            api_base_url: lookup("NAKO_METADATA_SCRAPER_TMDB_API_BASE_URL")
                .and_then(non_empty_trimmed)
                .unwrap_or_else(|| "https://api.themoviedb.org/3".to_owned()),
            language: lookup("NAKO_METADATA_SCRAPER_TMDB_LANGUAGE")
                .and_then(non_empty_trimmed)
                .unwrap_or_else(|| "en-US".to_owned()),
            include_adult: lookup("NAKO_METADATA_SCRAPER_TMDB_INCLUDE_ADULT")
                .and_then(|value| parse_bool(&value))
                .unwrap_or(false),
            proxy_url: lookup("NAKO_METADATA_SCRAPER_TMDB_PROXY_URL").and_then(non_empty_trimmed),
        }
    }

    #[must_use]
    pub const fn secret_field_id() -> &'static str {
        "tmdb_read_access_token"
    }
}

#[must_use]
pub(crate) fn catalog_entry() -> ProviderCatalogEntry {
    ProviderCatalogEntry {
        id: ProviderId::Tmdb,
        default_enabled: false,
        enabled_env_var: "NAKO_METADATA_SCRAPER_PROVIDER_TMDB_ENABLED",
        capabilities: &["metadata_suggestion", "movie_search", "tv_search"],
        field_quality: Default::default(),
        secret_reference: Some(AddonSecretReferenceFieldDeclaration::new(
            TmdbProviderConfig::secret_field_id(),
            "TMDB Read Access Token",
            Some(
                "Secret Reference for a TMDB API Read Access Token. The sidecar resolves it from NAKO_METADATA_SCRAPER_TMDB_READ_ACCESS_TOKEN."
                    .to_owned(),
            ),
            true,
        )),
        external_id_capabilities: TMDB_EXTERNAL_ID_CAPABILITIES,
        load_config: load_config,
        proxy_configured: tmdb_proxy_configured,
        network_policy_key: Some("tmdb_proxy_configured"),
        build: build_provider,
    }
}

fn load_config(input: ProviderConfigInput<'_>) -> ProviderConfig {
    let lookup = input.lookup;
    ProviderConfig::tmdb(
        input.enabled,
        TmdbProviderConfig::from_env_lookup(|name| lookup(name)),
    )
}

fn tmdb_proxy_configured(provider: &ProviderConfig) -> bool {
    provider
        .tmdb_config()
        .and_then(|config| config.proxy_url.as_ref())
        .is_some()
}

fn build_provider(config: &Config) -> ProviderBuildStatus {
    let Some(tmdb_config) = config
        .provider_config(ProviderId::Tmdb)
        .and_then(|provider| provider.tmdb_config().cloned())
    else {
        return ProviderBuildStatus::Unavailable;
    };
    if tmdb_config.read_access_token.is_none() {
        return ProviderBuildStatus::Unavailable;
    }
    match TmdbMetadataProvider::new(tmdb_config) {
        Ok(provider) => ProviderBuildStatus::Ready(Box::new(provider)),
        Err(_) => ProviderBuildStatus::Unavailable,
    }
}

#[derive(Clone, Debug)]
pub struct TmdbMetadataProvider<T = ReqwestProviderHttpTransport>
where
    T: ProviderHttpTransport,
{
    config: TmdbProviderConfig,
    runtime: ProviderHttpRuntime<T>,
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
        self.suggest_candidates(query).await
    }
}

#[cfg(test)]
mod tests {
    use crate::providers::http_runtime::{ProviderHttpResponse, ProviderHttpRuntimeConfig};

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
    fn tmdb_query_tv_ids_ignores_zero_and_invalid_values() {
        let query = MetadataQuery {
            title: "Breaking Bad".to_owned(),
            year: Some(2008),
            language: "en-US".to_owned(),
            external_ids: vec![
                crate::engine::QueryExternalId {
                    provider: "tmdb_tv".to_owned(),
                    value: "0".to_owned(),
                },
                crate::engine::QueryExternalId {
                    provider: "TMDB_TV".to_owned(),
                    value: "1396".to_owned(),
                },
                crate::engine::QueryExternalId {
                    provider: "tmdb_tv_id".to_owned(),
                    value: "1396".to_owned(),
                },
                crate::engine::QueryExternalId {
                    provider: "tmdb_tv".to_owned(),
                    value: "not-a-number".to_owned(),
                },
            ],
        };

        let tv_ids = tmdb_query_tv_ids(&query).collect::<Vec<_>>();

        assert_eq!(tv_ids, vec![1396]);
    }

    #[test]
    fn tmdb_find_response_ignores_zero_movie_ids() {
        let response = TmdbFindResponse {
            movie_results: vec![
                TmdbFindMovieResult { id: 0 },
                TmdbFindMovieResult { id: 603 },
            ],
            tv_results: Vec::new(),
        };

        assert_eq!(response.first_movie_id(), Some(603));
    }

    #[test]
    fn tmdb_find_response_ignores_zero_tv_ids() {
        let response = TmdbFindResponse {
            movie_results: Vec::new(),
            tv_results: vec![TmdbFindTvResult { id: 0 }, TmdbFindTvResult { id: 1396 }],
        };

        assert_eq!(response.first_tv_id(), Some(1396));
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

    #[test]
    fn tmdb_tv_search_response_skips_zero_id_items() {
        let response = TmdbTvSearchResponse::from_value(serde_json::json!({
            "results": [
                {"id": 0},
                {"id": 1396, "name": "Breaking Bad"}
            ]
        }))
        .unwrap();

        assert_eq!(response.results.len(), 1);
        assert_eq!(response.results[0].id, 1396);
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
            body: r#"{
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
                "vote_count": 23456,
                "external_ids": {
                    "imdb_id": "tt0133093",
                    "wikidata_id": "Q83495"
                },
                "alternative_titles": {
                    "titles": [
                        {"iso_3166_1": "CN", "title": "黑客帝国", "type": "localized"},
                        {"iso_3166_1": "US", "title": "The Matrix", "type": "original"}
                    ]
                }
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
        assert_eq!(requests.len(), 2);
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
            vec![
                ("language".to_owned(), "en-US".to_owned()),
                (
                    "append_to_response".to_owned(),
                    "external_ids,alternative_titles".to_owned()
                )
            ]
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
            body: r#"{
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
                "vote_count": 23456,
                "external_ids": {
                    "imdb_id": "tt0133093",
                    "wikidata_id": "Q83495"
                },
                "alternative_titles": {
                    "titles": [{"title": "黑客帝国"}]
                }
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
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].url, "https://tmdb.example/3/movie/603");
        assert_eq!(
            requests[0].query,
            vec![
                ("language".to_owned(), "en-US".to_owned()),
                (
                    "append_to_response".to_owned(),
                    "external_ids,alternative_titles".to_owned()
                )
            ]
        );
        assert!(
            requests
                .iter()
                .all(|request| request.url != "https://tmdb.example/3/search/movie")
        );
    }

    #[tokio::test]
    async fn tmdb_provider_uses_query_external_id_for_direct_tv_lookup() {
        let transport = FakeTransport::default();
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: r#"{
                "id": 1396,
                "name": "Breaking Bad",
                "original_name": "Breaking Bad",
                "overview": "A chemistry teacher turns to crime.",
                "first_air_date": "2008-01-20",
                "episode_run_time": [47],
                "tagline": "Change the equation.",
                "original_language": "en",
                "poster_path": "/bb-poster.jpg",
                "backdrop_path": "/bb-backdrop.jpg",
                "genres": [{"id": 18, "name": "Drama"}],
                "networks": [{"id": 174, "name": "AMC"}],
                "origin_country": ["US"],
                "number_of_episodes": 62,
                "number_of_seasons": 5,
                "status": "Ended",
                "vote_average": 8.9,
                "vote_count": 15000,
                "external_ids": {
                    "imdb_id": "tt0903747",
                    "tvdb_id": 81189,
                    "wikidata_id": "Q109630"
                },
                "alternative_titles": {
                    "results": [{"title": "絕命毒師"}]
                }
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
                year: Some(2008),
                language: "zh-CN".to_owned(),
                external_ids: vec![crate::engine::QueryExternalId {
                    provider: "tmdb_tv".to_owned(),
                    value: "1396".to_owned(),
                }],
            })
            .await
            .unwrap();

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].provider_id, "tmdb:tv:1396");
        assert_eq!(candidates[0].patch.title.as_deref(), Some("Breaking Bad"));
        assert_eq!(
            candidates[0].patch.overview.as_deref(),
            Some("A chemistry teacher turns to crime.")
        );
        assert_eq!(
            candidates[0].patch.release_date.as_deref(),
            Some("2008-01-20")
        );
        assert_eq!(candidates[0].patch.runtime_minutes, Some(47));
        assert_eq!(
            candidates[0].patch.tagline.as_deref(),
            Some("Change the equation.")
        );
        assert_eq!(candidates[0].facts.release_year, Some(2008));
        assert_eq!(candidates[0].facts.community_score_milli, Some(890));
        assert_eq!(candidates[0].facts.community_vote_count, Some(15000));
        assert!(
            candidates[0]
                .facts
                .external_ids
                .iter()
                .any(|id| id.provider == "tmdb_tv" && id.value == "1396")
        );
        assert!(
            candidates[0]
                .facts
                .external_ids
                .iter()
                .any(|id| id.provider == "tvdb" && id.value == "81189")
        );
        assert!(
            candidates[0]
                .facts
                .alternate_titles
                .iter()
                .any(|title| title == "絕命毒師")
        );
        assert!(
            candidates[0]
                .patch
                .tags
                .as_ref()
                .unwrap()
                .contains(&"tmdb_network:AMC".to_owned())
        );
        assert!(
            candidates[0]
                .artwork_candidates
                .iter()
                .any(|candidate| candidate.provider_id == "tmdb:tv:1396"
                    && candidate.facts.kind == AddonArtworkKind::Poster
                    && candidate.facts.source_url
                        == "https://image.tmdb.org/t/p/original/bb-poster.jpg")
        );

        let requests = transport.requests();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].url, "https://tmdb.example/3/tv/1396");
    }

    #[tokio::test]
    async fn tmdb_provider_searches_tv_when_movie_search_has_no_candidates() {
        let transport = FakeTransport::default();
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{"results": []}"#.to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{
                "results": [{
                    "id": 1396,
                    "name": "Breaking Bad",
                    "original_name": "Breaking Bad",
                    "overview": "Search TV result.",
                    "first_air_date": "2008-01-20",
                    "genre_ids": [18],
                    "vote_average": 8.9,
                    "vote_count": 15000
                }]
            }"#
            .to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{
                "id": 1396,
                "name": "Breaking Bad",
                "original_name": "Breaking Bad",
                "overview": "Detail TV result.",
                "first_air_date": "2008-01-20",
                "episode_run_time": [47],
                "tagline": null,
                "original_language": "en",
                "poster_path": null,
                "backdrop_path": null,
                "genres": [{"id": 18, "name": "Drama"}],
                "vote_average": 8.9,
                "vote_count": 15000,
                "external_ids": {
                    "imdb_id": "tt0903747",
                    "tvdb_id": 81189
                },
                "alternative_titles": {
                    "results": []
                }
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
                title: "Breaking Bad".to_owned(),
                year: Some(2008),
                language: "en-US".to_owned(),
                external_ids: Vec::new(),
            })
            .await
            .unwrap();

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].provider_id, "tmdb:tv:1396");
        assert_eq!(candidates[0].patch.title.as_deref(), Some("Breaking Bad"));
        assert_eq!(
            candidates[0].patch.overview.as_deref(),
            Some("Detail TV result.")
        );
        assert_eq!(candidates[0].patch.runtime_minutes, Some(47));

        let requests = transport.requests();
        assert_eq!(requests.len(), 3);
        assert_eq!(requests[0].url, "https://tmdb.example/3/search/movie");
        assert_eq!(requests[1].url, "https://tmdb.example/3/search/tv");
        assert!(
            requests[1]
                .query
                .contains(&("first_air_date_year".to_owned(), "2008".to_owned()))
        );
        assert_eq!(requests[2].url, "https://tmdb.example/3/tv/1396");
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
            body: r#"{
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
                "vote_count": 23456,
                "external_ids": {
                    "imdb_id": "tt0133093",
                    "wikidata_id": "Q83495"
                },
                "alternative_titles": {
                    "titles": [{"title": "黑客帝国"}]
                }
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
        assert_eq!(requests.len(), 2);
        assert_eq!(requests[0].url, "https://tmdb.example/3/find/tt0133093");
        assert!(
            requests[0]
                .query
                .contains(&("external_source".to_owned(), "imdb_id".to_owned()))
        );
        assert_eq!(requests[1].url, "https://tmdb.example/3/movie/603");
        assert_eq!(
            requests[1].query,
            vec![
                ("language".to_owned(), "en-US".to_owned()),
                (
                    "append_to_response".to_owned(),
                    "external_ids,alternative_titles".to_owned()
                )
            ]
        );
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
        let provider_note = crate::engine::render_provider_note(
            &candidates[0].facts.provider_outcomes,
            candidates[0].facts.provider_note.as_deref(),
        )
        .unwrap();
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
        let provider_note = crate::engine::render_provider_note(
            &candidates[0].facts.provider_outcomes,
            candidates[0].facts.provider_note.as_deref(),
        );
        assert!(provider_note.is_some_and(|note| note.contains("degraded")));
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
                "vote_count": 1000,
                "external_ids": 503,
                "alternative_titles": {
                    "titles": [
                        {"title": "Detail Alias"}
                    ]
                }
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
        let provider_note = crate::engine::render_provider_note(
            &candidates[0].facts.provider_outcomes,
            candidates[0].facts.provider_note.as_deref(),
        );
        assert!(provider_note.is_some_and(|note| note.contains("partial")));
        assert!(
            candidates[0]
                .artwork_candidates
                .iter()
                .any(|candidate| candidate.facts.source_url
                    == "https://image.tmdb.org/t/p/original/detail-poster.jpg")
        );

        let requests = transport.requests();
        assert_eq!(requests.len(), 2);
        assert_eq!(requests[1].url, "https://tmdb.example/3/movie/10");
        assert_eq!(
            requests[1].query,
            vec![
                ("language".to_owned(), "en-US".to_owned()),
                (
                    "append_to_response".to_owned(),
                    "external_ids,alternative_titles".to_owned()
                )
            ]
        );
    }
}
