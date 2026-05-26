use std::collections::{BTreeMap, HashMap};

use nako_addon_protocol::{
    ADDON_PROTOCOL_VERSION, AddonResource, AddonResourceRequest, AddonTaskRequest,
    AddonTaskResponse,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::engine::{MetadataScrapeRuntime, av};

pub const BULK_METADATA_SCRAPE_TASK_ID: &str = "bulk-metadata-scrape";
pub const BULK_METADATA_SCRAPE_TASK_NAME: &str = "Bulk metadata scrape";
pub const BULK_METADATA_SCRAPE_TASK_PATH: &str = "/tasks/bulk-metadata-scrape";
pub const BULK_METADATA_SCRAPE_TASK_DESCRIPTION: &str =
    "Runs metadata suggestions for a bounded batch of items";
pub const BULK_METADATA_SCRAPE_OUTPUT_SCHEMA: &str =
    "nako.official.metadata-scraper.bulk-metadata-scrape.result.v1";

const DEFAULT_BATCH_SIZE: usize = 4;
const MAX_BATCH_SIZE: usize = 12;

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
    resume_state: BulkMetadataScrapeResumeState,
}

#[derive(Clone, Debug, Serialize, Eq, PartialEq)]
struct BulkMetadataScrapeTaskOutput {
    schema: &'static str,
    library_id: Option<String>,
    source_id: Option<String>,
    cursor: usize,
    batch_size: usize,
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
    failure_reasons: BTreeMap<String, usize>,
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
    safe_failure_reason: Option<&'static str>,
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
    payload: Value,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
struct BulkMetadataScrapeResumeState {
    #[serde(default)]
    reusable_items: Vec<BulkMetadataScrapeResumeItem>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct BulkMetadataScrapeResumeItem {
    av_number: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    language_hint: Option<String>,
    index: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    av: Option<Value>,
    payload: Value,
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
        let plan = BulkMetadataScrapeBatchPlan::new(
            input.items.len(),
            input.cursor.unwrap_or(0),
            input.batch_size.unwrap_or(DEFAULT_BATCH_SIZE),
        );

        let mut items = Vec::with_capacity(plan.item_indexes.len());
        let mut summary = BulkMetadataScrapeTaskSummary::default();
        let mut reuse_cache = input.resume_state.into_reuse_cache();
        for index in &plan.item_indexes {
            let item_request_id = format!("{}:item-{}", request_id, index);
            let payload = input.items[*index].clone();
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
                let safe_failure_reason = safe_failure_reason_for_payload(&item_payload);
                summary.record_failure(safe_failure_reason);
                summary.reused_items += 1;
                items.push(BulkMetadataScrapeTaskItemOutput {
                    index: *index,
                    request_id: item_request_id,
                    av: item_av,
                    reused_from_index: Some(entry.index),
                    safe_failure_reason,
                    payload: item_payload,
                });
                continue;
            }

            let item_response = self
                .scrape(AddonResourceRequest {
                    protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
                    addon_id: addon_id.clone(),
                    resource: AddonResource::Metadata,
                    request_id: item_request_id.clone(),
                    payload,
                })
                .await;
            let av = item_response.payload["query"].get("av").cloned();
            let item_av = planned_av.or(av);
            let safe_failure_reason = safe_failure_reason_for_payload(&item_response.payload);
            summary.record_failure(safe_failure_reason);
            summary.record_provider_execution(&item_response.payload);
            summary.scraped_items += 1;
            if let Some(cache_key) = reuse_key {
                reuse_cache.insert(
                    cache_key,
                    BulkMetadataScrapeReuseEntry {
                        index: *index,
                        av: item_av.clone(),
                        payload: item_response.payload.clone(),
                    },
                );
            }
            items.push(BulkMetadataScrapeTaskItemOutput {
                index: *index,
                request_id: item_request_id,
                av: item_av,
                reused_from_index: None,
                safe_failure_reason,
                payload: item_response.payload,
            });
        }

