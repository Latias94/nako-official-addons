use std::collections::HashSet;

use crate::providers::MetadataProvider;

use super::{MAX_CANDIDATES_PER_QUERY, MetadataCandidate, MetadataQuery, ranking};

pub(crate) async fn suggest_candidates(
    providers: &[Box<dyn MetadataProvider>],
    query: &MetadataQuery,
) -> Vec<MetadataCandidate> {
    let mut candidates = Vec::new();

    for provider in providers {
        match provider.suggest(query).await {
            Ok(provider_candidates) => candidates.extend(
                provider_candidates
                    .into_iter()
                    .map(|candidate| ranking::rank_candidate(query, candidate)),
            ),
            Err(error) => {
                tracing::warn!(provider = provider.id().as_str(), %error, "metadata provider failed")
            }
        }
    }

    candidates.sort_by(|left, right| {
        right
            .confidence_milli
            .cmp(&left.confidence_milli)
            .then_with(|| left.provider.cmp(&right.provider))
            .then_with(|| left.provider_id.cmp(&right.provider_id))
    });
    let mut seen = HashSet::new();
    candidates.retain(|candidate| {
        seen.insert((candidate.provider.clone(), candidate.provider_id.clone()))
    });
    if candidates.len() > MAX_CANDIDATES_PER_QUERY {
        candidates.truncate(MAX_CANDIDATES_PER_QUERY);
    }
    candidates
}
