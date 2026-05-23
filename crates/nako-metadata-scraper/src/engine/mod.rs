use std::sync::Arc;

use nako_addon_protocol::{
    ADDON_PROTOCOL_VERSION, AddonArtifact, AddonResourceRequest, AddonResourceResponse,
};
use serde::Deserialize;

use crate::providers::MetadataProvider;

pub mod ranking;

pub use ranking::{
    CandidateEvidence, MetadataCandidate, ProviderCandidateFacts, ProviderExternalId,
    ProviderMetadataCandidate,
};

#[derive(Clone)]
pub struct MetadataScrapeRuntime {
    default_language: String,
    providers: Arc<Vec<Box<dyn MetadataProvider>>>,
}

impl MetadataScrapeRuntime {
    #[must_use]
    pub fn new(
        default_language: impl Into<String>,
        providers: Vec<Box<dyn MetadataProvider>>,
    ) -> Self {
        Self {
            default_language: default_language.into(),
            providers: Arc::new(providers),
        }
    }

    pub async fn scrape(&self, request: AddonResourceRequest) -> AddonResourceResponse {
        let query = MetadataQuery::from_payload(&request.payload, &self.default_language);
        let candidates = self.suggest_candidates(&query).await;
        let payload = serde_json::json!({
            "query": {
                "title": query.title,
                "year": query.year,
                "language": query.language
            },
            "candidates": candidates
        });

        AddonResourceResponse {
            protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
            addon_id: request.addon_id,
            resource: request.resource,
            request_id: request.request_id,
            payload: payload.clone(),
            artifacts: vec![AddonArtifact {
                kind: "metadata_suggestion".to_owned(),
                payload,
            }],
        }
    }

