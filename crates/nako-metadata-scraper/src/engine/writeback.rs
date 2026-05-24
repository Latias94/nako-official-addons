use serde::Deserialize;

use crate::nako_runtime::{
    NakoAccessCheckRequest, NakoPermission, NakoRuntimeClient, NakoRuntimeTransport,
    NakoSideEffectSummary, NakoSideEffectTarget, SubmitNakoArtworkWriteRequest,
    SubmitNakoMetadataWriteRequest,
};

use super::{MetadataCandidate, MetadataQuery, artwork};

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MetadataWritebackRequest {
    pub library_id: String,
    pub target: NakoSideEffectTarget,
    pub idempotency_key: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum MetadataWritebackInput {
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

impl MetadataWritebackInput {
    #[must_use]
    pub(crate) fn from_payload(payload: &serde_json::Value) -> Self {
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
    let Some(runtime) = runtime else {
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
    let Some(runtime) = runtime else {
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
