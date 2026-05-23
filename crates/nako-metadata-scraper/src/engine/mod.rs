use std::{collections::HashSet, sync::Arc};

use nako_addon_protocol::{
    ADDON_PROTOCOL_VERSION, AddonArtifact, AddonResourceRequest, AddonResourceResponse,
};
use serde::Deserialize;

use crate::nako_runtime::{
    NakoAccessCheckRequest, NakoPermission, NakoRuntimeClient, NakoRuntimeTransport,
    NakoSideEffectSummary, NakoSideEffectTarget, SubmitNakoArtworkWriteRequest,
    SubmitNakoMetadataWriteRequest,
};
use crate::providers::MetadataProvider;

pub mod artwork;
pub mod ranking;

const MAX_CANDIDATES_PER_QUERY: usize = 12;

pub use artwork::{
    ArtworkCandidate, ArtworkWritebackResult, ArtworkWritebackStatus, ProviderArtworkCandidate,
    ProviderArtworkCandidateFacts,
};
pub use ranking::{
    CandidateEvidence, MetadataCandidate, ProviderCandidateFacts, ProviderExternalId,
    ProviderMetadataCandidate,
};

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MetadataWritebackRequest {
    pub library_id: String,
    pub target: NakoSideEffectTarget,
    pub idempotency_key: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum MetadataWritebackInput {
    Absent,
    Invalid { safe_error_code: &'static str },
    Requested(MetadataWritebackRequest),
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct MetadataWritebackResult {
    pub status: MetadataWritebackStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safe_error_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub side_effect: Option<NakoSideEffectSummary>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MetadataWritebackStatus {
    Submitted,
    Skipped,
    Failed,
}

#[derive(Clone)]
pub struct MetadataScrapeRuntime<T = crate::nako_runtime::ReqwestNakoRuntimeTransport>
where
    T: NakoRuntimeTransport,
{
    default_language: String,
    providers: Arc<Vec<Box<dyn MetadataProvider>>>,
    nako_runtime: Option<NakoRuntimeClient<T>>,
}

impl<T> MetadataScrapeRuntime<T>
where
    T: NakoRuntimeTransport,
{
    #[must_use]
    pub fn new(
        default_language: impl Into<String>,
        providers: Vec<Box<dyn MetadataProvider>>,
        nako_runtime: Option<NakoRuntimeClient<T>>,
    ) -> Self {
        Self {
            default_language: default_language.into(),
            providers: Arc::new(providers),
            nako_runtime,
        }
    }

    pub async fn scrape(&self, request: AddonResourceRequest) -> AddonResourceResponse {
        let query = MetadataQuery::from_payload(&request.payload, &self.default_language);
        let writeback_request = MetadataWritebackInput::from_payload(&request.payload);
        let artwork_writeback_request =
            artwork::ArtworkWritebackInput::from_payload(&request.payload);
        let candidates = self.suggest_candidates(&query).await;
        let selected_candidate = candidates.first().cloned();
        let writeback_result = self
            .maybe_submit_writeback(
                &request.request_id,
                &query,
                selected_candidate.as_ref(),
                writeback_request,
            )
            .await;
        let artwork_writeback_result = self
            .maybe_submit_artwork_writeback(
                &request.request_id,
                &query,
                &candidates,
                artwork_writeback_request,
            )
            .await;
        let mut payload = serde_json::json!({
            "query": {
                "title": query.title,
                "year": query.year,
                "language": query.language
            },
            "candidates": candidates
        });
        if let Some(writeback_result) = writeback_result {
            payload["writeback"] = serde_json::to_value(writeback_result)
                .expect("writeback result is always serializable");
        }
        if let Some(artwork_writeback_result) = artwork_writeback_result {
            payload["artwork_writeback"] = serde_json::to_value(artwork_writeback_result)
                .expect("artwork writeback result is always serializable");
        }

        AddonResourceResponse {
            protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
            addon_id: request.addon_id,
            resource: request.resource,
            request_id: request.request_id,
            payload: payload.clone(),
            artifacts: vec![AddonArtifact {
                kind: "metadata_suggestion".to_owned(),
                payload,
            }],
        }
    }

    async fn maybe_submit_writeback(
        &self,
        request_id: &str,
        query: &MetadataQuery,
        selected_candidate: Option<&MetadataCandidate>,
        writeback_request: MetadataWritebackInput,
    ) -> Option<MetadataWritebackResult> {
        let writeback_request = match writeback_request {
            MetadataWritebackInput::Absent => return None,
            MetadataWritebackInput::Invalid { safe_error_code } => {
                return Some(MetadataWritebackResult {
                    status: MetadataWritebackStatus::Skipped,
                    safe_error_code: Some(safe_error_code.to_owned()),
                    side_effect: None,
                });
            }
            MetadataWritebackInput::Requested(writeback_request) => writeback_request,
        };
        let Some(selected_candidate) = selected_candidate else {
            return Some(MetadataWritebackResult {
                status: MetadataWritebackStatus::Skipped,
                safe_error_code: Some("no_candidates".to_owned()),
                side_effect: None,
            });
        };
        let Some(runtime) = self.nako_runtime.as_ref() else {
            return Some(MetadataWritebackResult {
                status: MetadataWritebackStatus::Skipped,
                safe_error_code: Some("nako_runtime_disabled".to_owned()),
                side_effect: None,
            });
        };

        let access = runtime
            .access_check(NakoAccessCheckRequest {
                permission: NakoPermission::MetadataWrite,
                library_id: Some(writeback_request.library_id.clone()),
            })
            .await;
        let Ok(access) = access else {
            tracing::warn!(request_id = %request_id, "metadata writeback access check failed");
            return Some(MetadataWritebackResult {
                status: MetadataWritebackStatus::Skipped,
                safe_error_code: Some("access_check_failed".to_owned()),
                side_effect: None,
            });
        };
        if !access.allowed {
            return Some(MetadataWritebackResult {
                status: MetadataWritebackStatus::Skipped,
                safe_error_code: Some("access_denied".to_owned()),
                side_effect: None,
            });
        }

        let provenance = serde_json::json!({
            "origin": "nako-metadata-scraper",
            "request_id": request_id,
            "query": {
                "title": query.title,
                "year": query.year,
                "language": query.language
            },
            "selected_candidate": {
                "provider": selected_candidate.provider,
                "provider_id": selected_candidate.provider_id,
                "confidence_milli": selected_candidate.confidence_milli
            }
        });
        let writeback = runtime
            .submit_metadata_write(SubmitNakoMetadataWriteRequest {
                library_id: writeback_request.library_id.clone(),
                target: writeback_request.target.clone(),
                idempotency_key: writeback_request.idempotency_key.clone(),
                provenance,
                patch: selected_candidate.patch.clone(),
            })
            .await;

        match writeback {
            Ok(response) => Some(MetadataWritebackResult {
                status: MetadataWritebackStatus::Submitted,
                safe_error_code: None,
                side_effect: Some(response.side_effect),
            }),
            Err(error) => {
                tracing::warn!(
                    request_id = %request_id,
                    safe_error_code = error.safe_code(),
                    "metadata writeback submission failed"
                );
                Some(MetadataWritebackResult {
                    status: MetadataWritebackStatus::Failed,
                    safe_error_code: Some(error.safe_code().to_owned()),
                    side_effect: None,
                })
            }
        }
    }

    async fn maybe_submit_artwork_writeback(
        &self,
        request_id: &str,
        query: &MetadataQuery,
        candidates: &[MetadataCandidate],
        writeback_request: artwork::ArtworkWritebackInput,
    ) -> Option<artwork::ArtworkWritebackResult> {
        let writeback_request = match writeback_request {
            artwork::ArtworkWritebackInput::Absent => return None,
            artwork::ArtworkWritebackInput::Invalid { safe_error_code } => {
                return Some(artwork::artwork_write_summary(
                    artwork::ArtworkWritebackStatus::Skipped,
                    Some(safe_error_code),
                    None,
                ));
            }
            artwork::ArtworkWritebackInput::Requested(writeback_request) => writeback_request,
        };
        if !artwork::valid_artwork_target(&writeback_request.target) {
            return Some(artwork::artwork_write_summary(
                artwork::ArtworkWritebackStatus::Skipped,
                Some("invalid_artwork_target_kind"),
                None,
            ));
        }
        let Some(selected_candidate) =
            artwork::select_artwork_candidate(candidates, writeback_request.kind)
        else {
            return Some(artwork::artwork_write_summary(
                artwork::ArtworkWritebackStatus::Skipped,
                Some("no_artwork_candidates"),
                None,
            ));
        };
        let Some(runtime) = self.nako_runtime.as_ref() else {
            return Some(artwork::artwork_write_summary(
                artwork::ArtworkWritebackStatus::Skipped,
                Some("nako_runtime_disabled"),
                None,
            ));
        };

        let access = runtime
            .access_check(NakoAccessCheckRequest {
                permission: NakoPermission::ArtworkWrite,
                library_id: Some(writeback_request.library_id.clone()),
            })
            .await;
        let Ok(access) = access else {
            tracing::warn!(request_id = %request_id, "artwork writeback access check failed");
            return Some(artwork::artwork_write_summary(
                artwork::ArtworkWritebackStatus::Skipped,
                Some("access_check_failed"),
                None,
            ));
        };
        if !access.allowed {
            return Some(artwork::artwork_write_summary(
                artwork::ArtworkWritebackStatus::Skipped,
                Some("access_denied"),
                None,
            ));
        }

        let provenance = artwork::artwork_write_provenance(
            "nako-metadata-scraper",
            request_id,
            &query.title,
            query.year,
            &query.language,
            selected_candidate,
        );
        let writeback = runtime
            .submit_artwork_write(SubmitNakoArtworkWriteRequest {
                library_id: writeback_request.library_id.clone(),
                target: writeback_request.target.clone(),
                idempotency_key: writeback_request.idempotency_key.clone(),
                provenance,
                artwork: selected_candidate.artwork.clone(),
            })
            .await;

        match writeback {
            Ok(response) => Some(artwork::artwork_write_summary(
                artwork::ArtworkWritebackStatus::Submitted,
                None,
                Some(response.side_effect),
            )),
            Err(error) => {
                tracing::warn!(
                    request_id = %request_id,
                    safe_error_code = error.safe_code(),
                    "artwork writeback submission failed"
                );
                Some(artwork::artwork_write_summary(
                    artwork::ArtworkWritebackStatus::Failed,
                    Some(error.safe_code()),
                    None,
                ))
            }
        }
    }

    async fn suggest_candidates(&self, query: &MetadataQuery) -> Vec<MetadataCandidate> {
        let mut candidates = Vec::new();

        for provider in self.providers.iter() {
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
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct MetadataQuery {
    pub title: String,
    pub year: Option<i32>,
    pub language: String,
    pub external_ids: Vec<QueryExternalId>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct QueryExternalId {
    pub provider: String,
    pub value: String,
}

impl MetadataQuery {
    #[must_use]
    pub fn from_payload(payload: &serde_json::Value, default_language: &str) -> Self {
        let raw_title = payload
            .get("title")
            .and_then(serde_json::Value::as_str)
            .or_else(|| payload.get("name").and_then(serde_json::Value::as_str))
            .unwrap_or("Unknown Title")
            .trim();
        let title = normalize_title(raw_title);
        let year = payload
            .get("year")
            .and_then(serde_json::Value::as_i64)
            .and_then(|value| i32::try_from(value).ok());
        let language = payload
            .get("language")
            .and_then(serde_json::Value::as_str)
            .unwrap_or(default_language)
            .to_owned();
        let external_ids = external_ids_from_payload(payload);

        Self {
            title: if title.is_empty() {
                "Unknown Title".to_owned()
            } else {
                title
            },
            year,
            language,
            external_ids,
        }
    }
}

impl MetadataWritebackInput {
    #[must_use]
    fn from_payload(payload: &serde_json::Value) -> Self {
        let Some(writeback) = payload.get("writeback") else {
            return Self::Absent;
        };

        match serde_json::from_value::<MetadataWritebackRequest>(writeback.clone()) {
            Ok(writeback_request) => Self::Requested(writeback_request),
            Err(_) => Self::Invalid {
                safe_error_code: "invalid_writeback_request",
            },
        }
    }
}

fn normalize_title(title: &str) -> String {
    title.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn external_ids_from_payload(payload: &serde_json::Value) -> Vec<QueryExternalId> {
    if let Some(values) = payload
        .get("external_ids")
        .and_then(serde_json::Value::as_object)
    {
        return values
            .iter()
            .filter_map(|(provider, value)| {
                value.as_str().map(|value| QueryExternalId {
                    provider: provider.clone(),
                    value: value.to_owned(),
                })
            })
            .collect();
    }

    payload
        .get("external_ids")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|value| {
            Some(QueryExternalId {
                provider: value.get("provider")?.as_str()?.to_owned(),
                value: value.get("value")?.as_str()?.to_owned(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::{
        collections::VecDeque,
        sync::{Arc, Mutex},
    };

    use async_trait::async_trait;
    use nako_addon_protocol::{AddonArtworkKind, AddonMetadataPatch, AddonResource};

    use super::*;
    use crate::{
        config::ProviderId,
        nako_runtime::{
            NakoRuntimeClient, NakoRuntimeClientConfig, NakoRuntimeError, NakoRuntimeHttpRequest,
            NakoRuntimeHttpResponse, NakoRuntimeResult, NakoRuntimeTransport,
        },
        providers::MetadataProvider,
    };

    struct CandidateProvider {
        provider_id: &'static str,
        title: &'static str,
        year: Option<i32>,
    }

    #[async_trait]
    impl MetadataProvider for CandidateProvider {
        fn id(&self) -> ProviderId {
            ProviderId::Fixture
        }

        async fn suggest(
            &self,
            query: &MetadataQuery,
        ) -> anyhow::Result<Vec<ProviderMetadataCandidate>> {
            Ok(vec![ProviderMetadataCandidate {
                provider: self.id().as_str().to_owned(),
                provider_id: self.provider_id.to_owned(),
                patch: AddonMetadataPatch {
                    title: Some(query.title.clone()),
                    original_title: None,
                    sort_title: None,
                    overview: None,
                    release_date: self.year.map(|year| format!("{year}-01-01")),
                    runtime_minutes: None,
                    tagline: None,
                    genres: None,
                    tags: Some(vec![query.language.clone()]),
                },
                facts: ProviderCandidateFacts {
                    title: Some(self.title.to_owned()),
                    alternate_titles: Vec::new(),
                    release_year: self.year,
                    language: Some(query.language.clone()),
                    community_score_milli: None,
                    community_vote_count: None,
                    external_ids: Vec::new(),
                    provider_note: Some("test candidate".to_owned()),
                },
                artwork_candidates: vec![ProviderArtworkCandidate {
                    provider: self.id().as_str().to_owned(),
                    provider_id: self.provider_id.to_owned(),
                    facts: ProviderArtworkCandidateFacts {
                        kind: AddonArtworkKind::Poster,
                        source_url: "https://example.test/poster.jpg".to_owned(),
                        language: None,
                        width: Some(1000),
                        height: Some(1500),
                    },
                }],
            }])
        }
    }

    struct FailingProvider;

    struct DuplicateProvider {
        candidate_count: usize,
    }

    #[async_trait]
    impl MetadataProvider for FailingProvider {
        fn id(&self) -> ProviderId {
            ProviderId::Fixture
        }

        async fn suggest(
            &self,
            _query: &MetadataQuery,
        ) -> anyhow::Result<Vec<ProviderMetadataCandidate>> {
            anyhow::bail!("synthetic provider failure")
        }
    }

    #[async_trait]
    impl MetadataProvider for DuplicateProvider {
        fn id(&self) -> ProviderId {
            ProviderId::Fixture
        }

        async fn suggest(
            &self,
            _query: &MetadataQuery,
        ) -> anyhow::Result<Vec<ProviderMetadataCandidate>> {
            let mut candidates = Vec::new();
            for index in 0..self.candidate_count {
                let provider_id = if index < 2 {
                    "fixture:duplicate".to_owned()
                } else {
                    format!("fixture:{index}")
                };
                candidates.push(ProviderMetadataCandidate {
                    provider: self.id().as_str().to_owned(),
                    provider_id,
                    patch: AddonMetadataPatch::default(),
                    facts: ProviderCandidateFacts {
                        title: Some(format!("Candidate {index}")),
                        alternate_titles: Vec::new(),
                        release_year: Some(2000 + index as i32),
                        language: Some("en-US".to_owned()),
                        community_score_milli: None,
                        community_vote_count: None,
                        external_ids: Vec::new(),
                        provider_note: None,
                    },
                    artwork_candidates: Vec::new(),
                });
            }

            Ok(candidates)
        }
    }

    #[tokio::test]
    async fn runtime_normalizes_request_and_shapes_metadata_response() {
        let runtime = MetadataScrapeRuntime::<FakeTransport>::new(
            "zh-CN",
            vec![Box::new(CandidateProvider {
                provider_id: "fixture:matrix",
                title: "The Matrix",
                year: Some(1999),
            })],
            None,
        );

        let response = runtime
            .scrape(AddonResourceRequest {
                protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
                addon_id: "addon-1".to_owned(),
                resource: AddonResource::Metadata,
                request_id: "request-1".to_owned(),
                payload: serde_json::json!({"name": "  The   Matrix  ", "year": 1999}),
            })
            .await;

        assert_eq!(response.request_id, "request-1");
        assert_eq!(response.artifacts[0].kind, "metadata_suggestion");
        assert_eq!(response.payload["query"]["title"], "The Matrix");
        assert_eq!(response.payload["query"]["language"], "zh-CN");
        assert!(response.payload.get("writeback").is_none());
        assert_eq!(
            response.payload["candidates"][0]["patch"]["release_date"],
            "1999-01-01"
        );
        assert_eq!(response.payload["candidates"][0]["confidence_milli"], 830);
        assert_eq!(
            response.payload["candidates"][0]["artwork_candidates"][0]["confidence_milli"],
            830
        );
        assert_eq!(
            response.payload["candidates"][0]["artwork_candidates"][0]["artwork"]["kind"],
            "poster"
        );
        assert_eq!(response.artifacts[0].payload, response.payload);
    }

    #[tokio::test]
    async fn ranking_evidence_runtime_sorts_candidates_and_skips_failed_providers() {
        let runtime = MetadataScrapeRuntime::<FakeTransport>::new(
            "en-US",
            vec![
                Box::new(CandidateProvider {
                    provider_id: "fixture:low",
                    title: "Other Movie",
                    year: Some(2001),
                }),
                Box::new(FailingProvider),
                Box::new(CandidateProvider {
                    provider_id: "fixture:high",
                    title: "Movie",
                    year: None,
                }),
            ],
            None,
        );

        let response = runtime
            .scrape(AddonResourceRequest {
                protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
                addon_id: "addon-1".to_owned(),
                resource: AddonResource::Metadata,
                request_id: "request-1".to_owned(),
                payload: serde_json::json!({"title": "Movie"}),
            })
            .await;

        let candidates = response.payload["candidates"].as_array().unwrap();
        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[0]["provider_id"], "fixture:high");
        assert_eq!(candidates[1]["provider_id"], "fixture:low");
    }

    #[tokio::test]
    async fn ranking_evidence_runtime_deduplicates_and_caps_candidates() {
        let runtime = MetadataScrapeRuntime::<FakeTransport>::new(
            "en-US",
            vec![Box::new(DuplicateProvider {
                candidate_count: 14,
            })],
            None,
        );

        let response = runtime
            .scrape(AddonResourceRequest {
                protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
                addon_id: "addon-1".to_owned(),
                resource: AddonResource::Metadata,
                request_id: "request-1".to_owned(),
                payload: serde_json::json!({"title": "Movie"}),
            })
            .await;

        let candidates = response.payload["candidates"].as_array().unwrap();
        assert_eq!(candidates.len(), MAX_CANDIDATES_PER_QUERY);

        let mut provider_ids = HashSet::new();
        for candidate in candidates {
            let provider_id = candidate["provider_id"].as_str().unwrap().to_owned();
            assert!(provider_ids.insert(provider_id));
        }
    }

    #[test]
    fn ranking_evidence_metadata_query_parses_external_ids() {
        let query = MetadataQuery::from_payload(
            &serde_json::json!({
                "title": "Movie",
                "external_ids": {
                    "imdb": "tt0133093",
                    "tmdb": "603"
                }
            }),
            "en-US",
        );

        assert_eq!(
            query.external_ids,
            vec![
                QueryExternalId {
                    provider: "imdb".to_owned(),
                    value: "tt0133093".to_owned(),
                },
                QueryExternalId {
                    provider: "tmdb".to_owned(),
                    value: "603".to_owned(),
                }
            ]
        );
    }

    #[tokio::test]
    async fn metadata_side_effect_request_submits_selected_patch_when_enabled() {
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
            "en-US",
            vec![Box::new(CandidateProvider {
                provider_id: "fixture:matrix",
                title: "The Matrix",
                year: Some(1999),
            })],
            Some(NakoRuntimeClient::<FakeTransport>::with_transport(
                NakoRuntimeClientConfig {
                    base_url: "https://nako.example/".to_owned(),
                    addon_token: "addon-token-secret".to_owned(),
                    timeout_ms: 1500,
                },
                transport.clone(),
            )),
        );

        let response = runtime
            .scrape(AddonResourceRequest {
                protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
                addon_id: "addon-1".to_owned(),
                resource: AddonResource::Metadata,
                request_id: "request-1".to_owned(),
                payload: serde_json::json!({
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
                    }
                }),
            })
            .await;

        assert_eq!(response.payload["writeback"]["status"], "submitted");
        assert_eq!(
            response.payload["writeback"]["side_effect"]["permission"],
            "metadata_write"
        );
        assert_eq!(
            response.payload["writeback"]["side_effect"]["applied_source"],
            "addon:addon-1"
        );

        let requests = transport.requests();
        assert_eq!(requests.len(), 2);
        assert_eq!(
            requests[0].url,
            "https://nako.example/addon/v1/access-check"
        );
        let access_check_body: serde_json::Value = serde_json::from_str(&requests[0].body).unwrap();
        assert_eq!(
            access_check_body,
            serde_json::json!({
                "permission": "metadata_write",
                "library_id": "library-1"
            })
        );
        assert_eq!(
            requests[1].url,
            "https://nako.example/addon/v1/side-effects"
        );
        let body: serde_json::Value = serde_json::from_str(&requests[1].body).unwrap();
        assert_eq!(body["permission"], "metadata_write");
        assert_eq!(body["library_id"], "library-1");
        assert_eq!(body["target"]["kind"], "media_source");
        assert_eq!(body["target"]["id"], "source-1");
        assert_eq!(body["idempotency_key"], "metadata-demo-1");
        assert_eq!(body["payload"]["title"], "The Matrix");
        assert_eq!(body["payload"]["release_date"], "1999-01-01");
        assert_eq!(body["payload"]["tags"][0], "en-US");
        assert_eq!(body["provenance"]["origin"], "nako-metadata-scraper");
        assert_eq!(body["provenance"]["request_id"], "request-1");
        assert_eq!(body["provenance"]["query"]["title"], "The Matrix");
    }

    #[tokio::test]
    async fn metadata_side_effect_request_skips_when_runtime_is_disabled() {
        let runtime =
            MetadataScrapeRuntime::<crate::nako_runtime::ReqwestNakoRuntimeTransport>::new(
                "en-US",
                vec![Box::new(CandidateProvider {
                    provider_id: "fixture:matrix",
                    title: "The Matrix",
                    year: Some(1999),
                })],
                None,
            );

        let response = runtime
            .scrape(AddonResourceRequest {
                protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
                addon_id: "addon-1".to_owned(),
                resource: AddonResource::Metadata,
                request_id: "request-1".to_owned(),
                payload: serde_json::json!({
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
                    }
                }),
            })
            .await;

        assert_eq!(response.payload["writeback"]["status"], "skipped");
        assert_eq!(
            response.payload["writeback"]["safe_error_code"],
            "nako_runtime_disabled"
        );
    }

    #[tokio::test]
    async fn artwork_side_effect_request_submits_selected_candidate_when_enabled() {
        let transport = FakeTransport::default();
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
            vec![Box::new(CandidateProvider {
                provider_id: "fixture:matrix",
                title: "The Matrix",
                year: Some(1999),
            })],
            Some(NakoRuntimeClient::<FakeTransport>::with_transport(
                NakoRuntimeClientConfig {
                    base_url: "https://nako.example/".to_owned(),
                    addon_token: "addon-token-secret".to_owned(),
                    timeout_ms: 1500,
                },
                transport.clone(),
            )),
        );

        let response = runtime
            .scrape(AddonResourceRequest {
                protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
                addon_id: "addon-1".to_owned(),
                resource: AddonResource::Metadata,
                request_id: "request-1".to_owned(),
                payload: serde_json::json!({
                    "title": "The Matrix",
                    "year": 1999,
                    "language": "en-US",
                    "artwork_writeback": {
                        "library_id": "library-1",
                        "target": {
                            "kind": "media_item",
                            "id": "item-1"
                        },
                        "idempotency_key": "artwork-demo-1",
                        "kind": "poster"
                    }
                }),
            })
            .await;

        assert_eq!(response.payload["artwork_writeback"]["status"], "submitted");
        assert_eq!(
            response.payload["artwork_writeback"]["side_effect"]["permission"],
            "artwork_write"
        );
        assert_eq!(
            response.payload["artwork_writeback"]["side_effect"]["applied_source"],
            "addon:addon-1"
        );

        let requests = transport.requests();
        assert_eq!(requests.len(), 2);
        let access_check_body: serde_json::Value = serde_json::from_str(&requests[0].body).unwrap();
        assert_eq!(
            access_check_body,
            serde_json::json!({
                "permission": "artwork_write",
                "library_id": "library-1"
            })
        );
        let body: serde_json::Value = serde_json::from_str(&requests[1].body).unwrap();
        assert_eq!(body["permission"], "artwork_write");
        assert_eq!(body["library_id"], "library-1");
        assert_eq!(body["target"]["kind"], "media_item");
        assert_eq!(body["target"]["id"], "item-1");
        assert_eq!(body["idempotency_key"], "artwork-demo-1");
        assert_eq!(body["payload"]["intent"], "propose_artwork");
        assert_eq!(body["payload"]["kind"], "poster");
        assert_eq!(body["payload"]["source"]["kind"], "remote_url");
        assert_eq!(
            body["payload"]["source"]["url"],
            "https://example.test/poster.jpg"
        );
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
                    Err(NakoRuntimeError::Transport {
                        message: "fake response queue was empty".to_owned(),
                    })
                })
        }
    }
}
