use std::collections::{BTreeMap, HashMap};

use nako_addon_protocol::{ADDON_PROTOCOL_VERSION, AddonTaskRequest, AddonTaskResponse};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::engine::{
    MetadataScrapeRuntime, av,
    provider_execution::{ProviderExecutionReport, ProviderExecutionStatus},
    response,
};

pub const BULK_METADATA_SCRAPE_TASK_ID: &str = "bulk-metadata-scrape";
pub const BULK_METADATA_SCRAPE_TASK_NAME: &str = "Bulk metadata scrape";
pub const BULK_METADATA_SCRAPE_TASK_PATH: &str = "/tasks/bulk-metadata-scrape";
pub const BULK_METADATA_SCRAPE_TASK_DESCRIPTION: &str =
    "Runs metadata suggestions for a bounded batch of items";
pub const BULK_METADATA_SCRAPE_OUTPUT_SCHEMA: &str =
    "nako.official.metadata-scraper.bulk-metadata-scrape.result.v1";

const DEFAULT_BATCH_SIZE: usize = 4;
const MAX_BATCH_SIZE: usize = 12;
const DEFAULT_PROVIDER_SUPPRESS_AFTER_FAILURES: usize = 2;
const DEFAULT_PROVIDER_COOLDOWN_ITEMS: usize = 3;
const DEFAULT_REUSABLE_ITEM_LIMIT: usize = 128;
const DEFAULT_PROVIDER_STATE_LIMIT: usize = 64;

#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub struct BulkMetadataScrapeError {
    message: String,
}

