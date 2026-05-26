use crate::engine::{
    MetadataQuery, ProviderMetadataCandidate,
    av::{AvNumberRoute, AvNumberSource, facts_from_query, facts_from_text},
};

use super::{
    DMM_PROVIDER_ID, DMM_URL_EXTERNAL_ID_PROVIDER, DmmMetadataProvider,
    parser::{DmmSearchResult, cid_from_url, parse_detail_page, parse_search_results},
};

impl<T> DmmMetadataProvider<T>
where
    T: crate::providers::http_runtime::ProviderHttpTransport,
{
    pub(super) async fn suggest_candidates(
        &self,
        query: &MetadataQuery,
    ) -> anyhow::Result<Vec<ProviderMetadataCandidate>> {
        if let Some(detail_url) = explicit_dmm_url(query) {
            let detail_url = self.absolute_url(&detail_url);
            let detail = self.render(detail_url.clone()).await?;
            let cid = cid_from_url(&detail_url).unwrap_or_else(|| query.title.clone());
            let search_result = DmmSearchResult {
                cid,
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

        if let Some(cid) = explicit_dmm_id(query) {
            let detail_url = self.detail_url(&cid);
            let detail = self.render(detail_url.clone()).await?;
            let search_result = DmmSearchResult {
                cid,
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
        if av_facts.route != AvNumberRoute::Censored {
            return Ok(Vec::new());
        }

        let search = self.render(self.search_url(&av_facts.number)).await?;
        let search_results = parse_search_results(&search.html, &av_facts)
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

fn explicit_dmm_id(query: &MetadataQuery) -> Option<String> {
    query
        .external_ids
        .iter()
        .find(|external_id| external_id.provider.eq_ignore_ascii_case(DMM_PROVIDER_ID))
        .map(|external_id| external_id.value.trim())
        .filter(|value| !value.is_empty())
        .and_then(normalize_dmm_id)
}

fn explicit_dmm_url(query: &MetadataQuery) -> Option<String> {
    query
        .external_ids
        .iter()
        .find(|external_id| {
            external_id
                .provider
                .eq_ignore_ascii_case(DMM_URL_EXTERNAL_ID_PROVIDER)
        })
        .map(|external_id| external_id.value.trim())
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

fn normalize_dmm_id(value: &str) -> Option<String> {
    if let Some(cid) = cid_from_url(value) {
        return Some(cid);
    }

    let value = value
        .trim()
        .trim_start_matches("cid=")
        .trim_matches(['/', '?', '#', '&']);
    let end = value.find(['/', '?', '#', '&']).unwrap_or(value.len());
    let cid = &value[..end];
    (!cid.is_empty()
        && cid
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '_'))
    .then(|| cid.to_owned())
}
