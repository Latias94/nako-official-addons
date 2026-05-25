use std::{collections::HashSet, future::Future, hash::Hash};

use crate::engine::{MetadataQuery, ProviderMetadataCandidate, ranking};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SearchEnrichmentPolicy {
    provider_id: &'static str,
    provider_name: &'static str,
    enrichment_limit: usize,
    partial_search_note: &'static str,
}

impl SearchEnrichmentPolicy {
    #[must_use]
    pub(crate) const fn new(
        provider_id: &'static str,
        provider_name: &'static str,
        enrichment_limit: usize,
        partial_search_note: &'static str,
    ) -> Self {
        Self {
            provider_id,
            provider_name,
            enrichment_limit,
            partial_search_note,
        }
    }
}

pub(crate) async fn first_direct_lookup<Attempt, Attempts, Lookup, LookupFuture>(
    policy: SearchEnrichmentPolicy,
    attempts: Attempts,
    mut lookup: Lookup,
) -> Option<ProviderMetadataCandidate>
where
    Attempt: Send,
    Attempts: IntoIterator<Item = Attempt> + Send,
    Attempts::IntoIter: Send,
    Lookup: FnMut(Attempt) -> LookupFuture,
    LookupFuture: Future<Output = anyhow::Result<Option<ProviderMetadataCandidate>>> + Send,
{
    for attempt in attempts {
        match lookup(attempt).await {
            Ok(Some(candidate)) => return Some(candidate),
            Ok(None) => {
                tracing::warn!(
                    provider = policy.provider_id,
                    provider_name = policy.provider_name,
                    "provider direct lookup returned no candidate; trying next direct lookup"
                );
            }
            Err(error) => {
                tracing::warn!(
                    provider = policy.provider_id,
                    provider_name = policy.provider_name,
                    %error,
                    "provider direct lookup failed; trying next direct lookup"
                );
            }
        }
    }

    None
}

pub(crate) async fn search_and_enrich<
    SearchResult,
    SearchResultKey,
    SearchTitleVariant,
    SearchFuture,
    SearchResultKeyFn,
    DegradedCandidate,
    AppendProviderNote,
    EnrichSearchResult,
    EnrichFuture,