impl BulkMetadataScrapeError {
    fn invalid(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
struct BulkMetadataScrapeTaskInput {
    items: Vec<Value>,
    #[serde(default)]
    cursor: Option<usize>,
    #[serde(default)]
    batch_size: Option<usize>,
    #[serde(default)]
    provider_policy: BulkProviderPolicy,
    #[serde(default)]
    resume_state: BulkMetadataScrapeResumeState,
}

#[derive(Clone, Debug, Serialize, Eq, PartialEq)]
struct BulkMetadataScrapeTaskOutput {
    schema: &'static str,
    library_id: Option<String>,
    source_id: Option<String>,
    cursor: usize,
    batch_size: usize,
    provider_policy: BulkProviderPolicy,
    total_items: usize,
    processed_items: usize,
    remaining_items: usize,
    next_cursor: Option<usize>,
    summary: BulkMetadataScrapeTaskSummary,
    resume_state: BulkMetadataScrapeResumeState,
    items: Vec<BulkMetadataScrapeTaskItemOutput>,
}

#[derive(Clone, Debug, Default, Serialize, Eq, PartialEq)]
struct BulkMetadataScrapeTaskSummary {
    scraped_items: usize,
    reused_items: usize,
    av_items: usize,
    empty_candidate_items: usize,
    failed_items: usize,
    suppressed_items: usize,
    budget_exhausted_items: usize,
    failure_reasons: BTreeMap<String, usize>,
    retry_classes: BTreeMap<String, usize>,
    provider_execution: Vec<BulkMetadataScrapeProviderSummary>,
}

#[derive(Clone, Debug, Default, Serialize, Eq, PartialEq)]
struct BulkMetadataScrapeProviderSummary {
    provider_id: String,
    selected_items: usize,
    skipped_by_route_items: usize,
    returned_items: usize,
    empty_items: usize,
    failed_items: usize,
    suppressed_items: usize,
    budget_exhausted_items: usize,
    retryable_failed_items: usize,
    permanent_failed_items: usize,
    operator_action_failed_items: usize,
    cooldown_remaining_items: usize,
    returned_candidate_count: usize,
}

#[derive(Clone, Debug, Serialize, Eq, PartialEq)]
struct BulkMetadataScrapeTaskItemOutput {
    index: usize,
    request_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    av: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reused_from_index: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    safe_failure_reason: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    suppressed_provider_ids: Vec<String>,
    payload: Value,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct BulkMetadataScrapeReuseKey {
    av_number: String,
    language_hint: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct BulkMetadataScrapeReuseEntry {
    index: usize,
    av: Option<Value>,
    safe_failure_reason: Option<String>,
    suppressed_provider_ids: Vec<String>,
    payload: Value,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
struct BulkMetadataScrapeResumeState {
    #[serde(default)]
    reusable_items: Vec<BulkMetadataScrapeResumeItem>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    provider_states: Vec<BulkProviderResumeState>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct BulkMetadataScrapeResumeItem {
    av_number: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    language_hint: Option<String>,
    index: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    av: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    safe_failure_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    suppressed_provider_ids: Vec<String>,
    payload: Value,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct BulkProviderPolicy {
    #[serde(default = "default_provider_suppress_after_failures")]
    suppress_after_failures: usize,
    #[serde(default = "default_provider_cooldown_items")]
    cooldown_items: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    max_selected_providers_per_item: Option<usize>,
    #[serde(default = "default_reusable_item_limit")]
    max_reusable_items: usize,
    #[serde(default = "default_provider_state_limit")]
    max_provider_states: usize,
}

impl Default for BulkProviderPolicy {
    fn default() -> Self {
        Self {
            suppress_after_failures: DEFAULT_PROVIDER_SUPPRESS_AFTER_FAILURES,
            cooldown_items: DEFAULT_PROVIDER_COOLDOWN_ITEMS,
            max_selected_providers_per_item: None,
            max_reusable_items: DEFAULT_REUSABLE_ITEM_LIMIT,
            max_provider_states: DEFAULT_PROVIDER_STATE_LIMIT,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct BulkProviderResumeState {
    provider_id: String,
    #[serde(default)]
    consecutive_failures: usize,
    #[serde(default)]
    cooldown_remaining_items: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    last_failure_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    retry_class: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct BulkProviderState {
    consecutive_failures: usize,
    cooldown_remaining_items: usize,
    last_failure_reason: Option<String>,
    retry_class: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct BulkProviderStateTable {
    states: BTreeMap<String, BulkProviderState>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct BulkMetadataScrapeBatchPlan {
    cursor: usize,
    batch_size: usize,
    total_items: usize,
    next_cursor: Option<usize>,
    item_indexes: Vec<usize>,
}

impl BulkMetadataScrapeBatchPlan {
    fn new(total_items: usize, cursor: usize, batch_size: usize) -> Self {
        let cursor = cursor.min(total_items);
        let batch_size = batch_size.clamp(1, MAX_BATCH_SIZE);
        let end = cursor.saturating_add(batch_size).min(total_items);
        let item_indexes = (cursor..end).collect::<Vec<_>>();

        Self {
            cursor,
            batch_size,
            total_items,
            next_cursor: (end < total_items).then_some(end),
            item_indexes,
        }
    }

    fn remaining_items(&self) -> usize {
        self.total_items
            .saturating_sub(self.cursor.saturating_add(self.item_indexes.len()))
    }
}

impl<T> MetadataScrapeRuntime<T>
where
    T: crate::nako_runtime::NakoRuntimeTransport,
{
    pub async fn bulk_scrape(
        &self,
        request: AddonTaskRequest,
    ) -> Result<AddonTaskResponse, BulkMetadataScrapeError> {
        if request.task_id != BULK_METADATA_SCRAPE_TASK_ID {
            return Err(BulkMetadataScrapeError::invalid(format!(
                "unexpected task_id {}",
                request.task_id
            )));
        }

        let AddonTaskRequest {
            addon_id,
            task_id,
            job_id,
            request_id,
            library_id,
            source_id,
            payload,
            ..
        } = request;
        let input =
            serde_json::from_value::<BulkMetadataScrapeTaskInput>(payload).map_err(|error| {
                BulkMetadataScrapeError::invalid(format!("invalid bulk task payload: {error}"))
            })?;
        let BulkMetadataScrapeTaskInput {
            items: input_items,
            cursor,
            batch_size,
            mut provider_policy,
            resume_state,
        } = input;
        provider_policy.normalize();
        let plan = BulkMetadataScrapeBatchPlan::new(
            input_items.len(),
            cursor.unwrap_or(0),
            batch_size.unwrap_or(DEFAULT_BATCH_SIZE),
        );

        let mut items = Vec::with_capacity(plan.item_indexes.len());
        let mut summary = BulkMetadataScrapeTaskSummary::default();
        let BulkMetadataScrapeResumeState {
            reusable_items,
            provider_states,
        } = resume_state;
        let mut reuse_cache = BulkMetadataScrapeResumeState::reuse_cache_from_items(
            reusable_items,
            provider_policy.max_reusable_items,
        );
        let mut provider_states = BulkProviderStateTable::from_resume(
            provider_states,
            provider_policy.max_provider_states,
        );
        for index in &plan.item_indexes {
            let item_request_id = format!("{}:item-{}", request_id, index);
            let payload = input_items[*index].clone();
            let planned_av = av_facts_value_from_payload(&payload);
            let reuse_key = BulkMetadataScrapeReuseKey::from_payload(&payload);
            if planned_av.is_some() {
                summary.av_items += 1;
            }

            if let Some(cache_key) = reuse_key.as_ref()
                && let Some(entry) = reuse_cache.get(cache_key)
            {
                let item_av = planned_av.or_else(|| entry.av.clone());
                let item_payload = reused_item_payload(&entry.payload, item_av.as_ref());
                let safe_failure_reason = entry.safe_failure_reason.clone();
                let suppressed_provider_ids = entry.suppressed_provider_ids.clone();
                summary.record_failure(safe_failure_reason.as_deref());
                summary.record_suppressed_item(&suppressed_provider_ids);
                summary.reused_items += 1;
                items.push(BulkMetadataScrapeTaskItemOutput {
                    index: *index,
                    request_id: item_request_id,
                    av: item_av,
                    reused_from_index: Some(entry.index),
                    safe_failure_reason,
                    suppressed_provider_ids,
                    payload: item_payload,
                });
                continue;
            }

            let disabled_provider_ids = provider_states.disabled_provider_ids();
            let payload = payload_with_provider_execution_policy(
                payload,
                &disabled_provider_ids,
                &provider_policy,
            );
            let item_outcome = self.scrape_outcome(&item_request_id, &payload).await;
            let item_payload = response::metadata_payload(&item_outcome);
            let item_av = planned_av.or_else(|| item_outcome.av_value());
            let safe_failure_reason = item_outcome
                .safe_failure_reason()
                .map(std::borrow::ToOwned::to_owned);
            let suppressed_provider_ids = item_outcome.suppressed_provider_ids();
            summary.record_failure(safe_failure_reason.as_deref());
            summary.record_suppressed_item(&suppressed_provider_ids);
            summary.record_provider_execution(&item_outcome.provider_execution.providers);
            provider_states
                .record_reports(&item_outcome.provider_execution.providers, &provider_policy);
            provider_states.consume_cooldowns(&disabled_provider_ids);
            summary.scraped_items += 1;
            if let Some(cache_key) = reuse_key {
                reuse_cache.insert(
                    cache_key,
                    BulkMetadataScrapeReuseEntry {
                        index: *index,
                        av: item_av.clone(),
                        safe_failure_reason: safe_failure_reason.clone(),
                        suppressed_provider_ids: suppressed_provider_ids.clone(),
                        payload: item_payload.clone(),
                    },
                );
            }
            items.push(BulkMetadataScrapeTaskItemOutput {
                index: *index,
                request_id: item_request_id,
                av: item_av,
                reused_from_index: None,
                safe_failure_reason,
                suppressed_provider_ids,
                payload: item_payload,
            });
        }
        summary.record_provider_states(
            provider_states.resume_states_with_limit(provider_policy.max_provider_states),
        );
        let output_provider_policy = provider_policy.clone();

        let output = BulkMetadataScrapeTaskOutput {
            schema: BULK_METADATA_SCRAPE_OUTPUT_SCHEMA,
            library_id,
            source_id,
            cursor: plan.cursor,
            batch_size: plan.batch_size,
            provider_policy,
            total_items: plan.total_items,
            processed_items: items.len(),
            remaining_items: plan.remaining_items(),
            next_cursor: plan.next_cursor,
            summary,
            resume_state: BulkMetadataScrapeResumeState::from_caches(
                &reuse_cache,
                &provider_states,
                output_provider_policy.max_reusable_items,
                output_provider_policy.max_provider_states,
            ),
            items,
        };

        Ok(AddonTaskResponse {
            protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
            addon_id,
            task_id,
            job_id,
            request_id,
            output: serde_json::to_value(output).expect("bulk task output is serializable"),
        })
    }
}

impl BulkMetadataScrapeTaskSummary {
    fn record_failure(&mut self, safe_failure_reason: Option<&str>) {
        let Some(reason) = safe_failure_reason else {
            return;
        };

        self.empty_candidate_items += 1;
        self.failed_items += 1;
        *self.failure_reasons.entry(reason.to_owned()).or_default() += 1;
    }

    fn record_suppressed_item(&mut self, suppressed_provider_ids: &[String]) {
        if !suppressed_provider_ids.is_empty() {
            self.suppressed_items += 1;
        }
    }

    fn record_provider_execution(&mut self, reports: &[ProviderExecutionReport]) {
        for report in reports {
            let retry_class = report
                .safe_failure_reason
                .and_then(retry_class_for_failure_reason);
            if let Some(retry_class) = retry_class {
                *self
                    .retry_classes
                    .entry(retry_class.to_owned())
                    .or_default() += 1;
            }
            let provider = self.provider_summary_mut(&report.provider_id);

            match report.status {
                ProviderExecutionStatus::SkippedByAvRoute => {
                    provider.skipped_by_route_items += 1;
                }
                ProviderExecutionStatus::Suppressed => provider.suppressed_items += 1,
                ProviderExecutionStatus::BudgetExhausted => {
                    provider.budget_exhausted_items += 1;
                    self.budget_exhausted_items += 1;
                }
                ProviderExecutionStatus::ReturnedCandidates => {
                    provider.selected_items += 1;
                    provider.returned_items += 1;
                    provider.returned_candidate_count += report.candidate_count.unwrap_or(0);
                }
                ProviderExecutionStatus::Empty => {
                    provider.selected_items += 1;
                    provider.empty_items += 1;
                }
                ProviderExecutionStatus::Failed => {
                    provider.selected_items += 1;
                    provider.failed_items += 1;
                    match retry_class {
                        Some("retryable") => provider.retryable_failed_items += 1,
                        Some("permanent") => provider.permanent_failed_items += 1,
                        Some("operator_action") => provider.operator_action_failed_items += 1,
                        _ => {}
                    }
                }
            }
        }
    }

    fn record_provider_states(&mut self, provider_states: Vec<BulkProviderResumeState>) {
        for state in provider_states {
            let provider = self.provider_summary_mut(&state.provider_id);
            provider.cooldown_remaining_items = state.cooldown_remaining_items;
        }
    }

    fn provider_summary_mut(
        &mut self,
        provider_id: &str,
    ) -> &mut BulkMetadataScrapeProviderSummary {
        if let Some(index) = self
            .provider_execution
            .iter()
            .position(|summary| summary.provider_id == provider_id)
        {
            return &mut self.provider_execution[index];
        }

        self.provider_execution
            .push(BulkMetadataScrapeProviderSummary {
                provider_id: provider_id.to_owned(),
                ..BulkMetadataScrapeProviderSummary::default()
            });
        self.provider_execution
            .last_mut()
            .expect("provider summary was just pushed")
    }
}

impl BulkMetadataScrapeReuseKey {
    fn from_payload(payload: &Value) -> Option<Self> {
        if has_side_effect_request(payload, "writeback")
            || has_side_effect_request(payload, "artwork_writeback")
        {
            return None;
        }

        let av_facts = av::facts_from_payload(payload)?;
        Some(Self {
            av_number: av_facts.number,
            language_hint: language_hint(payload),
        })
    }
}

impl BulkMetadataScrapeResumeState {
    fn reuse_cache_from_items(
        reusable_items: Vec<BulkMetadataScrapeResumeItem>,
        max_reusable_items: usize,
    ) -> HashMap<BulkMetadataScrapeReuseKey, BulkMetadataScrapeReuseEntry> {
        let mut entries = reusable_items
            .into_iter()
            .filter_map(|item| {
                let key = item.reuse_key()?;
                let entry = BulkMetadataScrapeReuseEntry {
                    index: item.index,
                    av: item.av,
                    safe_failure_reason: item
                        .safe_failure_reason
                        .map(|reason| reason.trim().to_ascii_lowercase())
                        .filter(|reason| !reason.is_empty()),
                    suppressed_provider_ids: item
                        .suppressed_provider_ids
                        .into_iter()
                        .filter_map(|provider_id| normalize_provider_id(&provider_id))
                        .fold(Vec::new(), |mut provider_ids, provider_id| {
                            push_unique_string(&mut provider_ids, provider_id);
                            provider_ids
                        }),
                    payload: item.payload,
                };
                Some((key, entry))
            })
            .collect::<Vec<_>>();
        entries.sort_by(|left, right| {
            left.0
                .av_number
                .cmp(&right.0.av_number)
                .then_with(|| left.0.language_hint.cmp(&right.0.language_hint))
        });
        entries.into_iter().take(max_reusable_items).collect()
    }

    fn from_caches(
        reuse_cache: &HashMap<BulkMetadataScrapeReuseKey, BulkMetadataScrapeReuseEntry>,
        provider_states: &BulkProviderStateTable,
        max_reusable_items: usize,
        max_provider_states: usize,
    ) -> Self {
        let mut reusable_items = reuse_cache
            .iter()
            .map(|(key, entry)| BulkMetadataScrapeResumeItem {
                av_number: key.av_number.clone(),
                language_hint: key.language_hint.clone(),
                index: entry.index,
                av: entry.av.clone(),
                safe_failure_reason: entry.safe_failure_reason.clone(),
                suppressed_provider_ids: entry.suppressed_provider_ids.clone(),
                payload: entry.payload.clone(),
            })
            .collect::<Vec<_>>();
        reusable_items.sort_by(|left, right| {
            left.av_number
                .cmp(&right.av_number)
                .then_with(|| left.language_hint.cmp(&right.language_hint))
                .then_with(|| left.index.cmp(&right.index))
        });
        reusable_items.truncate(max_reusable_items);
        Self {
            reusable_items,
            provider_states: provider_states.resume_states_with_limit(max_provider_states),
        }
    }
}

impl BulkMetadataScrapeResumeItem {
    fn reuse_key(&self) -> Option<BulkMetadataScrapeReuseKey> {
        let av_facts = av::facts_from_text(&self.av_number, av::AvNumberSource::ExternalId)?;
        Some(BulkMetadataScrapeReuseKey {
            av_number: av_facts.number,
            language_hint: self
                .language_hint
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_owned),
        })
    }
}

impl BulkProviderPolicy {
    fn enables_suppression(&self) -> bool {
        self.suppress_after_failures > 0 && self.cooldown_items > 0
    }

    fn normalize(&mut self) {
        self.max_selected_providers_per_item = self
            .max_selected_providers_per_item
            .filter(|value| *value > 0);
    }
}

impl BulkProviderStateTable {
    fn from_resume(states: Vec<BulkProviderResumeState>, max_provider_states: usize) -> Self {
        let states = states
            .into_iter()
            .filter_map(|state| {
                let provider_id = normalize_provider_id(&state.provider_id)?;
                Some((
                    provider_id,
                    BulkProviderState {
                        consecutive_failures: state.consecutive_failures,
                        cooldown_remaining_items: state.cooldown_remaining_items,
                        last_failure_reason: state
                            .last_failure_reason
                            .map(|reason| reason.trim().to_ascii_lowercase())
                            .filter(|reason| !reason.is_empty()),
                        retry_class: state
                            .retry_class
                            .map(|retry_class| retry_class.trim().to_ascii_lowercase())
                            .filter(|retry_class| !retry_class.is_empty()),
                    },
                ))
            })
            .take(max_provider_states)
            .collect();

        Self { states }
    }

    fn disabled_provider_ids(&self) -> Vec<String> {
        self.states
            .iter()
            .filter_map(|(provider_id, state)| {
                (state.cooldown_remaining_items > 0).then(|| provider_id.clone())
            })
            .collect()
    }

    fn record_reports(&mut self, reports: &[ProviderExecutionReport], policy: &BulkProviderPolicy) {
        for report in reports {
            match report.status {
                ProviderExecutionStatus::ReturnedCandidates | ProviderExecutionStatus::Empty => {
                    self.states.remove(&report.provider_id);
                }
                ProviderExecutionStatus::Failed => {
                    self.record_failure(
                        &report.provider_id,
                        report.safe_failure_reason.unwrap_or("provider_error"),
                        policy,
                    );
                }
                _ => {}
            }
        }
    }

    fn record_failure(&mut self, provider_id: &str, reason: &str, policy: &BulkProviderPolicy) {
        let retry_class = retry_class_for_failure_reason(reason).unwrap_or("retryable");
        let state = self
            .states
            .entry(provider_id.to_owned())
            .or_insert_with(|| BulkProviderState {
                consecutive_failures: 0,
                cooldown_remaining_items: 0,
                last_failure_reason: None,
                retry_class: None,
            });
        state.last_failure_reason = Some(reason.to_owned());
        state.retry_class = Some(retry_class.to_owned());

        match retry_class {
            "retryable" => {
                state.consecutive_failures += 1;
                if policy.enables_suppression()
                    && state.consecutive_failures >= policy.suppress_after_failures
                {
                    state.cooldown_remaining_items = policy.cooldown_items;
                }
            }
            "operator_action" => {
                state.consecutive_failures = 1;
                if policy.enables_suppression() {
                    state.cooldown_remaining_items = policy.cooldown_items;
                }
            }
            "permanent" => {
                state.consecutive_failures = 0;
                state.cooldown_remaining_items = 0;
            }
            _ => {}
        }
    }

    fn consume_cooldowns(&mut self, provider_ids: &[String]) {
        for provider_id in provider_ids {
            let Some(provider_id) = normalize_provider_id(provider_id) else {
                continue;
            };
            let Some(state) = self.states.get_mut(&provider_id) else {
                continue;
            };
            if state.cooldown_remaining_items == 0 {
                continue;
            }
            state.cooldown_remaining_items -= 1;
            if state.cooldown_remaining_items == 0 {
                state.consecutive_failures = 0;
            }
        }
    }

    fn resume_states_with_limit(&self, max_provider_states: usize) -> Vec<BulkProviderResumeState> {
        self.states
            .iter()
            .filter_map(|(provider_id, state)| {
                if state.consecutive_failures == 0
                    && state.cooldown_remaining_items == 0
                    && state.last_failure_reason.is_none()
                    && state.retry_class.is_none()
                {
                    return None;
                }

                Some(BulkProviderResumeState {
                    provider_id: provider_id.clone(),
                    consecutive_failures: state.consecutive_failures,
                    cooldown_remaining_items: state.cooldown_remaining_items,
                    last_failure_reason: state.last_failure_reason.clone(),
                    retry_class: state.retry_class.clone(),
                })
            })
            .take(max_provider_states)
            .collect()
    }
}

fn av_facts_value_from_payload(payload: &Value) -> Option<Value> {
    av::facts_from_payload(payload)
        .map(|facts| serde_json::to_value(facts).expect("AV query facts are serializable"))
}

fn reused_item_payload(payload: &Value, av: Option<&Value>) -> Value {
    let mut payload = payload.clone();
    if let Some(av) = av {
        payload["query"]["av"] = av.clone();
    }
    payload
}

fn payload_with_provider_execution_policy(
    mut payload: Value,
    provider_ids: &[String],
    provider_policy: &BulkProviderPolicy,
) -> Value {
    let max_selected_providers = provider_policy.max_selected_providers_per_item;
    if provider_ids.is_empty() && max_selected_providers.is_none() {
        return payload;
    }

    let mut disabled_provider_ids =
        existing_provider_ids(&payload, "provider_execution_policy.disabled_provider_ids");
    if disabled_provider_ids.is_empty() {
        disabled_provider_ids = existing_provider_ids(
            &payload,
            "provider_execution_policy.suppressed_provider_ids",
        );
    }
    if disabled_provider_ids.is_empty() {
        disabled_provider_ids = existing_provider_ids(&payload, "disabled_provider_ids");
    }
    if disabled_provider_ids.is_empty() {
        disabled_provider_ids = existing_provider_ids(&payload, "suppressed_provider_ids");
    }
    for provider_id in provider_ids {
        if let Some(provider_id) = normalize_provider_id(provider_id) {
            push_unique_string(&mut disabled_provider_ids, provider_id);
        }
    }

    let mut execution_policy = payload
        .get("provider_execution_policy")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    if !disabled_provider_ids.is_empty() {
        execution_policy.insert(
            "disabled_provider_ids".to_owned(),
            Value::Array(
                disabled_provider_ids
                    .into_iter()
                    .map(Value::String)
                    .collect::<Vec<_>>(),
            ),
        );
    }
    if let Some(max_selected_providers) = max_selected_providers {
        execution_policy.insert(
            "max_selected_providers".to_owned(),
            Value::Number((max_selected_providers as u64).into()),
        );
    }
    payload["provider_execution_policy"] = Value::Object(execution_policy);
    payload
}

fn existing_provider_ids(payload: &Value, key: &str) -> Vec<String> {
    value_at_path(payload, key)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .filter_map(normalize_provider_id)
        .fold(Vec::new(), |mut provider_ids, provider_id| {
            push_unique_string(&mut provider_ids, provider_id);
            provider_ids
        })
}

fn value_at_path<'a>(payload: &'a Value, path: &str) -> Option<&'a Value> {
    path.split('.')
        .try_fold(payload, |value, segment| value.get(segment))
}

fn retry_class_for_failure_reason(reason: &str) -> Option<&'static str> {
    match reason {
        "timeout" | "rate_limited" | "provider_error" => Some("retryable"),
        "auth_or_forbidden" => Some("operator_action"),
        "not_found" | "parse_error" => Some("permanent"),
        _ => None,
    }
}

fn normalize_provider_id(provider_id: &str) -> Option<String> {
    let provider_id = provider_id.trim().to_ascii_lowercase();
    (!provider_id.is_empty()).then_some(provider_id)
}

fn push_unique_string(values: &mut Vec<String>, value: String) {
    if !values.iter().any(|existing| existing == &value) {
        values.push(value);
    }
}

fn default_provider_suppress_after_failures() -> usize {
    DEFAULT_PROVIDER_SUPPRESS_AFTER_FAILURES
}

fn default_provider_cooldown_items() -> usize {
    DEFAULT_PROVIDER_COOLDOWN_ITEMS
}

fn default_reusable_item_limit() -> usize {
    DEFAULT_REUSABLE_ITEM_LIMIT
}

fn default_provider_state_limit() -> usize {
    DEFAULT_PROVIDER_STATE_LIMIT
}

fn has_side_effect_request(payload: &Value, key: &str) -> bool {
    payload.get(key).is_some_and(|value| !value.is_null())
}

fn language_hint(payload: &Value) -> Option<String> {
    payload
        .get("language")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

#[cfg(test)]
mod tests {
    use std::{
        collections::VecDeque,
        sync::{Arc, Mutex},
    };

    use async_trait::async_trait;
    use nako_addon_protocol::{ADDON_PROTOCOL_VERSION, AddonArtworkKind};

    use crate::{
        config::ProviderId,
        nako_runtime::{
            NakoRuntimeClient, NakoRuntimeClientConfig, NakoRuntimeError, NakoRuntimeHttpRequest,
            NakoRuntimeHttpResponse, NakoRuntimeResult, NakoRuntimeTransport,
        },
        providers::MetadataProvider,
    };

    use super::*;
    use crate::engine::{
        MetadataQuery, ProviderArtworkCandidate, ProviderArtworkCandidateFacts,
        ProviderCandidateFacts, ProviderMetadataCandidate,
    };

    #[tokio::test]
    async fn bulk_metadata_scrape_plans_bounded_batches() {
        let runtime =
            MetadataScrapeRuntime::<crate::nako_runtime::ReqwestNakoRuntimeTransport>::new(
                "en-US",
                vec![Box::new(BulkCandidateProvider)],
                None,
            );

        let response = runtime
            .bulk_scrape(AddonTaskRequest {
                protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
                addon_id: "addon-1".to_owned(),
                task_id: BULK_METADATA_SCRAPE_TASK_ID.to_owned(),
                job_id: "job-1".to_owned(),
                request_id: "request-1".to_owned(),
                attempt: 1,
                retry_of_job_id: None,
                library_id: Some("library-1".to_owned()),
                source_id: Some("source-1".to_owned()),
                payload: serde_json::json!({
                    "cursor": 0,
                    "batch_size": 1,
                    "items": [
                        {"title": "The Matrix", "year": 1999},
                        {"title": "Inception", "year": 2010}
                    ]
                }),
            })
            .await
            .unwrap();

        assert_eq!(response.task_id, BULK_METADATA_SCRAPE_TASK_ID);
        assert_eq!(
            response.output["schema"],
            BULK_METADATA_SCRAPE_OUTPUT_SCHEMA
        );
        assert_eq!(response.output["cursor"], 0);
        assert_eq!(response.output["batch_size"], 1);
        assert_eq!(response.output["total_items"], 2);
        assert_eq!(response.output["processed_items"], 1);
        assert_eq!(response.output["remaining_items"], 1);
        assert_eq!(response.output["next_cursor"], 1);
        assert_eq!(response.output["summary"]["scraped_items"], 1);
        assert_eq!(response.output["summary"]["reused_items"], 0);
        assert_eq!(response.output["items"][0]["index"], 0);
        assert_eq!(
            response.output["items"][0]["payload"]["query"]["title"],
            "The Matrix"
        );
        assert_eq!(
            response.output["items"][0]["payload"]["query"]["language"],
            "en-US"
        );
    }

    #[tokio::test]
    async fn bulk_metadata_scrape_includes_av_planning_summary() {
        let runtime =
            MetadataScrapeRuntime::<crate::nako_runtime::ReqwestNakoRuntimeTransport>::new(
                "zh-CN",
                vec![Box::new(BulkCandidateProvider)],
                None,
            );

        let response = runtime
            .bulk_scrape(AddonTaskRequest {
                protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
                addon_id: "addon-1".to_owned(),
                task_id: BULK_METADATA_SCRAPE_TASK_ID.to_owned(),
                job_id: "job-1".to_owned(),
                request_id: "request-1".to_owned(),
                attempt: 1,
                retry_of_job_id: None,
                library_id: Some("library-1".to_owned()),
                source_id: Some("source-1".to_owned()),
                payload: serde_json::json!({
                    "batch_size": 1,
                    "items": [
                        {"file_name": "[HD] ssni00644 1080p x264.mkv"}
                    ]
                }),
            })
            .await
            .unwrap();

        assert_eq!(response.output["items"][0]["av"]["number"], "SSNI-644");
        assert_eq!(response.output["items"][0]["av"]["route"], "censored");
        assert_eq!(
            response.output["items"][0]["payload"]["query"]["av"]["number"],
            "SSNI-644"
        );
        assert_eq!(response.output["summary"]["av_items"], 1);
    }

    #[tokio::test]
    async fn bulk_metadata_scrape_reuses_duplicate_av_numbers_without_side_effects() {
        let runtime =
            MetadataScrapeRuntime::<crate::nako_runtime::ReqwestNakoRuntimeTransport>::new(
                "zh-CN",
                vec![Box::new(BulkCandidateProvider)],
                None,
            );

        let response = runtime
            .bulk_scrape(AddonTaskRequest {
                protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
                addon_id: "addon-1".to_owned(),
                task_id: BULK_METADATA_SCRAPE_TASK_ID.to_owned(),
                job_id: "job-1".to_owned(),
                request_id: "request-1".to_owned(),
                attempt: 1,
                retry_of_job_id: None,
                library_id: Some("library-1".to_owned()),
                source_id: Some("source-1".to_owned()),
                payload: serde_json::json!({
                    "batch_size": 3,
                    "items": [
                        {"file_name": "SSNI-00644-CD1.mp4"},
                        {"file_name": "[HD] ssni00644 1080p x264.mkv"},
                        {"file_name": "FC2PPV-1723984.mp4"}
                    ]
                }),
            })
            .await
            .unwrap();

        assert_eq!(response.output["summary"]["scraped_items"], 2);
        assert_eq!(response.output["summary"]["reused_items"], 1);
        assert_eq!(response.output["summary"]["av_items"], 3);
        assert_eq!(response.output["items"][0]["av"]["number"], "SSNI-644");
        assert_eq!(response.output["items"][1]["av"]["number"], "SSNI-644");
        assert_eq!(response.output["items"][1]["av"]["source"], "file_name");
        assert_eq!(response.output["items"][1]["reused_from_index"], 0);
        assert_eq!(
            response.output["items"][1]["payload"]["query"]["title"],
            "SSNI-644"
        );
        assert_eq!(
            response.output["items"][1]["payload"]["query"]["av"]["source"],
            "file_name"
        );
        assert_eq!(response.output["items"][2]["av"]["number"], "FC2-1723984");
        assert!(
            response.output["items"][0]
                .get("reused_from_index")
                .is_none()
        );
    }

    #[tokio::test]
    async fn bulk_metadata_scrape_reuses_resume_state_across_batches() {
        let runtime =
            MetadataScrapeRuntime::<crate::nako_runtime::ReqwestNakoRuntimeTransport>::new(
                "zh-CN",
                vec![Box::new(BulkCandidateProvider)],
                None,
            );

        let first_response = runtime
            .bulk_scrape(AddonTaskRequest {
                protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
                addon_id: "addon-1".to_owned(),
                task_id: BULK_METADATA_SCRAPE_TASK_ID.to_owned(),
                job_id: "job-1".to_owned(),
                request_id: "request-1".to_owned(),
                attempt: 1,
                retry_of_job_id: None,
                library_id: Some("library-1".to_owned()),
                source_id: Some("source-1".to_owned()),
                payload: serde_json::json!({
                    "cursor": 0,
                    "batch_size": 1,
                    "items": [
                        {"file_name": "SSNI-00644-CD1.mp4"},
                        {"file_name": "ssni00644-CD2.mp4"}
                    ]
                }),
            })
            .await
            .unwrap();

        let resume_state = first_response.output["resume_state"].clone();
        let second_response = runtime
            .bulk_scrape(AddonTaskRequest {
                protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
                addon_id: "addon-1".to_owned(),
                task_id: BULK_METADATA_SCRAPE_TASK_ID.to_owned(),
                job_id: "job-1".to_owned(),
                request_id: "request-2".to_owned(),
                attempt: 1,
                retry_of_job_id: None,
                library_id: Some("library-1".to_owned()),
                source_id: Some("source-1".to_owned()),
                payload: serde_json::json!({
                    "cursor": 1,
                    "batch_size": 1,
                    "resume_state": resume_state,
                    "items": [
                        {"file_name": "SSNI-00644-CD1.mp4"},
                        {"file_name": "ssni00644-CD2.mp4"}
                    ]
                }),
            })
            .await
            .unwrap();

        assert_eq!(
            first_response.output["resume_state"]["reusable_items"][0]["av_number"],
            "SSNI-644"
        );
        assert_eq!(second_response.output["summary"]["scraped_items"], 0);
        assert_eq!(second_response.output["summary"]["reused_items"], 1);
        assert_eq!(
            second_response.output["items"][0]["payload"]["query"]["title"],
            "SSNI-644"
        );
        assert_eq!(second_response.output["items"][0]["reused_from_index"], 0);
    }

    #[tokio::test]
    async fn bulk_metadata_scrape_does_not_reuse_items_with_side_effect_requests() {
        let transport = FakeTransport::default();
        transport.push(Ok(NakoRuntimeHttpResponse {
            status: 200,
            body: serde_json::json!({
                "addon_id": "addon-1",
                "token_id": "token-1",
                "permission": "metadata_write",
                "library_id": "library-1",
                "allowed": true
            })
            .to_string(),
        }));
        transport.push(Ok(NakoRuntimeHttpResponse {
            status: 200,
            body: serde_json::json!({
                "side_effect": {
                    "id": "effect-1",
                    "addon_id": "addon-1",
                    "token_id": "token-1",
                    "permission": "metadata_write",
                    "library_id": "library-1",
                    "target": {"kind": "media_source", "id": "source-1"},
                    "idempotency_key": "metadata-demo-1",
                    "validation_status": "accepted",
                    "safe_error_code": null,
                    "apply_status": "applied",
                    "apply_error_code": null,
                    "applied_item_id": "item-1",
                    "applied_source": "addon:addon-1",
                    "apply_report": null
                },
                "idempotent_replay": false
            })
            .to_string(),
        }));
        let runtime = MetadataScrapeRuntime::<FakeTransport>::new(
            "zh-CN",
            vec![Box::new(BulkCandidateProvider)],
            Some(NakoRuntimeClient::<FakeTransport>::with_transport(
                NakoRuntimeClientConfig {
                    base_url: "https://nako.example/".to_owned(),
                    addon_token: "addon-token-secret".to_owned(),
                    timeout_ms: 1_500,
                },
                transport.clone(),
            )),
        );

        let response = runtime
            .bulk_scrape(AddonTaskRequest {
                protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
                addon_id: "addon-1".to_owned(),
                task_id: BULK_METADATA_SCRAPE_TASK_ID.to_owned(),
                job_id: "job-1".to_owned(),
                request_id: "request-1".to_owned(),
                attempt: 1,
                retry_of_job_id: None,
                library_id: Some("library-1".to_owned()),
                source_id: Some("source-1".to_owned()),
                payload: serde_json::json!({
                    "batch_size": 2,
                    "items": [
                        {
                            "file_name": "SSNI-00644-CD1.mp4",
                            "writeback": {
                                "library_id": "library-1",
                                "target": {
                                    "kind": "media_source",
                                    "id": "source-1"
                                },
                                "idempotency_key": "metadata-demo-1"
                            }
                        },
                        {"file_name": "ssni00644-CD2.mp4"}
                    ]
                }),
            })
            .await
            .unwrap();

        assert_eq!(response.output["summary"]["scraped_items"], 2);
        assert_eq!(response.output["summary"]["reused_items"], 0);
        assert!(
            response.output["items"][1]
                .get("reused_from_index")
                .is_none()
        );
        assert_eq!(transport.requests().len(), 2);
    }

    #[tokio::test]
    async fn bulk_metadata_scrape_marks_empty_candidate_items() {
        let runtime =
            MetadataScrapeRuntime::<crate::nako_runtime::ReqwestNakoRuntimeTransport>::new(
                "zh-CN",
                Vec::new(),
                None,
            );

        let response = runtime
            .bulk_scrape(AddonTaskRequest {
                protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
                addon_id: "addon-1".to_owned(),
                task_id: BULK_METADATA_SCRAPE_TASK_ID.to_owned(),
                job_id: "job-1".to_owned(),
                request_id: "request-1".to_owned(),
                attempt: 1,
                retry_of_job_id: None,
                library_id: Some("library-1".to_owned()),
                source_id: Some("source-1".to_owned()),
                payload: serde_json::json!({
                    "batch_size": 1,
                    "items": [
                        {"file_name": "SSNI-00644.mp4"}
                    ]
                }),
            })
            .await
            .unwrap();

        assert_eq!(response.output["summary"]["empty_candidate_items"], 1);
        assert_eq!(response.output["summary"]["failed_items"], 1);
        assert_eq!(
            response.output["summary"]["failure_reasons"]["no_candidates"],
            1
        );
        assert_eq!(
            response.output["items"][0]["safe_failure_reason"],
            "no_candidates"
        );
    }

    #[tokio::test]
    async fn bulk_metadata_scrape_summarizes_provider_execution_failures() {
        let runtime = MetadataScrapeRuntime::<FakeTransport>::new(
            "zh-CN",
            vec![Box::new(FailingBulkProvider)],
            None,
        );

        let response = runtime
            .bulk_scrape(AddonTaskRequest {
                protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
                addon_id: "addon-1".to_owned(),
                task_id: BULK_METADATA_SCRAPE_TASK_ID.to_owned(),
                job_id: "job-1".to_owned(),
                request_id: "request-1".to_owned(),
                attempt: 1,
                retry_of_job_id: None,
                library_id: Some("library-1".to_owned()),
                source_id: Some("source-1".to_owned()),
                payload: serde_json::json!({
                    "batch_size": 1,
                    "items": [
                        {"file_name": "SSNI-00644.mp4"}
                    ]
                }),
            })
            .await
            .unwrap();

        assert_eq!(response.output["summary"]["failed_items"], 1);
        assert_eq!(
            response.output["summary"]["failure_reasons"]["provider_failed"],
            1
        );
        assert_eq!(
            response.output["items"][0]["safe_failure_reason"],
            "provider_failed"
        );
        assert_eq!(
            response.output["summary"]["provider_execution"][0]["provider_id"],
            "fixture"
        );
        assert_eq!(
            response.output["summary"]["provider_execution"][0]["selected_items"],
            1
        );
        assert_eq!(
            response.output["summary"]["provider_execution"][0]["failed_items"],
            1
        );

        let output_text = serde_json::to_string(&response.output).unwrap();
        assert!(!output_text.contains("raw provider failure"));
    }

    #[tokio::test]
    async fn bulk_metadata_scrape_suppresses_retryable_provider_across_resume_state() {
        let calls = Arc::new(Mutex::new(0));
        let runtime = MetadataScrapeRuntime::<FakeTransport>::new(
            "zh-CN",
            vec![Box::new(CountingFailingBulkProvider {
                calls: calls.clone(),
            })],
            None,
        );

        let first_response = runtime
            .bulk_scrape(AddonTaskRequest {
                protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
                addon_id: "addon-1".to_owned(),
                task_id: BULK_METADATA_SCRAPE_TASK_ID.to_owned(),
                job_id: "job-1".to_owned(),
                request_id: "request-1".to_owned(),
                attempt: 1,
                retry_of_job_id: None,
                library_id: Some("library-1".to_owned()),
                source_id: Some("source-1".to_owned()),
                payload: serde_json::json!({
                    "batch_size": 2,
                    "provider_policy": {
                        "suppress_after_failures": 1,
                        "cooldown_items": 2
                    },
                    "items": [
                        {"file_name": "SSNI-00644.mp4"},
                        {"file_name": "IPX-001.mp4"},
                        {"file_name": "MIDE-010.mp4"}
                    ]
                }),
            })
            .await
            .unwrap();

        assert_eq!(*calls.lock().unwrap(), 1);
        assert_eq!(first_response.output["summary"]["failed_items"], 2);
        assert_eq!(first_response.output["summary"]["suppressed_items"], 1);
        assert_eq!(
            first_response.output["summary"]["failure_reasons"]["provider_failed"],
            1
        );
        assert_eq!(
            first_response.output["summary"]["failure_reasons"]["provider_suppressed"],
            1
        );
        assert_eq!(
            first_response.output["summary"]["retry_classes"]["retryable"],
            1
        );
        assert_eq!(
            first_response.output["summary"]["provider_execution"][0]["provider_id"],
            "fixture"
        );
        assert_eq!(
            first_response.output["summary"]["provider_execution"][0]["selected_items"],
            1
        );
        assert_eq!(
            first_response.output["summary"]["provider_execution"][0]["failed_items"],
            1
        );
        assert_eq!(
            first_response.output["summary"]["provider_execution"][0]["retryable_failed_items"],
            1
        );
        assert_eq!(
            first_response.output["summary"]["provider_execution"][0]["suppressed_items"],
            1
        );
        assert_eq!(
            first_response.output["summary"]["provider_execution"][0]["cooldown_remaining_items"],
            1
        );
        assert_eq!(
            first_response.output["items"][1]["safe_failure_reason"],
            "provider_suppressed"
        );
        assert_eq!(
            first_response.output["items"][1]["suppressed_provider_ids"][0],
            "fixture"
        );
        assert_eq!(
            first_response.output["items"][1]["payload"]["provider_execution"]["providers"][0]["status"],
            "suppressed"
        );
        assert_eq!(
            first_response.output["resume_state"]["provider_states"][0]["cooldown_remaining_items"],
            1
        );

        let second_response = runtime
            .bulk_scrape(AddonTaskRequest {
                protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
                addon_id: "addon-1".to_owned(),
                task_id: BULK_METADATA_SCRAPE_TASK_ID.to_owned(),
                job_id: "job-1".to_owned(),
                request_id: "request-2".to_owned(),
                attempt: 1,
                retry_of_job_id: None,
                library_id: Some("library-1".to_owned()),
                source_id: Some("source-1".to_owned()),
                payload: serde_json::json!({
                    "cursor": 2,
                    "batch_size": 1,
                    "provider_policy": {
                        "suppress_after_failures": 1,
                        "cooldown_items": 2
                    },
                    "resume_state": first_response.output["resume_state"].clone(),
                    "items": [
                        {"file_name": "SSNI-00644.mp4"},
                        {"file_name": "IPX-001.mp4"},
                        {"file_name": "MIDE-010.mp4"}
                    ]
                }),
            })
            .await
            .unwrap();

        assert_eq!(*calls.lock().unwrap(), 1);
        assert_eq!(second_response.output["summary"]["suppressed_items"], 1);
        assert_eq!(
            second_response.output["items"][0]["safe_failure_reason"],
            "provider_suppressed"
        );
        assert_eq!(
            second_response.output["items"][0]["suppressed_provider_ids"][0],
            "fixture"
        );
        assert_eq!(
            second_response.output["resume_state"]["provider_states"][0]["cooldown_remaining_items"],
            0
        );
    }

    #[tokio::test]
    async fn bulk_provider_guard_applies_visible_budget_and_bounds_reuse_cache() {
        let javdb_calls = Arc::new(Mutex::new(0));
        let dmm_calls = Arc::new(Mutex::new(0));
        let runtime = MetadataScrapeRuntime::<FakeTransport>::new(
            "zh-CN",
            vec![
                Box::new(BulkCountingCandidateProvider {
                    provider_id: ProviderId::Javdb,
                    candidate_provider: "javdb",
                    calls: javdb_calls.clone(),
                }),
                Box::new(BulkCountingCandidateProvider {
                    provider_id: ProviderId::Dmm,
                    candidate_provider: "dmm",
                    calls: dmm_calls.clone(),
                }),
            ],
            None,
        );

        let response = runtime
            .bulk_scrape(AddonTaskRequest {
                protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
                addon_id: "addon-1".to_owned(),
                task_id: BULK_METADATA_SCRAPE_TASK_ID.to_owned(),
                job_id: "job-1".to_owned(),
                request_id: "request-1".to_owned(),
                attempt: 1,
                retry_of_job_id: None,
                library_id: Some("library-1".to_owned()),
                source_id: Some("source-1".to_owned()),
                payload: serde_json::json!({
                    "batch_size": 2,
                    "provider_policy": {
                        "max_selected_providers_per_item": 1,
                        "max_reusable_items": 1,
                        "max_provider_states": 1
                    },
                    "items": [
                        {"file_name": "SSNI-00644.mp4"},
                        {"file_name": "IPX-001.mp4"}
                    ]
                }),
            })
            .await
            .unwrap();

        assert_eq!(*javdb_calls.lock().unwrap(), 2);
        assert_eq!(*dmm_calls.lock().unwrap(), 0);
        assert_eq!(response.output["provider_policy"]["max_reusable_items"], 1);
        assert_eq!(
            response.output["provider_policy"]["max_selected_providers_per_item"],
            1
        );
        assert_eq!(response.output["summary"]["budget_exhausted_items"], 2);
        assert_eq!(
            response.output["summary"]["provider_execution"][1]["provider_id"],
            "dmm"
        );
        assert_eq!(
            response.output["summary"]["provider_execution"][1]["budget_exhausted_items"],
            2
        );
        assert_eq!(
            response.output["items"][0]["payload"]["provider_execution"]["applied_policy"]["max_selected_providers"],
            1
        );
        assert_eq!(
            response.output["items"][0]["payload"]["provider_execution"]["providers"][1]["status"],
            "budget_exhausted"
        );
        assert_eq!(
            response.output["resume_state"]["reusable_items"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
    }

    #[tokio::test]
    async fn bulk_metadata_scrape_submits_existing_side_effect_paths() {
        let transport = FakeTransport::default();
        transport.push(Ok(NakoRuntimeHttpResponse {
            status: 200,
            body: serde_json::json!({
                "addon_id": "addon-1",
                "token_id": "token-1",
                "permission": "metadata_write",
                "library_id": "library-1",
                "allowed": true
            })
            .to_string(),
        }));
        transport.push(Ok(NakoRuntimeHttpResponse {
            status: 200,
            body: serde_json::json!({
                "side_effect": {
                    "id": "effect-1",
                    "addon_id": "addon-1",
                    "token_id": "token-1",
                    "permission": "metadata_write",
                    "library_id": "library-1",
                    "target": {"kind": "media_source", "id": "source-1"},
                    "idempotency_key": "metadata-demo-1",
                    "validation_status": "accepted",
                    "safe_error_code": null,
                    "apply_status": "applied",
                    "apply_error_code": null,
                    "applied_item_id": "item-1",
                    "applied_source": "addon:addon-1",
                    "apply_report": null
                },
                "idempotent_replay": false
            })
            .to_string(),
        }));
        transport.push(Ok(NakoRuntimeHttpResponse {
            status: 200,
            body: serde_json::json!({
                "addon_id": "addon-1",
                "token_id": "token-1",
                "permission": "artwork_write",
                "library_id": "library-1",
                "allowed": true
            })
            .to_string(),
        }));
        transport.push(Ok(NakoRuntimeHttpResponse {
            status: 200,
            body: serde_json::json!({
                "side_effect": {
                    "id": "effect-2",
                    "addon_id": "addon-1",
                    "token_id": "token-1",
                    "permission": "artwork_write",
                    "library_id": "library-1",
                    "target": {"kind": "media_item", "id": "item-1"},
                    "idempotency_key": "artwork-demo-1",
                    "validation_status": "accepted",
                    "safe_error_code": null,
                    "apply_status": "applied",
                    "apply_error_code": null,
                    "applied_item_id": "item-1",
                    "applied_source": "addon:addon-1",
                    "apply_report": null
                },
                "idempotent_replay": false
            })
            .to_string(),
        }));
        let runtime = MetadataScrapeRuntime::<FakeTransport>::new(
            "en-US",
            vec![Box::new(BulkCandidateProvider)],
            Some(NakoRuntimeClient::<FakeTransport>::with_transport(
                NakoRuntimeClientConfig {
                    base_url: "https://nako.example/".to_owned(),
                    addon_token: "addon-token-secret".to_owned(),
                    timeout_ms: 1_500,
                },
                transport.clone(),
            )),
        );

        let response = runtime
            .bulk_scrape(AddonTaskRequest {
                protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
                addon_id: "addon-1".to_owned(),
                task_id: BULK_METADATA_SCRAPE_TASK_ID.to_owned(),
                job_id: "job-1".to_owned(),
                request_id: "request-1".to_owned(),
                attempt: 1,
                retry_of_job_id: None,
                library_id: Some("library-1".to_owned()),
                source_id: Some("source-1".to_owned()),
                payload: serde_json::json!({
                    "batch_size": 1,
                    "items": [
                        {
                            "title": "The Matrix",
                            "year": 1999,
                            "language": "en-US",
                            "writeback": {
                                "library_id": "library-1",
                                "target": {
                                    "kind": "media_source",
                                    "id": "source-1"
                                },
                                "idempotency_key": "metadata-demo-1"
                            },
                            "artwork_writeback": {
                                "library_id": "library-1",
                                "target": {
                                    "kind": "media_item",
                                    "id": "item-1"
                                },
                                "idempotency_key": "artwork-demo-1",
                                "kind": "poster"
                            }
                        }
                    ]
                }),
            })
            .await
            .unwrap();

        assert_eq!(
            response.output["items"][0]["payload"]["writeback"]["status"],
            "submitted"
        );
        assert_eq!(
            response.output["items"][0]["payload"]["writeback"]["side_effect"]["permission"],
            "metadata_write"
        );
        assert_eq!(
            response.output["items"][0]["payload"]["artwork_writeback"]["status"],
            "submitted"
        );
        assert_eq!(
            response.output["items"][0]["payload"]["artwork_writeback"]["side_effect"]["permission"],
            "artwork_write"
        );
        assert_eq!(transport.requests().len(), 4);
    }

    #[derive(Clone, Default)]
    struct FakeTransport {
        responses: Arc<Mutex<VecDeque<NakoRuntimeResult<NakoRuntimeHttpResponse>>>>,
        requests: Arc<Mutex<Vec<NakoRuntimeHttpRequest>>>,
    }

    impl FakeTransport {
        fn push(&self, response: NakoRuntimeResult<NakoRuntimeHttpResponse>) {
            self.responses.lock().unwrap().push_back(response);
        }

        fn requests(&self) -> Vec<NakoRuntimeHttpRequest> {
            self.requests.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl NakoRuntimeTransport for FakeTransport {
        async fn post(
            &self,
            request: NakoRuntimeHttpRequest,
        ) -> NakoRuntimeResult<NakoRuntimeHttpResponse> {
            self.requests.lock().unwrap().push(request);
            self.responses
                .lock()
                .unwrap()
                .pop_front()
                .unwrap_or_else(|| {
                    Err(NakoRuntimeError::Http {
                        message: "fake response queue was empty".to_owned(),
                    })
                })
        }
    }

    struct BulkCandidateProvider;

    struct BulkCountingCandidateProvider {
        provider_id: ProviderId,
        candidate_provider: &'static str,
        calls: Arc<Mutex<usize>>,
    }

    struct FailingBulkProvider;

    struct CountingFailingBulkProvider {
        calls: Arc<Mutex<usize>>,
    }

    #[async_trait]
    impl MetadataProvider for BulkCandidateProvider {
        fn id(&self) -> crate::config::ProviderId {
            crate::config::ProviderId::Fixture
        }

        async fn suggest(
            &self,
            query: &MetadataQuery,
        ) -> anyhow::Result<Vec<ProviderMetadataCandidate>> {
            Ok(vec![ProviderMetadataCandidate {
                provider: self.id().as_str().to_owned(),
                provider_id: "fixture:bulk".to_owned(),
                patch: nako_addon_protocol::AddonMetadataPatch {
                    title: Some(query.title.clone()),
                    original_title: None,
                    sort_title: None,
                    overview: None,
                    release_date: query.year.map(|year| format!("{year}-01-01")),
                    runtime_minutes: None,
                    tagline: None,
                    genres: None,
                    tags: Some(vec![query.language.clone()]),
                    ..nako_addon_protocol::AddonMetadataPatch::default()
                },
                facts: ProviderCandidateFacts {
                    title: Some(query.title.clone()),
                    alternate_titles: Vec::new(),
                    release_year: query.year,
                    language: Some(query.language.clone()),
                    av: None,
                    community_score_milli: None,
                    community_vote_count: None,
                    external_ids: Vec::new(),
                    provider_outcomes: Vec::new(),
                    provider_note: Some("bulk test candidate".to_owned()),
                },
                artwork_candidates: vec![ProviderArtworkCandidate {
                    provider: self.id().as_str().to_owned(),
                    provider_id: "fixture:bulk".to_owned(),
                    facts: ProviderArtworkCandidateFacts {
                        kind: AddonArtworkKind::Poster,
                        source_url: "https://example.test/poster.jpg".to_owned(),
                        language: Some(query.language.clone()),
                        width: Some(1000),
                        height: Some(1500),
                    },
                }],
            }])
        }
    }

    #[async_trait]
    impl MetadataProvider for BulkCountingCandidateProvider {
        fn id(&self) -> ProviderId {
            self.provider_id
        }

        async fn suggest(
            &self,
            query: &MetadataQuery,
        ) -> anyhow::Result<Vec<ProviderMetadataCandidate>> {
            *self.calls.lock().unwrap() += 1;
            Ok(vec![ProviderMetadataCandidate {
                provider: self.candidate_provider.to_owned(),
                provider_id: format!("{}:bulk:{}", self.candidate_provider, query.title),
                patch: nako_addon_protocol::AddonMetadataPatch {
                    title: Some(query.title.clone()),
                    ..nako_addon_protocol::AddonMetadataPatch::default()
                },
                facts: ProviderCandidateFacts {
                    title: Some(query.title.clone()),
                    alternate_titles: Vec::new(),
                    release_year: query.year,
                    language: Some(query.language.clone()),
                    av: None,
                    community_score_milli: None,
                    community_vote_count: None,
                    external_ids: Vec::new(),
                    provider_outcomes: Vec::new(),
                    provider_note: Some("bulk counting test candidate".to_owned()),
                },
                artwork_candidates: Vec::new(),
            }])
        }
    }

    #[async_trait]
    impl MetadataProvider for FailingBulkProvider {
        fn id(&self) -> crate::config::ProviderId {
            crate::config::ProviderId::Fixture
        }

        async fn suggest(
            &self,
            _query: &MetadataQuery,
        ) -> anyhow::Result<Vec<ProviderMetadataCandidate>> {
            anyhow::bail!("raw provider failure with https://private.example/secret")
        }
    }

    #[async_trait]
    impl MetadataProvider for CountingFailingBulkProvider {
        fn id(&self) -> crate::config::ProviderId {
            crate::config::ProviderId::Fixture
        }

        async fn suggest(
            &self,
            _query: &MetadataQuery,
        ) -> anyhow::Result<Vec<ProviderMetadataCandidate>> {
            *self.calls.lock().unwrap() += 1;
            anyhow::bail!("provider timeout while fetching fixture")
        }
    }
}
