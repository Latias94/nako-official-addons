use crate::engine::{
    MetadataQuery, ProviderMetadataCandidate,
    av::{AvNumberRoute, facts_from_query},
};

use super::{
    Fc2MetadataProvider,
    parser::{article_id_from_av_number, parse_detail_page},
};

impl<T> Fc2MetadataProvider<T>
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
        if av_facts.route != AvNumberRoute::Fc2 {
            return Ok(Vec::new());
        }

        let Some(article_id) = article_id_from_av_number(&av_facts.number) else {
            return Ok(Vec::new());
        };
        let detail_url = self.detail_url(&article_id);
        let detail = self.render(detail_url.clone()).await?;
        let Some(detail) = parse_detail_page(&detail.html, &detail_url, av_facts) else {
            return Ok(Vec::new());
        };

        Ok(vec![detail.into_candidate(query)])
    }
}
