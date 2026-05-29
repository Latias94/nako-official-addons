use crate::engine::{
    MetadataQuery, ProviderMetadataCandidate,
    av::{AvNumberRoute, AvNumberSource, facts_from_query, facts_from_text},
};

use super::{
    JavdbMetadataProvider,
    parser::{JavdbSearchResult, parse_detail_page, parse_search_results},
};

impl<T> JavdbMetadataProvider<T>
where
    T: crate::providers::http_runtime::ProviderHttpTransport,
{
    pub(super) async fn suggest_candidates(
        &self,
        query: &MetadataQuery,
    ) -> anyhow::Result<Vec<ProviderMetadataCandidate>> {
        if let Some(movie_id) = explicit_javdb_id(query) {
            let detail_url = self.detail_url(&movie_id);
            let detail = self.render(detail_url.clone()).await?;
            let search_result = JavdbSearchResult {
                movie_id,
                url: detail_url.clone(),
                title: query.title.clone(),
                number: query.title.clone(),
            };
            if let Some(detail) = parse_detail_page(
                &detail.html,
                &search_result,
                &detail_url,
                facts_from_query(query)
                    .or_else(|| facts_from_text(&query.title, AvNumberSource::QueryTitle)),
            ) {
                return Ok(vec![detail.into_candidate(query)]);
            }
            return Ok(Vec::new());
        }

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
                parse_detail_page(&detail.html, &result, &detail_url, Some(detail_av_facts))
            {
                candidates.push(detail.into_candidate(query));
            }
        }

        Ok(candidates)
    }
}

fn explicit_javdb_id(query: &MetadataQuery) -> Option<String> {
    query
        .external_ids
        .iter()
        .find(|external_id| external_id.provider.eq_ignore_ascii_case("javdb"))
        .map(|external_id| external_id.value.trim())
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}
