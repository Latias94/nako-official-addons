use nako_addon_protocol::AddonMetadataPatch;
use serde::Serialize;

use super::{
    MetadataQuery, QueryExternalId,
    artwork::{ArtworkCandidate, ProviderArtworkCandidate},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderMetadataCandidate {
    pub provider: String,
    pub provider_id: String,
    pub patch: AddonMetadataPatch,
    pub facts: ProviderCandidateFacts,
    pub artwork_candidates: Vec<ProviderArtworkCandidate>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ProviderCandidateFacts {
    pub title: Option<String>,
    pub release_year: Option<i32>,
    pub language: Option<String>,
    pub community_score_milli: Option<u16>,
    pub community_vote_count: Option<u32>,
    pub external_ids: Vec<ProviderExternalId>,
    pub provider_note: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderExternalId {
    pub provider: String,
    pub value: String,
}

#[derive(Clone, Debug, Serialize, Eq, PartialEq)]
pub struct MetadataCandidate {
    pub provider: String,
    pub provider_id: String,
    pub confidence_milli: u16,
    pub patch: AddonMetadataPatch,
    pub artwork_candidates: Vec<ArtworkCandidate>,
    pub evidence: CandidateEvidence,
}

#[derive(Clone, Debug, Serialize, Eq, PartialEq)]
pub struct CandidateEvidence {
    pub title_match: TitleMatchEvidence,
    pub year_match: YearMatchEvidence,
    pub language_match: LanguageMatchEvidence,
    pub external_id_match: ExternalIdMatchEvidence,
    pub score_reasons: Vec<CandidateScoreReason>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_note: Option<String>,
}

#[derive(Clone, Copy, Debug, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TitleMatchEvidence {
    Exact,
    Normalized,
    MissingCandidate,
    Mismatch,
}

#[derive(Clone, Copy, Debug, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum YearMatchEvidence {
    Exact,
    QueryMissing,
    CandidateMissing,
    Mismatch,
}

#[derive(Clone, Copy, Debug, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum LanguageMatchEvidence {
    Exact,
    QueryMissing,
    CandidateMissing,
    Mismatch,
}

#[derive(Clone, Copy, Debug, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ExternalIdMatchEvidence {
    Exact,
    QueryMissing,
    CandidateMissing,
    Mismatch,
}

#[derive(Clone, Debug, Serialize, Eq, PartialEq)]
pub struct CandidateScoreReason {
    pub kind: &'static str,
    pub outcome: &'static str,
    pub delta_milli: i16,
}

#[must_use]
pub fn rank_candidate(
    query: &MetadataQuery,
    candidate: ProviderMetadataCandidate,
) -> MetadataCandidate {
    let mut score = 250_i16;
    let mut reasons = vec![CandidateScoreReason {
        kind: "base",
        outcome: "provider_candidate",
        delta_milli: 250,
    }];

    let title_match = title_match(query.title.as_str(), &candidate.patch);
    push_reason(&mut reasons, title_reason(title_match));
    score += title_delta(title_match);

    let year_match = year_match(query.year, candidate.facts.release_year);
    push_reason(&mut reasons, year_reason(year_match));
    score += year_delta(year_match);

    let external_id_match = external_id_match(&query.external_ids, &candidate.facts.external_ids);
    push_reason(&mut reasons, external_id_reason(external_id_match));
    score += external_id_delta(external_id_match);

    let language_match =
        language_match(query.language.as_str(), candidate.facts.language.as_deref());
    push_reason(&mut reasons, language_reason(language_match));
    score += language_delta(language_match);

    let community_score = community_score(
        candidate.facts.community_score_milli,
        candidate.facts.community_vote_count,
    );
    push_reason(&mut reasons, community_score_reason(community_score));
    score += community_score;

    MetadataCandidate {
        provider: candidate.provider,
        provider_id: candidate.provider_id,
        confidence_milli: score.clamp(0, 1000) as u16,
        patch: candidate.patch,
        artwork_candidates: candidate
            .artwork_candidates
            .into_iter()
            .map(|artwork_candidate| ArtworkCandidate {
                provider: artwork_candidate.provider,
                provider_id: artwork_candidate.provider_id,
                confidence_milli: score.clamp(0, 1000) as u16,
                artwork: artwork_candidate.facts.into_artwork(),
            })
            .collect(),
        evidence: CandidateEvidence {
            title_match,
            year_match,
            language_match,
            external_id_match,
            score_reasons: reasons,
            provider_note: candidate.facts.provider_note,
        },
    }
}

