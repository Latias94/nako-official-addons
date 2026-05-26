use async_trait::async_trait;
use scraper::{Html, Selector};

use crate::{
    engine::{
        MetadataQuery, ProviderMetadataCandidate,
        av::{AvNumberRoute, AvNumberSource, AvQueryFacts, facts_from_query, facts_from_text},
    },
    providers::rendered_page::RenderedHtmlPage,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RenderedAvSearchResult {
    pub(crate) url: String,
}

impl RenderedAvSearchResult {
    #[must_use]
    pub(crate) fn new(url: impl Into<String>) -> Self {
        Self { url: url.into() }
    }
}

#[async_trait]
pub(crate) trait RenderedAvFlow: Sync {
    fn provider_id(&self) -> &'static str;

    fn url_external_id_provider(&self) -> &'static str;

    fn supports_route(&self, route: AvNumberRoute) -> bool;

    async fn render_html_page(&self, url: String) -> anyhow::Result<RenderedHtmlPage>;

    fn absolute_url(&self, value: &str) -> String;

    fn detail_url(&self, id: &str) -> String;

    fn direct_lookup_av(&self, query: &MetadataQuery) -> Option<AvQueryFacts> {
        facts_from_query(query)
    }

    fn search_url(&self, _av: &AvQueryFacts) -> Option<String> {
        None
    }

    fn search_results(&self, _html: &str, _av: &AvQueryFacts) -> Vec<RenderedAvSearchResult> {
        Vec::new()
    }

    fn detail_candidates(
        &self,
        html: &str,
        detail_url: &str,
        av: Option<AvQueryFacts>,
        query: &MetadataQuery,
    ) -> Vec<ProviderMetadataCandidate>;
}

pub(crate) async fn suggest_candidates<F>(
    flow: &F,
    query: &MetadataQuery,
) -> anyhow::Result<Vec<ProviderMetadataCandidate>>
where
    F: RenderedAvFlow + ?Sized,
{
    if let Some(url) = direct_external_id(query, flow.url_external_id_provider()) {
        return render_detail_candidates(
            flow,
            flow.absolute_url(&url),
            flow.direct_lookup_av(query),
            query,
        )
        .await;
    }

    if let Some(id) = direct_external_id(query, flow.provider_id()) {
        return render_detail_candidates(
            flow,
            flow.detail_url(&id),
            flow.direct_lookup_av(query),
            query,
        )
        .await;
    }

    let Some(av) = facts_from_query(query) else {
        return Ok(Vec::new());
    };
    if !flow.supports_route(av.route) {
        return Ok(Vec::new());
    }

    let detail_url = if let Some(search_url) = flow.search_url(&av) {
        let search = flow.render_html_page(search_url).await?;
        let Some(result) = flow.search_results(&search.html, &av).into_iter().next() else {
            return Ok(Vec::new());
        };
        result.url
    } else {
        flow.detail_url(&av.number)
    };

    render_detail_candidates(flow, detail_url, Some(av), query).await
}

async fn render_detail_candidates<F>(
    flow: &F,
    detail_url: String,
    av: Option<AvQueryFacts>,
    query: &MetadataQuery,
) -> anyhow::Result<Vec<ProviderMetadataCandidate>>
where
    F: RenderedAvFlow + ?Sized,
{
    let detail = flow.render_html_page(detail_url.clone()).await?;
    Ok(flow.detail_candidates(&detail.html, &detail_url, av, query))
}

pub(crate) fn direct_external_id(query: &MetadataQuery, provider: &str) -> Option<String> {
    query
        .external_ids
        .iter()
        .find(|external_id| external_id.provider.eq_ignore_ascii_case(provider))
        .map(|external_id| external_id.value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

pub(crate) fn element_text(document: &Html, selector: &str) -> Option<String> {
    let selector = Selector::parse(selector).ok()?;
    document
        .select(&selector)
        .next()
        .map(|element| normalize_whitespace(&element.text().collect::<Vec<_>>().join(" ")))
        .filter(|value| !value.is_empty())
}

pub(crate) fn attr_value(document: &Html, selector: &str, attr: &str) -> Option<String> {
    let selector = Selector::parse(selector).ok()?;
    document
        .select(&selector)
        .find_map(|element| element.value().attr(attr))
        .map(str::to_owned)
        .filter(|value| !value.trim().is_empty())
}

pub(crate) fn link_texts(document: &Html, selector: &str) -> Vec<String> {
    let Ok(selector) = Selector::parse(selector) else {
        return Vec::new();
    };
    document
        .select(&selector)
        .map(|element| normalize_whitespace(&element.text().collect::<Vec<_>>().join(" ")))
        .filter(|value| !value.is_empty())
        .fold(Vec::new(), |mut values, value| {
            if !values.contains(&value) {
                values.push(value);
            }
            values
        })
}

pub(crate) fn image_urls(document: &Html, selector: &str, base_url: &str) -> Vec<String> {
    let Ok(selector) = Selector::parse(selector) else {
        return Vec::new();
    };
    document
        .select(&selector)
        .filter_map(|element| {
            element
                .value()
                .attr("href")
                .or_else(|| element.value().attr("src"))
                .or_else(|| {
                    element
                        .select(&Selector::parse("img").ok()?)
                        .next()
                        .and_then(|image| image.value().attr("src"))
                })
        })
        .map(|value| normalize_url(absolute_url(base_url, value)))
        .filter(|value| !value.trim().is_empty())
        .fold(Vec::new(), |mut values, value| {
            if !values.contains(&value) {
                values.push(value);
            }
            values
        })
}

pub(crate) fn first_non_empty(values: &[Option<&str>]) -> Option<String> {
    values
        .iter()
        .flatten()
        .map(|value| normalize_whitespace(value))
        .find(|value| !value.is_empty())
}

pub(crate) fn labeled_value(text: &str, labels: &[&str], known_labels: &[&str]) -> Option<String> {
    labels
        .iter()
        .find_map(|label| labeled_value_by_label(text, label, known_labels))
}

pub(crate) fn first_iso_date(text: &str) -> Option<String> {
    for token in text.split_whitespace() {
        if token.len() >= 10 && token.as_bytes().get(4) == Some(&b'-') {
            let value = &token[..10];
            if value
                .chars()
                .enumerate()
                .all(|(index, character)| matches!(index, 4 | 7) || character.is_ascii_digit())
            {
                return Some(value.to_owned());
            }
        }
    }
    None
}

pub(crate) fn first_year(text: &str) -> Option<i32> {
    for token in text.split(|character: char| !character.is_ascii_digit()) {
        if token.len() == 4 {
            let year = token.parse::<i32>().ok()?;
            if (1888..=2100).contains(&year) {
                return Some(year);
            }
        }
    }
    None
}

pub(crate) fn parse_minutes(value: &str) -> Option<u32> {
    value
        .split(|character: char| !character.is_ascii_digit())
        .find(|value| !value.is_empty())
        .and_then(|value| value.parse::<u32>().ok())
}

pub(crate) fn parse_rating_milli(value: &str) -> Option<u16> {
    let rating = value
        .split_whitespace()
        .find_map(|token| token.trim().parse::<f64>().ok())?;
    let scaled = if rating <= 5.0 {
        rating * 200.0
    } else {
        rating * 100.0
    };
    Some(scaled.round().clamp(0.0, 1000.0) as u16)
}

pub(crate) fn first_u32(value: &str) -> Option<u32> {
    value
        .split(|character: char| !character.is_ascii_digit())
        .find(|value| !value.is_empty())
        .and_then(|value| value.parse::<u32>().ok())
}

pub(crate) fn absolute_url(base_url: &str, value: &str) -> String {
    let value = value.trim();
    if value.starts_with("http://") || value.starts_with("https://") {
        return value.to_owned();
    }
    if let Some(value) = value.strip_prefix("//") {
        return format!("https://{value}");
    }
    format!(
        "{}/{}",
        base_url.trim_end_matches('/'),
        value.trim_start_matches('/')
    )
}

pub(crate) fn normalize_url(value: String) -> String {
    if let Some(value) = value.strip_prefix("//") {
        return format!("https://{value}");
    }
    value
}

pub(crate) fn normalize_whitespace(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub(crate) fn percent_encode(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.as_bytes() {
        match *byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(char::from(*byte));
            }
            b' ' => encoded.push_str("%20"),
            byte => encoded.push_str(&format!("%{byte:02X}")),
        }
    }
    encoded
}

pub(crate) fn text_or_url_matches_av(text: &str, url: &str, av: &AvQueryFacts) -> bool {
    [text, url]
        .into_iter()
        .filter_map(|value| facts_from_text(value, AvNumberSource::ExternalId))
        .any(|facts| facts.number.eq_ignore_ascii_case(&av.number))
        || compact(text).contains(&compact(&av.number))
        || compact(url).contains(&compact(&av.number))
}

pub(crate) fn id_query_value(url: &str, key: &str) -> Option<String> {
    let marker = format!("{key}=");
    let start = url.find(&marker)? + marker.len();
    let rest = &url[start..];
    let end = rest.find(['/', '?', '#', '&']).unwrap_or(rest.len());
    let id = &rest[..end];
    (!id.is_empty()).then(|| id.to_owned())
}

fn labeled_value_by_label(text: &str, label: &str, known_labels: &[&str]) -> Option<String> {
    let markers = [format!("{label}:"), format!("{label}：")];
    for marker in markers {
        let Some(start) = text.find(&marker).map(|index| index + marker.len()) else {
            continue;
        };
        let rest = text[start..].trim();
        let end = known_labels
            .iter()
            .flat_map(|known_label| [format!("{known_label}:"), format!("{known_label}：")])
            .filter(|next_marker| next_marker != &marker)
            .filter_map(|next_marker| rest.find(&next_marker))
            .min()
            .unwrap_or(rest.len());
        if let Some(value) =
            Some(normalize_whitespace(&rest[..end])).filter(|value| !value.is_empty())
        {
            return Some(value);
        }
    }

    None
}

fn compact(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .map(|character| character.to_ascii_uppercase())
        .collect()
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;
    use nako_addon_protocol::AddonMetadataPatch;

    use crate::engine::{
        MetadataQuery, ProviderCandidateFacts, ProviderMetadataCandidate, QueryExternalId,
        av::{AV_NUMBER_EXTERNAL_ID_PROVIDER, AvNumberRoute, AvQueryFacts},
    };

    use super::*;

    #[tokio::test]
    async fn rendered_av_flow_searches_then_renders_first_detail_result() {
        let flow = FakeRenderedAvFlow::default();
        let query = MetadataQuery {
            title: "SSNI-644".to_owned(),
            year: None,
            language: "zh-CN".to_owned(),
            external_ids: vec![QueryExternalId {
                provider: AV_NUMBER_EXTERNAL_ID_PROVIDER.to_owned(),
                value: "SSNI-644".to_owned(),
            }],
        };

        let candidates = suggest_candidates(&flow, &query).await.unwrap();

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].provider_id, "fake:detail");
        assert_eq!(
            flow.rendered_urls(),
            vec![
                "https://site.test/search/SSNI-644".to_owned(),
                "https://site.test/detail/SSNI-644".to_owned()
            ]
        );
    }

    #[derive(Clone, Default)]
    struct FakeRenderedAvFlow {
        rendered_urls: Arc<Mutex<Vec<String>>>,
    }

    impl FakeRenderedAvFlow {
        fn rendered_urls(&self) -> Vec<String> {
            self.rendered_urls.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl RenderedAvFlow for FakeRenderedAvFlow {
        fn provider_id(&self) -> &'static str {
            "fake"
        }

        fn url_external_id_provider(&self) -> &'static str {
            "fake_url"
        }

        fn supports_route(&self, route: AvNumberRoute) -> bool {
            route == AvNumberRoute::Censored
        }

        async fn render_html_page(
            &self,
            url: String,
        ) -> anyhow::Result<crate::providers::rendered_page::RenderedHtmlPage> {
            self.rendered_urls.lock().unwrap().push(url.clone());
            let html = if url.contains("/search/") {
                "search"
            } else {
                "detail"
            };
            Ok(crate::providers::rendered_page::RenderedHtmlPage {
                html: html.to_owned(),
            })
        }

        fn absolute_url(&self, value: &str) -> String {
            absolute_url("https://site.test", value)
        }

        fn detail_url(&self, id: &str) -> String {
            format!("https://site.test/detail/{id}")
        }

        fn search_url(&self, av: &AvQueryFacts) -> Option<String> {
            Some(format!("https://site.test/search/{}", av.number))
        }

        fn search_results(&self, html: &str, av: &AvQueryFacts) -> Vec<RenderedAvSearchResult> {
            assert_eq!(html, "search");
            vec![RenderedAvSearchResult::new(format!(
                "https://site.test/detail/{}",
                av.number
            ))]
        }

        fn detail_candidates(
            &self,
            html: &str,
            _detail_url: &str,
            _av: Option<AvQueryFacts>,
            _query: &MetadataQuery,
        ) -> Vec<ProviderMetadataCandidate> {
            assert_eq!(html, "detail");
            vec![ProviderMetadataCandidate {
                provider: "fake".to_owned(),
                provider_id: "fake:detail".to_owned(),
                patch: AddonMetadataPatch::default(),
                facts: ProviderCandidateFacts::default(),
                artwork_candidates: Vec::new(),
            }]
        }
    }
}
