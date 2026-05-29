use serde::{Deserialize, Serialize};

use super::ResourceLinkType;

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

    #[must_use]
    pub fn to_query(&self, default_limit: usize, max_limit: usize) -> Option<ResourceSearchQuery> {
        let query = self.normalized_query()?;
        Some(ResourceSearchQuery {
            intent: ResourceSearchIntent::infer(&query, &self.ext),
            query,
            limit: self.effective_limit(default_limit, max_limit),
            sources: self.sources.clone(),
            link_types: self.link_types.clone(),
            refresh: self.refresh,
            ext: self.ext.clone(),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ResourceSearchIntent {
    FreeText,
    MediaTitle {
        title: String,
        year: Option<i32>,
        media_kind: Option<String>,
    },
    ExternalId {
        kind: String,
        value: String,
    },
    ExactLink {
        url: String,
    },
}

impl ResourceSearchIntent {
    #[must_use]
    pub fn infer(query: &str, ext: &serde_json::Value) -> Self {
        if looks_like_resource_link(query) {
            return Self::ExactLink {
                url: query.to_owned(),
            };
        }

        if let Some(intent) = external_id_intent(ext) {
            return intent;
        }
        if let Some(intent) = media_title_intent(ext) {
            return intent;
        }

        Self::FreeText
    }

    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::FreeText => "free_text",
            Self::MediaTitle { .. } => "media_title",
            Self::ExternalId { .. } => "external_id",
            Self::ExactLink { .. } => "exact_link",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ResourceSearchQuery {
    pub intent: ResourceSearchIntent,
    pub query: String,
    pub limit: usize,
    pub sources: Vec<String>,
    pub link_types: Vec<ResourceLinkType>,
    pub refresh: bool,
    pub ext: serde_json::Value,
}

impl ResourceSearchQuery {
    #[must_use]
    pub fn free_text(query: impl Into<String>, limit: usize) -> Self {
        let query = query.into();
        Self {
            intent: ResourceSearchIntent::FreeText,
            query,
            limit,
            sources: Vec::new(),
            link_types: Vec::new(),
            refresh: false,
            ext: serde_json::Value::Null,
        }
    }

    #[must_use]
    pub fn source_requested(&self, provider_id: &str) -> bool {
        self.sources.is_empty()
            || self
                .sources
                .iter()
                .any(|source| source.eq_ignore_ascii_case(provider_id))
    }
}

fn looks_like_resource_link(query: &str) -> bool {
    let query = query.trim().to_ascii_lowercase();
    query.starts_with("magnet:?")
        || query.starts_with("ed2k://")
        || query.starts_with("http://")
        || query.starts_with("https://")
}

fn external_id_intent(ext: &serde_json::Value) -> Option<ResourceSearchIntent> {
    let object = ext.as_object()?;
    if let Some(external_id) = object
        .get("external_id")
        .and_then(serde_json::Value::as_object)
    {
        let kind = non_empty_value(external_id.get("kind")?)?;
        let value = non_empty_value(external_id.get("value")?)?;
        return Some(ResourceSearchIntent::ExternalId { kind, value });
    }

    let kind = object.get("external_id_kind").and_then(non_empty_value)?;
    let value = object.get("external_id_value").and_then(non_empty_value)?;
    Some(ResourceSearchIntent::ExternalId { kind, value })
}

fn media_title_intent(ext: &serde_json::Value) -> Option<ResourceSearchIntent> {
    let object = ext.as_object()?;
    let title = object
        .get("media_title")
        .or_else(|| object.get("title"))
        .and_then(non_empty_value)?;
    let year = object
        .get("year")
        .and_then(serde_json::Value::as_i64)
        .and_then(|year| i32::try_from(year).ok())
        .filter(|year| *year > 0);
    let media_kind = object.get("media_kind").and_then(non_empty_value);

    Some(ResourceSearchIntent::MediaTitle {
        title,
        year,
        media_kind,
    })
}

fn non_empty_value(value: &serde_json::Value) -> Option<String> {
    value.as_str().and_then(|value| {
        let value = value.trim();
        if value.is_empty() {
            None
        } else {
            Some(value.to_owned())
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_request_normalizes_query_limit_and_intent() {
        let request = ResourceSearchRequest {
            query: "  movie title  ".to_owned(),
            limit: Some(200),
            ..ResourceSearchRequest::default()
        };

        assert_eq!(request.normalized_query().as_deref(), Some("movie title"));
        assert_eq!(request.effective_limit(20, 100), 100);

        let query = request.to_query(20, 100).unwrap();
        assert_eq!(query.intent, ResourceSearchIntent::FreeText);
        assert_eq!(query.limit, 100);
    }

    #[test]
    fn search_intent_detects_exact_links_before_ext_context() {
        let intent = ResourceSearchIntent::infer(
            "magnet:?xt=urn:btih:abcdef",
            &serde_json::json!({
                "title": "Ignored",
                "year": 2026
            }),
        );

        assert_eq!(
            intent,
            ResourceSearchIntent::ExactLink {
                url: "magnet:?xt=urn:btih:abcdef".to_owned()
            }
        );
    }

    #[test]
    fn search_intent_detects_external_ids() {
        let intent = ResourceSearchIntent::infer(
            "demo",
            &serde_json::json!({
                "external_id": {
                    "kind": "tmdb",
                    "value": "123"
                }
            }),
        );

        assert_eq!(
            intent,
            ResourceSearchIntent::ExternalId {
                kind: "tmdb".to_owned(),
                value: "123".to_owned()
            }
        );
    }

    #[test]
    fn search_intent_detects_media_title_context() {
        let intent = ResourceSearchIntent::infer(
            "demo",
            &serde_json::json!({
                "media_title": "Demo Movie",
                "year": 2026,
                "media_kind": "movie"
            }),
        );

        assert_eq!(
            intent,
            ResourceSearchIntent::MediaTitle {
                title: "Demo Movie".to_owned(),
                year: Some(2026),
                media_kind: Some("movie".to_owned())
            }
        );
    }
}
