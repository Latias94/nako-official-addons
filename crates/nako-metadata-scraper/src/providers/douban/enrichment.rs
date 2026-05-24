use crate::engine::{MetadataQuery, ProviderMetadataCandidate};

use super::{
    DOUBAN_DETAIL_ENRICHMENT_LIMIT, DoubanMetadataProvider,
    parser::{parse_detail_page, parse_search_results},
};

impl<T> DoubanMetadataProvider<T>
where
    T: crate::providers::http_runtime::ProviderHttpTransport,
{
    pub(super) async fn suggest_candidates(
        &self,
        query: &MetadataQuery,
    ) -> anyhow::Result<Vec<ProviderMetadataCandidate>> {
        let search = self.render(self.search_url(&query.title)).await?;
        let search_results = parse_search_results(&search.html);
        let mut candidates = Vec::new();

        for result in search_results
            .into_iter()
            .take(DOUBAN_DETAIL_ENRICHMENT_LIMIT)
        {
            let detail = self.render(result.url.clone()).await?;
            if let Some(detail) = parse_detail_page(&detail.html, &result, query) {
                candidates.push(detail.into_candidate(query));
            }
        }

        Ok(candidates)
    }
}
