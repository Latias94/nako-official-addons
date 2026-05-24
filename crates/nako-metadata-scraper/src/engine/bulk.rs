use nako_addon_protocol::{
    ADDON_PROTOCOL_VERSION, AddonResource, AddonResourceRequest, AddonTaskRequest,
    AddonTaskResponse,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::engine::MetadataScrapeRuntime;

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
    items: Vec<BulkMetadataScrapeTaskItemOutput>,
}

#[derive(Clone, Debug, Serialize, Eq, PartialEq)]
struct BulkMetadataScrapeTaskItemOutput {
    index: usize,
    request_id: String,
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
        for index in &plan.item_indexes {
            let item_request_id = format!("{}:item-{}", request_id, index);
            let payload = input.items[*index].clone();
            let item_response = self
                .scrape(AddonResourceRequest {
                    protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
                    addon_id: addon_id.clone(),
                    resource: AddonResource::Metadata,
                    request_id: item_request_id.clone(),
                    payload,
                })
                .await;
            items.push(BulkMetadataScrapeTaskItemOutput {
                index: *index,
                request_id: item_request_id,
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
}