>(
    policy: SearchEnrichmentPolicy,
    query: &MetadataQuery,
    mut search_title_variant: SearchTitleVariant,
    mut search_result_key: SearchResultKeyFn,
    mut degraded_candidate: DegradedCandidate,
    mut append_provider_note: AppendProviderNote,
    mut enrich_search_result: EnrichSearchResult,
) -> anyhow::Result<Vec<ProviderMetadataCandidate>>
where
    SearchResult: Clone + Send,
    SearchResultKey: Eq + Hash + Send,
    SearchTitleVariant: FnMut(String) -> SearchFuture,
    SearchFuture: Future<Output = anyhow::Result<Vec<SearchResult>>> + Send,
    SearchResultKeyFn: FnMut(&SearchResult) -> SearchResultKey,
    DegradedCandidate: FnMut(SearchResult) -> ProviderMetadataCandidate,
    AppendProviderNote: FnMut(&mut ProviderMetadataCandidate, &'static str),
    EnrichSearchResult: FnMut(SearchResult) -> EnrichFuture,
    EnrichFuture: Future<Output = anyhow::Result<ProviderMetadataCandidate>> + Send,
{
    let mut search_results = Vec::new();
    let mut seen_result_ids = HashSet::new();
    let mut last_search_error = None;

    for search_title in query.search_title_variants() {
        let results = match search_title_variant(search_title).await {
            Ok(results) => results,
            Err(error) => {
                tracing::warn!(
                    provider = policy.provider_id,
                    provider_name = policy.provider_name,
                    %error,
                    "provider title-variant search failed"
                );
                last_search_error = Some(error);
                continue;
            }
        };

        for result in results {
            if seen_result_ids.insert(search_result_key(&result)) {
                search_results.push(result);
            }
        }
    }

    if search_results.is_empty()
        && let Some(error) = last_search_error.take()
    {
        return Err(error);
    }

    let partial_search = last_search_error.is_some();
    let search_results = ranking::select_ranked_provider_inputs(
        query,
        search_results,
        policy.enrichment_limit,
        |result| degraded_candidate(result.clone()),
    );

    let mut candidates = Vec::new();
    for result in search_results {
        match enrich_search_result(result.clone()).await {
            Ok(mut candidate) => {
                if partial_search {
                    append_provider_note(&mut candidate, policy.partial_search_note);
                }
                candidates.push(candidate);
            }
            Err(error) => {
                tracing::warn!(
                    provider = policy.provider_id,
                    provider_name = policy.provider_name,
                    %error,
                    "returning degraded provider candidate after enrichment failure"
                );
                let mut candidate = degraded_candidate(result);
                if partial_search {
                    append_provider_note(&mut candidate, policy.partial_search_note);
                }
                candidates.push(candidate);
            }
        }
    }

    Ok(candidates)
}

#[cfg(test)]
mod tests {
    use nako_addon_protocol::AddonMetadataPatch;

    use crate::engine::{ProviderCandidateFacts, ProviderExternalId};

    use super::*;

    const TEST_POLICY: SearchEnrichmentPolicy = SearchEnrichmentPolicy::new(
        "fixture",
        "Fixture",
        2,
        "Fixture provider preserved candidates after partial title-variant search failure.",
    );

    #[derive(Clone, Debug)]
    struct FakeSearchResult {
        id: u64,
        title: &'static str,
        enrichment_fails: bool,
    }

    #[tokio::test]
    async fn search_policy_dedupes_ranks_preserves_partial_failure_and_degrades() {
        let query = query("Movie [Cut]");

        let candidates = search_and_enrich(
            TEST_POLICY,
            &query,
            |search_title| async move {
                match search_title.as_str() {
                    "Movie [Cut]" => Ok(vec![
                        FakeSearchResult {
                            id: 2,
                            title: "Other",
                            enrichment_fails: false,
                        },
                        FakeSearchResult {
                            id: 1,
                            title: "Movie [Cut]",
                            enrichment_fails: true,
                        },
                        FakeSearchResult {
                            id: 1,
                            title: "Movie [Cut]",
                            enrichment_fails: true,
                        },
                    ]),
                    "Movie" => anyhow::bail!("synthetic title-variant failure"),
                    _ => Ok(Vec::new()),
                }
            },
            |result| result.id,
            |result| candidate(&query, result.id, result.title, "degraded"),
            |candidate, note| {
                candidate
                    .facts
                    .provider_note
                    .get_or_insert_with(String::new)
                    .push_str(note);
            },
            |result| {
                let query = &query;
                async move {
                    if result.enrichment_fails {
                        anyhow::bail!("synthetic enrichment failure");
                    }

                    Ok(candidate(query, result.id, result.title, "enriched"))
                }
            },
        )
        .await
        .unwrap();

        assert_eq!(
            candidates
                .iter()
                .map(|candidate| candidate.provider_id.as_str())
                .collect::<Vec<_>>(),
            vec!["fixture:1", "fixture:2"]
        );
        assert!(
            candidates[0]
                .facts
                .provider_note
                .as_deref()
                .is_some_and(|note| note.contains("degraded")
                    && note.contains("partial title-variant search failure"))
        );
        assert!(
            candidates[1]
                .facts
                .provider_note
                .as_deref()
                .is_some_and(|note| note.contains("enriched")
                    && note.contains("partial title-variant search failure"))
        );
    }

    #[tokio::test]
    async fn direct_lookup_policy_returns_first_successful_candidate() {
        let query = query("Movie");

        let candidate = first_direct_lookup(TEST_POLICY, [1, 2, 3], |attempt| {
            let query = &query;
            async move {
                match attempt {
                    1 => anyhow::bail!("synthetic direct lookup failure"),
                    2 => Ok(None),
                    id => Ok(Some(candidate(query, id, "Movie", "direct"))),
                }
            }
        })
        .await
        .unwrap();

        assert_eq!(candidate.provider_id, "fixture:3");
        assert_eq!(candidate.facts.provider_note.as_deref(), Some("direct"));
    }

    fn query(title: &str) -> MetadataQuery {
        MetadataQuery {
            title: title.to_owned(),
            year: Some(2021),
            language: "en-US".to_owned(),
            external_ids: Vec::new(),
        }
    }

    fn candidate(
        query: &MetadataQuery,
        id: u64,
        title: &'static str,
        note: &'static str,
    ) -> ProviderMetadataCandidate {
        ProviderMetadataCandidate {
            provider: "fixture".to_owned(),
            provider_id: format!("fixture:{id}"),
            patch: AddonMetadataPatch {
                title: Some(title.to_owned()),
                ..AddonMetadataPatch::default()
            },
            facts: ProviderCandidateFacts {
                title: Some(title.to_owned()),
                alternate_titles: Vec::new(),
                release_year: query.year,
                language: Some(query.language.clone()),
                community_score_milli: None,
                community_vote_count: None,
                external_ids: vec![ProviderExternalId {
                    provider: "fixture".to_owned(),
                    value: id.to_string(),
                }],
                provider_note: Some(note.to_owned()),
            },
            artwork_candidates: Vec::new(),
        }
    }
}
