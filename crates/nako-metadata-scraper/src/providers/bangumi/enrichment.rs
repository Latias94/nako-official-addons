use crate::engine::{MetadataQuery, ProviderMetadataCandidate, ProviderOutcome};

use super::{
    BANGUMI_PROVIDER_ID, BangumiMetadataProvider,
    mapper::BangumiSubjectCandidate,
    parser::{
        BangumiSubject, BangumiSubjectSearchFilter, BangumiSubjectSearchRequest,
        BangumiSubjectSearchResponse,
    },
    search::{BANGUMI_DETAIL_ENRICHMENT_LIMIT, bangumi_air_date_filter, bangumi_query_subject_ids},
};
use crate::providers::{
    http_runtime::ProviderHttpTransport,
    search_policy::{SearchEnrichmentPolicy, first_direct_lookup, search_and_enrich},
};

const BANGUMI_SEARCH_POLICY: SearchEnrichmentPolicy = SearchEnrichmentPolicy::new(
    BANGUMI_PROVIDER_ID,
    "Bangumi",
    BANGUMI_DETAIL_ENRICHMENT_LIMIT,
    ProviderOutcome::BangumiPartialTitleVariantSearchFailure,
);

impl<T> BangumiMetadataProvider<T>
where
    T: ProviderHttpTransport,
{
    pub(super) async fn suggest_candidates(
        &self,
        query: &MetadataQuery,
    ) -> anyhow::Result<Vec<ProviderMetadataCandidate>> {
        if let Some(candidate) = first_direct_lookup(
            BANGUMI_SEARCH_POLICY,
            bangumi_query_subject_ids(query),
            |subject_id| async move {
                self.enrich_subject_candidate_by_id(query, subject_id)
                    .await
                    .map(Some)
            },
        )
        .await
        {
            return Ok(vec![candidate]);
        }

        search_and_enrich(
            BANGUMI_SEARCH_POLICY,
            query,
            |search_title| async move {
                self.search_subjects(query, &search_title)
                    .await
                    .map(|search| search.data)
            },
            |subject: &BangumiSubject| subject.id,
            |subject| subject.into_degraded_candidate(query),
            |candidate, outcome| candidate.facts.provider_outcomes.push(outcome),
            |subject| async move { self.enrich_subject_candidate(query, subject).await },
        )
        .await
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