    async fn suggest_candidates(&self, query: &MetadataQuery) -> Vec<MetadataCandidate> {
        let mut candidates = Vec::new();

        for provider in self.providers.iter() {
            match provider.suggest(query).await {
                Ok(provider_candidates) => candidates.extend(
                    provider_candidates
                        .into_iter()
                        .map(|candidate| ranking::rank_candidate(query, candidate)),
                ),
                Err(error) => {
                    tracing::warn!(provider = provider.id().as_str(), %error, "metadata provider failed")
                }
            }
        }

        candidates.sort_by(|left, right| {
            right
                .confidence_milli
                .cmp(&left.confidence_milli)
                .then_with(|| left.provider.cmp(&right.provider))
                .then_with(|| left.provider_id.cmp(&right.provider_id))
        });
        candidates
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct MetadataQuery {
    pub title: String,
    pub year: Option<i32>,
    pub language: String,
    pub external_ids: Vec<QueryExternalId>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct QueryExternalId {
    pub provider: String,
    pub value: String,
}

impl MetadataQuery {
    #[must_use]
    pub fn from_payload(payload: &serde_json::Value, default_language: &str) -> Self {
        let raw_title = payload
            .get("title")
            .and_then(serde_json::Value::as_str)
            .or_else(|| payload.get("name").and_then(serde_json::Value::as_str))
            .unwrap_or("Unknown Title")
            .trim();
        let title = normalize_title(raw_title);
        let year = payload
            .get("year")
            .and_then(serde_json::Value::as_i64)
            .and_then(|value| i32::try_from(value).ok());
        let language = payload
            .get("language")
            .and_then(serde_json::Value::as_str)
            .unwrap_or(default_language)
            .to_owned();
        let external_ids = external_ids_from_payload(payload);

        Self {
            title: if title.is_empty() {
                "Unknown Title".to_owned()
            } else {
                title
            },
            year,
            language,
            external_ids,
        }
    }
}

fn normalize_title(title: &str) -> String {
    title.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn external_ids_from_payload(payload: &serde_json::Value) -> Vec<QueryExternalId> {
    if let Some(values) = payload
        .get("external_ids")
        .and_then(serde_json::Value::as_object)
    {
        return values
            .iter()
            .filter_map(|(provider, value)| {
                value.as_str().map(|value| QueryExternalId {
                    provider: provider.clone(),
                    value: value.to_owned(),
                })
            })
            .collect();
    }

    payload
        .get("external_ids")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|value| {
            Some(QueryExternalId {
                provider: value.get("provider")?.as_str()?.to_owned(),
                value: value.get("value")?.as_str()?.to_owned(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use nako_addon_protocol::{AddonMetadataPatch, AddonResource};

    use super::*;
    use crate::{config::ProviderId, providers::MetadataProvider};

    struct CandidateProvider {
        provider_id: &'static str,
        title: &'static str,
        year: Option<i32>,
    }

    #[async_trait]
    impl MetadataProvider for CandidateProvider {
        fn id(&self) -> ProviderId {
            ProviderId::Fixture
        }

        async fn suggest(
            &self,
            query: &MetadataQuery,
        ) -> anyhow::Result<Vec<ProviderMetadataCandidate>> {
            Ok(vec![ProviderMetadataCandidate {
                provider: self.id().as_str().to_owned(),
                provider_id: self.provider_id.to_owned(),
                patch: AddonMetadataPatch {
                    title: Some(query.title.clone()),
                    original_title: None,
                    sort_title: None,
                    overview: None,
                    release_date: self.year.map(|year| format!("{year}-01-01")),
                    runtime_minutes: None,
                    tagline: None,
                    genres: None,
                    tags: Some(vec![query.language.clone()]),
                },
                facts: ProviderCandidateFacts {
                    title: Some(self.title.to_owned()),
                    release_year: self.year,
                    language: Some(query.language.clone()),
                    external_ids: Vec::new(),
                    provider_note: Some("test candidate".to_owned()),
                },
            }])
        }
    }

    struct FailingProvider;

    #[async_trait]
    impl MetadataProvider for FailingProvider {
        fn id(&self) -> ProviderId {
            ProviderId::Fixture
        }

        async fn suggest(
            &self,
            _query: &MetadataQuery,
        ) -> anyhow::Result<Vec<ProviderMetadataCandidate>> {
            anyhow::bail!("synthetic provider failure")
        }
    }

    #[tokio::test]
    async fn runtime_normalizes_request_and_shapes_metadata_response() {
        let runtime = MetadataScrapeRuntime::new(
            "zh-CN",
            vec![Box::new(CandidateProvider {
                provider_id: "fixture:matrix",
                title: "The Matrix",
                year: Some(1999),
            })],
        );

        let response = runtime
            .scrape(AddonResourceRequest {
                protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
                addon_id: "addon-1".to_owned(),
                resource: AddonResource::Metadata,
                request_id: "request-1".to_owned(),
                payload: serde_json::json!({"name": "  The   Matrix  ", "year": 1999}),
            })
            .await;

        assert_eq!(response.request_id, "request-1");
        assert_eq!(response.artifacts[0].kind, "metadata_suggestion");
        assert_eq!(response.payload["query"]["title"], "The Matrix");
        assert_eq!(response.payload["query"]["language"], "zh-CN");
        assert_eq!(
            response.payload["candidates"][0]["patch"]["release_date"],
            "1999-01-01"
        );
        assert_eq!(response.payload["candidates"][0]["confidence_milli"], 830);
        assert_eq!(response.artifacts[0].payload, response.payload);
    }

    #[tokio::test]
    async fn ranking_evidence_runtime_sorts_candidates_and_skips_failed_providers() {
        let runtime = MetadataScrapeRuntime::new(
            "en-US",
            vec![
                Box::new(CandidateProvider {
                    provider_id: "fixture:low",
                    title: "Other Movie",
                    year: Some(2001),
                }),
                Box::new(FailingProvider),
                Box::new(CandidateProvider {
                    provider_id: "fixture:high",
                    title: "Movie",
                    year: None,
                }),
            ],
        );

        let response = runtime
            .scrape(AddonResourceRequest {
                protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
                addon_id: "addon-1".to_owned(),
                resource: AddonResource::Metadata,
                request_id: "request-1".to_owned(),
                payload: serde_json::json!({"title": "Movie"}),
            })
            .await;

        let candidates = response.payload["candidates"].as_array().unwrap();
        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[0]["provider_id"], "fixture:high");
        assert_eq!(candidates[1]["provider_id"], "fixture:low");
    }

    #[test]
    fn ranking_evidence_metadata_query_parses_external_ids() {
        let query = MetadataQuery::from_payload(
            &serde_json::json!({
                "title": "Movie",
                "external_ids": {
                    "imdb": "tt0133093",
                    "tmdb": "603"
                }
            }),
            "en-US",
        );

        assert_eq!(
            query.external_ids,
            vec![
                QueryExternalId {
                    provider: "imdb".to_owned(),
                    value: "tt0133093".to_owned(),
                },
                QueryExternalId {
                    provider: "tmdb".to_owned(),
                    value: "603".to_owned(),
                }
            ]
        );
    }
}
