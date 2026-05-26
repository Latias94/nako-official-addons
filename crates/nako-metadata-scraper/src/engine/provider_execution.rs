use serde::Serialize;
use serde_json::Value;

use super::av::AvNumberRoute;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct ProviderSuggestionSet {
    pub(crate) candidates: Vec<super::MetadataCandidate>,
    pub(crate) execution: ProviderExecutionSummary,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct ProviderRunPolicy {
    disabled_provider_ids: Vec<String>,
    max_selected_providers: Option<usize>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub(crate) struct ProviderExecutionSummary {
    #[serde(skip_serializing_if = "ProviderExecutionPolicyReport::is_empty")]
    pub(crate) applied_policy: ProviderExecutionPolicyReport,
    pub(crate) selected_provider_ids: Vec<String>,
    pub(crate) skipped_provider_ids: Vec<String>,
    pub(crate) returned_provider_ids: Vec<String>,
    pub(crate) failed_provider_ids: Vec<String>,
    pub(crate) suppressed_provider_ids: Vec<String>,
    pub(crate) budget_exhausted_provider_ids: Vec<String>,
    pub(crate) returned_candidate_count: usize,
    pub(crate) providers: Vec<ProviderExecutionReport>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub(crate) struct ProviderExecutionPolicyReport {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    disabled_provider_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_selected_providers: Option<usize>,
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
    Suppressed,
    BudgetExhausted,
    ReturnedCandidates,
    Empty,
    Failed,
}

impl ProviderRunPolicy {
    #[must_use]
    pub(crate) fn from_max_selected_providers(max_selected_providers: Option<usize>) -> Self {
        Self {
            disabled_provider_ids: Vec::new(),
            max_selected_providers: normalize_positive_usize(max_selected_providers),
        }
    }

    #[must_use]
    pub(crate) fn from_payload_or_default(payload: &Value, default_policy: &Self) -> Self {
        let mut disabled_provider_ids = default_policy.disabled_provider_ids.clone();
        let mut max_selected_providers = default_policy.max_selected_providers;

        if let Some(policy) = payload
            .get("provider_execution_policy")
            .or_else(|| payload.get("provider_run_policy"))
            .and_then(Value::as_object)
        {
            merge_provider_ids(
                &mut disabled_provider_ids,
                policy
                    .get("disabled_provider_ids")
                    .or_else(|| policy.get("suppressed_provider_ids")),
            );
            max_selected_providers = policy
                .get("max_selected_providers")
                .or_else(|| policy.get("max_selected_providers_per_item"))
                .and_then(value_as_usize)
                .or(max_selected_providers);
        } else {
            merge_provider_ids(
                &mut disabled_provider_ids,
                payload
                    .get("disabled_provider_ids")
                    .or_else(|| payload.get("suppressed_provider_ids")),
            );
        }

        Self {
            disabled_provider_ids,
            max_selected_providers: normalize_positive_usize(max_selected_providers),
        }
    }

    #[must_use]
    pub(crate) fn disables(&self, provider_id: &str) -> bool {
        let Some(provider_id) = normalize_provider_id(provider_id) else {
            return false;
        };
        self.disabled_provider_ids
            .iter()
            .any(|disabled| disabled == &provider_id)
    }

    #[must_use]
    pub(crate) fn can_select_more(&self, selected_provider_count: usize) -> bool {
        self.max_selected_providers
            .is_none_or(|max_selected| selected_provider_count < max_selected)
    }
}

impl ProviderExecutionSummary {
    #[must_use]
    pub(crate) fn for_policy(provider_run_policy: &ProviderRunPolicy) -> Self {
        Self {
            applied_policy: ProviderExecutionPolicyReport {
                disabled_provider_ids: provider_run_policy.disabled_provider_ids.clone(),
                max_selected_providers: provider_run_policy.max_selected_providers,
            },
            ..Self::default()
        }
    }

    pub(crate) fn record_selected(&mut self, provider_id: &str) {
        push_unique(&mut self.selected_provider_ids, provider_id);
    }

    pub(crate) fn record_skipped_by_av_route(&mut self, provider_id: &str, route: AvNumberRoute) {
        push_unique(&mut self.skipped_provider_ids, provider_id);
        self.providers.push(ProviderExecutionReport {
            provider_id: provider_id.to_owned(),
            status: ProviderExecutionStatus::SkippedByAvRoute,
            av_route: Some(route),
            candidate_count: None,
            safe_failure_reason: None,
        });
    }

    pub(crate) fn record_suppressed(&mut self, provider_id: &str) {
        push_unique(&mut self.skipped_provider_ids, provider_id);
        push_unique(&mut self.suppressed_provider_ids, provider_id);
        self.providers.push(ProviderExecutionReport {
            provider_id: provider_id.to_owned(),
            status: ProviderExecutionStatus::Suppressed,
            av_route: None,
            candidate_count: None,
            safe_failure_reason: None,
        });
    }

    pub(crate) fn record_budget_exhausted(&mut self, provider_id: &str) {
        push_unique(&mut self.skipped_provider_ids, provider_id);
        push_unique(&mut self.budget_exhausted_provider_ids, provider_id);
        self.providers.push(ProviderExecutionReport {
            provider_id: provider_id.to_owned(),
            status: ProviderExecutionStatus::BudgetExhausted,
            av_route: None,
            candidate_count: None,
            safe_failure_reason: None,
        });
    }

    pub(crate) fn record_returned(&mut self, provider_id: &str, candidate_count: usize) {
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

    pub(crate) fn record_failed(&mut self, provider_id: &str, safe_failure_reason: &'static str) {
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

impl ProviderExecutionPolicyReport {
    fn is_empty(&self) -> bool {
        self.disabled_provider_ids.is_empty() && self.max_selected_providers.is_none()
    }
}

pub(crate) fn safe_provider_failure_reason(error: &anyhow::Error) -> &'static str {
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

pub(crate) fn normalize_provider_id(provider_id: &str) -> Option<String> {
    let provider_id = provider_id.trim().to_ascii_lowercase();
    (!provider_id.is_empty()).then_some(provider_id)
}

fn merge_provider_ids(values: &mut Vec<String>, input: Option<&Value>) {
    let Some(input) = input else {
        return;
    };

    if let Some(provider_id) = input.as_str().and_then(normalize_provider_id) {
        push_unique(values, &provider_id);
        return;
    }

    for provider_id in input
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .filter_map(normalize_provider_id)
    {
        push_unique(values, &provider_id);
    }
}

fn value_as_usize(value: &Value) -> Option<usize> {
    value
        .as_u64()
        .and_then(|value| usize::try_from(value).ok())
        .or_else(|| value.as_str()?.trim().parse::<usize>().ok())
}

fn normalize_positive_usize(value: Option<usize>) -> Option<usize> {
    value.filter(|value| *value > 0)
}

fn push_unique(values: &mut Vec<String>, value: &str) {
    if !values.iter().any(|existing| existing == value) {
        values.push(value.to_owned());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_execution_policy_parses_request_visible_budget_and_suppression() {
        let policy = ProviderRunPolicy::from_payload_or_default(
            &serde_json::json!({
                "provider_execution_policy": {
                    "disabled_provider_ids": [" JavDB ", "javdb", "dmm"],
                    "max_selected_providers": "2"
                }
            }),
            &ProviderRunPolicy::default(),
        );

        assert!(policy.disables("javdb"));
        assert!(policy.disables("DMM"));
        assert!(policy.can_select_more(1));
        assert!(!policy.can_select_more(2));
        assert_eq!(
            ProviderExecutionSummary::for_policy(&policy).applied_policy,
            ProviderExecutionPolicyReport {
                disabled_provider_ids: vec!["javdb".to_owned(), "dmm".to_owned()],
                max_selected_providers: Some(2),
            }
        );
    }
}
