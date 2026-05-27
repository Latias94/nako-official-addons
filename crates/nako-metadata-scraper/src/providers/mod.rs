use async_trait::async_trait;

use crate::config::ProviderId;
use crate::engine::{MetadataQuery, ProviderMetadataCandidate, av::AvNumberRoute};

pub mod airav;
pub mod anilist;
pub mod avsox;
pub mod bangumi;
pub mod browser_worker;
pub mod caribbean;
pub mod dmm;
pub mod douban;
pub mod fc2;
pub mod fc2ppvdb;
pub mod fixture;
pub mod http_runtime;
pub mod jav321;
pub mod javbus;
pub mod javdb;
pub mod javlibrary;
pub mod mgstage;
mod official_uncensored;
pub mod onepondo;
pub mod prestige;
mod registry;
mod rendered_av;
#[cfg(test)]
pub(crate) mod rendered_av_fixture;
mod rendered_page;
mod rendered_recipe;
mod rendered_search_av;
mod search_policy;
pub mod tenmusume;
pub mod theporndb;
pub mod tmdb;
pub mod xcity;

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
        xcity::catalog_entry(),
        fc2::catalog_entry(),
        fc2ppvdb::catalog_entry(),
        caribbean::catalog_entry(),
        onepondo::catalog_entry(),
        tenmusume::catalog_entry(),
        jav321::catalog_entry(),
        javbus::catalog_entry(),
        javlibrary::catalog_entry(),
        airav::catalog_entry(),
        avsox::catalog_entry(),
        mgstage::catalog_entry(),
        prestige::catalog_entry(),
        theporndb::catalog_entry(),
        anilist::catalog_entry(),
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