fn push_reason(reasons: &mut Vec<CandidateScoreReason>, reason: Option<CandidateScoreReason>) {
    if let Some(reason) = reason {
        reasons.push(reason);
    }
}

fn title_match(query_title: &str, patch: &AddonMetadataPatch) -> TitleMatchEvidence {
    let candidate_titles = [
        patch.title.as_deref(),
        patch.original_title.as_deref(),
        patch.sort_title.as_deref(),
    ];
    let mut saw_candidate_title = false;
    let mut saw_normalized_match = false;

    for candidate_title in candidate_titles
        .into_iter()
        .flatten()
        .filter(|value| !value.trim().is_empty())
    {
        saw_candidate_title = true;

        if query_title == candidate_title {
            return TitleMatchEvidence::Exact;
        }

        if normalize_title(query_title) == normalize_title(candidate_title) {
            saw_normalized_match = true;
        }
    }

    if saw_normalized_match {
        TitleMatchEvidence::Normalized
    } else if saw_candidate_title {
        TitleMatchEvidence::Mismatch
    } else {
        TitleMatchEvidence::MissingCandidate
    }
}

fn title_delta(value: TitleMatchEvidence) -> i16 {
    match value {
        TitleMatchEvidence::Exact => 360,
        TitleMatchEvidence::Normalized => 320,
        TitleMatchEvidence::MissingCandidate => 0,
        TitleMatchEvidence::Mismatch => -160,
    }
}

fn title_reason(value: TitleMatchEvidence) -> Option<CandidateScoreReason> {
    Some(match value {
        TitleMatchEvidence::Exact => CandidateScoreReason {
            kind: "title",
            outcome: "exact",
            delta_milli: title_delta(value),
        },
        TitleMatchEvidence::Normalized => CandidateScoreReason {
            kind: "title",
            outcome: "normalized",
            delta_milli: title_delta(value),
        },
        TitleMatchEvidence::MissingCandidate => return None,
        TitleMatchEvidence::Mismatch => CandidateScoreReason {
            kind: "title",
            outcome: "mismatch",
            delta_milli: title_delta(value),
        },
    })
}

fn year_match(query_year: Option<i32>, candidate_year: Option<i32>) -> YearMatchEvidence {
    match (query_year, candidate_year) {
        (Some(left), Some(right)) if left == right => YearMatchEvidence::Exact,
        (Some(_), Some(_)) => YearMatchEvidence::Mismatch,
        (None, _) => YearMatchEvidence::QueryMissing,
        (Some(_), None) => YearMatchEvidence::CandidateMissing,
    }
}

fn year_delta(value: YearMatchEvidence) -> i16 {
    match value {
        YearMatchEvidence::Exact => 180,
        YearMatchEvidence::Mismatch => -120,
        YearMatchEvidence::QueryMissing | YearMatchEvidence::CandidateMissing => 0,
    }
}

fn year_reason(value: YearMatchEvidence) -> Option<CandidateScoreReason> {
    match value {
        YearMatchEvidence::Exact => Some(CandidateScoreReason {
            kind: "year",
            outcome: "exact",
            delta_milli: year_delta(value),
        }),
        YearMatchEvidence::Mismatch => Some(CandidateScoreReason {
            kind: "year",
            outcome: "mismatch",
            delta_milli: year_delta(value),
        }),
        YearMatchEvidence::QueryMissing | YearMatchEvidence::CandidateMissing => None,
    }
}

