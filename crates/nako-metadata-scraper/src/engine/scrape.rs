use serde_json::Value;

use super::{
    MetadataCandidate, MetadataQuery, MetadataWritebackResult,
    artwork::ArtworkWritebackResult,
    av::{self, AvQueryFacts},
    provider_execution::{ProviderExecutionStatus, ProviderExecutionSummary},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct MetadataScrapeOutcome {
    pub(crate) query: MetadataQuery,
    pub(crate) av: Option<AvQueryFacts>,
    pub(crate) candidates: Vec<MetadataCandidate>,
    pub(crate) provider_execution: ProviderExecutionSummary,
    pub(crate) writeback_result: Option<MetadataWritebackResult>,
    pub(crate) artwork_writeback_result: Option<ArtworkWritebackResult>,
}

impl MetadataScrapeOutcome {
    pub(crate) fn av_value(&self) -> Option<Value> {
        self.av
            .as_ref()
            .map(|facts| serde_json::to_value(facts).expect("AV query facts are serializable"))
    }

    pub(crate) fn safe_failure_reason(&self) -> Option<&'static str> {
        if !self.candidates.is_empty() {
            return None;
        }

        if !self.provider_execution.failed_provider_ids.is_empty() {
            return Some("provider_failed");
        }
        if self.provider_execution.selected_provider_ids.is_empty()
            && !self.provider_execution.suppressed_provider_ids.is_empty()
        {
            return Some("provider_suppressed");
        }
        if self.provider_execution.selected_provider_ids.is_empty()
            && !self
                .provider_execution
                .budget_exhausted_provider_ids
                .is_empty()
        {
            return Some("provider_budget_exhausted");
        }
        if self.provider_execution.selected_provider_ids.is_empty()
            && !self.provider_execution.skipped_provider_ids.is_empty()
        {
            return Some("provider_skipped_by_route");
        }

        Some("no_candidates")
    }

    pub(crate) fn suppressed_provider_ids(&self) -> Vec<String> {
        if !self.provider_execution.suppressed_provider_ids.is_empty() {
            return self.provider_execution.suppressed_provider_ids.clone();
        }

        self.provider_execution
            .providers
            .iter()
            .filter(|report| report.status == ProviderExecutionStatus::Suppressed)
            .fold(Vec::new(), |mut provider_ids, report| {
                if !provider_ids
                    .iter()
                    .any(|provider_id| provider_id == &report.provider_id)
                {
                    provider_ids.push(report.provider_id.clone());
                }
                provider_ids
            })
    }
}

pub(crate) fn av_facts_for_outcome(payload: &Value, query: &MetadataQuery) -> Option<AvQueryFacts> {
    av::facts_from_payload(payload).or_else(|| av::facts_from_query(query))
}
