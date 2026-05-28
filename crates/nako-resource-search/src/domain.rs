mod link;
mod query;
mod result;

pub use link::{ResourceLink, ResourceLinkType};
pub use query::{ResourceSearchIntent, ResourceSearchQuery, ResourceSearchRequest};
pub use result::{
    MergedResourceLink, ProviderExecutionStatus, RESOURCE_SEARCH_RESPONSE_SCHEMA,
    ResourceSearchProviderExecution, ResourceSearchResponse, ResourceSearchResult,
};

pub const RESOURCE_SEARCH_REQUEST_SCHEMA: &str = "nako.official.resource-search.alpha.request.v1";
