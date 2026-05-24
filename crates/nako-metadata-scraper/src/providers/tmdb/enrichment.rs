use std::collections::HashSet;

use crate::engine::{MetadataQuery, ProviderMetadataCandidate};

use super::{
    TMDB_PROVIDER_ID, TmdbMetadataProvider,
    mapper::{TmdbMovieCandidate, TmdbMovieSearchResult, append_provider_note},
    search::{tmdb_query_imdb_ids, tmdb_query_movie_ids, tmdb_ranked_enrichment_results},
};

const TMDB_PARTIAL_SEARCH_NOTE: &str =
    "TMDB provider preserved candidates after partial title-variant search failure.";

impl<T> TmdbMetadataProvider<T>
where
    T: crate::providers::http_runtime::ProviderHttpTransport,
{
    pub(super) async fn suggest_candidates(
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

    pub(super) async fn enrich_movie_candidate(
        &self,
        query: &MetadataQuery,
        result: TmdbMovieSearchResult,
    ) -> anyhow::Result<ProviderMetadataCandidate> {
        let movie_id = result.id;
        self.enrich_movie_candidate_from_seed(query, result, movie_id)
            .await
    }

    pub(super) async fn enrich_movie_candidate_by_id(
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

    pub(super) async fn enrich_movie_candidate_from_seed(
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
                Default::default()
            }
        };
        let alternative_titles = match self.fetch_movie_alternative_titles(movie_id).await {
            Ok(alternative_titles) => alternative_titles,
            Err(error) => {
                partial_enrichment = true;
                tracing::warn!(provider = TMDB_PROVIDER_ID, %error, "TMDB alternative titles enrichment failed for detail candidate");
                Default::default()
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
}
