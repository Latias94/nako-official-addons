use std::collections::HashSet;

use crate::engine::{MetadataQuery, ProviderMetadataCandidate};

use super::{
    BANGUMI_PROVIDER_ID, BangumiMetadataProvider,
    mapper::{BangumiSubjectCandidate, append_provider_note},
    parser::{
        BangumiSubject, BangumiSubjectSearchFilter, BangumiSubjectSearchRequest,
        BangumiSubjectSearchResponse,
    },
    search::{
        BANGUMI_DETAIL_ENRICHMENT_LIMIT, BANGUMI_PARTIAL_SEARCH_NOTE, bangumi_air_date_filter,
        bangumi_query_subject_ids, bangumi_ranked_enrichment_subjects,
    },
};

impl<T> BangumiMetadataProvider<T>
where
    T: crate::providers::http_runtime::ProviderHttpTransport,
{
    pub(super) async fn suggest_candidates(
        &self,
        query: &MetadataQuery,
    ) -> anyhow::Result<Vec<ProviderMetadataCandidate>> {
        for subject_id in bangumi_query_subject_ids(query) {
            match self.enrich_subject_candidate_by_id(query, subject_id).await {
                Ok(candidate) => return Ok(vec![candidate]),
                Err(error) => {
                    tracing::warn!(provider = BANGUMI_PROVIDER_ID, %error, "Bangumi direct subject lookup failed; falling back to title search");
                }
            }
        }

        let mut search_subjects = Vec::new();
        let mut seen_subject_ids = HashSet::new();
        let mut last_search_error = None;

        for search_title in query.search_title_variants() {
            let search = match self.search_subjects(query, &search_title).await {
                Ok(search) => search,
                Err(error) => {
                    tracing::warn!(provider = BANGUMI_PROVIDER_ID, %error, "Bangumi title-variant search failed");
                    last_search_error = Some(error);
                    continue;
                }
            };
            for subject in search.data {
                if seen_subject_ids.insert(subject.id) {
                    search_subjects.push(subject);
                }
            }
        }
        if search_subjects.is_empty()
            && let Some(error) = last_search_error.take()
        {
            return Err(error);
        }
        let partial_search = last_search_error.is_some();
        search_subjects = bangumi_ranked_enrichment_subjects(query, search_subjects);

        let mut candidates = Vec::new();
        for subject in search_subjects {
            match self.enrich_subject_candidate(query, subject.clone()).await {
                Ok(mut candidate) => {
                    if partial_search {
                        append_provider_note(
                            &mut candidate.facts.provider_note,
                            BANGUMI_PARTIAL_SEARCH_NOTE,
                        );
                    }
                    candidates.push(candidate);
                }
                Err(error) => {
                    tracing::warn!(provider = BANGUMI_PROVIDER_ID, %error, "returning degraded Bangumi candidate after enrichment failure");
                    let mut candidate = subject.into_degraded_candidate(query);
                    if partial_search {
                        append_provider_note(
                            &mut candidate.facts.provider_note,
                            BANGUMI_PARTIAL_SEARCH_NOTE,
                        );
                    }
                    candidates.push(candidate);
                }
            }
        }

        Ok(candidates)
    }

    pub(super) async fn search_subjects(
        &self,
        query: &MetadataQuery,
        search_title: &str,
    ) -> anyhow::Result<BangumiSubjectSearchResponse> {
        let search_body = BangumiSubjectSearchRequest {
            keyword: search_title.to_owned(),
            sort: "match",
            filter: BangumiSubjectSearchFilter {
                subject_type: self.config.subject_types.clone(),
                nsfw: self.config.include_nsfw,
                air_date: bangumi_air_date_filter(query.year),
            },
        };
        let response = self
            .runtime
            .post_json(
                BANGUMI_PROVIDER_ID,
                "search subjects",
                self.endpoint("v0/search/subjects"),
                vec![
                    (
                        "limit".to_owned(),
                        BANGUMI_DETAIL_ENRICHMENT_LIMIT.to_string(),
                    ),
                    ("offset".to_owned(), "0".to_owned()),
                ],
                self.bearer_headers(),
                &search_body,
            )
            .await?;

        BangumiSubjectSearchResponse::from_value(response.body)
    }

    pub(super) async fn enrich_subject_candidate(
        &self,
        query: &MetadataQuery,
        search_subject: BangumiSubject,
    ) -> anyhow::Result<ProviderMetadataCandidate> {
        let detail = self.fetch_subject(search_subject.id).await?;
        if detail.id == 0 {
            anyhow::bail!(
                "Bangumi subject detail response returned zero id for requested subject {}",
                search_subject.id
            );
        }
        if detail.id != search_subject.id {
            anyhow::bail!(
                "Bangumi subject detail response id {} did not match requested subject {}",
                detail.id,
                search_subject.id
            );
        }
        Ok(BangumiSubjectCandidate {
            search: search_subject,
            detail,
            degraded: false,
        }
        .into_candidate(query))
    }

    pub(super) async fn enrich_subject_candidate_by_id(
        &self,
        query: &MetadataQuery,
        subject_id: u64,
    ) -> anyhow::Result<ProviderMetadataCandidate> {
        let detail = self.fetch_subject(subject_id).await?;
        if detail.id == 0 {
            anyhow::bail!(
                "Bangumi subject detail response returned zero id for requested subject {subject_id}"
            );
        }
        if detail.id != subject_id {
            anyhow::bail!(
                "Bangumi subject detail response id {} did not match requested subject {subject_id}",
                detail.id
            );
        }
        Ok(BangumiSubjectCandidate {
            search: BangumiSubject::default(),
            detail,
            degraded: false,
        }
        .into_candidate(query))
    }

    async fn fetch_subject(&self, subject_id: u64) -> anyhow::Result<BangumiSubject> {
        let response = self
            .runtime
            .get_json(
                BANGUMI_PROVIDER_ID,
                "subject detail",
                self.endpoint(format!("v0/subjects/{subject_id}")),
                Vec::new(),
                self.bearer_headers(),
            )
            .await?;

        BangumiSubject::from_value(response.body)
    }
}
