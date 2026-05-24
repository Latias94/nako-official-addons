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
        TmdbSearchResponse,
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

    pub(super) async fn fetch_movie_detail(
        &self,
        movie_id: u64,
    ) -> anyhow::Result<TmdbMovieDetail> {
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

    pub(super) async fn fetch_movie_external_ids(
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

    pub(super) async fn fetch_movie_alternative_titles(
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

    pub(super) async fn find_movie_id_by_imdb_id(
        &self,
        imdb_id: &str,
    ) -> anyhow::Result<Option<u64>> {
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