        let output = BulkMetadataScrapeTaskOutput {
            schema: BULK_METADATA_SCRAPE_OUTPUT_SCHEMA,
            library_id,
            source_id,
            cursor: plan.cursor,
            batch_size: plan.batch_size,
            total_items: plan.total_items,
            processed_items: items.len(),
            remaining_items: plan.remaining_items(),
            next_cursor: plan.next_cursor,
            summary,
            resume_state: BulkMetadataScrapeResumeState::from_reuse_cache(&reuse_cache),
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
    fn record_failure(&mut self, safe_failure_reason: Option<&'static str>) {
        let Some(reason) = safe_failure_reason else {
            return;
        };

        self.empty_candidate_items += 1;
        self.failed_items += 1;
        *self.failure_reasons.entry(reason.to_owned()).or_default() += 1;
    }

    fn record_provider_execution(&mut self, payload: &Value) {
        let Some(reports) = payload
            .get("provider_execution")
            .and_then(|value| value.get("providers"))
            .and_then(Value::as_array)
        else {
            return;
        };

        for report in reports {
            let Some(provider_id) = report.get("provider_id").and_then(Value::as_str) else {
                continue;
            };
            let Some(status) = report.get("status").and_then(Value::as_str) else {
                continue;
            };
            let candidate_count = report
                .get("candidate_count")
                .and_then(Value::as_u64)
                .unwrap_or(0) as usize;
            let provider = self.provider_summary_mut(provider_id);

            match status {
                "skipped_by_av_route" => provider.skipped_by_route_items += 1,
                "returned_candidates" => {
                    provider.selected_items += 1;
                    provider.returned_items += 1;
                    provider.returned_candidate_count += candidate_count;
                }
                "empty" => {
                    provider.selected_items += 1;
                    provider.empty_items += 1;
                }
                "failed" => {
                    provider.selected_items += 1;
                    provider.failed_items += 1;
                }
                _ => {}
            }
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
    fn into_reuse_cache(self) -> HashMap<BulkMetadataScrapeReuseKey, BulkMetadataScrapeReuseEntry> {
        self.reusable_items
            .into_iter()
            .filter_map(|item| {
                let key = item.reuse_key()?;
                let entry = BulkMetadataScrapeReuseEntry {
                    index: item.index,
                    av: item.av,
                    payload: item.payload,
                };
                Some((key, entry))
            })
            .collect()
    }

    fn from_reuse_cache(
        reuse_cache: &HashMap<BulkMetadataScrapeReuseKey, BulkMetadataScrapeReuseEntry>,
    ) -> Self {
        let mut reusable_items = reuse_cache
            .iter()
            .map(|(key, entry)| BulkMetadataScrapeResumeItem {
                av_number: key.av_number.clone(),
                language_hint: key.language_hint.clone(),
                index: entry.index,
                av: entry.av.clone(),
                payload: entry.payload.clone(),
            })
            .collect::<Vec<_>>();
        reusable_items.sort_by(|left, right| {
            left.av_number
                .cmp(&right.av_number)
                .then_with(|| left.language_hint.cmp(&right.language_hint))
                .then_with(|| left.index.cmp(&right.index))
        });
        Self { reusable_items }
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

fn safe_failure_reason_for_payload(payload: &Value) -> Option<&'static str> {
    let has_no_candidates = payload
        .get("candidates")
        .and_then(Value::as_array)
        .is_some_and(Vec::is_empty);
    if !has_no_candidates {
        return None;
    }

    let provider_execution = payload.get("provider_execution");
    if provider_execution
        .and_then(|value| value.get("failed_provider_ids"))
        .and_then(Value::as_array)
        .is_some_and(|values| !values.is_empty())
    {
        return Some("provider_failed");
    }
    if provider_execution
        .and_then(|value| value.get("selected_provider_ids"))
        .and_then(Value::as_array)
        .is_some_and(Vec::is_empty)
        && provider_execution
            .and_then(|value| value.get("skipped_provider_ids"))
            .and_then(Value::as_array)
            .is_some_and(|values| !values.is_empty())
    {
        return Some("provider_skipped_by_route");
    }

    Some("no_candidates")
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

    struct FailingBulkProvider;

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
                },
                facts: ProviderCandidateFacts {
                    title: Some(query.title.clone()),
                    alternate_titles: Vec::new(),
                    release_year: query.year,
                    language: Some(query.language.clone()),
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
}
