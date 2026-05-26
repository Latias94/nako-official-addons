use async_trait::async_trait;
use serde::Deserialize;

use crate::nako_runtime::{
    NakoPermission, NakoRuntimeClient, NakoRuntimeResult, NakoRuntimeTransport,
    NakoSideEffectResponse, NakoSideEffectSummary, NakoSideEffectTarget, NakoSideEffectTargetKind,
    SubmitNakoArtworkWriteRequest, SubmitNakoMetadataWriteRequest,
};

use super::{
    MetadataCandidate, MetadataQuery, artwork,
    side_effect::{
        SideEffectWritebackAdapter, SideEffectWritebackInput, SideEffectWritebackRequest,
        run_side_effect_writeback,
    },
};

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MetadataWritebackRequest {
    pub library_id: String,
    pub target: NakoSideEffectTarget,
    pub idempotency_key: String,
}

pub(crate) type MetadataWritebackInput = SideEffectWritebackInput<MetadataWritebackRequest>;

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

impl SideEffectWritebackRequest for MetadataWritebackRequest {
    fn library_id(&self) -> &str {
        &self.library_id
    }

    fn target(&self) -> &NakoSideEffectTarget {
        &self.target
    }

    fn idempotency_key(&self) -> &str {
        &self.idempotency_key
    }
}

#[must_use]
pub(crate) fn metadata_writeback_input_from_payload(
    payload: &serde_json::Value,
) -> MetadataWritebackInput {
    MetadataWritebackInput::from_payload(payload, "writeback", "invalid_writeback_request")
}

pub(crate) async fn maybe_submit_metadata_writeback<T>(
    runtime: Option<&NakoRuntimeClient<T>>,
    request_id: &str,
    query: &MetadataQuery,
    selected_candidate: Option<&MetadataCandidate>,
    writeback_request: MetadataWritebackInput,
) -> Option<MetadataWritebackResult>
where
    T: NakoRuntimeTransport,
{
    run_side_effect_writeback(
        runtime,
        request_id,
        writeback_request,
        MetadataWritebackAdapter {
            request_id,
            query,
            selected_candidate,
        },
    )
    .await
}

pub(crate) async fn maybe_submit_artwork_writeback<T>(
    runtime: Option<&NakoRuntimeClient<T>>,
    request_id: &str,
    query: &MetadataQuery,
    candidates: &[MetadataCandidate],
    writeback_request: artwork::ArtworkWritebackInput,
) -> Option<artwork::ArtworkWritebackResult>
where
    T: NakoRuntimeTransport,
{
    run_side_effect_writeback(
        runtime,
        request_id,
        writeback_request,
        ArtworkWritebackAdapter {
            request_id,
            query,
            candidates,
        },
    )
    .await
}

#[must_use]
fn valid_metadata_target(target: &NakoSideEffectTarget) -> bool {
    target.kind == NakoSideEffectTargetKind::MediaSource
}

#[must_use]
fn metadata_write_summary(
    status: MetadataWritebackStatus,
    safe_error_code: Option<String>,
    side_effect: Option<NakoSideEffectSummary>,
) -> MetadataWritebackResult {
    MetadataWritebackResult {
        status,
        safe_error_code,
        side_effect,
    }
}

struct MetadataWritebackAdapter<'a> {
    request_id: &'a str,
    query: &'a MetadataQuery,
    selected_candidate: Option<&'a MetadataCandidate>,
}

struct MetadataWritebackSubmission {
    provenance: serde_json::Value,
    patch: nako_addon_protocol::AddonMetadataPatch,
}