fn external_id_match(
    query_ids: &[QueryExternalId],
    candidate_ids: &[ProviderExternalId],
) -> ExternalIdMatchEvidence {
    if query_ids.is_empty() {
        return ExternalIdMatchEvidence::QueryMissing;
    }
    if candidate_ids.is_empty() {
        return ExternalIdMatchEvidence::CandidateMissing;
    }

    if query_ids.iter().any(|query| {
        candidate_ids.iter().any(|candidate| {
            query.provider.eq_ignore_ascii_case(&candidate.provider)
                && query.value == candidate.value
        })
    }) {
        ExternalIdMatchEvidence::Exact
    } else {
        ExternalIdMatchEvidence::Mismatch
    }
}

fn external_id_delta(value: ExternalIdMatchEvidence) -> i16 {
    match value {
        ExternalIdMatchEvidence::Exact => 300,
        ExternalIdMatchEvidence::Mismatch => -100,
        ExternalIdMatchEvidence::QueryMissing | ExternalIdMatchEvidence::CandidateMissing => 0,
    }
}

fn external_id_reason(value: ExternalIdMatchEvidence) -> Option<CandidateScoreReason> {
    match value {
        ExternalIdMatchEvidence::Exact => Some(CandidateScoreReason {
            kind: "external_id",
            outcome: "exact",
            delta_milli: external_id_delta(value),
        }),
        ExternalIdMatchEvidence::Mismatch => Some(CandidateScoreReason {
            kind: "external_id",
            outcome: "mismatch",
            delta_milli: external_id_delta(value),
        }),
        ExternalIdMatchEvidence::QueryMissing | ExternalIdMatchEvidence::CandidateMissing => None,
    }
}

fn language_match(query_language: &str, candidate_language: Option<&str>) -> LanguageMatchEvidence {
    if query_language.trim().is_empty() {
        return LanguageMatchEvidence::QueryMissing;
    }
    let Some(candidate_language) = candidate_language.filter(|value| !value.trim().is_empty())
    else {
        return LanguageMatchEvidence::CandidateMissing;
    };

    if query_language.eq_ignore_ascii_case(candidate_language) {
        LanguageMatchEvidence::Exact
    } else {
        LanguageMatchEvidence::Mismatch
    }
}

fn language_delta(value: LanguageMatchEvidence) -> i16 {
    match value {
        LanguageMatchEvidence::Exact => 40,
        LanguageMatchEvidence::Mismatch
        | LanguageMatchEvidence::QueryMissing
        | LanguageMatchEvidence::CandidateMissing => 0,
    }
}

fn language_reason(value: LanguageMatchEvidence) -> Option<CandidateScoreReason> {
    match value {
        LanguageMatchEvidence::Exact => Some(CandidateScoreReason {
            kind: "language",
            outcome: "exact",
            delta_milli: language_delta(value),
        }),
        LanguageMatchEvidence::Mismatch => Some(CandidateScoreReason {
            kind: "language",
            outcome: "mismatch",
            delta_milli: language_delta(value),
        }),
        LanguageMatchEvidence::QueryMissing | LanguageMatchEvidence::CandidateMissing => None,
    }
}

fn community_score(score_milli: Option<u16>, vote_count: Option<u32>) -> i16 {
    let Some(score_milli) = score_milli else {
        return 0;
    };
    let Some(vote_count) = vote_count else {
        return 0;
    };
    if vote_count < 25 {
        return 0;
    }

    let score_bonus = ((score_milli.saturating_sub(650)) / 25).min(10) as i16;
    let count_bonus = match vote_count {
        0..=99 => 0,
        100..=999 => 2,
        1000..=9999 => 4,
        _ => 6,
    };

    score_bonus + count_bonus
}

