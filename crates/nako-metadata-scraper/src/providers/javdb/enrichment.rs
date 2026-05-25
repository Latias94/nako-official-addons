use crate::engine::{
    MetadataQuery, ProviderMetadataCandidate,
    av::{AvNumberRoute, AvNumberSource, facts_from_query, facts_from_text},
};

use super::{
    JavdbMetadataProvider,
    parser::{parse_detail_page, parse_search_results},
};

impl<T> JavdbMetadataProvider<T>
where
    T: crate::providers::http_runtime::ProviderHttpTransport,
{
    pub(super) async fn suggest_candidates(
        &self,
        query: &MetadataQuery,
    ) -> anyhow::Result<Vec<ProviderMetadataCandidate>> {
        let Some(av_facts) = facts_from_query(query) else {
            return Ok(Vec::new());
        };
        if av_facts.route == AvNumberRoute::Fc2 {
            return Ok(Vec::new());
        }

        let search = self.render(self.search_url(&av_facts.number)).await?;
        let search_results = parse_search_results(&search.html, &av_facts.number)
            .into_iter()
            .take(1)
            .collect::<Vec<_>>();
        let mut candidates = Vec::new();

        for result in search_results {
            let detail_url = self.absolute_url(&result.url);
            let detail = self.render(detail_url.clone()).await?;
            let detail_av_facts = facts_from_text(&result.number, AvNumberSource::ExternalId)
                .unwrap_or_else(|| av_facts.clone());
            if let Some(detail) =
                parse_detail_page(&detail.html, &result, &detail_url, detail_av_facts)
            {
                candidates.push(detail.into_candidate(query));
            }
        }

        Ok(candidates)
    }
}
