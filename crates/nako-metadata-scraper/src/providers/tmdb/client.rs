use crate::{
    config::TmdbProviderConfig,
    engine::MetadataQuery,
    providers::http_runtime::{
        ProviderHttpResult, ProviderHttpRuntime, ProviderHttpRuntimeConfig, ProviderHttpTransport,
        ReqwestProviderHttpTransport,
    },
};

use super::{
    TMDB_PROVIDER_ID, TmdbMetadataProvider,
    parser::{
        TmdbFindResponse, TmdbMovieAlternativeTitles, TmdbMovieDetail, TmdbMovieExternalIds,
        TmdbSearchResponse, TmdbTvAlternativeTitles, TmdbTvDetail, TmdbTvExternalIds,
        TmdbTvSearchResponse,
    },
};

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
    #[cfg(test)]
    #[must_use]
    pub(super) fn with_runtime(
        config: TmdbProviderConfig,
        runtime: ProviderHttpRuntime<T>,
    ) -> Self {
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

    pub(super) async fn search_movies(
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

    pub(super) async fn search_tvs(
        &self,
        query: &MetadataQuery,
        search_title: &str,
    ) -> anyhow::Result<TmdbTvSearchResponse> {
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
            request_query.push(("first_air_date_year".to_owned(), year.to_string()));
        }

        let response = self
            .runtime
            .get_json(
                TMDB_PROVIDER_ID,
                "search TV",
                self.endpoint("search/tv"),
                request_query,
                self.bearer_headers(),
            )
            .await?;

        TmdbTvSearchResponse::from_value(response.body)
    }

    pub(super) async fn fetch_movie_detail_bundle(
        &self,
        movie_id: u64,
    ) -> anyhow::Result<TmdbMovieDetailBundle> {
        let response = self
            .runtime
            .get_json(
                TMDB_PROVIDER_ID,
                "movie detail",
                self.endpoint(format!("movie/{movie_id}")),
                vec![
                    ("language".to_owned(), self.config.language.clone()),
                    (
                        "append_to_response".to_owned(),
                        "external_ids,alternative_titles".to_owned(),
                    ),
                ],
                self.bearer_headers(),
            )
            .await?;

        TmdbMovieDetailBundle::from_value(response.body)
    }

    pub(super) async fn fetch_tv_detail_bundle(
        &self,
        tv_id: u64,
    ) -> anyhow::Result<TmdbTvDetailBundle> {
        let response = self
            .runtime
            .get_json(
                TMDB_PROVIDER_ID,
                "TV detail",
                self.endpoint(format!("tv/{tv_id}")),
                vec![
                    ("language".to_owned(), self.config.language.clone()),
                    (
                        "append_to_response".to_owned(),
                        "external_ids,alternative_titles".to_owned(),
                    ),
                ],
                self.bearer_headers(),
            )
            .await?;

        TmdbTvDetailBundle::from_value(response.body)
    }

    pub(super) async fn find_by_imdb_id(&self, imdb_id: &str) -> anyhow::Result<TmdbFindResponse> {
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

        TmdbFindResponse::from_value(response.body)
    }
}

#[derive(Clone, Debug)]
pub(super) struct TmdbMovieDetailBundle {
    pub(super) detail: TmdbMovieDetail,
    pub(super) external_ids: TmdbMovieExternalIds,
    pub(super) alternative_titles: TmdbMovieAlternativeTitles,
    pub(super) partial_enrichment: bool,
}

impl TmdbMovieDetailBundle {
    fn from_value(value: serde_json::Value) -> anyhow::Result<Self> {
        let detail = TmdbMovieDetail::from_value(value.clone())?;
        let (external_ids, external_ids_partial) = parse_optional_nested_value(
            value.get("external_ids"),
            "TMDB appended external IDs",
            TmdbMovieExternalIds::from_value,
        );
        let (alternative_titles, alternative_titles_partial) = parse_optional_nested_value(
            value.get("alternative_titles"),
            "TMDB appended alternative titles",
            TmdbMovieAlternativeTitles::from_value,
        );

        Ok(Self {
            detail,
            external_ids,
            alternative_titles,
            partial_enrichment: external_ids_partial || alternative_titles_partial,
        })
    }
}

#[derive(Clone, Debug)]
pub(super) struct TmdbTvDetailBundle {
    pub(super) detail: TmdbTvDetail,
    pub(super) external_ids: TmdbTvExternalIds,
    pub(super) alternative_titles: TmdbTvAlternativeTitles,
    pub(super) partial_enrichment: bool,
}

impl TmdbTvDetailBundle {
    fn from_value(value: serde_json::Value) -> anyhow::Result<Self> {
        let detail = TmdbTvDetail::from_value(value.clone())?;
        let (external_ids, external_ids_partial) = parse_optional_nested_value(
            value.get("external_ids"),
            "TMDB appended TV external IDs",
            TmdbTvExternalIds::from_value,
        );
        let (alternative_titles, alternative_titles_partial) = parse_optional_nested_value(
            value.get("alternative_titles"),
            "TMDB appended TV alternative titles",
            TmdbTvAlternativeTitles::from_value,
        );

        Ok(Self {
            detail,
            external_ids,
            alternative_titles,
            partial_enrichment: external_ids_partial || alternative_titles_partial,
        })
    }
}

fn parse_optional_nested_value<T>(
    value: Option<&serde_json::Value>,
    label: &'static str,
    parser: impl FnOnce(serde_json::Value) -> anyhow::Result<T>,
) -> (T, bool)
where
    T: Default,
{
    let Some(value) = value else {
        return (T::default(), false);
    };
    if value.is_null() {
        return (T::default(), false);
    }

    match parser(value.clone()) {
        Ok(parsed) => (parsed, false),
        Err(error) => {
            tracing::warn!(provider = TMDB_PROVIDER_ID, %error, %label, "TMDB appended detail field could not be parsed; continuing with partial enrichment");
            (T::default(), true)
        }
    }
}
