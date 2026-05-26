use std::collections::{BTreeMap, BTreeSet};

use nako_addon_protocol::{AddonArtworkKind, AddonMetadataPatch};

use super::{
    MetadataCandidate, MetadataQuery, ProviderArtworkCandidate, ProviderExternalId,
    ProviderExternalIdCapability, ProviderFieldPolicy, ProviderMetadataCandidate,
    ranking::{self, CandidateFieldSource, CandidateMergeReason, CandidateProviderSource},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ResolvedProviderFact {
    source: ProviderFactSource,
    external_ids: Vec<ResolvedExternalId>,
    candidate: ProviderMetadataCandidate,
}

impl ResolvedProviderFact {
    fn from_candidate(
        candidate: ProviderMetadataCandidate,
        external_id_capabilities: &[ProviderExternalIdCapability],
    ) -> Self {
        let source = ProviderFactSource::from_candidate(&candidate);
        let external_ids =
            resolved_external_ids(&candidate.facts.external_ids, external_id_capabilities);

        Self {
            source,
            external_ids,
            candidate,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(test, derive(serde::Serialize))]
pub(crate) struct ProviderFactSource {
    pub provider: String,
    pub provider_id: String,
}

impl ProviderFactSource {
    fn from_candidate(candidate: &ProviderMetadataCandidate) -> Self {
        Self {
            provider: normalize_provider(&candidate.provider),
            provider_id: candidate.provider_id.trim().to_owned(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct ResolvedExternalId {
    provider: String,
    value: String,
}

impl ResolvedExternalId {
    fn from_external_id(external_id: &ProviderExternalId) -> Option<Self> {
        let provider = normalize_provider(&external_id.provider);
        let value = normalize_external_id_value(&external_id.value);
        (!provider.is_empty() && !value.is_empty()).then_some(Self { provider, value })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ResolvedCandidateCluster {
    facts: Vec<ResolvedProviderFact>,
}

impl ResolvedCandidateCluster {
    fn new(fact: ResolvedProviderFact) -> Self {
        Self { facts: vec![fact] }
    }

    #[must_use]
    pub(crate) fn into_ranked_candidate(
        self,
        query: &MetadataQuery,
        provider_field_policy: &ProviderFieldPolicy,
    ) -> MetadataCandidate {
        let provider_sources = self.provider_sources_for_evidence();
        let merge_reasons = self.merge_reasons_for_evidence(provider_sources.len());
        if !provider_field_policy.is_empty() && self.facts.len() > 1 {
            return self.into_policy_ranked_candidate(
                query,
                provider_sources,
                merge_reasons,
                provider_field_policy,
            );
        }

        let mut candidates = self
            .facts
            .into_iter()
            .map(|fact| {
                ranking::rank_candidate_with_source_evidence(
                    query,
                    fact.candidate,
                    provider_sources.clone(),
                    merge_reasons.clone(),
                )
            })
            .collect::<Vec<_>>();
        candidates.sort_by(ranking::compare_metadata_candidates);
        candidates
            .into_iter()
            .next()
            .expect("resolved candidate cluster always contains at least one fact")
    }

    fn into_policy_ranked_candidate(
        self,
        query: &MetadataQuery,
        provider_sources: Vec<CandidateProviderSource>,
        merge_reasons: Vec<CandidateMergeReason>,
        provider_field_policy: &ProviderFieldPolicy,
    ) -> MetadataCandidate {
        let facts = self.facts;
        let mut ranked_facts = facts
            .iter()
            .enumerate()
            .map(|(index, fact)| {
                (
                    index,
                    ranking::rank_candidate_with_source_evidence(
                        query,
                        fact.candidate.clone(),
                        provider_sources.clone(),
                        merge_reasons.clone(),
                    ),
                )
            })
            .collect::<Vec<_>>();
        ranked_facts
            .sort_by(|(_, left), (_, right)| ranking::compare_metadata_candidates(left, right));
        let base_index = ranked_facts
            .first()
            .map(|(index, _)| *index)
            .expect("resolved candidate cluster always contains at least one fact");

        let mut candidate = facts[base_index].candidate.clone();
        let field_sources = apply_provider_field_policy(
            &mut candidate.patch,
            &facts,
            base_index,
            provider_field_policy,
        );
        apply_provider_artwork_policy(
            &mut candidate.artwork_candidates,
            &facts,
            base_index,
            provider_field_policy,
        );

        ranking::rank_candidate_with_evidence_overrides(
            query,
            candidate,
            provider_sources,
            merge_reasons,
            Some(field_sources),
        )
    }

    fn provider_sources_for_evidence(&self) -> Vec<CandidateProviderSource> {
        let sources = self
            .facts
            .iter()
            .map(|fact| CandidateProviderSource {
                provider: fact.source.provider.clone(),
                provider_id: fact.source.provider_id.clone(),
            })
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();

        if sources.len() > 1 {
            sources
        } else {
            Vec::new()
        }
    }

    fn merge_reasons_for_evidence(&self, source_count: usize) -> Vec<CandidateMergeReason> {
        if source_count <= 1 {
            return Vec::new();
        }

        self.shared_external_ids()
            .into_iter()
            .map(|external_id| CandidateMergeReason {
                kind: "shared_external_id",
                provider: external_id.provider,
                source_count,
            })
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }

    #[cfg(test)]
    #[must_use]
    pub(crate) fn facts(&self) -> &[ResolvedProviderFact] {
        &self.facts
    }

    #[cfg(test)]
    #[must_use]
    pub(crate) fn sources(&self) -> Vec<ProviderFactSource> {
        self.facts
            .iter()
            .map(|fact| fact.source.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }

    #[cfg(test)]
    #[must_use]
    pub(crate) fn shared_external_id_providers(&self) -> Vec<String> {
        self.shared_external_ids()
            .into_iter()
            .map(|external_id| external_id.provider)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }

    #[cfg(test)]
    #[must_use]
    pub(crate) fn evidence(&self) -> ResolverClusterEvidence {
        let sources = self.sources();
        let merge_reasons = self
            .shared_external_id_providers()
            .into_iter()
            .map(|provider| ResolverMergeReason {
                kind: "shared_external_id",
                provider,
                source_count: sources.len(),
            })
            .collect();

        ResolverClusterEvidence {
            source_count: sources.len(),
            sources,
            merge_reasons,
        }
    }

    fn matches(&self, fact: &ResolvedProviderFact) -> bool {
        self.facts.iter().any(|existing| {
            existing.source == fact.source
                || existing
                    .external_ids
                    .iter()
                    .any(|external_id| fact.external_ids.contains(external_id))
        })
    }

    fn push_fact(&mut self, fact: ResolvedProviderFact) {
        self.facts.push(fact);
    }

    fn merge(&mut self, other: Self) {
        self.facts.extend(other.facts);
    }

    fn shared_external_ids(&self) -> Vec<ResolvedExternalId> {
        let mut sources_by_external_id = BTreeMap::<ResolvedExternalId, BTreeSet<_>>::new();
        for fact in &self.facts {
            for external_id in &fact.external_ids {
                sources_by_external_id
                    .entry(external_id.clone())
                    .or_default()
                    .insert(fact.source.clone());
            }
        }

        sources_by_external_id
            .into_iter()
            .filter_map(|(external_id, sources)| (sources.len() > 1).then_some(external_id))
            .collect()
    }
}

fn apply_provider_field_policy(
    patch: &mut AddonMetadataPatch,
    facts: &[ResolvedProviderFact],
    base_index: usize,
    provider_field_policy: &ProviderFieldPolicy,
) -> Vec<CandidateFieldSource> {
    let mut field_sources = Vec::new();

    let (title, source) =
        select_string_field(facts, base_index, provider_field_policy, "title", |patch| {
            patch.title.as_deref()
        });
    patch.title = title;
    push_selected_field_source(&mut field_sources, "title", facts, source);

    let (original_title, source) = select_string_field(
        facts,
        base_index,
        provider_field_policy,
        "original_title",
        |patch| patch.original_title.as_deref(),
    );
    patch.original_title = original_title;
    push_selected_field_source(&mut field_sources, "original_title", facts, source);

    let (sort_title, source) = select_string_field(
        facts,
        base_index,
        provider_field_policy,
        "sort_title",
        |patch| patch.sort_title.as_deref(),
    );
    patch.sort_title = sort_title;
    push_selected_field_source(&mut field_sources, "sort_title", facts, source);

    let (overview, source) = select_string_field(
        facts,
        base_index,
        provider_field_policy,
        "overview",
        |patch| patch.overview.as_deref(),
    );
    patch.overview = overview;
    push_selected_field_source(&mut field_sources, "overview", facts, source);

    let (release_date, source) = select_string_field(
        facts,
        base_index,
        provider_field_policy,
        "release_date",
        |patch| patch.release_date.as_deref(),
    );
    patch.release_date = release_date;
    push_selected_field_source(&mut field_sources, "release_date", facts, source);

    let (runtime_minutes, source) = select_copy_field(
        facts,
        base_index,
        provider_field_policy,
        "runtime_minutes",
        |patch| patch.runtime_minutes,
    );
    patch.runtime_minutes = runtime_minutes;
    push_selected_field_source(&mut field_sources, "runtime_minutes", facts, source);

    let (tagline, source) = select_string_field(
        facts,
        base_index,
        provider_field_policy,
        "tagline",
        |patch| patch.tagline.as_deref(),
    );
    patch.tagline = tagline;
    push_selected_field_source(&mut field_sources, "tagline", facts, source);

    let (genres, source) = select_vec_field(
        facts,
        base_index,
        provider_field_policy,
        "genres",
        |patch| patch.genres.as_ref(),
    );
    patch.genres = genres;
    push_selected_field_source(&mut field_sources, "genres", facts, source);

    let (tags, source) =
        select_vec_field(facts, base_index, provider_field_policy, "tags", |patch| {
            patch.tags.as_ref()
        });
    patch.tags = tags;
    push_selected_field_source(&mut field_sources, "tags", facts, source);

    field_sources
}

fn select_string_field(
    facts: &[ResolvedProviderFact],
    base_index: usize,
    provider_field_policy: &ProviderFieldPolicy,
    field: &'static str,
    get_value: impl Fn(&AddonMetadataPatch) -> Option<&str>,
) -> (Option<String>, Option<usize>) {
    let Some(index) = select_fact_index(facts, base_index, provider_field_policy, field, |patch| {
        get_value(patch).is_some_and(|value| !value.trim().is_empty())
    }) else {
        return (None, None);
    };

    (
        get_value(&facts[index].candidate.patch).map(str::to_owned),
        Some(index),
    )
}

fn select_vec_field(
    facts: &[ResolvedProviderFact],
    base_index: usize,
    provider_field_policy: &ProviderFieldPolicy,
    field: &'static str,
    get_value: impl Fn(&AddonMetadataPatch) -> Option<&Vec<String>>,
) -> (Option<Vec<String>>, Option<usize>) {
    let Some(index) = select_fact_index(facts, base_index, provider_field_policy, field, |patch| {
        get_value(patch).is_some_and(|value| !value.is_empty())
    }) else {
        return (None, None);
    };

    (
        get_value(&facts[index].candidate.patch).cloned(),
        Some(index),
    )
}

fn select_copy_field<T: Copy>(
    facts: &[ResolvedProviderFact],
    base_index: usize,
    provider_field_policy: &ProviderFieldPolicy,
    field: &'static str,
    get_value: impl Fn(&AddonMetadataPatch) -> Option<T>,
) -> (Option<T>, Option<usize>) {
    let Some(index) = select_fact_index(facts, base_index, provider_field_policy, field, |patch| {
        get_value(patch).is_some()
    }) else {
        return (None, None);
    };

    (get_value(&facts[index].candidate.patch), Some(index))
}

fn select_fact_index(
    facts: &[ResolvedProviderFact],
    base_index: usize,
    provider_field_policy: &ProviderFieldPolicy,
    field: &str,
    has_value: impl Fn(&AddonMetadataPatch) -> bool,
) -> Option<usize> {
    let preferences = provider_field_policy.providers_for(field);
    for provider in preferences {
        if let Some(index) = facts
            .iter()
            .position(|fact| fact.source.provider == *provider && has_value(&fact.candidate.patch))
        {
            return Some(index);
        }
    }

    if has_value(&facts[base_index].candidate.patch) {
        return Some(base_index);
    }

    if preferences.is_empty() {
        return None;
    }

    facts
        .iter()
        .position(|fact| has_value(&fact.candidate.patch))
}

fn apply_provider_artwork_policy(
    artwork_candidates: &mut Vec<ProviderArtworkCandidate>,
    facts: &[ResolvedProviderFact],
    base_index: usize,
    provider_field_policy: &ProviderFieldPolicy,
) {
    let mut selected = Vec::new();
    push_selected_artwork_kind(
        &mut selected,
        facts,
        base_index,
        provider_field_policy,
        "poster",
        AddonArtworkKind::Poster,
    );
    push_selected_artwork_kind(
        &mut selected,
        facts,
        base_index,
        provider_field_policy,
        "backdrop",
        AddonArtworkKind::Backdrop,
    );

    if selected.is_empty() {
        return;
    }

    selected.extend(
        artwork_candidates
            .iter()
            .filter(|candidate| {
                candidate.facts.kind != AddonArtworkKind::Poster
                    && candidate.facts.kind != AddonArtworkKind::Backdrop
            })
            .cloned(),
    );
    *artwork_candidates = selected;
}

fn push_selected_artwork_kind(
    selected: &mut Vec<ProviderArtworkCandidate>,
    facts: &[ResolvedProviderFact],
    base_index: usize,
    provider_field_policy: &ProviderFieldPolicy,
    field: &str,
    kind: AddonArtworkKind,
) {
    let Some(index) =
        select_artwork_fact_index(facts, base_index, provider_field_policy, field, kind)
    else {
        return;
    };
    selected.extend(
        facts[index]
            .candidate
            .artwork_candidates
            .iter()
            .filter(|candidate| candidate.facts.kind == kind)
            .cloned(),
    );
}

fn select_artwork_fact_index(
    facts: &[ResolvedProviderFact],
    base_index: usize,
    provider_field_policy: &ProviderFieldPolicy,
    field: &str,
    kind: AddonArtworkKind,
) -> Option<usize> {
    let mut preferences = provider_field_policy
        .providers_for(field)
        .iter()
        .collect::<Vec<_>>();
    for provider in provider_field_policy.providers_for("artwork") {
        if !preferences.contains(&provider) {
            preferences.push(provider);
        }
    }

    for provider in &preferences {
        if let Some(index) = facts.iter().position(|fact| {
            fact.source.provider.eq_ignore_ascii_case(provider)
                && fact
                    .candidate
                    .artwork_candidates
                    .iter()
                    .any(|candidate| candidate.facts.kind == kind)
        }) {
            return Some(index);
        }
    }

    if facts[base_index]
        .candidate
        .artwork_candidates
        .iter()
        .any(|candidate| candidate.facts.kind == kind)
    {
        return Some(base_index);
    }

    if preferences.is_empty() {
        return None;
    }

    facts.iter().position(|fact| {
        fact.candidate
            .artwork_candidates
            .iter()
            .any(|candidate| candidate.facts.kind == kind)
    })
}

fn push_selected_field_source(
    field_sources: &mut Vec<CandidateFieldSource>,
    field: &'static str,
    facts: &[ResolvedProviderFact],
    source_index: Option<usize>,
) {
    let Some(source_index) = source_index else {
        return;
    };
    let source = &facts[source_index].source;
    field_sources.push(CandidateFieldSource {
        field,
        provider: source.provider.clone(),
        provider_id: source.provider_id.clone(),
    });
}

#[cfg(test)]
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub(crate) struct ResolverClusterEvidence {
    pub source_count: usize,
    pub sources: Vec<ProviderFactSource>,
    pub merge_reasons: Vec<ResolverMergeReason>,
}

#[cfg(test)]
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub(crate) struct ResolverMergeReason {
    pub kind: &'static str,
    pub provider: String,
    pub source_count: usize,
}

#[must_use]
pub(crate) fn resolve_provider_candidates(
    candidates: Vec<ProviderMetadataCandidate>,
    external_id_capabilities: &[ProviderExternalIdCapability],
) -> Vec<ResolvedCandidateCluster> {
    let mut clusters = Vec::<ResolvedCandidateCluster>::new();

    for candidate in candidates {
        let fact = ResolvedProviderFact::from_candidate(candidate, external_id_capabilities);
        let matching_cluster_indexes = clusters
            .iter()
            .enumerate()
            .filter_map(|(index, cluster)| cluster.matches(&fact).then_some(index))
            .collect::<Vec<_>>();

        let Some(first_index) = matching_cluster_indexes.first().copied() else {
            clusters.push(ResolvedCandidateCluster::new(fact));
            continue;
        };

        clusters[first_index].push_fact(fact);
        for merge_index in matching_cluster_indexes.into_iter().skip(1).rev() {
            let other = clusters.remove(merge_index);
            clusters[first_index].merge(other);
        }
    }

    clusters
}

fn resolved_external_ids(
    external_ids: &[ProviderExternalId],
    external_id_capabilities: &[ProviderExternalIdCapability],
) -> Vec<ResolvedExternalId> {
    external_ids
        .iter()
        .filter(|external_id| {
            emitted_external_id_supported(&external_id.provider, external_id_capabilities)
        })
        .filter_map(ResolvedExternalId::from_external_id)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn emitted_external_id_supported(
    provider: &str,
    external_id_capabilities: &[ProviderExternalIdCapability],
) -> bool {
    external_id_capabilities.is_empty()
        || external_id_capabilities.iter().any(|capability| {
            capability.emits && provider.eq_ignore_ascii_case(capability.provider)
        })
}

fn normalize_provider(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

fn normalize_external_id_value(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use nako_addon_protocol::AddonMetadataPatch;

    use crate::engine::{
        ExternalIdValueKind, ProviderCandidateFacts, ProviderExternalId,
        ProviderExternalIdCapability, ProviderMetadataCandidate,
        resolver::resolve_provider_candidates,
    };

    #[test]
    fn resolver_clusters_candidates_that_share_external_ids() {
        let clusters = resolve_provider_candidates(
            vec![
                candidate(
                    "tmdb",
                    "tmdb:movie:603",
                    &[("tmdb", "603"), ("imdb", "tt0133093")],
                ),
                candidate(
                    "douban",
                    "douban:subject:1291843",
                    &[("imdb", "TT0133093"), ("douban", "1291843")],
                ),
                candidate("bangumi", "bangumi:subject:265", &[("bangumi", "265")]),
            ],
            &[],
        );

        assert_eq!(clusters.len(), 2);
        assert_eq!(clusters[0].sources().len(), 2);
        assert_eq!(clusters[0].facts().len(), 2);
        assert!(
            clusters[0]
                .shared_external_id_providers()
                .contains(&"imdb".to_owned())
        );
        assert_eq!(clusters[1].sources().len(), 1);
    }

    #[test]
    fn resolver_deduplicates_exact_provider_identity() {
        let clusters = resolve_provider_candidates(
            vec![
                candidate("tmdb", "tmdb:movie:603", &[("tmdb", "603")]),
                candidate("tmdb", "tmdb:movie:603", &[("tmdb", "603")]),
            ],
            &[],
        );

        assert_eq!(clusters.len(), 1);
        assert_eq!(clusters[0].facts().len(), 2);
        assert_eq!(clusters[0].sources().len(), 1);
    }

    #[test]
    fn resolver_evidence_does_not_serialize_raw_external_id_values() {
        let clusters = resolve_provider_candidates(
            vec![
                candidate("tmdb", "tmdb:movie:603", &[("imdb", "tt0133093")]),
                candidate("douban", "douban:subject:1291843", &[("imdb", "tt0133093")]),
            ],
            &[],
        );

        let evidence = clusters[0].evidence();
        let text = serde_json::to_string(&evidence).unwrap();

        assert!(text.contains("shared_external_id"));
        assert!(text.contains("imdb"));
        assert!(!text.contains("tt0133093"));
        assert!(!text.contains("The Matrix"));
    }

    #[test]
    fn resolver_uses_emitted_external_id_capabilities_when_catalog_is_provided() {
        let clusters = resolve_provider_candidates(
            vec![
                candidate("tmdb", "tmdb:movie:unlisted", &[("unlisted", "shared")]),
                candidate(
                    "douban",
                    "douban:subject:1291843",
                    &[("unlisted", "shared")],
                ),
                candidate("bangumi", "bangumi:subject:265", &[("imdb", "tt0133093")]),
                candidate("tmdb", "tmdb:movie:603", &[("imdb", "TT0133093")]),
            ],
            &[ProviderExternalIdCapability::new(
                "imdb",
                ExternalIdValueKind::Opaque,
                true,
                true,
                &["imdb_id"],
                true,
            )],
        );

        assert_eq!(clusters.len(), 3);
        assert_eq!(clusters[0].facts().len(), 1);
        assert_eq!(clusters[1].facts().len(), 1);
        assert_eq!(clusters[2].facts().len(), 2);
        assert_eq!(clusters[2].shared_external_id_providers(), vec!["imdb"]);
    }

    fn candidate(
        provider: &str,
        provider_id: &str,
        external_ids: &[(&str, &str)],
    ) -> ProviderMetadataCandidate {
        ProviderMetadataCandidate {
            provider: provider.to_owned(),
            provider_id: provider_id.to_owned(),
            patch: AddonMetadataPatch {
                title: Some("The Matrix".to_owned()),
                ..AddonMetadataPatch::default()
            },
            facts: ProviderCandidateFacts {
                title: Some("The Matrix".to_owned()),
                alternate_titles: Vec::new(),
                release_year: Some(1999),
                language: Some("en-US".to_owned()),
                community_score_milli: None,
                community_vote_count: None,
                external_ids: external_ids
                    .iter()
                    .map(|(provider, value)| ProviderExternalId {
                        provider: (*provider).to_owned(),
                        value: (*value).to_owned(),
                    })
                    .collect(),
                provider_outcomes: Vec::new(),
                provider_note: None,
            },
            artwork_candidates: Vec::new(),
        }
    }
}
