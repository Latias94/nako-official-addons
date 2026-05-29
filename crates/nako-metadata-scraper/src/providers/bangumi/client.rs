use crate::{
    config::BangumiProviderConfig,
    providers::http_runtime::{
        ProviderHttpResult, ProviderHttpRuntime, ProviderHttpRuntimeConfig, ProviderHttpTransport,
        ReqwestProviderHttpTransport,
    },
};

use super::BangumiMetadataProvider;

impl BangumiMetadataProvider<ReqwestProviderHttpTransport> {
    pub fn new(config: BangumiProviderConfig) -> ProviderHttpResult<Self> {
        let runtime = ProviderHttpRuntime::new(ProviderHttpRuntimeConfig {
            user_agent: config.user_agent.clone(),
            proxy_url: config.proxy_url.clone(),
            ..ProviderHttpRuntimeConfig::default()
        })?;
        Ok(Self { config, runtime })
    }
}

impl<T> BangumiMetadataProvider<T>
where
    T: ProviderHttpTransport,
{
    #[must_use]
    pub fn with_runtime(config: BangumiProviderConfig, runtime: ProviderHttpRuntime<T>) -> Self {
        Self { config, runtime }
    }

    pub(super) fn endpoint(&self, path: impl AsRef<str>) -> String {
        let path = path.as_ref();
        format!(
            "{}/{}",
            self.config.api_base_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }

    pub(super) fn bearer_headers(&self) -> Vec<(String, String)> {
        self.config
            .access_token
            .as_ref()
            .map(|token| vec![("authorization".to_owned(), format!("Bearer {token}"))])
            .unwrap_or_default()
    }
}
