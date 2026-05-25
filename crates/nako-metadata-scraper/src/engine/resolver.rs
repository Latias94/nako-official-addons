#[cfg(test)]
use std::collections::BTreeMap;
use std::collections::BTreeSet;

use super::{
    MetadataCandidate, MetadataQuery, ProviderExternalId, ProviderMetadataCandidate, ranking,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ResolvedProviderFact {
    source: ProviderFactSource,
    external_ids: Vec<ResolvedExternalId>,
    candidate: ProviderMetadataCandidate,
}

impl ResolvedProviderFact {
    fn from_candidate(candidate: ProviderMetadataCandidate) -> Self {
        let source = ProviderFactSource::from_candidate(&candidate);
        let external_ids = resolved_external_ids(&candidate.facts.external_ids);

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
    pub(crate) fn into_ranked_candidate(self, query: &MetadataQuery) -> MetadataCandidate {
        let mut candidates = self
            .facts
            .into_iter()
            .map(|fact| ranking::rank_candidate(query, fact.candidate))
            .collect::<Vec<_>>();
        candidates.sort_by(ranking::compare_metadata_candidates);
        candidates
            .into_iter()
            .next()
            .expect("resolved candidate cluster always contains at least one fact")
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

    #[cfg(test)]
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
) -> Vec<ResolvedCandidateCluster> {
    let mut clusters = Vec::<ResolvedCandidateCluster>::new();

    for candidate in candidates {
        let fact = ResolvedProviderFact::from_candidate(candidate);
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

fn resolved_external_ids(external_ids: &[ProviderExternalId]) -> Vec<ResolvedExternalId> {
    external_ids
        .iter()
        .filter_map(ResolvedExternalId::from_external_id)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
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
        ProviderCandidateFacts, ProviderExternalId, ProviderMetadataCandidate,
        resolver::resolve_provider_candidates,
    };

    #[test]
    fn resolver_clusters_candidates_that_share_external_ids() {
        let clusters = resolve_provider_candidates(vec![
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
        ]);

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
        let clusters = resolve_provider_candidates(vec![
            candidate("tmdb", "tmdb:movie:603", &[("tmdb", "603")]),
            candidate("tmdb", "tmdb:movie:603", &[("tmdb", "603")]),
        ]);

        assert_eq!(clusters.len(), 1);
        assert_eq!(clusters[0].facts().len(), 2);
        assert_eq!(clusters[0].sources().len(), 1);
    }

    #[test]
    fn resolver_evidence_does_not_serialize_raw_external_id_values() {
        let clusters = resolve_provider_candidates(vec![
            candidate("tmdb", "tmdb:movie:603", &[("imdb", "tt0133093")]),
            candidate("douban", "douban:subject:1291843", &[("imdb", "tt0133093")]),
        ]);

        let evidence = clusters[0].evidence();
        let text = serde_json::to_string(&evidence).unwrap();

        assert!(text.contains("shared_external_id"));
        assert!(text.contains("imdb"));
        assert!(!text.contains("tt0133093"));
        assert!(!text.contains("The Matrix"));
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
