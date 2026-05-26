use serde::Serialize;

use crate::providers::MetadataProvider;

use super::{
    MAX_CANDIDATES_PER_QUERY, MetadataCandidate, MetadataQuery, ProviderExternalIdCapability,
    av::{AvNumberRoute, facts_from_query},
    ranking, resolver,
};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct ProviderSuggestionSet {
    pub(crate) candidates: Vec<MetadataCandidate>,
    pub(crate) execution: ProviderExecutionSummary,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub(crate) struct ProviderExecutionSummary {
    pub(crate) selected_provider_ids: Vec<String>,
    pub(crate) skipped_provider_ids: Vec<String>,
    pub(crate) returned_provider_ids: Vec<String>,
    pub(crate) failed_provider_ids: Vec<String>,
    pub(crate) returned_candidate_count: usize,
    pub(crate) providers: Vec<ProviderExecutionReport>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct ProviderExecutionReport {
    pub(crate) provider_id: String,
    pub(crate) status: ProviderExecutionStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) av_route: Option<AvNumberRoute>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) candidate_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) safe_failure_reason: Option<&'static str>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ProviderExecutionStatus {
    SkippedByAvRoute,
    ReturnedCandidates,
    Empty,
    Failed,
}

pub(crate) async fn suggest_candidates(
    providers: &[Box<dyn MetadataProvider>],
    query: &MetadataQuery,
    external_id_capabilities: &[ProviderExternalIdCapability],
) -> ProviderSuggestionSet {
    let mut provider_candidates = Vec::new();
    let mut execution = ProviderExecutionSummary::default();
    let av_route = facts_from_query(query).map(|facts| facts.route);

    for provider in providers {
        let provider_id = provider.id().as_str();
        if let Some(route) = av_route
            && !provider.supports_av_route(route)
        {
            execution.record_skipped_by_av_route(provider_id, route);
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
            .map(|cluster| cluster.into_ranked_candidate(query))
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

impl ProviderExecutionSummary {
    fn record_selected(&mut self, provider_id: &str) {
        push_unique(&mut self.selected_provider_ids, provider_id);
    }

    fn record_skipped_by_av_route(&mut self, provider_id: &str, route: AvNumberRoute) {
        push_unique(&mut self.skipped_provider_ids, provider_id);
        self.providers.push(ProviderExecutionReport {
            provider_id: provider_id.to_owned(),
            status: ProviderExecutionStatus::SkippedByAvRoute,
            av_route: Some(route),
            candidate_count: None,
            safe_failure_reason: None,
        });
    }

    fn record_returned(&mut self, provider_id: &str, candidate_count: usize) {
        if candidate_count > 0 {
            push_unique(&mut self.returned_provider_ids, provider_id);
        }
        self.returned_candidate_count += candidate_count;
        self.providers.push(ProviderExecutionReport {
            provider_id: provider_id.to_owned(),
            status: if candidate_count == 0 {
                ProviderExecutionStatus::Empty
            } else {
                ProviderExecutionStatus::ReturnedCandidates
            },
            av_route: None,
            candidate_count: Some(candidate_count),
            safe_failure_reason: None,
        });
    }

    fn record_failed(&mut self, provider_id: &str, safe_failure_reason: &'static str) {
        push_unique(&mut self.failed_provider_ids, provider_id);
        self.providers.push(ProviderExecutionReport {
            provider_id: provider_id.to_owned(),
            status: ProviderExecutionStatus::Failed,
            av_route: None,
            candidate_count: None,
            safe_failure_reason: Some(safe_failure_reason),
        });
    }
}

fn push_unique(values: &mut Vec<String>, value: &str) {
    if !values.iter().any(|existing| existing == value) {
        values.push(value.to_owned());
    }
}

fn safe_provider_failure_reason(error: &anyhow::Error) -> &'static str {
    let message = error.to_string().to_ascii_lowercase();
    if message.contains("timed out") || message.contains("timeout") {
        "timeout"
    } else if message.contains("429") || message.contains("rate limit") {
        "rate_limited"
    } else if message.contains("401")
        || message.contains("403")
        || message.contains("unauthorized")
        || message.contains("forbidden")
    {
        "auth_or_forbidden"
    } else if message.contains("404") || message.contains("not found") {
        "not_found"
    } else if message.contains("parse") || message.contains("malformed") {
        "parse_error"
    } else {
        "provider_error"
    }
}
