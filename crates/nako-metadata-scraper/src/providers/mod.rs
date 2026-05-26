use async_trait::async_trait;

use crate::config::ProviderId;
use crate::engine::{MetadataQuery, ProviderMetadataCandidate, av::AvNumberRoute};

pub mod bangumi;
pub mod browser_worker;
pub mod dmm;
pub mod douban;
pub mod fc2;
pub mod fixture;
pub mod http_runtime;
pub mod javdb;
mod registry;
mod rendered_page;
mod search_policy;
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
        javdb::catalog_entry(),
        dmm::catalog_entry(),
        fc2::catalog_entry(),
    ]
}

#[async_trait]
pub trait MetadataProvider: Send + Sync {
    fn id(&self) -> ProviderId;

    fn supports_av_route(&self, _route: AvNumberRoute) -> bool {
        true
    }

    async fn suggest(
        &self,
        query: &MetadataQuery,
    ) -> anyhow::Result<Vec<ProviderMetadataCandidate>>;
}