#[async_trait]
impl<T> SideEffectWritebackAdapter<T> for MetadataWritebackAdapter<'_>
where
    T: NakoRuntimeTransport,
{
    type Request = MetadataWritebackRequest;
    type Prepared = MetadataWritebackSubmission;
    type Result = MetadataWritebackResult;

    fn operation_name(&self) -> &'static str {
        "metadata writeback"
    }

    fn permission(&self) -> NakoPermission {
        NakoPermission::MetadataWrite
    }

    fn validate_target(&self, target: &NakoSideEffectTarget) -> Result<(), &'static str> {
        valid_metadata_target(target)
            .then_some(())
            .ok_or("invalid_metadata_target_kind")
    }

    fn prepare(&self, _request: &Self::Request) -> Result<Self::Prepared, &'static str> {
        let Some(selected_candidate) = self.selected_candidate else {
            return Err("no_candidates");
        };

        Ok(MetadataWritebackSubmission {
            provenance: serde_json::json!({
                "origin": "nako-metadata-scraper",
                "request_id": self.request_id,
                "query": {
                    "title": self.query.title,
                    "year": self.query.year,
                    "language": self.query.language
                },
                "selected_candidate": {
                    "provider": selected_candidate.provider,
                    "provider_id": selected_candidate.provider_id,
                    "confidence_milli": selected_candidate.confidence_milli
                }
            }),
            patch: selected_candidate.patch.clone(),
        })
    }

    fn skipped(&self, safe_error_code: String) -> Self::Result {
        metadata_write_summary(
            MetadataWritebackStatus::Skipped,
            Some(safe_error_code),
            None,
        )
    }

    fn failed(&self, safe_error_code: String) -> Self::Result {
        metadata_write_summary(MetadataWritebackStatus::Failed, Some(safe_error_code), None)
    }

    fn submitted(&self, side_effect: NakoSideEffectSummary) -> Self::Result {
        metadata_write_summary(MetadataWritebackStatus::Submitted, None, Some(side_effect))
    }

    async fn submit(
        &self,
        runtime: &NakoRuntimeClient<T>,
        request: &Self::Request,
        prepared: Self::Prepared,
    ) -> NakoRuntimeResult<NakoSideEffectResponse> {
        runtime
            .submit_metadata_write(SubmitNakoMetadataWriteRequest {
                library_id: request.library_id().to_owned(),
                target: request.target().clone(),
                idempotency_key: request.idempotency_key().to_owned(),
                provenance: prepared.provenance,
                patch: prepared.patch,
            })
            .await
    }
}

impl SideEffectWritebackRequest for artwork::ArtworkWritebackRequest {
    fn library_id(&self) -> &str {
        &self.library_id
    }

    fn target(&self) -> &NakoSideEffectTarget {
        &self.target
    }

    fn idempotency_key(&self) -> &str {
        &self.idempotency_key
    }
}

struct ArtworkWritebackAdapter<'a> {
    request_id: &'a str,
    query: &'a MetadataQuery,
    candidates: &'a [MetadataCandidate],
}

struct ArtworkWritebackSubmission {
    provenance: serde_json::Value,
    artwork: nako_addon_protocol::AddonArtworkWritePayload,
}

#[async_trait]
impl<T> SideEffectWritebackAdapter<T> for ArtworkWritebackAdapter<'_>
where
    T: NakoRuntimeTransport,
{
    type Request = artwork::ArtworkWritebackRequest;
    type Prepared = ArtworkWritebackSubmission;
    type Result = artwork::ArtworkWritebackResult;

    fn operation_name(&self) -> &'static str {
        "artwork writeback"
    }

    fn permission(&self) -> NakoPermission {
        NakoPermission::ArtworkWrite
    }

    fn validate_target(&self, target: &NakoSideEffectTarget) -> Result<(), &'static str> {
        artwork::valid_artwork_target(target)
            .then_some(())
            .ok_or("invalid_artwork_target_kind")
    }

    fn prepare(&self, request: &Self::Request) -> Result<Self::Prepared, &'static str> {
        let Some(selected_candidate) =
            artwork::select_artwork_candidate(self.candidates, request.kind)
        else {
            return Err("no_artwork_candidates");
        };

        Ok(ArtworkWritebackSubmission {
            provenance: artwork::artwork_write_provenance(
                "nako-metadata-scraper",
                self.request_id,
                &self.query.title,
                self.query.year,
                &self.query.language,
                selected_candidate,
            ),
            artwork: selected_candidate.artwork.clone(),
        })
    }

    fn skipped(&self, safe_error_code: String) -> Self::Result {
        artwork::artwork_write_summary(
            artwork::ArtworkWritebackStatus::Skipped,
            Some(safe_error_code),
            None,
        )
    }

    fn failed(&self, safe_error_code: String) -> Self::Result {
        artwork::artwork_write_summary(
            artwork::ArtworkWritebackStatus::Failed,
            Some(safe_error_code),
            None,
        )
    }

    fn submitted(&self, side_effect: NakoSideEffectSummary) -> Self::Result {
        artwork::artwork_write_summary(
            artwork::ArtworkWritebackStatus::Submitted,
            None,
            Some(side_effect),
        )
    }

    async fn submit(
        &self,
        runtime: &NakoRuntimeClient<T>,
        request: &Self::Request,
        prepared: Self::Prepared,
    ) -> NakoRuntimeResult<NakoSideEffectResponse> {
        runtime
            .submit_artwork_write(SubmitNakoArtworkWriteRequest {
                library_id: request.library_id().to_owned(),
                target: request.target().clone(),
                idempotency_key: request.idempotency_key().to_owned(),
                provenance: prepared.provenance,
                artwork: prepared.artwork,
            })
            .await
    }
}
