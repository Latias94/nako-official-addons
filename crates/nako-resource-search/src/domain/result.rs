use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::{ResourceLink, ResourceLinkType};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ResourceSearchResult {
    pub id: String,
    pub title: String,
    pub source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub links: Vec<ResourceLink>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub images: Vec<String>,
    #[serde(default)]
    pub score: u16,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MergedResourceLink {
    pub url: String,
    pub normalized_url: String,
    pub link_type: ResourceLinkType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    pub sources: Vec<String>,
}

impl MergedResourceLink {
    #[must_use]
    pub fn from_link(link: &ResourceLink) -> Self {
        Self {
            url: link.url.clone(),
            normalized_url: link.normalized_url.clone(),
            link_type: link.link_type,
            password: link.password.clone(),
            note: link.note.clone(),
            sources: vec![link.source.clone()],
        }
    }

    pub fn merge_source(&mut self, source: &str) {
        if !self.sources.iter().any(|candidate| candidate == source) {
            self.sources.push(source.to_owned());
            self.sources.sort();
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderExecutionStatus {
    Ok,
    Error,
    Skipped,
}

impl ProviderExecutionStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Error => "error",
            Self::Skipped => "skipped",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderExecutionFinality {
    Complete,
    Partial,
    Unknown,
}

impl ProviderExecutionFinality {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Complete => "complete",
            Self::Partial => "partial",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ResourceSearchProviderExecution {
    pub provider_id: String,
    pub status: ProviderExecutionStatus,
    pub result_count: usize,
    pub finality: ProviderExecutionFinality,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub safe_message: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ResourceSearchResponse {
    pub query: String,
    pub total: usize,
    pub results: Vec<ResourceSearchResult>,
    pub merged_by_type: BTreeMap<ResourceLinkType, Vec<MergedResourceLink>>,
    pub provider_executions: Vec<ResourceSearchProviderExecution>,
}
