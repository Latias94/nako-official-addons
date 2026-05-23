use nako_addon_protocol::{
    AddonArtworkIntent, AddonArtworkKind, AddonArtworkSourceKind, AddonArtworkSourcePayload,
    AddonArtworkWritePayload,
};
use serde::{Deserialize, Serialize};

use crate::nako_runtime::{NakoSideEffectSummary, NakoSideEffectTarget, NakoSideEffectTargetKind};

use super::ranking::MetadataCandidate;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderArtworkCandidate {
    pub provider: String,
    pub provider_id: String,
    pub facts: ProviderArtworkCandidateFacts,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderArtworkCandidateFacts {
    pub kind: AddonArtworkKind,
    pub source_url: String,
    pub language: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

impl ProviderArtworkCandidateFacts {
    #[must_use]
    pub fn into_artwork(self) -> AddonArtworkWritePayload {
        AddonArtworkWritePayload {
            intent: AddonArtworkIntent::ProposeArtwork,
            kind: self.kind,
            source: AddonArtworkSourcePayload {
                kind: AddonArtworkSourceKind::RemoteUrl,
                url: self.source_url,
            },
            language: self.language,
            width: self.width,
            height: self.height,
        }
    }
}

#[derive(Clone, Debug, Serialize, Eq, PartialEq)]
pub struct ArtworkCandidate {
    pub provider: String,
    pub provider_id: String,
    pub confidence_milli: u16,
    pub artwork: AddonArtworkWritePayload,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct ArtworkWritebackRequest {
    pub library_id: String,
    pub target: NakoSideEffectTarget,
    pub idempotency_key: String,
    pub kind: AddonArtworkKind,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ArtworkWritebackInput {
    Absent,
    Invalid { safe_error_code: &'static str },
    Requested(ArtworkWritebackRequest),
}

#[derive(Clone, Debug, Serialize, Eq, PartialEq)]
pub struct ArtworkWritebackResult {
    pub status: ArtworkWritebackStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safe_error_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub side_effect: Option<NakoSideEffectSummary>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtworkWritebackStatus {
    Submitted,
    Skipped,
    Failed,
}

impl ArtworkWritebackInput {
    #[must_use]
    pub fn from_payload(payload: &serde_json::Value) -> Self {
        let Some(writeback) = payload.get("artwork_writeback") else {
            return Self::Absent;
        };

        match serde_json::from_value::<ArtworkWritebackRequest>(writeback.clone()) {
            Ok(writeback_request) => Self::Requested(writeback_request),
            Err(_) => Self::Invalid {
                safe_error_code: "invalid_artwork_writeback_request",
            },
        }
    }
}

#[must_use]
pub fn select_artwork_candidate<'a>(
    candidates: &'a [MetadataCandidate],
    kind: AddonArtworkKind,
) -> Option<&'a ArtworkCandidate> {
    candidates.iter().find_map(|candidate| {
        candidate
            .artwork_candidates
            .iter()
            .find(|artwork_candidate| artwork_candidate.artwork.kind == kind)
    })
}

#[must_use]
pub fn valid_artwork_target(target: &NakoSideEffectTarget) -> bool {
    target.kind == NakoSideEffectTargetKind::MediaItem
}

#[must_use]
pub fn artwork_write_provenance(
    origin: &str,
    request_id: &str,
    query_title: &str,
    query_year: Option<i32>,
    query_language: &str,
    selected_candidate: &ArtworkCandidate,
) -> serde_json::Value {
    serde_json::json!({
        "origin": origin,
        "request_id": request_id,
        "query": {
            "title": query_title,
            "year": query_year,
            "language": query_language
        },
        "selected_candidate": {
            "provider": selected_candidate.provider,
            "provider_id": selected_candidate.provider_id,
            "confidence_milli": selected_candidate.confidence_milli,
            "kind": selected_candidate.artwork.kind,
        }
    })
}

#[must_use]
pub fn artwork_write_summary(
    status: ArtworkWritebackStatus,
    safe_error_code: Option<&'static str>,
    side_effect: Option<NakoSideEffectSummary>,
) -> ArtworkWritebackResult {
    ArtworkWritebackResult {
        status,
        safe_error_code: safe_error_code.map(str::to_owned),
        side_effect,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_artwork_candidate_facts_map_to_artwork_payload() {
        let artwork = ProviderArtworkCandidateFacts {
            kind: AddonArtworkKind::Poster,
            source_url: "https://example.test/poster.jpg".to_owned(),
            language: Some("en".to_owned()),
            width: Some(1000),
            height: Some(1500),
        }
        .into_artwork();

        assert_eq!(artwork.intent, AddonArtworkIntent::ProposeArtwork);
        assert_eq!(artwork.kind, AddonArtworkKind::Poster);
        assert_eq!(artwork.source.kind, AddonArtworkSourceKind::RemoteUrl);
        assert_eq!(artwork.source.url, "https://example.test/poster.jpg");
    }

    #[test]
    fn artwork_writeback_input_parses_explicit_payload() {
        let input = ArtworkWritebackInput::from_payload(&serde_json::json!({
            "artwork_writeback": {
                "library_id": "library-1",
                "target": {
                    "kind": "media_item",
                    "id": "item-1"
                },
                "idempotency_key": "artwork-demo-1",
                "kind": "poster"
            }
        }));

        match input {
            ArtworkWritebackInput::Requested(request) => {
                assert_eq!(request.library_id, "library-1");
                assert_eq!(request.target.kind, NakoSideEffectTargetKind::MediaItem);
                assert_eq!(request.kind, AddonArtworkKind::Poster);
            }
            other => panic!("unexpected artwork writeback input: {other:?}"),
        }
    }

    #[test]
    fn artwork_target_validation_is_media_item_only() {
        assert!(valid_artwork_target(&NakoSideEffectTarget {
            kind: NakoSideEffectTargetKind::MediaItem,
            id: "item-1".to_owned(),
        }));
        assert!(!valid_artwork_target(&NakoSideEffectTarget {
            kind: NakoSideEffectTargetKind::MediaSource,
            id: "source-1".to_owned(),
        }));
    }
}
