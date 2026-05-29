use crate::providers::MetadataProvider;

use super::{
    MAX_CANDIDATES_PER_QUERY, MetadataQuery, ProviderExternalIdCapability, ProviderFieldPolicy,
    ProviderRunPolicy,
    av::facts_from_query,
    provider_execution::{
        ProviderExecutionSummary, ProviderSuggestionSet, safe_provider_failure_reason,
    },
    ranking, resolver,
};

pub(crate) async fn suggest_candidates(
    providers: &[Box<dyn MetadataProvider>],
    query: &MetadataQuery,
    external_id_capabilities: &[ProviderExternalIdCapability],
    provider_field_policy: &ProviderFieldPolicy,
    provider_run_policy: &ProviderRunPolicy,
) -> ProviderSuggestionSet {
    let mut provider_candidates = Vec::new();
    let mut execution = ProviderExecutionSummary::for_policy(provider_run_policy);
    let av_route = facts_from_query(query).map(|facts| facts.route);

    for provider in providers {
        let provider_id = provider.id().as_str();
        if provider_run_policy.disables(provider_id) {
            execution.record_suppressed(provider_id);
            continue;
        }
        if let Some(route) = av_route
            && !provider.supports_av_route(route)
        {
            execution.record_skipped_by_av_route(provider_id, route);
            continue;
        }
        if !provider_run_policy.can_select_more(execution.selected_provider_ids.len()) {
            execution.record_budget_exhausted(provider_id);
            continue;
        }

        execution.record_selected(provider_id);
        match provider.suggest(query).await {
            Ok(candidates) => {
                execution.record_returned(provider_id, candidates.len());
                provider_candidates.extend(candidates);
            }
            Err(error) => {
                let safe_failure_reason = safe_provider_failure_reason(&error);
                execution.record_failed(provider_id, safe_failure_reason);
                tracing::warn!(
                    provider = provider_id,
                    safe_failure_reason,
                    "metadata provider failed"
                );
            }
        }
    }

    let mut candidates =
        resolver::resolve_provider_candidates(provider_candidates, external_id_capabilities)
            .into_iter()
            .map(|cluster| cluster.into_ranked_candidate(query, provider_field_policy))
            .collect::<Vec<_>>();
    candidates.sort_by(ranking::compare_metadata_candidates);
    if candidates.len() > MAX_CANDIDATES_PER_QUERY {
        candidates.truncate(MAX_CANDIDATES_PER_QUERY);
    }
    ProviderSuggestionSet {
        candidates,
        execution,
    }
}
