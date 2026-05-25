use crate::engine::{MetadataQuery, ProviderMetadataCandidate, ProviderOutcome};

use super::{
    TMDB_PROVIDER_ID, TmdbMetadataProvider,
    mapper::{TmdbMovieCandidate, TmdbMovieSearchResult},
    search::{tmdb_query_imdb_ids, tmdb_query_movie_ids},
};
use crate::providers::{
    http_runtime::ProviderHttpTransport,
    search_policy::{SearchEnrichmentPolicy, first_direct_lookup, search_and_enrich},
};

const TMDB_DETAIL_ENRICHMENT_LIMIT: usize = 3;
const TMDB_SEARCH_POLICY: SearchEnrichmentPolicy = SearchEnrichmentPolicy::new(
    TMDB_PROVIDER_ID,
    "TMDB",
    TMDB_DETAIL_ENRICHMENT_LIMIT,
    ProviderOutcome::TmdbPartialTitleVariantSearchFailure,
);

impl<T> TmdbMetadataProvider<T>
where
    T: ProviderHttpTransport,
{
    pub(super) async fn suggest_candidates(
        &self,
        query: &MetadataQuery,
    ) -> anyhow::Result<Vec<ProviderMetadataCandidate>> {
        if let Some(candidate) = first_direct_lookup(
            TMDB_SEARCH_POLICY,
            tmdb_query_movie_ids(query),
            |movie_id| async move {
                self.enrich_movie_candidate_by_id(query, movie_id)
                    .await
                    .map(Some)
            },
        )
        .await
        {
            return Ok(vec![candidate]);
        }
        if let Some(candidate) = first_direct_lookup(
            TMDB_SEARCH_POLICY,
            tmdb_query_imdb_ids(query),
            |imdb_id| async move {
                let Some(movie_id) = self.find_movie_id_by_imdb_id(&imdb_id).await? else {
                    return Ok(None);
                };

                self.enrich_movie_candidate_by_id(query, movie_id)
                    .await
                    .map(Some)
            },
        )
        .await
        {
            return Ok(vec![candidate]);
        }

        search_and_enrich(
            TMDB_SEARCH_POLICY,
            query,
            |search_title| async move {
                self.search_movies(query, &search_title)
                    .await
                    .map(|search| search.results)
            },
            |result: &TmdbMovieSearchResult| result.id,
            |result| result.into_degraded_candidate(query),
            |candidate, outcome| candidate.facts.provider_outcomes.push(outcome),
            |result| async move { self.enrich_movie_candidate(query, result).await },
        )
        .await
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
        let detail_bundle = self.fetch_movie_detail_bundle(movie_id).await?;
        let detail = detail_bundle.detail;
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

        Ok(TmdbMovieCandidate {
            search: result,
            detail,
            external_ids: detail_bundle.external_ids,
            alternative_titles: detail_bundle.alternative_titles,
            partial_enrichment: detail_bundle.partial_enrichment,
        }
        .into_candidate(query))
    }
}
