use std::collections::BTreeMap;

use crate::{
    domain::{ResourceLink, ResourceLinkType, ResourceSearchQuery, ResourceSearchResult},
    links::{classify_resource_url, resource_link_with_type},
};

use super::{
    PANSOU_COMPATIBLE_PROVIDER_ID,
    wire::{PansouLink, PansouMergedLink, PansouSearchResponse, PansouSearchResult},
};

pub(super) fn map_pansou_response(
    query: &ResourceSearchQuery,
    response: PansouSearchResponse,
) -> Vec<ResourceSearchResult> {
    if !response.results.is_empty() {
        return response
            .results
            .into_iter()
            .enumerate()
            .filter_map(|(index, result)| map_pansou_result(index, result))
            .collect();
    }

    map_pansou_merged_response(query, response.merged_by_type)
}

fn map_pansou_result(index: usize, result: PansouSearchResult) -> Option<ResourceSearchResult> {
    let source = source_from_channel(&result.channel);
    let links = result
        .links
        .into_iter()
        .filter_map(|link| map_pansou_link(link, &source))
        .collect::<Vec<_>>();
    if links.is_empty() {
        return None;
    }

    let id = first_non_empty(&[&result.unique_id, &result.message_id])
        .map(str::to_owned)
        .unwrap_or_else(|| format!("pansou:{index}"));
    let title =
        non_empty_trimmed(&result.title).unwrap_or_else(|| format!("PanSou result {index}"));

    Some(ResourceSearchResult {
        id,
        title,
        source,
        content: non_empty_trimmed(&result.content),
        links,
        tags: result.tags,
        images: result.images,
        score: 700,
    })
}

fn map_pansou_merged_response(
    query: &ResourceSearchQuery,
    merged_by_type: BTreeMap<String, Vec<PansouMergedLink>>,
) -> Vec<ResourceSearchResult> {
    let mut results = Vec::new();
    for (link_type, links) in merged_by_type {
        for (index, merged_link) in links.into_iter().enumerate() {
            let source = non_empty_trimmed(&merged_link.source)
                .unwrap_or_else(|| PANSOU_COMPATIBLE_PROVIDER_ID.to_owned());
            let Some(link) = map_pansou_merged_link(&link_type, &source, &merged_link) else {
                continue;
            };
            let title = format!("{} {} resource", query.query, link.link_type.as_str());
            results.push(ResourceSearchResult {
                id: format!("pansou:merged:{link_type}:{index}"),
                title,
                source,
                content: non_empty_trimmed(&merged_link.note),
                links: vec![link],
                tags: Vec::new(),
                images: merged_link.images,
                score: 650,
            });
        }
    }

    results
}

fn map_pansou_link(link: PansouLink, source: &str) -> Option<ResourceLink> {
    let mut mapped = pansou_resource_link(&link.url, &link.link_type, source)?;
    if let Some(password) = non_empty_trimmed(&link.password) {
        mapped = mapped.with_password(password);
    }
    if let Some(work_title) = non_empty_trimmed(&link.work_title) {
        mapped = mapped.with_note(work_title);
    }

    Some(mapped)
}

fn map_pansou_merged_link(
    link_type: &str,
    source: &str,
    link: &PansouMergedLink,
) -> Option<ResourceLink> {
    let mut mapped = pansou_resource_link(&link.url, link_type, source)?;
    if let Some(password) = non_empty_trimmed(&link.password) {
        mapped = mapped.with_password(password);
    }
    if let Some(note) = non_empty_trimmed(&link.note) {
        mapped = mapped.with_note(note);
    }

    Some(mapped)
}

fn pansou_resource_link(url: &str, pansou_link_type: &str, source: &str) -> Option<ResourceLink> {
    let link_type = pansou_link_type
        .parse::<ResourceLinkType>()
        .unwrap_or_else(|_| classify_resource_url(url));
    resource_link_with_type(url, link_type, source)
}

fn source_from_channel(channel: &str) -> String {
    non_empty_trimmed(channel)
        .map(|channel| format!("pansou:{channel}"))
        .unwrap_or_else(|| PANSOU_COMPATIBLE_PROVIDER_ID.to_owned())
}

fn first_non_empty<'a>(values: &[&'a str]) -> Option<&'a str> {
    values
        .iter()
        .copied()
        .find(|value| !value.trim().is_empty())
}

fn non_empty_trimmed(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_pansou_results_into_resource_search_results() {
        let response = serde_json::from_value::<PansouSearchResponse>(serde_json::json!({
            "results": [{
                "message_id": "m1",
                "unique_id": "u1",
                "channel": "movies",
                "title": "Demo Movie",
                "content": "content",
                "links": [
                    { "type": "quark", "url": "https://pan.quark.cn/s/demo", "password": "1234", "work_title": "disc 1" },
                    { "type": "magnet", "url": "magnet:?xt=urn:btih:ABCDEF" }
                ],
                "tags": ["tag1"],
                "images": ["https://example.test/image.jpg"]
            }]
        }))
        .unwrap();

        let results = map_pansou_response(&query(), response);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "u1");
        assert_eq!(results[0].source, "pansou:movies");
        assert_eq!(results[0].links[0].link_type, ResourceLinkType::Quark);
        assert_eq!(results[0].links[0].password.as_deref(), Some("1234"));
        assert_eq!(results[0].links[0].note.as_deref(), Some("disc 1"));
        assert_eq!(results[0].links[1].link_type, ResourceLinkType::Magnet);
    }

    #[test]
    fn maps_merged_only_pansou_response_into_synthetic_results() {
        let response = serde_json::from_value::<PansouSearchResponse>(serde_json::json!({
            "merged_by_type": {
                "quark": [{
                    "url": "https://pan.quark.cn/s/demo",
                    "password": "abcd",
                    "note": "merged",
                    "source": "plugin:jikepan",
                    "images": ["https://example.test/image.jpg"]
                }]
            }
        }))
        .unwrap();

        let results = map_pansou_response(&query(), response);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "pansou:merged:quark:0");
        assert_eq!(results[0].source, "plugin:jikepan");
        assert_eq!(results[0].links[0].link_type, ResourceLinkType::Quark);
        assert_eq!(results[0].links[0].password.as_deref(), Some("abcd"));
        assert_eq!(results[0].images, vec!["https://example.test/image.jpg"]);
    }

    fn query() -> ResourceSearchQuery {
        ResourceSearchQuery::free_text("Demo Movie", 10)
    }
}
