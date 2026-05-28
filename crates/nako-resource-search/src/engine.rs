use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
};

use async_trait::async_trait;
use thiserror::Error;

use crate::{
    Config,
    domain::{
        MergedResourceLink, ProviderExecutionStatus, RESOURCE_SEARCH_RESPONSE_SCHEMA,
        ResourceLinkType, ResourceSearchProviderExecution, ResourceSearchQuery,
        ResourceSearchRequest, ResourceSearchResponse, ResourceSearchResult,
    },
    providers::{ProviderDiagnostic, ProviderRegistry},
};

#[async_trait]
pub trait ResourceSearchProvider: Send + Sync {
    fn id(&self) -> &'static str;

    fn priority(&self) -> u16 {
        100
    }

    async fn search(
        &self,
        query: &ResourceSearchQuery,
    ) -> anyhow::Result<Vec<ResourceSearchResult>>;
}

#[derive(Clone)]
pub struct ResourceSearchRuntime {
    config: Config,
    provider_registry: ProviderRegistry,
    providers: Vec<Arc<dyn ResourceSearchProvider>>,
}

impl ResourceSearchRuntime {
    #[must_use]
    pub fn new(config: Config) -> Self {
        let provider_registry = ProviderRegistry::from_config(&config);
        let providers = provider_registry.enabled_providers();

        Self {
            config,
            provider_registry,
            providers,
        }
    }

    #[must_use]
    pub fn provider_count(&self) -> usize {
        self.providers.len()
    }

    #[must_use]
    pub fn provider_ids(&self) -> Vec<&'static str> {
        self.providers
            .iter()
            .map(|provider| provider.id())
            .collect()
    }

    #[must_use]
    pub fn provider_diagnostics(&self) -> Vec<ProviderDiagnostic> {
        self.provider_registry.diagnostics()
    }

    pub async fn search(
        &self,
        request: ResourceSearchRequest,
    ) -> Result<ResourceSearchResponse, ResourceSearchError> {
        let query = request
            .to_query(self.config.default_limit, self.config.max_limit)
            .ok_or(ResourceSearchError::EmptyQuery)?;

        let mut results = Vec::new();
        let mut provider_executions = Vec::new();
        for provider in &self.providers {
            if !query.source_requested(provider.id()) {
                provider_executions.push(ResourceSearchProviderExecution {
                    provider_id: provider.id().to_owned(),
                    status: ProviderExecutionStatus::Skipped,
                    result_count: 0,
                    safe_message: Some("source_not_requested".to_owned()),
                });
                continue;
            }

            match provider.search(&query).await {
                Ok(mut provider_results) => {
                    let result_count = provider_results.len();
                    results.append(&mut provider_results);
                    provider_executions.push(ResourceSearchProviderExecution {
                        provider_id: provider.id().to_owned(),
                        status: ProviderExecutionStatus::Ok,
                        result_count,
                        safe_message: None,
                    });
                }
                Err(error) => provider_executions.push(ResourceSearchProviderExecution {
                    provider_id: provider.id().to_owned(),
                    status: ProviderExecutionStatus::Error,
                    result_count: 0,
                    safe_message: Some(format!("{error:#}")),
                }),
            }
        }

        let (results, merged_by_type) = fuse_results(results, &query.link_types, query.limit);

        Ok(ResourceSearchResponse {
            schema: RESOURCE_SEARCH_RESPONSE_SCHEMA.to_owned(),
            query: query.query,
            total: results.len(),
            results,
            merged_by_type,
            provider_executions,
        })
    }
}

#[derive(Debug, Error)]
pub enum ResourceSearchError {
    #[error("empty resource search query")]
    EmptyQuery,
}

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
    use crate::{domain::ResourceSearchRequest, links::resource_link};

    use super::*;

    #[tokio::test]
    async fn runtime_search_returns_fixture_results_and_grouped_links() {
        let runtime = ResourceSearchRuntime::new(Config::default());
        let response = runtime
            .search(ResourceSearchRequest {
                query: "demo movie".to_owned(),
                ..ResourceSearchRequest::default()
            })
            .await
            .unwrap();

        assert_eq!(response.schema, RESOURCE_SEARCH_RESPONSE_SCHEMA);
        assert_eq!(response.query, "demo movie");
        assert_eq!(response.provider_executions.len(), 1);
        assert_eq!(
            response.provider_executions[0].status,
            ProviderExecutionStatus::Ok
        );
        assert_eq!(response.total, 2);
        assert!(
            response
                .merged_by_type
                .contains_key(&ResourceLinkType::Quark)
        );
        assert!(
            response
                .merged_by_type
                .contains_key(&ResourceLinkType::Magnet)
        );
    }

    #[tokio::test]
    async fn runtime_search_can_filter_requested_source() {
        let runtime = ResourceSearchRuntime::new(Config::default());
        let response = runtime
            .search(ResourceSearchRequest {
                query: "demo movie".to_owned(),
                sources: vec!["missing".to_owned()],
                ..ResourceSearchRequest::default()
            })
            .await
            .unwrap();

        assert_eq!(response.total, 0);
        assert_eq!(
            response.provider_executions[0].status,
            ProviderExecutionStatus::Skipped
        );
    }

    #[test]
    fn runtime_registers_pansou_provider_only_when_active() {
        let mut inactive_config = Config::default();
        inactive_config.pansou.enabled = true;
        assert_eq!(
            ResourceSearchRuntime::new(inactive_config).provider_ids(),
            vec!["fixture"]
        );

        let mut active_config = Config::default();
        active_config.pansou.enabled = true;
        active_config.pansou.base_url = Some("http://127.0.0.1:8888".to_owned());
        assert_eq!(
            ResourceSearchRuntime::new(active_config).provider_ids(),
            vec!["fixture", "pansou_compatible"]
        );
    }

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