fn community_score_reason(delta_milli: i16) -> Option<CandidateScoreReason> {
    (delta_milli > 0).then_some(CandidateScoreReason {
        kind: "community_score",
        outcome: "boost",
        delta_milli,
    })
}

fn normalize_title(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

#[cfg(test)]
mod tests {
    use nako_addon_protocol::AddonMetadataPatch;

    use super::*;

    #[test]
    fn ranking_evidence_rewards_exact_title_year_external_id_and_language() {
        let candidate = rank_candidate(
            &MetadataQuery {
                title: "The Matrix".to_owned(),
                year: Some(1999),
                language: "en-US".to_owned(),
                external_ids: vec![QueryExternalId {
                    provider: "tmdb".to_owned(),
                    value: "603".to_owned(),
                }],
            },
            ProviderMetadataCandidate {
                provider: "tmdb".to_owned(),
                provider_id: "tmdb:movie:603".to_owned(),
                patch: AddonMetadataPatch {
                    title: Some("The Matrix".to_owned()),
                    ..AddonMetadataPatch::default()
                },
                facts: ProviderCandidateFacts {
                    title: Some("The Matrix".to_owned()),
                    release_year: Some(1999),
                    language: Some("en-US".to_owned()),
                    community_score_milli: None,
                    community_vote_count: None,
                    external_ids: vec![ProviderExternalId {
                        provider: "tmdb".to_owned(),
                        value: "603".to_owned(),
                    }],
                    provider_note: Some("synthetic test candidate".to_owned()),
                },
                artwork_candidates: Vec::new(),
            },
        );

        assert_eq!(candidate.confidence_milli, 1000);
        assert_eq!(candidate.evidence.title_match, TitleMatchEvidence::Exact);
        assert_eq!(candidate.evidence.year_match, YearMatchEvidence::Exact);
        assert_eq!(
            candidate.evidence.external_id_match,
            ExternalIdMatchEvidence::Exact
        );
        assert_eq!(
            candidate.evidence.language_match,
            LanguageMatchEvidence::Exact
        );
    }

    #[test]
    fn ranking_evidence_matches_original_and_sort_title_variants() {
        let candidate = rank_candidate(
            &MetadataQuery {
                title: "The Matrix".to_owned(),
                year: None,
                language: "en-US".to_owned(),
                external_ids: Vec::new(),
            },
            ProviderMetadataCandidate {
                provider: "tmdb".to_owned(),
                provider_id: "tmdb:movie:603".to_owned(),
                patch: AddonMetadataPatch {
                    title: Some("Matrix".to_owned()),
                    original_title: Some("The Matrix".to_owned()),
                    sort_title: Some("The Matrix".to_owned()),
                    ..AddonMetadataPatch::default()
                },
                facts: ProviderCandidateFacts {
                    title: Some("Matrix".to_owned()),
                    release_year: None,
                    language: Some("en-US".to_owned()),
                    community_score_milli: None,
                    community_vote_count: None,
                    external_ids: Vec::new(),
                    provider_note: None,
                },
                artwork_candidates: Vec::new(),
            },
        );

        assert_eq!(candidate.confidence_milli, 650);
        assert_eq!(candidate.evidence.title_match, TitleMatchEvidence::Exact);
    }

    #[test]
    fn ranking_evidence_penalizes_title_year_and_external_id_mismatch() {
        let candidate = rank_candidate(
            &MetadataQuery {
                title: "The Matrix".to_owned(),
                year: Some(1999),
                language: "en-US".to_owned(),
                external_ids: vec![QueryExternalId {
                    provider: "tmdb".to_owned(),
                    value: "603".to_owned(),
                }],
            },
            ProviderMetadataCandidate {
                provider: "tmdb".to_owned(),
                provider_id: "tmdb:movie:1".to_owned(),
                patch: AddonMetadataPatch::default(),
                facts: ProviderCandidateFacts {
                    title: Some("Other Movie".to_owned()),
                    release_year: Some(2001),
                    language: Some("ja-JP".to_owned()),
                    community_score_milli: None,
                    community_vote_count: None,
                    external_ids: vec![ProviderExternalId {
                        provider: "tmdb".to_owned(),
                        value: "1".to_owned(),
                    }],
                    provider_note: None,
                },
                artwork_candidates: Vec::new(),
            },
        );

        assert_eq!(candidate.confidence_milli, 30);
        assert_eq!(
            candidate.evidence.title_match,
            TitleMatchEvidence::MissingCandidate
        );
        assert_eq!(candidate.evidence.year_match, YearMatchEvidence::Mismatch);
        assert_eq!(
            candidate.evidence.external_id_match,
            ExternalIdMatchEvidence::Mismatch
        );
        assert_eq!(
            candidate.evidence.language_match,
            LanguageMatchEvidence::Mismatch
        );
    }

    #[test]
    fn ranking_evidence_serialization_is_redaction_safe() {
        let candidate = rank_candidate(
            &MetadataQuery {
                title: "Hidden Title".to_owned(),
                year: Some(1999),
                language: "en-US".to_owned(),
                external_ids: vec![QueryExternalId {
                    provider: "tmdb".to_owned(),
                    value: "secret-external-id".to_owned(),
                }],
            },
            ProviderMetadataCandidate {
                provider: "tmdb".to_owned(),
                provider_id: "tmdb:movie:redacted".to_owned(),
                patch: AddonMetadataPatch::default(),
                facts: ProviderCandidateFacts {
                    title: Some("Hidden Title".to_owned()),
                    release_year: Some(1999),
                    language: Some("en-US".to_owned()),
                    community_score_milli: None,
                    community_vote_count: None,
                    external_ids: vec![ProviderExternalId {
                        provider: "tmdb".to_owned(),
                        value: "secret-external-id".to_owned(),
                    }],
                    provider_note: Some("safe note".to_owned()),
                },
                artwork_candidates: Vec::new(),
            },
        );

        let text = serde_json::to_string(&candidate.evidence).unwrap();

        assert!(!text.contains("Hidden Title"));
        assert!(!text.contains("secret-external-id"));
        assert!(!text.contains("Bearer"));
        assert!(text.contains("external_id"));
    }

    #[test]
    fn ranking_evidence_rewards_community_score_bonus() {
        let query = MetadataQuery {
            title: "The Matrix".to_owned(),
            year: None,
            language: "en-US".to_owned(),
            external_ids: Vec::new(),
        };

        let baseline = rank_candidate(
            &query,
            ProviderMetadataCandidate {
                provider: "tmdb".to_owned(),
                provider_id: "tmdb:movie:baseline".to_owned(),
                patch: AddonMetadataPatch::default(),
                facts: ProviderCandidateFacts {
                    title: Some("The Matrix".to_owned()),
                    release_year: None,
                    language: Some("ja-JP".to_owned()),
                    community_score_milli: None,
                    community_vote_count: None,
                    external_ids: Vec::new(),
                    provider_note: None,
                },
                artwork_candidates: Vec::new(),
            },
        );

        let boosted = rank_candidate(
            &query,
            ProviderMetadataCandidate {
                provider: "tmdb".to_owned(),
                provider_id: "tmdb:movie:boosted".to_owned(),
                patch: AddonMetadataPatch::default(),
                facts: ProviderCandidateFacts {
                    title: Some("The Matrix".to_owned()),
                    release_year: None,
                    language: Some("ja-JP".to_owned()),
                    community_score_milli: Some(880),
                    community_vote_count: Some(12_000),
                    external_ids: Vec::new(),
                    provider_note: None,
                },
                artwork_candidates: Vec::new(),
            },
        );

        assert!(boosted.confidence_milli > baseline.confidence_milli);
        assert!(
            boosted
                .evidence
                .score_reasons
                .iter()
                .any(|reason| reason.kind == "community_score")
        );
    }
}
