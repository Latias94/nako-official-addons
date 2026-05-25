use crate::providers::MetadataProvider;

use super::{
    MAX_CANDIDATES_PER_QUERY, MetadataCandidate, MetadataQuery, ProviderExternalIdCapability,
    ranking, resolver,
};

pub(crate) async fn suggest_candidates(
    providers: &[Box<dyn MetadataProvider>],
    query: &MetadataQuery,
    external_id_capabilities: &[ProviderExternalIdCapability],
) -> Vec<MetadataCandidate> {
    let mut provider_candidates = Vec::new();

    for provider in providers {
        match provider.suggest(query).await {
            Ok(candidates) => provider_candidates.extend(candidates),
            Err(error) => {
                tracing::warn!(provider = provider.id().as_str(), %error, "metadata provider failed")
            }
        }
    }

    let mut candidates =
        resolver::resolve_provider_candidates(provider_candidates, external_id_capabilities)
            .into_iter()
            .map(|cluster| cluster.into_ranked_candidate(query))
            .collect::<Vec<_>>();
    candidates.sort_by(ranking::compare_metadata_candidates);
    if candidates.len() > MAX_CANDIDATES_PER_QUERY {
        candidates.truncate(MAX_CANDIDATES_PER_QUERY);
    }
    candidates
}
