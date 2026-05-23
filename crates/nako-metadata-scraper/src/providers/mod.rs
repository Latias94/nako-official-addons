use async_trait::async_trait;

use crate::config::ProviderId;
use crate::engine::{MetadataQuery, ProviderMetadataCandidate};

pub mod bangumi;
pub mod fixture;
pub mod http_runtime;
mod registry;
pub mod tmdb;

pub use registry::{ProviderDescriptor, ProviderDiagnostics, ProviderRegistry, ProviderStatus};

#[async_trait]
pub trait MetadataProvider: Send + Sync {
    fn id(&self) -> ProviderId;

    async fn suggest(
        &self,
        query: &MetadataQuery,
    ) -> anyhow::Result<Vec<ProviderMetadataCandidate>>;
}
