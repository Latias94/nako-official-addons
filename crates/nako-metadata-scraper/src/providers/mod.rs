use async_trait::async_trait;

use crate::config::ProviderId;
use crate::engine::{MetadataQuery, ProviderMetadataCandidate};

pub mod bangumi;
pub mod browser_worker;
pub mod douban;
pub mod fixture;
pub mod http_runtime;
mod registry;
pub mod tmdb;

pub use registry::{
    ProviderAssembly, ProviderBuildStatus, ProviderDescriptor, ProviderDiagnostics,
    ProviderRegistry, ProviderStatus,
};
pub(crate) use registry::{ProviderCatalogEntry, ProviderConfigInput};

#[must_use]
pub(crate) fn provider_catalog() -> Vec<ProviderCatalogEntry> {
    vec![
        fixture::catalog_entry(),
        tmdb::catalog_entry(),
        bangumi::catalog_entry(),
        browser_worker::catalog_entry(),
        douban::catalog_entry(),
    ]
}

#[async_trait]
pub trait MetadataProvider: Send + Sync {
    fn id(&self) -> ProviderId;

    async fn suggest(
        &self,
        query: &MetadataQuery,
    ) -> anyhow::Result<Vec<ProviderMetadataCandidate>>;
}
