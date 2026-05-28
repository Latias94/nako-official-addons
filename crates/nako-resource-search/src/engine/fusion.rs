use std::collections::{BTreeMap, HashMap};

use crate::domain::{MergedResourceLink, ResourceLinkType, ResourceSearchResult};

#[must_use]
pub fn fuse_results(
    mut results: Vec<ResourceSearchResult>,
    requested_link_types: &[ResourceLinkType],
    limit: usize,
) -> (
    Vec<ResourceSearchResult>,
    BTreeMap<ResourceLinkType, Vec<MergedResourceLink>>,
) {
    for result in &mut results {
        result.links.retain(|link| {
            requested_link_types.is_empty() || requested_link_types.contains(&link.link_type)
        });
    }

    results.retain(|result| !result.links.is_empty());
    results.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| left.title.cmp(&right.title))
            .then_with(|| left.id.cmp(&right.id))
    });
    results.truncate(limit);

    let mut merged = HashMap::<String, MergedResourceLink>::new();
    for result in &results {
        for link in &result.links {
            merged
                .entry(link.normalized_url.clone())
                .and_modify(|existing| existing.merge_source(&link.source))
                .or_insert_with(|| MergedResourceLink::from_link(link));
        }
    }

    let mut merged_by_type = BTreeMap::<ResourceLinkType, Vec<MergedResourceLink>>::new();
    for link in merged.into_values() {
        merged_by_type.entry(link.link_type).or_default().push(link);
    }
    for links in merged_by_type.values_mut() {
        links.sort_by(|left, right| left.normalized_url.cmp(&right.normalized_url));
    }

    (results, merged_by_type)
}

#[cfg(test)]
mod tests {
    use crate::links::resource_link;

    use super::*;

    #[test]
    fn fusion_deduplicates_normalized_urls_and_preserves_sources() {
        let result_a = result_with_links(
            "a",
            "fixture-a",
            vec![
                "https://PAN.QUARK.CN/s/demo#frag",
                "magnet:?xt=urn:btih:ABC",
            ],
        );
        let result_b = result_with_links("b", "fixture-b", vec!["https://pan.quark.cn/s/demo"]);

        let (_results, merged_by_type) = fuse_results(vec![result_a, result_b], &[], 10);
        let quark_links = merged_by_type.get(&ResourceLinkType::Quark).unwrap();

        assert_eq!(quark_links.len(), 1);
        assert_eq!(quark_links[0].normalized_url, "https://pan.quark.cn/s/demo");
        assert_eq!(
            quark_links[0].sources,
            vec!["fixture-a".to_owned(), "fixture-b".to_owned()]
        );
        assert_eq!(
            merged_by_type.get(&ResourceLinkType::Magnet).unwrap()[0].normalized_url,
            "magnet:?xt=urn:btih:abc"
        );
    }

    #[test]
    fn fusion_filters_link_types_and_drops_empty_results() {
        let result = result_with_links(
            "a",
            "fixture",
            vec!["https://pan.quark.cn/s/demo", "magnet:?xt=urn:btih:ABC"],
        );

        let (results, merged_by_type) = fuse_results(vec![result], &[ResourceLinkType::Magnet], 10);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].links.len(), 1);
        assert_eq!(results[0].links[0].link_type, ResourceLinkType::Magnet);
        assert!(!merged_by_type.contains_key(&ResourceLinkType::Quark));
        assert!(merged_by_type.contains_key(&ResourceLinkType::Magnet));
    }

    fn result_with_links(id: &str, source: &str, links: Vec<&str>) -> ResourceSearchResult {
        ResourceSearchResult {
            id: id.to_owned(),
            title: id.to_owned(),
            source: source.to_owned(),
            content: None,
            links: links
                .into_iter()
                .map(|url| resource_link(url, source).unwrap())
                .collect(),
            tags: Vec::new(),
            images: Vec::new(),
            score: 100,
        }
    }
}
