use std::sync::Arc;

use thiserror::Error;

use crate::{
    Config,
    checkers::{ConservativeResourceLinkCheckProvider, ResourceLinkCheckProvider},
    domain::{
        ProviderExecutionFinality, ProviderExecutionStatus, ResourceLinkCheckRequest,
        ResourceLinkCheckResponse, ResourceLinkCheckStatus, ResourceSearchProviderExecution,
        ResourceSearchRequest, ResourceSearchResponse,
    },
    providers::{
        ProviderDiagnostic, ProviderRegistry, ProviderSearchFinality, ResourceSearchProvider,
    },
};

use super::fusion::fuse_results;

#[derive(Clone)]
pub struct ResourceSearchRuntime {
    config: Config,
    provider_registry: ProviderRegistry,
    providers: Vec<Arc<dyn ResourceSearchProvider>>,
    link_check_provider: Arc<dyn ResourceLinkCheckProvider>,
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
            link_check_provider: Arc::new(ConservativeResourceLinkCheckProvider),
        }
    }

    #[must_use]
    pub fn provider_count(&self) -> usize {
        self.providers.len()
    }

    #[must_use]
    pub fn active_provider_count(&self) -> usize {
        self.provider_registry.active_provider_count()
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

    #[must_use]
    pub fn link_check_provider_id(&self) -> &'static str {
        self.link_check_provider.id()
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
                    finality: ProviderExecutionFinality::Unknown,
                    safe_message: Some("source_not_requested".to_owned()),
                });
                continue;
            }

            match provider.search(&query).await {
                Ok(mut batch) => {
                    let result_count = batch.results.len();
                    let safe_message = batch.safe_message();
                    results.append(&mut batch.results);
                    provider_executions.push(ResourceSearchProviderExecution {
                        provider_id: provider.id().to_owned(),
                        status: ProviderExecutionStatus::Ok,
                        result_count,
                        finality: provider_finality(batch.finality),
                        safe_message,
                    });
                }
                Err(_error) => provider_executions.push(ResourceSearchProviderExecution {
                    provider_id: provider.id().to_owned(),
                    status: ProviderExecutionStatus::Error,
                    result_count: 0,
                    finality: ProviderExecutionFinality::Unknown,
                    safe_message: Some("provider_search_failed".to_owned()),
                }),
            }
        }

        let (results, merged_by_type) = fuse_results(results, &query.link_types, query.limit);

        Ok(ResourceSearchResponse {
            query: query.query,
            total: results.len(),
            results,
            merged_by_type,
            provider_executions,
        })
    }

    pub async fn check_link(&self, request: ResourceLinkCheckRequest) -> ResourceLinkCheckResponse {
        match self.link_check_provider.check(&request).await {
            Ok(response) => response,
            Err(_error) => ResourceLinkCheckResponse::new(
                request.link.link_type,
                ResourceLinkCheckStatus::Error,
                current_time_ms(),
            )
            .with_safe_message("link_check_provider_failed")
            .with_safe_fact("checker_provider", self.link_check_provider.id()),
        }
    }
}

#[derive(Debug, Error)]
pub enum ResourceSearchError {
    #[error("empty resource search query")]
    EmptyQuery,
}

const fn provider_finality(finality: ProviderSearchFinality) -> ProviderExecutionFinality {
    match finality {
        ProviderSearchFinality::Complete => ProviderExecutionFinality::Complete,
        ProviderSearchFinality::Partial => ProviderExecutionFinality::Partial,
    }
}

fn current_time_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis().try_into().unwrap_or(u64::MAX))
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{
        domain::{
            ResourceLink, ResourceLinkCheckRequest, ResourceLinkCheckStatus, ResourceLinkType,
            ResourceSearchQuery, ResourceSearchRequest,
        },
        providers::ProviderSearchBatch,
    };

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

        assert_eq!(response.query, "demo movie");
        assert_eq!(response.provider_executions.len(), 1);
        assert_eq!(
            response.provider_executions[0].status,
            ProviderExecutionStatus::Ok
        );
        assert_eq!(
            response.provider_executions[0].finality,
            ProviderExecutionFinality::Complete
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

    #[tokio::test]
    async fn runtime_provider_errors_use_redaction_safe_accounting() {
        let runtime = ResourceSearchRuntime::with_providers(
            Config::default(),
            vec![Arc::new(FailingProvider)],
        );
        let response = runtime
            .search(ResourceSearchRequest {
                query: "demo movie".to_owned(),
                ..ResourceSearchRequest::default()
            })
            .await
            .unwrap();

        assert_eq!(response.total, 0);
        assert_eq!(
            response.provider_executions[0].status,
            ProviderExecutionStatus::Error
        );
        assert_eq!(
            response.provider_executions[0].safe_message.as_deref(),
            Some("provider_search_failed")
        );
    }

    #[tokio::test]
    async fn runtime_check_link_uses_conservative_checker() {
        let runtime = ResourceSearchRuntime::new(Config::default());
        let response = runtime
            .check_link(ResourceLinkCheckRequest {
                link: ResourceLink {
                    url: "https://pan.quark.cn/s/demo".to_owned(),
                    normalized_url: "https://pan.quark.cn/s/demo".to_owned(),
                    link_type: ResourceLinkType::Quark,
                    source: "fixture".to_owned(),
                    password: None,
                    note: None,
                },
                refresh: false,
            })
            .await;

        assert_eq!(response.status, ResourceLinkCheckStatus::Reachable);
        assert_eq!(
            response.safe_facts.get("live_network").map(String::as_str),
            Some("false")
        );
    }

    #[async_trait::async_trait]
    impl ResourceSearchProvider for FailingProvider {
        fn id(&self) -> &'static str {
            "failing"
        }

        async fn search(
            &self,
            _query: &ResourceSearchQuery,
        ) -> anyhow::Result<ProviderSearchBatch> {
            anyhow::bail!("secret-token-must-not-leak")
        }
    }

    struct FailingProvider;

    impl ResourceSearchRuntime {
        fn with_providers(config: Config, providers: Vec<Arc<dyn ResourceSearchProvider>>) -> Self {
            let provider_registry = ProviderRegistry::from_config(&config);

            Self {
                config,
                provider_registry,
                providers,
                link_check_provider: Arc::new(ConservativeResourceLinkCheckProvider),
            }
        }
    }
}
