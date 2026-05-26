use crate::engine::{
    MetadataQuery, ProviderMetadataCandidate,
    av::{AvNumberRoute, AvNumberSource, facts_from_query, facts_from_text},
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
        if let Some(article_id) = explicit_fc2_id(query) {
            let detail_url = self.detail_url(&article_id);
            let detail = self.render(detail_url.clone()).await?;
            let av_facts =
                facts_from_text(&format!("FC2-{article_id}"), AvNumberSource::ExternalId)
                    .expect("FC2 article ID direct lookup builds a parseable AV number");
            let Some(detail) = parse_detail_page(&detail.html, &detail_url, av_facts) else {
                return Ok(Vec::new());
            };
            return Ok(vec![detail.into_candidate(query)]);
        }

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

fn explicit_fc2_id(query: &MetadataQuery) -> Option<String> {
    query
        .external_ids
        .iter()
        .find(|external_id| external_id.provider.eq_ignore_ascii_case("fc2"))
        .map(|external_id| external_id.value.trim())
        .filter(|value| !value.is_empty())
        .map(|value| value.trim_start_matches("FC2-"))
        .filter(|value| value.chars().all(|character| character.is_ascii_digit()))
        .map(str::to_owned)
}
