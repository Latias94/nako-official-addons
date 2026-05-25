use std::cmp::Ordering;

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
pub fn select_artwork_candidate(
    candidates: &[MetadataCandidate],
    kind: AddonArtworkKind,
) -> Option<&ArtworkCandidate> {
    candidates
        .iter()
        .flat_map(|candidate| candidate.artwork_candidates.iter())
        .filter(|artwork_candidate| artwork_candidate.artwork.kind == kind)
        .max_by(|left, right| compare_artwork_candidates(left, right))
}

fn compare_artwork_candidates(left: &ArtworkCandidate, right: &ArtworkCandidate) -> Ordering {
    left.confidence_milli
        .cmp(&right.confidence_milli)
        .then_with(|| artwork_area(&left.artwork).cmp(&artwork_area(&right.artwork)))
        .then_with(|| left.provider.cmp(&right.provider))
        .then_with(|| left.provider_id.cmp(&right.provider_id))
        .then_with(|| left.artwork.source.url.cmp(&right.artwork.source.url))
}

fn artwork_area(artwork: &AddonArtworkWritePayload) -> u64 {
    match (artwork.width, artwork.height) {
        (Some(width), Some(height)) => u64::from(width) * u64::from(height),
        _ => 0,
    }
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
    use crate::engine::ranking::{
        ExternalIdMatchEvidence, LanguageMatchEvidence, TitleMatchEvidence, YearMatchEvidence,
    };
    use crate::engine::{CandidateEvidence, MetadataCandidate};

    fn candidate(
        provider: &str,
        provider_id: &str,
        confidence_milli: u16,
        source_url: &str,
        width: Option<u32>,
        height: Option<u32>,
    ) -> MetadataCandidate {
        MetadataCandidate {
            provider: provider.to_owned(),
            provider_id: provider_id.to_owned(),
            confidence_milli,
            patch: nako_addon_protocol::AddonMetadataPatch::default(),
            artwork_candidates: vec![ArtworkCandidate {
                provider: provider.to_owned(),
                provider_id: provider_id.to_owned(),
                confidence_milli,
                artwork: AddonArtworkWritePayload {
                    intent: AddonArtworkIntent::ProposeArtwork,
                    kind: AddonArtworkKind::Poster,
                    source: AddonArtworkSourcePayload {
                        kind: AddonArtworkSourceKind::RemoteUrl,
                        url: source_url.to_owned(),
                    },
                    language: Some("en".to_owned()),
                    width,
                    height,
                },
            }],
            evidence: CandidateEvidence {
                title_match: TitleMatchEvidence::MissingCandidate,
                year_match: YearMatchEvidence::QueryMissing,
                language_match: LanguageMatchEvidence::Exact,
                external_id_match: ExternalIdMatchEvidence::QueryMissing,
                score_reasons: Vec::new(),
                field_sources: Vec::new(),
                provider_sources: Vec::new(),
                merge_reasons: Vec::new(),
                provider_note: None,
            },
        }
    }

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

    #[test]
    fn select_artwork_candidate_prefers_higher_confidence_candidate() {
        let candidates = vec![
            candidate(
                "tmdb",
                "tmdb:poster:1",
                600,
                "https://example.test/low.jpg",
                Some(1000),
                Some(1500),
            ),
            candidate(
                "bangumi",
                "bangumi:poster:2",
                850,
                "https://example.test/high.jpg",
                Some(900),
                Some(1350),
            ),
        ];

        let selected = select_artwork_candidate(&candidates, AddonArtworkKind::Poster)
            .expect("expected a poster candidate");

        assert_eq!(selected.provider, "bangumi");
        assert_eq!(selected.provider_id, "bangumi:poster:2");
        assert_eq!(selected.artwork.source.url, "https://example.test/high.jpg");
    }

    #[test]
    fn select_artwork_candidate_uses_resolution_as_tiebreaker() {
        let candidates = vec![
            candidate(
                "tmdb",
                "tmdb:poster:1",
                750,
                "https://example.test/small.jpg",
                Some(1000),
                Some(1500),
            ),
            candidate(
                "bangumi",
                "bangumi:poster:2",
                750,
                "https://example.test/large.jpg",
                Some(1200),
                Some(1800),
            ),
        ];

        let selected = select_artwork_candidate(&candidates, AddonArtworkKind::Poster)
            .expect("expected a poster candidate");

        assert_eq!(selected.provider, "bangumi");
        assert_eq!(
            selected.artwork.source.url,
            "https://example.test/large.jpg"
        );
    }
}
