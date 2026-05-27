use std::{collections::BTreeMap, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::links::{classify_resource_url, normalize_resource_url};

pub const RESOURCE_SEARCH_REQUEST_SCHEMA: &str = "nako.official.resource-search.alpha.request.v1";
pub const RESOURCE_SEARCH_RESPONSE_SCHEMA: &str = "nako.official.resource-search.alpha.response.v1";

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceLinkType {
    Aliyun,
    Baidu,
    Quark,
    Tianyi,
    Uc,
    Mobile,
    #[serde(rename = "115")]
    OneOneFive,
    Pikpak,
    Xunlei,
    #[serde(rename = "123")]
    OneTwoThree,
    Magnet,
    Ed2k,
    Web,
    Other,
}

impl ResourceLinkType {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Aliyun => "aliyun",
            Self::Baidu => "baidu",
            Self::Quark => "quark",
            Self::Tianyi => "tianyi",
            Self::Uc => "uc",
            Self::Mobile => "mobile",
            Self::OneOneFive => "115",
            Self::Pikpak => "pikpak",
            Self::Xunlei => "xunlei",
            Self::OneTwoThree => "123",
            Self::Magnet => "magnet",
            Self::Ed2k => "ed2k",
            Self::Web => "web",
            Self::Other => "other",
        }
    }
}

impl FromStr for ResourceLinkType {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "aliyun" | "ali" => Ok(Self::Aliyun),
            "baidu" => Ok(Self::Baidu),
            "quark" => Ok(Self::Quark),
            "tianyi" | "189" => Ok(Self::Tianyi),
            "uc" => Ok(Self::Uc),
            "mobile" | "139" => Ok(Self::Mobile),
            "115" => Ok(Self::OneOneFive),
            "pikpak" => Ok(Self::Pikpak),
            "xunlei" => Ok(Self::Xunlei),
            "123" | "123pan" => Ok(Self::OneTwoThree),
            "magnet" => Ok(Self::Magnet),
            "ed2k" => Ok(Self::Ed2k),
            "web" => Ok(Self::Web),
            "other" | "others" => Ok(Self::Other),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct ResourceSearchRequest {
    pub query: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sources: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub link_types: Vec<ResourceLinkType>,
    #[serde(default)]
    pub refresh: bool,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub ext: serde_json::Value,
}

impl ResourceSearchRequest {
    #[must_use]
    pub fn normalized_query(&self) -> Option<String> {
        let query = self.query.trim();
        if query.is_empty() {
            None
        } else {
            Some(query.to_owned())
        }
    }

    #[must_use]
    pub fn effective_limit(&self, default_limit: usize, max_limit: usize) -> usize {
        self.limit.unwrap_or(default_limit).clamp(1, max_limit)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ResourceSearchQuery {
    pub query: String,
    pub limit: usize,
    pub sources: Vec<String>,
    pub link_types: Vec<ResourceLinkType>,
    pub refresh: bool,
    pub ext: serde_json::Value,
}

impl ResourceSearchQuery {
    #[must_use]
    pub fn source_requested(&self, provider_id: &str) -> bool {
        self.sources.is_empty()
            || self
                .sources
                .iter()
                .any(|source| source.eq_ignore_ascii_case(provider_id))
    }
}

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
pub struct ResourceLink {
    pub url: String,
    pub normalized_url: String,
    pub link_type: ResourceLinkType,
    pub source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

impl ResourceLink {
    #[must_use]
    pub fn new(url: impl Into<String>, source: impl Into<String>) -> Option<Self> {
        let url = url.into();
        let normalized_url = normalize_resource_url(&url)?;
        let link_type = classify_resource_url(&url);

        Some(Self {
            url: url.trim().to_owned(),
            normalized_url,
            link_type,
            source: source.into(),
            password: None,
            note: None,
        })
    }

    #[must_use]
    pub fn with_password(mut self, password: impl Into<String>) -> Self {
        let password = password.into();
        if !password.trim().is_empty() {
            self.password = Some(password.trim().to_owned());
        }
        self
    }

    #[must_use]
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        let note = note.into();
        if !note.trim().is_empty() {
            self.note = Some(note.trim().to_owned());
        }
        self
    }
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

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ResourceSearchProviderExecution {
    pub provider_id: String,
    pub status: ProviderExecutionStatus,
    pub result_count: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub safe_message: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ResourceSearchResponse {
    pub schema: String,
    pub query: String,
    pub total: usize,
    pub results: Vec<ResourceSearchResult>,
    pub merged_by_type: BTreeMap<ResourceLinkType, Vec<MergedResourceLink>>,
    pub provider_executions: Vec<ResourceSearchProviderExecution>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numeric_link_types_keep_wire_names() {
        assert_eq!(
            serde_json::to_value(ResourceLinkType::OneOneFive).unwrap(),
            serde_json::json!("115")
        );
        assert_eq!(
            serde_json::to_value(ResourceLinkType::OneTwoThree).unwrap(),
            serde_json::json!("123")
        );
        assert_eq!(
            serde_json::from_value::<ResourceLinkType>(serde_json::json!("115")).unwrap(),
            ResourceLinkType::OneOneFive
        );
    }

    #[test]
    fn search_request_normalizes_query_and_limit() {
        let request = ResourceSearchRequest {
            query: "  movie title  ".to_owned(),
            limit: Some(200),
            ..ResourceSearchRequest::default()
        };

        assert_eq!(request.normalized_query().as_deref(), Some("movie title"));
        assert_eq!(request.effective_limit(20, 100), 100);
    }
}
