pub mod artwork;
pub mod av;
pub mod bulk;
mod native_writeback;
mod orchestration;
mod outcome;
mod query;
pub mod ranking;
pub(crate) mod resolver;
mod response;
mod runtime;
pub mod title;
mod writeback;

pub(crate) const MAX_CANDIDATES_PER_QUERY: usize = 12;

pub use artwork::{
    ArtworkCandidate, ArtworkWritebackResult, ArtworkWritebackStatus, ProviderArtworkCandidate,
    ProviderArtworkCandidateFacts,
};
pub use av::AvMetadataFacts;
pub use outcome::{ProviderOutcome, render_provider_note};
pub(crate) use query::ProviderRunPolicy;
pub use query::{
    ExternalIdValueKind, MetadataQuery, ProviderExternalIdCapability, ProviderFieldPolicy,
    QueryExternalId, QueryExternalIdAlias,
};
pub use ranking::{
    CandidateEvidence, MetadataCandidate, ProviderCandidateFacts, ProviderExternalId,
    ProviderMetadataCandidate,
};
pub use runtime::MetadataScrapeRuntime;
pub use writeback::{MetadataWritebackRequest, MetadataWritebackResult, MetadataWritebackStatus};
#[cfg(test)]
mod tests {
    use std::{
        collections::{HashSet, VecDeque},
        sync::{Arc, Mutex},
    };

    use async_trait::async_trait;
    use nako_addon_protocol::{
        ADDON_PROTOCOL_VERSION, AddonArtworkKind, AddonMetadataPatch, AddonResource,
        AddonResourceRequest,
    };

    use super::*;
    use crate::{
        config::ProviderId,
        nako_runtime::{
            NakoRuntimeClient, NakoRuntimeClientConfig, NakoRuntimeError, NakoRuntimeHttpRequest,
            NakoRuntimeHttpResponse, NakoRuntimeResult, NakoRuntimeTransport,
        },
        providers::MetadataProvider,
    };

    const TEST_EXTERNAL_ID_ALIASES: &[QueryExternalIdAlias] = &[
        QueryExternalIdAlias::new("tmdb_id", "tmdb", true),
        QueryExternalIdAlias::new("imdb_id", "imdb", true),
        QueryExternalIdAlias::new("bangumi_id", "bangumi", true),
        QueryExternalIdAlias::new("browser_worker_url", "browser_worker", false),
    ];
    const TEST_EXTERNAL_ID_CAPABILITIES: &[ProviderExternalIdCapability] = &[
        ProviderExternalIdCapability::new(
            "tmdb",
            ExternalIdValueKind::Numeric,
            true,
            true,
            &["tmdb_id"],
            true,
        ),
        ProviderExternalIdCapability::new(
            "imdb",
            ExternalIdValueKind::Opaque,
            true,
            true,
            &["imdb_id"],
            true,
        ),
        ProviderExternalIdCapability::new(
            "bangumi",
            ExternalIdValueKind::Numeric,
            true,
            true,
            &["bangumi_id"],
            true,
        ),
        ProviderExternalIdCapability::new(
            "av_number",
            ExternalIdValueKind::Opaque,
            true,
            true,
            &["av_number"],
            false,
        ),
        ProviderExternalIdCapability::new(
            "browser_worker",
            ExternalIdValueKind::Url,
            true,
            true,
            &["browser_worker_url"],
            false,
        ),
    ];

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
                    ..AddonMetadataPatch::default()
                },
                facts: ProviderCandidateFacts {
                    title: Some(self.title.to_owned()),
                    alternate_titles: Vec::new(),
                    release_year: self.year,
                    language: Some(query.language.clone()),
                    av: None,
                    community_score_milli: None,
                    community_vote_count: None,
                    external_ids: Vec::new(),
                    provider_outcomes: Vec::new(),
                    provider_note: Some("test candidate".to_owned()),
                },
                artwork_candidates: vec![ProviderArtworkCandidate {
                    provider: self.id().as_str().to_owned(),
                    provider_id: self.provider_id.to_owned(),
                    facts: ProviderArtworkCandidateFacts {
                        kind: AddonArtworkKind::Poster,
                        source_url: "https://example.test/poster.jpg".to_owned(),
                        language: None,
                        width: Some(1000),
                        height: Some(1500),
                    },
                }],
            }])
        }
    }

    struct FailingProvider;

    struct DuplicateProvider {
        candidate_count: usize,
    }

    struct ExternalIdCandidateProvider {
        candidate_provider: &'static str,
        provider_id: &'static str,
        title: &'static str,
        year: Option<i32>,
        external_ids: &'static [(&'static str, &'static str)],
    }

    struct CountingProvider {
        provider_id: ProviderId,
        candidate_provider: &'static str,
        candidate_id: &'static str,
        calls: Arc<Mutex<usize>>,
    }

    struct PolicyCandidateProvider {
        candidate_provider: &'static str,
        provider_id: &'static str,
        title: &'static str,
        overview: Option<&'static str>,
        tags: &'static [&'static str],
        av_actors: &'static [&'static str],
        av_studio: Option<&'static str>,
        av_wanted_count: Option<u32>,
        artwork: &'static [(&'static str, &'static str)],
        external_ids: &'static [(&'static str, &'static str)],
    }

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

    #[async_trait]
    impl MetadataProvider for DuplicateProvider {
        fn id(&self) -> ProviderId {
            ProviderId::Fixture
        }

        async fn suggest(
            &self,
            _query: &MetadataQuery,
        ) -> anyhow::Result<Vec<ProviderMetadataCandidate>> {
            let mut candidates = Vec::new();
            for index in 0..self.candidate_count {
                let provider_id = if index < 2 {
                    "fixture:duplicate".to_owned()
                } else {
                    format!("fixture:{index}")
                };
                candidates.push(ProviderMetadataCandidate {
                    provider: self.id().as_str().to_owned(),
                    provider_id,
                    patch: AddonMetadataPatch::default(),
                    facts: ProviderCandidateFacts {
                        title: Some(format!("Candidate {index}")),
                        alternate_titles: Vec::new(),
                        release_year: Some(2000 + index as i32),
                        language: Some("en-US".to_owned()),
                        av: None,
                        community_score_milli: None,
                        community_vote_count: None,
                        external_ids: Vec::new(),
                        provider_outcomes: Vec::new(),
                        provider_note: None,
                    },
                    artwork_candidates: Vec::new(),
                });
            }

            Ok(candidates)
        }
    }

    #[async_trait]
    impl MetadataProvider for ExternalIdCandidateProvider {
        fn id(&self) -> ProviderId {
            ProviderId::Fixture
        }

        async fn suggest(
            &self,
            _query: &MetadataQuery,
        ) -> anyhow::Result<Vec<ProviderMetadataCandidate>> {
            Ok(vec![ProviderMetadataCandidate {
                provider: self.candidate_provider.to_owned(),
                provider_id: self.provider_id.to_owned(),
                patch: AddonMetadataPatch {
                    title: Some(self.title.to_owned()),
                    original_title: None,
                    sort_title: None,
                    overview: None,
                    release_date: self.year.map(|year| format!("{year}-01-01")),
                    runtime_minutes: None,
                    tagline: None,
                    genres: None,
                    tags: None,
                    ..AddonMetadataPatch::default()
                },
                facts: ProviderCandidateFacts {
                    title: Some(self.title.to_owned()),
                    alternate_titles: Vec::new(),
                    release_year: self.year,
                    language: Some("en-US".to_owned()),
                    av: None,
                    community_score_milli: None,
                    community_vote_count: None,
                    external_ids: self
                        .external_ids
                        .iter()
                        .map(|(provider, value)| ProviderExternalId {
                            provider: (*provider).to_owned(),
                            value: (*value).to_owned(),
                        })
                        .collect(),
                    provider_outcomes: Vec::new(),
                    provider_note: None,
                },
                artwork_candidates: Vec::new(),
            }])
        }
    }

    #[async_trait]
    impl MetadataProvider for CountingProvider {
        fn id(&self) -> ProviderId {
            self.provider_id
        }

        fn supports_av_route(&self, route: av::AvNumberRoute) -> bool {
            match self.provider_id {
                ProviderId::Javdb => route != av::AvNumberRoute::Fc2,
                ProviderId::Dmm => route == av::AvNumberRoute::Censored,
                ProviderId::Fc2 => route == av::AvNumberRoute::Fc2,
                _ => true,
            }
        }

        async fn suggest(
            &self,
            query: &MetadataQuery,
        ) -> anyhow::Result<Vec<ProviderMetadataCandidate>> {
            *self.calls.lock().unwrap() += 1;
            Ok(vec![ProviderMetadataCandidate {
                provider: self.candidate_provider.to_owned(),
                provider_id: self.candidate_id.to_owned(),
                patch: AddonMetadataPatch {
                    title: Some(query.title.clone()),
                    original_title: None,
                    sort_title: None,
                    overview: None,
                    release_date: None,
                    runtime_minutes: None,
                    tagline: None,
                    genres: None,
                    tags: None,
                    ..AddonMetadataPatch::default()
                },
                facts: ProviderCandidateFacts {
                    title: Some(query.title.clone()),
                    alternate_titles: Vec::new(),
                    release_year: query.year,
                    language: Some(query.language.clone()),
                    av: None,
                    community_score_milli: None,
                    community_vote_count: None,
                    external_ids: vec![ProviderExternalId {
                        provider: "av_number".to_owned(),
                        value: query.title.clone(),
                    }],
                    provider_outcomes: Vec::new(),
                    provider_note: None,
                },
                artwork_candidates: Vec::new(),
            }])
        }
    }

    #[async_trait]
    impl MetadataProvider for PolicyCandidateProvider {
        fn id(&self) -> ProviderId {
            ProviderId::Fixture
        }

        async fn suggest(
            &self,
            _query: &MetadataQuery,
        ) -> anyhow::Result<Vec<ProviderMetadataCandidate>> {
            Ok(vec![ProviderMetadataCandidate {
                provider: self.candidate_provider.to_owned(),
                provider_id: self.provider_id.to_owned(),
                patch: AddonMetadataPatch {
                    title: Some(self.title.to_owned()),
                    original_title: None,
                    sort_title: None,
                    overview: self.overview.map(str::to_owned),
                    release_date: None,
                    runtime_minutes: None,
                    tagline: None,
                    genres: None,
                    tags: (!self.tags.is_empty())
                        .then(|| self.tags.iter().map(|tag| (*tag).to_owned()).collect()),
                    ..AddonMetadataPatch::default()
                },
                facts: ProviderCandidateFacts {
                    title: Some(self.title.to_owned()),
                    alternate_titles: Vec::new(),
                    release_year: None,
                    language: Some("zh-CN".to_owned()),
                    av: AvMetadataFacts {
                        actors: self
                            .av_actors
                            .iter()
                            .map(|actor| (*actor).to_owned())
                            .collect(),
                        all_actors: self
                            .av_actors
                            .iter()
                            .map(|actor| (*actor).to_owned())
                            .collect(),
                        studio: self.av_studio.map(str::to_owned),
                        wanted_count: self.av_wanted_count,
                        ..AvMetadataFacts::default()
                    }
                    .non_empty(),
                    community_score_milli: None,
                    community_vote_count: None,
                    external_ids: self
                        .external_ids
                        .iter()
                        .map(|(provider, value)| ProviderExternalId {
                            provider: (*provider).to_owned(),
                            value: (*value).to_owned(),
                        })
                        .collect(),
                    provider_outcomes: Vec::new(),
                    provider_note: None,
                },
                artwork_candidates: self
                    .artwork
                    .iter()
                    .enumerate()
                    .map(|(index, (kind, source_url))| ProviderArtworkCandidate {
                        provider: self.candidate_provider.to_owned(),
                        provider_id: format!("{}:artwork:{index}", self.provider_id),
                        facts: ProviderArtworkCandidateFacts {
                            kind: match *kind {
                                "backdrop" => AddonArtworkKind::Backdrop,
                                _ => AddonArtworkKind::Poster,
                            },
                            source_url: (*source_url).to_owned(),
                            language: None,
                            width: None,
                            height: None,
                        },
                    })
                    .collect(),
            }])
        }
    }

    #[tokio::test]
    async fn runtime_normalizes_request_and_shapes_metadata_response() {
        let runtime = MetadataScrapeRuntime::<FakeTransport>::new(
            "zh-CN",
            vec![Box::new(CandidateProvider {
                provider_id: "fixture:matrix",
                title: "The Matrix",
                year: Some(1999),
            })],
            None,
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
        assert!(response.payload.get("writeback").is_none());
        assert_eq!(
            response.payload["candidates"][0]["patch"]["release_date"],
            "1999-01-01"
        );
        assert_eq!(response.payload["candidates"][0]["confidence_milli"], 830);
        assert_eq!(
            response.payload["candidates"][0]["artwork_candidates"][0]["confidence_milli"],
            830
        );
        assert_eq!(
            response.payload["candidates"][0]["artwork_candidates"][0]["artwork"]["kind"],
            "poster"
        );
        assert_eq!(response.artifacts[0].payload, response.payload);
    }

    #[tokio::test]
    async fn runtime_exposes_redaction_safe_av_query_facts() {
        let runtime = MetadataScrapeRuntime::<FakeTransport>::new(
            "zh-CN",
            vec![Box::new(CandidateProvider {
                provider_id: "fixture:av",
                title: "FC2-1723984",
                year: None,
            })],
            None,
        );

        let response = runtime
            .scrape(AddonResourceRequest {
                protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
                addon_id: "addon-1".to_owned(),
                resource: AddonResource::Metadata,
                request_id: "request-1".to_owned(),
                payload: serde_json::json!({
                    "path": "D:\\Private\\FC2PPV-1723984-4K.mp4"
                }),
            })
            .await;

        assert_eq!(response.payload["query"]["title"], "FC2-1723984");
        assert_eq!(response.payload["query"]["av"]["number"], "FC2-1723984");
        assert_eq!(response.payload["query"]["av"]["route"], "fc2");
        assert_eq!(response.payload["query"]["av"]["source"], "path");
        assert!(
            !serde_json::to_string(&response.payload)
                .unwrap()
                .contains("Private")
        );
    }

    #[tokio::test]
    async fn ranking_evidence_runtime_sorts_candidates_and_skips_failed_providers() {
        let runtime = MetadataScrapeRuntime::<FakeTransport>::new(
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
            None,
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
        assert_eq!(
            response.payload["provider_execution"]["selected_provider_ids"],
            serde_json::json!(["fixture"])
        );
        assert_eq!(
            response.payload["provider_execution"]["failed_provider_ids"],
            serde_json::json!(["fixture"])
        );
        assert_eq!(
            response.payload["provider_execution"]["providers"][1]["status"],
            "failed"
        );
        assert_eq!(
            response.payload["provider_execution"]["providers"][1]["safe_failure_reason"],
            "provider_error"
        );

        let payload_text = serde_json::to_string(&response.payload).unwrap();
        assert!(!payload_text.contains("synthetic provider failure"));
    }

    #[tokio::test]
    async fn ranking_evidence_runtime_deduplicates_and_caps_candidates() {
        let runtime = MetadataScrapeRuntime::<FakeTransport>::with_external_id_aliases(
            "en-US",
            TEST_EXTERNAL_ID_ALIASES.to_vec(),
            vec![Box::new(DuplicateProvider {
                candidate_count: 14,
            })],
            None,
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
        assert_eq!(candidates.len(), MAX_CANDIDATES_PER_QUERY);

        let mut provider_ids = HashSet::new();
        for candidate in candidates {
            let provider_id = candidate["provider_id"].as_str().unwrap().to_owned();
            assert!(provider_ids.insert(provider_id));
        }
    }

    #[tokio::test]
    async fn resolver_runtime_clusters_candidates_with_shared_external_ids() {
        let runtime = MetadataScrapeRuntime::<FakeTransport>::with_external_id_capabilities(
            "en-US",
            TEST_EXTERNAL_ID_CAPABILITIES.to_vec(),
            vec![
                Box::new(ExternalIdCandidateProvider {
                    candidate_provider: "douban",
                    provider_id: "douban:subject:1291843",
                    title: "Other Movie",
                    year: Some(2001),
                    external_ids: &[("imdb", "TT0133093"), ("douban", "1291843")],
                }),
                Box::new(ExternalIdCandidateProvider {
                    candidate_provider: "tmdb",
                    provider_id: "tmdb:movie:603",
                    title: "The Matrix",
                    year: Some(1999),
                    external_ids: &[("tmdb", "603"), ("imdb", "tt0133093")],
                }),
            ],
            None,
        );

        let response = runtime
            .scrape(AddonResourceRequest {
                protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
                addon_id: "addon-1".to_owned(),
                resource: AddonResource::Metadata,
                request_id: "request-1".to_owned(),
                payload: serde_json::json!({"title": "The Matrix", "year": 1999}),
            })
            .await;

        let candidates = response.payload["candidates"].as_array().unwrap();
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0]["provider"], "tmdb");
        assert_eq!(candidates[0]["provider_id"], "tmdb:movie:603");
        assert_eq!(candidates[0]["patch"]["title"], "The Matrix");
    }

    #[tokio::test]
    async fn runtime_exposes_redaction_safe_provider_source_evidence_for_merged_av_candidates() {
        let runtime = MetadataScrapeRuntime::<FakeTransport>::with_external_id_capabilities(
            "zh-CN",
            TEST_EXTERNAL_ID_CAPABILITIES.to_vec(),
            vec![
                Box::new(ExternalIdCandidateProvider {
                    candidate_provider: "javdb",
                    provider_id: "javdb:movie:abc123",
                    title: "SSNI-644",
                    year: None,
                    external_ids: &[("av_number", "SSNI-644"), ("javdb", "abc123")],
                }),
                Box::new(ExternalIdCandidateProvider {
                    candidate_provider: "dmm",
                    provider_id: "dmm:cid:abc123",
                    title: "SSNI-644",
                    year: None,
                    external_ids: &[("av_number", "ssni-644")],
                }),
            ],
            None,
        );

        let response = runtime
            .scrape(AddonResourceRequest {
                protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
                addon_id: "addon-1".to_owned(),
                resource: AddonResource::Metadata,
                request_id: "request-1".to_owned(),
                payload: serde_json::json!({"av_number": "SSNI-644"}),
            })
            .await;

        let candidates = response.payload["candidates"].as_array().unwrap();
        assert_eq!(candidates.len(), 1);
        assert_eq!(
            candidates[0]["evidence"]["provider_sources"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
        assert_eq!(
            candidates[0]["evidence"]["merge_reasons"][0]["provider"],
            "av_number"
        );
        assert!(
            candidates[0]["evidence"]["field_sources"]
                .as_array()
                .unwrap()
                .iter()
                .any(|source| source["field"] == "title")
        );

        let evidence_text = serde_json::to_string(&candidates[0]["evidence"]).unwrap();
        assert!(!evidence_text.contains("SSNI-644"));
        assert!(!evidence_text.contains("ssni-644"));
    }

    #[tokio::test]
    async fn runtime_applies_default_av_provider_field_policy_within_merged_cluster() {
        let runtime = MetadataScrapeRuntime::<FakeTransport>::with_external_id_capabilities(
            "zh-CN",
            TEST_EXTERNAL_ID_CAPABILITIES.to_vec(),
            vec![
                Box::new(PolicyCandidateProvider {
                    candidate_provider: "javdb",
                    provider_id: "javdb:movie:abc123",
                    title: "SSNI-644 JavDB Title",
                    overview: Some("JavDB overview"),
                    tags: &["javdb-tag"],
                    av_actors: &["JavDB Actor"],
                    av_studio: Some("JavDB Studio"),
                    av_wanted_count: Some(77),
                    artwork: &[
                        ("poster", "https://img.example/javdb-poster.jpg"),
                        ("backdrop", "https://img.example/javdb-backdrop.jpg"),
                    ],
                    external_ids: &[("av_number", "SSNI-644"), ("javdb", "abc123")],
                }),
                Box::new(PolicyCandidateProvider {
                    candidate_provider: "dmm",
                    provider_id: "dmm:cid:ssni00644",
                    title: "DMM Official Title",
                    overview: Some("DMM overview"),
                    tags: &["dmm-tag"],
                    av_actors: &["DMM Actor"],
                    av_studio: Some("DMM Studio"),
                    av_wanted_count: None,
                    artwork: &[
                        ("poster", "https://img.example/dmm-poster.jpg"),
                        ("backdrop", "https://img.example/dmm-backdrop.jpg"),
                    ],
                    external_ids: &[("av_number", "ssni-644")],
                }),
            ],
            None,
        );

        let response = runtime
            .scrape(AddonResourceRequest {
                protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
                addon_id: "addon-1".to_owned(),
                resource: AddonResource::Metadata,
                request_id: "request-1".to_owned(),
                payload: serde_json::json!({"av_number": "SSNI-644"}),
            })
            .await;

        let candidate = &response.payload["candidates"][0];
        assert_eq!(candidate["patch"]["title"], "DMM Official Title");
        assert_eq!(candidate["patch"]["overview"], "DMM overview");
        assert_eq!(candidate["patch"]["tags"], serde_json::json!(["dmm-tag"]));
        assert_eq!(
            candidate["av"]["actors"],
            serde_json::json!(["JavDB Actor"])
        );
        assert_eq!(candidate["av"]["studio"], "DMM Studio");
        assert_eq!(candidate["av"]["wanted_count"], 77);
        assert!(
            candidate["evidence"]["field_sources"]
                .as_array()
                .unwrap()
                .iter()
                .any(|source| source["field"] == "title" && source["provider"] == "dmm")
        );
        assert!(
            candidate["evidence"]["field_sources"]
                .as_array()
                .unwrap()
                .iter()
                .any(|source| source["field"] == "actors" && source["provider"] == "javdb")
        );
        assert!(
            candidate["evidence"]["field_sources"]
                .as_array()
                .unwrap()
                .iter()
                .any(|source| source["field"] == "wanted_count" && source["provider"] == "javdb")
        );
        assert!(
            candidate["artwork_candidates"]
                .as_array()
                .unwrap()
                .iter()
                .all(|artwork| artwork["provider"] == "dmm")
        );
    }

    #[tokio::test]
    async fn runtime_applies_request_provider_field_policy_within_merged_av_cluster() {
        let runtime = MetadataScrapeRuntime::<FakeTransport>::with_external_id_capabilities(
            "zh-CN",
            TEST_EXTERNAL_ID_CAPABILITIES.to_vec(),
            vec![
                Box::new(PolicyCandidateProvider {
                    candidate_provider: "javdb",
                    provider_id: "javdb:movie:abc123",
                    title: "SSNI-644 JavDB Title",
                    overview: None,
                    tags: &["javdb-tag"],
                    av_actors: &["JavDB Actor"],
                    av_studio: Some("JavDB Studio"),
                    av_wanted_count: Some(77),
                    artwork: &[],
                    external_ids: &[("av_number", "SSNI-644"), ("javdb", "abc123")],
                }),
                Box::new(PolicyCandidateProvider {
                    candidate_provider: "dmm",
                    provider_id: "dmm:cid:ssni00644",
                    title: "DMM Title",
                    overview: Some("DMM overview"),
                    tags: &["dmm-tag"],
                    av_actors: &["DMM Actor"],
                    av_studio: Some("DMM Studio"),
                    av_wanted_count: None,
                    artwork: &[],
                    external_ids: &[("av_number", "ssni-644")],
                }),
            ],
            None,
        );

        let response = runtime
            .scrape(AddonResourceRequest {
                protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
                addon_id: "addon-1".to_owned(),
                resource: AddonResource::Metadata,
                request_id: "request-1".to_owned(),
                payload: serde_json::json!({
                    "av_number": "SSNI-644",
                    "provider_field_policy": {
                        "title": ["javdb"],
                        "overview": ["dmm"],
                        "tags": ["dmm"],
                        "actors": ["javdb"],
                        "studio": ["dmm"]
                    }
                }),
            })
            .await;

        let candidate = &response.payload["candidates"][0];
        assert_eq!(candidate["patch"]["title"], "SSNI-644 JavDB Title");
        assert_eq!(candidate["patch"]["overview"], "DMM overview");
        assert_eq!(candidate["patch"]["tags"], serde_json::json!(["dmm-tag"]));
        assert_eq!(
            candidate["av"]["actors"],
            serde_json::json!(["JavDB Actor"])
        );
        assert_eq!(candidate["av"]["studio"], "DMM Studio");
        assert!(
            candidate["evidence"]["field_sources"]
                .as_array()
                .unwrap()
                .iter()
                .any(|source| source["field"] == "title" && source["provider"] == "javdb")
        );
        assert!(
            candidate["evidence"]["field_sources"]
                .as_array()
                .unwrap()
                .iter()
                .any(|source| source["field"] == "overview" && source["provider"] == "dmm")
        );
        assert!(
            candidate["evidence"]["field_sources"]
                .as_array()
                .unwrap()
                .iter()
                .any(|source| source["field"] == "tags" && source["provider"] == "dmm")
        );
        assert!(
            candidate["evidence"]["field_sources"]
                .as_array()
                .unwrap()
                .iter()
                .any(|source| source["field"] == "actors" && source["provider"] == "javdb")
        );
        assert!(
            candidate["evidence"]["field_sources"]
                .as_array()
                .unwrap()
                .iter()
                .any(|source| source["field"] == "studio" && source["provider"] == "dmm")
        );
    }

    #[tokio::test]
    async fn runtime_routes_fc2_av_queries_only_to_fc2_provider_when_av_providers_are_enabled() {
        let javdb_calls = Arc::new(Mutex::new(0));
        let fc2_calls = Arc::new(Mutex::new(0));
        let runtime = MetadataScrapeRuntime::<FakeTransport>::new(
            "zh-CN",
            vec![
                Box::new(CountingProvider {
                    provider_id: ProviderId::Javdb,
                    candidate_provider: "javdb",
                    candidate_id: "javdb:movie:fc2-should-not-run",
                    calls: javdb_calls.clone(),
                }),
                Box::new(CountingProvider {
                    provider_id: ProviderId::Fc2,
                    candidate_provider: "fc2",
                    candidate_id: "fc2:article:1723984",
                    calls: fc2_calls.clone(),
                }),
            ],
            None,
        );

        let response = runtime
            .scrape(AddonResourceRequest {
                protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
                addon_id: "addon-1".to_owned(),
                resource: AddonResource::Metadata,
                request_id: "request-1".to_owned(),
                payload: serde_json::json!({"file_name": "FC2PPV-1723984.mp4"}),
            })
            .await;

        assert_eq!(*javdb_calls.lock().unwrap(), 0);
        assert_eq!(*fc2_calls.lock().unwrap(), 1);
        assert_eq!(response.payload["candidates"][0]["provider"], "fc2");
        assert_eq!(response.payload["query"]["av"]["route"], "fc2");
        assert_eq!(
            response.payload["provider_execution"]["selected_provider_ids"],
            serde_json::json!(["fc2"])
        );
        assert_eq!(
            response.payload["provider_execution"]["skipped_provider_ids"],
            serde_json::json!(["javdb"])
        );
        assert_eq!(
            response.payload["provider_execution"]["returned_provider_ids"],
            serde_json::json!(["fc2"])
        );
        assert_eq!(
            response.payload["provider_execution"]["providers"][0]["status"],
            "skipped_by_av_route"
        );
        assert_eq!(
            response.payload["provider_execution"]["providers"][0]["av_route"],
            "fc2"
        );
    }

    #[tokio::test]
    async fn runtime_routes_non_fc2_av_queries_to_javdb_provider_when_av_providers_are_enabled() {
        let javdb_calls = Arc::new(Mutex::new(0));
        let fc2_calls = Arc::new(Mutex::new(0));
        let runtime = MetadataScrapeRuntime::<FakeTransport>::new(
            "zh-CN",
            vec![
                Box::new(CountingProvider {
                    provider_id: ProviderId::Javdb,
                    candidate_provider: "javdb",
                    candidate_id: "javdb:movie:ssni",
                    calls: javdb_calls.clone(),
                }),
                Box::new(CountingProvider {
                    provider_id: ProviderId::Fc2,
                    candidate_provider: "fc2",
                    candidate_id: "fc2:article:should-not-run",
                    calls: fc2_calls.clone(),
                }),
            ],
            None,
        );

        let response = runtime
            .scrape(AddonResourceRequest {
                protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
                addon_id: "addon-1".to_owned(),
                resource: AddonResource::Metadata,
                request_id: "request-1".to_owned(),
                payload: serde_json::json!({"file_name": "SSNI-00644.mp4"}),
            })
            .await;

        assert_eq!(*javdb_calls.lock().unwrap(), 1);
        assert_eq!(*fc2_calls.lock().unwrap(), 0);
        assert_eq!(response.payload["candidates"][0]["provider"], "javdb");
        assert_eq!(response.payload["query"]["av"]["route"], "censored");
        assert_eq!(
            response.payload["provider_execution"]["selected_provider_ids"],
            serde_json::json!(["javdb"])
        );
        assert_eq!(
            response.payload["provider_execution"]["skipped_provider_ids"],
            serde_json::json!(["fc2"])
        );
    }

    #[test]
    fn metadata_query_parses_string_year() {
        let query = MetadataQuery::from_payload(
            &serde_json::json!({
                "title": "Movie",
                "year": " 1999 "
            }),
            "en-US",
        );

        assert_eq!(query.year, Some(1999));
    }

    #[test]
    fn metadata_query_parses_year_aliases() {
        let query = MetadataQuery::from_payload(
            &serde_json::json!({
                "title": "Movie",
                "release_year": "1995"
            }),
            "en-US",
        );

        assert_eq!(query.year, Some(1995));

        let query = MetadataQuery::from_payload(
            &serde_json::json!({
                "title": "Movie",
                "original_year": 2001
            }),
            "en-US",
        );

        assert_eq!(query.year, Some(2001));
    }

    #[test]
    fn metadata_query_parses_year_from_date_fields() {
        let query = MetadataQuery::from_payload(
            &serde_json::json!({
                "title": "Movie",
                "release_date": "1999-03-31"
            }),
            "en-US",
        );

        assert_eq!(query.year, Some(1999));

        let query = MetadataQuery::from_payload(
            &serde_json::json!({
                "title": "Movie",
                "date": "invalid"
            }),
            "en-US",
        );

        assert_eq!(query.year, None);
    }

    #[test]
    fn metadata_query_ignores_non_positive_years() {
        let query = MetadataQuery::from_payload(
            &serde_json::json!({
                "title": "Movie",
                "year": 0,
                "release_date": "1999-03-31"
            }),
            "en-US",
        );

        assert_eq!(query.year, Some(1999));

        let query = MetadataQuery::from_payload(
            &serde_json::json!({
                "title": "Movie",
                "year": " -1 "
            }),
            "en-US",
        );

        assert_eq!(query.year, None);

        let query = MetadataQuery::from_payload(
            &serde_json::json!({
                "title": "Movie",
                "release_date": "0000-03-31"
            }),
            "en-US",
        );

        assert_eq!(query.year, None);
    }

    #[test]
    fn metadata_query_ignores_out_of_range_years() {
        let query = MetadataQuery::from_payload(
            &serde_json::json!({
                "title": "Movie",
                "year": 10000
            }),
            "en-US",
        );

        assert_eq!(query.year, None);

        let query = MetadataQuery::from_payload(
            &serde_json::json!({
                "title": "Movie",
                "release_date": "10000-03-31"
            }),
            "en-US",
        );

        assert_eq!(query.year, None);

        let query = MetadataQuery::from_payload(
            &serde_json::json!({
                "title": "Movie",
                "release_date": "9999-12-31"
            }),
            "en-US",
        );

        assert_eq!(query.year, Some(9999));
    }

    #[test]
    fn metadata_query_trims_language() {
        let query = MetadataQuery::from_payload(
            &serde_json::json!({
                "title": "Movie",
                "language": " zh-CN "
            }),
            "en-US",
        );

        assert_eq!(query.language, "zh-CN");
    }

    #[test]
    fn metadata_query_uses_default_language_when_payload_language_is_blank() {
        let query = MetadataQuery::from_payload(
            &serde_json::json!({
                "title": "Movie",
                "language": " "
            }),
            "en-US",
        );

        assert_eq!(query.language, "en-US");
    }

    #[test]
    fn metadata_query_uses_first_non_empty_title_field() {
        let query = MetadataQuery::from_payload(
            &serde_json::json!({
                "title": " ",
                "name": "  The   Matrix  "
            }),
            "en-US",
        );

        assert_eq!(query.title, "The Matrix");
    }

    #[test]
    fn metadata_query_falls_back_to_original_title() {
        let query = MetadataQuery::from_payload(
            &serde_json::json!({
                "title": "",
                "name": " ",
                "original_title": "  千と千尋の神隠し  "
            }),
            "ja-JP",
        );

        assert_eq!(query.title, "千と千尋の神隠し");
    }

    #[test]
    fn ranking_evidence_metadata_query_parses_external_ids() {
        let query = MetadataQuery::from_payload_with_external_id_aliases(
            &serde_json::json!({
                "title": "Movie",
                "external_ids": {
                    "imdb": "tt0133093",
                    "tmdb": "603"
                }
            }),
            "en-US",
            TEST_EXTERNAL_ID_ALIASES,
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

    #[test]
    fn metadata_query_parses_external_id_object_arrays() {
        let query = MetadataQuery::from_payload_with_external_id_aliases(
            &serde_json::json!({
                "title": "Movie",
                "external_ids": {
                    "bangumi": ["not-a-number", "265"],
                    "imdb": ["bad", "tt0133093"],
                    "tmdb": ["invalid", "603"]
                }
            }),
            "en-US",
            TEST_EXTERNAL_ID_ALIASES,
        );

        assert_eq!(
            query.external_ids,
            vec![
                QueryExternalId {
                    provider: "bangumi".to_owned(),
                    value: "not-a-number".to_owned(),
                },
                QueryExternalId {
                    provider: "bangumi".to_owned(),
                    value: "265".to_owned(),
                },
                QueryExternalId {
                    provider: "imdb".to_owned(),
                    value: "bad".to_owned(),
                },
                QueryExternalId {
                    provider: "imdb".to_owned(),
                    value: "tt0133093".to_owned(),
                },
                QueryExternalId {
                    provider: "tmdb".to_owned(),
                    value: "invalid".to_owned(),
                },
                QueryExternalId {
                    provider: "tmdb".to_owned(),
                    value: "603".to_owned(),
                }
            ]
        );
    }

    #[test]
    fn metadata_query_parses_external_id_array_object_value_aliases() {
        let query = MetadataQuery::from_payload_with_external_id_aliases(
            &serde_json::json!({
                "title": "Movie",
                "external_ids": [
                    {"provider": "tmdb", "value": "603"},
                    {"provider": "imdb", "id": "TT0133093"},
                    {"provider": "bangumi", "external_id": "265"},
                    {"provider": "ignored", "external_id": 123}
                ]
            }),
            "en-US",
            TEST_EXTERNAL_ID_ALIASES,
        );

        assert_eq!(
            query.external_ids,
            vec![
                QueryExternalId {
                    provider: "tmdb".to_owned(),
                    value: "603".to_owned(),
                },
                QueryExternalId {
                    provider: "imdb".to_owned(),
                    value: "TT0133093".to_owned(),
                },
                QueryExternalId {
                    provider: "bangumi".to_owned(),
                    value: "265".to_owned(),
                }
            ]
        );
    }

    #[test]
    fn metadata_query_trims_external_ids_and_skips_empty_entries() {
        let query = MetadataQuery::from_payload_with_external_id_aliases(
            &serde_json::json!({
                "title": "Movie",
                "external_ids": [
                    {"provider": " tmdb ", "value": " 603 "},
                    {"provider": " ", "value": "tt0133093"},
                    {"provider": "imdb", "id": " "},
                    {"provider": " bangumi ", "external_id": " 265 "}
                ]
            }),
            "en-US",
            TEST_EXTERNAL_ID_ALIASES,
        );

        assert_eq!(
            query.external_ids,
            vec![
                QueryExternalId {
                    provider: "tmdb".to_owned(),
                    value: "603".to_owned(),
                },
                QueryExternalId {
                    provider: "bangumi".to_owned(),
                    value: "265".to_owned(),
                }
            ]
        );
    }

    #[test]
    fn metadata_query_parses_top_level_external_id_aliases() {
        let query = MetadataQuery::from_payload_with_external_id_aliases(
            &serde_json::json!({
                "title": "Movie",
                "tmdb_id": " 603 ",
                "imdb_id": " tt0133093 ",
                "bangumi_id": " 265 "
            }),
            "en-US",
            TEST_EXTERNAL_ID_ALIASES,
        );

        assert_eq!(
            query.external_ids,
            vec![
                QueryExternalId {
                    provider: "tmdb".to_owned(),
                    value: "603".to_owned(),
                },
                QueryExternalId {
                    provider: "imdb".to_owned(),
                    value: "tt0133093".to_owned(),
                },
                QueryExternalId {
                    provider: "bangumi".to_owned(),
                    value: "265".to_owned(),
                }
            ]
        );
    }

    #[test]
    fn metadata_query_preserves_external_ids_before_top_level_aliases() {
        let query = MetadataQuery::from_payload_with_external_id_aliases(
            &serde_json::json!({
                "title": "Movie",
                "external_ids": {
                    "tmdb": "603"
                },
                "tmdb_id": "604"
            }),
            "en-US",
            TEST_EXTERNAL_ID_ALIASES,
        );

        assert_eq!(
            query.external_ids,
            vec![
                QueryExternalId {
                    provider: "tmdb".to_owned(),
                    value: "603".to_owned(),
                },
                QueryExternalId {
                    provider: "tmdb".to_owned(),
                    value: "604".to_owned(),
                }
            ]
        );
    }

    #[test]
    fn metadata_query_parses_numeric_top_level_external_id_aliases() {
        let query = MetadataQuery::from_payload_with_external_id_aliases(
            &serde_json::json!({
                "title": "Movie",
                "tmdb_id": 603,
                "bangumi_id": 265,
                "imdb_id": "tt0133093"
            }),
            "en-US",
            TEST_EXTERNAL_ID_ALIASES,
        );

        assert_eq!(
            query.external_ids,
            vec![
                QueryExternalId {
                    provider: "tmdb".to_owned(),
                    value: "603".to_owned(),
                },
                QueryExternalId {
                    provider: "imdb".to_owned(),
                    value: "tt0133093".to_owned(),
                },
                QueryExternalId {
                    provider: "bangumi".to_owned(),
                    value: "265".to_owned(),
                }
            ]
        );
    }

    #[test]
    fn metadata_query_parses_numeric_external_id_values() {
        let query = MetadataQuery::from_payload_with_external_id_aliases(
            &serde_json::json!({
                "title": "Movie",
                "external_ids": {
                    "tmdb": 603,
                    "bangumi": [265, "266"]
                }
            }),
            "en-US",
            TEST_EXTERNAL_ID_ALIASES,
        );

        assert_eq!(
            query.external_ids,
            vec![
                QueryExternalId {
                    provider: "bangumi".to_owned(),
                    value: "265".to_owned(),
                },
                QueryExternalId {
                    provider: "bangumi".to_owned(),
                    value: "266".to_owned(),
                },
                QueryExternalId {
                    provider: "tmdb".to_owned(),
                    value: "603".to_owned(),
                }
            ]
        );
    }

    #[test]
    fn metadata_query_skips_non_positive_numeric_external_ids() {
        let query = MetadataQuery::from_payload_with_external_id_aliases(
            &serde_json::json!({
                "title": "Movie",
                "external_ids": {
                    "tmdb": [0, "0", "-1", "603"],
                    "bangumi": [0, "265"],
                    "imdb": 0
                },
                "tmdb_id": 0,
                "bangumi_id": "0",
                "imdb_id": 0
            }),
            "en-US",
            TEST_EXTERNAL_ID_ALIASES,
        );

        assert_eq!(
            query.external_ids,
            vec![
                QueryExternalId {
                    provider: "bangumi".to_owned(),
                    value: "265".to_owned(),
                },
                QueryExternalId {
                    provider: "tmdb".to_owned(),
                    value: "603".to_owned(),
                }
            ]
        );
    }

    #[test]
    fn metadata_query_parses_browser_worker_top_level_external_id_alias() {
        let query = MetadataQuery::from_payload_with_external_id_aliases(
            &serde_json::json!({
                "title": "Rendered Page",
                "browser_worker_url": " https://example.test/page "
            }),
            "en-US",
            TEST_EXTERNAL_ID_ALIASES,
        );

        assert_eq!(
            query.external_ids,
            vec![QueryExternalId {
                provider: "browser_worker".to_owned(),
                value: "https://example.test/page".to_owned(),
            }]
        );
    }

    #[test]
    fn metadata_query_parses_external_id_capabilities() {
        let query = MetadataQuery::from_payload_with_external_id_capabilities(
            &serde_json::json!({
                "title": "Movie",
                "external_ids": {
                    "tmdb": [0, "603"],
                    "imdb": 0
                },
                "tmdb_id": 604,
                "bangumi_id": "265",
                "browser_worker_url": " https://example.test/page "
            }),
            "en-US",
            TEST_EXTERNAL_ID_CAPABILITIES,
        );

        assert_eq!(
            query.external_ids,
            vec![
                QueryExternalId {
                    provider: "tmdb".to_owned(),
                    value: "603".to_owned(),
                },
                QueryExternalId {
                    provider: "tmdb".to_owned(),
                    value: "604".to_owned(),
                },
                QueryExternalId {
                    provider: "bangumi".to_owned(),
                    value: "265".to_owned(),
                },
                QueryExternalId {
                    provider: "browser_worker".to_owned(),
                    value: "https://example.test/page".to_owned(),
                }
            ]
        );
    }

    #[tokio::test]
    async fn metadata_side_effect_request_submits_selected_patch_when_enabled() {
        let transport = FakeTransport::default();
        transport.push(Ok(NakoRuntimeHttpResponse {
            status: 200,
            body: serde_json::json!({
                "addon_id": "addon-1",
                "token_id": "token-1",
                "permission": "metadata_write",
                "library_id": "library-1",
                "allowed": true
            })
            .to_string(),
        }));
        transport.push(Ok(NakoRuntimeHttpResponse {
            status: 200,
            body: serde_json::json!({
                "side_effect": {
                    "id": "effect-1",
                    "addon_id": "addon-1",
                    "token_id": "token-1",
                    "permission": "metadata_write",
                    "library_id": "library-1",
                    "target": {"kind": "media_source", "id": "source-1"},
                    "idempotency_key": "metadata-demo-1",
                    "validation_status": "accepted",
                    "safe_error_code": null,
                    "apply_status": "applied",
                    "apply_error_code": null,
                    "applied_item_id": "item-1",
                    "applied_source": "addon:addon-1",
                    "apply_report": null
                },
                "idempotent_replay": false
            })
            .to_string(),
        }));
        let runtime = MetadataScrapeRuntime::<FakeTransport>::new(
            "en-US",
            vec![Box::new(CandidateProvider {
                provider_id: "fixture:matrix",
                title: "The Matrix",
                year: Some(1999),
            })],
            Some(NakoRuntimeClient::<FakeTransport>::with_transport(
                NakoRuntimeClientConfig {
                    base_url: "https://nako.example/".to_owned(),
                    addon_token: "addon-token-secret".to_owned(),
                    timeout_ms: 1500,
                },
                transport.clone(),
            )),
        );

        let response = runtime
            .scrape(AddonResourceRequest {
                protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
                addon_id: "addon-1".to_owned(),
                resource: AddonResource::Metadata,
                request_id: "request-1".to_owned(),
                payload: serde_json::json!({
                    "title": "The Matrix",
                    "year": 1999,
                    "language": "en-US",
                    "writeback": {
                        "library_id": "library-1",
                        "target": {
                            "kind": "media_source",
                            "id": "source-1"
                        },
                        "idempotency_key": "metadata-demo-1"
                    }
                }),
            })
            .await;

        assert_eq!(response.payload["writeback"]["status"], "submitted");
        assert_eq!(
            response.payload["writeback"]["side_effect"]["permission"],
            "metadata_write"
        );
        assert_eq!(
            response.payload["writeback"]["side_effect"]["applied_source"],
            "addon:addon-1"
        );

        let requests = transport.requests();
        assert_eq!(requests.len(), 2);
        assert_eq!(
            requests[0].url,
            "https://nako.example/addon/v1/access-check"
        );
        let access_check_body: serde_json::Value = serde_json::from_str(&requests[0].body).unwrap();
        assert_eq!(
            access_check_body,
            serde_json::json!({
                "permission": "metadata_write",
                "library_id": "library-1"
            })
        );
        assert_eq!(
            requests[1].url,
            "https://nako.example/addon/v1/side-effects"
        );
        let body: serde_json::Value = serde_json::from_str(&requests[1].body).unwrap();
        assert_eq!(body["permission"], "metadata_write");
        assert_eq!(body["library_id"], "library-1");
        assert_eq!(body["target"]["kind"], "media_source");
        assert_eq!(body["target"]["id"], "source-1");
        assert_eq!(body["idempotency_key"], "metadata-demo-1");
        assert_eq!(body["payload"]["title"], "The Matrix");
        assert_eq!(body["payload"]["release_date"], "1999-01-01");
        assert_eq!(body["payload"]["tags"][0], "en-US");
        assert_eq!(body["provenance"]["origin"], "nako-metadata-scraper");
        assert_eq!(body["provenance"]["request_id"], "request-1");
        assert_eq!(body["provenance"]["query"]["title"], "The Matrix");
    }

    #[tokio::test]
    async fn metadata_side_effect_request_skips_when_runtime_is_disabled() {
        let runtime =
            MetadataScrapeRuntime::<crate::nako_runtime::ReqwestNakoRuntimeTransport>::new(
                "en-US",
                vec![Box::new(CandidateProvider {
                    provider_id: "fixture:matrix",
                    title: "The Matrix",
                    year: Some(1999),
                })],
                None,
            );

        let response = runtime
            .scrape(AddonResourceRequest {
                protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
                addon_id: "addon-1".to_owned(),
                resource: AddonResource::Metadata,
                request_id: "request-1".to_owned(),
                payload: serde_json::json!({
                    "title": "The Matrix",
                    "year": 1999,
                    "language": "en-US",
                    "writeback": {
                        "library_id": "library-1",
                        "target": {
                            "kind": "media_source",
                            "id": "source-1"
                        },
                        "idempotency_key": "metadata-demo-1"
                    }
                }),
            })
            .await;

        assert_eq!(response.payload["writeback"]["status"], "skipped");
        assert_eq!(
            response.payload["writeback"]["safe_error_code"],
            "nako_runtime_disabled"
        );
    }

    #[tokio::test]
    async fn artwork_side_effect_request_submits_selected_candidate_when_enabled() {
        let transport = FakeTransport::default();
        transport.push(Ok(NakoRuntimeHttpResponse {
            status: 200,
            body: serde_json::json!({
                "addon_id": "addon-1",
                "token_id": "token-1",
                "permission": "artwork_write",
                "library_id": "library-1",
                "allowed": true
            })
            .to_string(),
        }));
        transport.push(Ok(NakoRuntimeHttpResponse {
            status: 200,
            body: serde_json::json!({
                "side_effect": {
                    "id": "effect-2",
                    "addon_id": "addon-1",
                    "token_id": "token-1",
                    "permission": "artwork_write",
                    "library_id": "library-1",
                    "target": {"kind": "media_item", "id": "item-1"},
                    "idempotency_key": "artwork-demo-1",
                    "validation_status": "accepted",
                    "safe_error_code": null,
                    "apply_status": "applied",
                    "apply_error_code": null,
                    "applied_item_id": "item-1",
                    "applied_source": "addon:addon-1",
                    "apply_report": null
                },
                "idempotent_replay": false
            })
            .to_string(),
        }));
        let runtime = MetadataScrapeRuntime::<FakeTransport>::new(
            "en-US",
            vec![Box::new(CandidateProvider {
                provider_id: "fixture:matrix",
                title: "The Matrix",
                year: Some(1999),
            })],
            Some(NakoRuntimeClient::<FakeTransport>::with_transport(
                NakoRuntimeClientConfig {
                    base_url: "https://nako.example/".to_owned(),
                    addon_token: "addon-token-secret".to_owned(),
                    timeout_ms: 1500,
                },
                transport.clone(),
            )),
        );

        let response = runtime
            .scrape(AddonResourceRequest {
                protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
                addon_id: "addon-1".to_owned(),
                resource: AddonResource::Metadata,
                request_id: "request-1".to_owned(),
                payload: serde_json::json!({
                    "title": "The Matrix",
                    "year": 1999,
                    "language": "en-US",
                    "artwork_writeback": {
                        "library_id": "library-1",
                        "target": {
                            "kind": "media_item",
                            "id": "item-1"
                        },
                        "idempotency_key": "artwork-demo-1",
                        "kind": "poster"
                    }
                }),
            })
            .await;

        assert_eq!(response.payload["artwork_writeback"]["status"], "submitted");
        assert_eq!(
            response.payload["artwork_writeback"]["side_effect"]["permission"],
            "artwork_write"
        );
        assert_eq!(
            response.payload["artwork_writeback"]["side_effect"]["applied_source"],
            "addon:addon-1"
        );

        let requests = transport.requests();
        assert_eq!(requests.len(), 2);
        let access_check_body: serde_json::Value = serde_json::from_str(&requests[0].body).unwrap();
        assert_eq!(
            access_check_body,
            serde_json::json!({
                "permission": "artwork_write",
                "library_id": "library-1"
            })
        );
        let body: serde_json::Value = serde_json::from_str(&requests[1].body).unwrap();
        assert_eq!(body["permission"], "artwork_write");
        assert_eq!(body["library_id"], "library-1");
        assert_eq!(body["target"]["kind"], "media_item");
        assert_eq!(body["target"]["id"], "item-1");
        assert_eq!(body["idempotency_key"], "artwork-demo-1");
        assert_eq!(body["payload"]["intent"], "propose_artwork");
        assert_eq!(body["payload"]["kind"], "poster");
        assert_eq!(body["payload"]["source"]["kind"], "remote_url");
        assert_eq!(
            body["payload"]["source"]["url"],
            "https://example.test/poster.jpg"
        );
    }

    #[derive(Clone, Default)]
    struct FakeTransport {
        responses: Arc<Mutex<VecDeque<NakoRuntimeResult<NakoRuntimeHttpResponse>>>>,
        requests: Arc<Mutex<Vec<NakoRuntimeHttpRequest>>>,
    }

    impl FakeTransport {
        fn push(&self, response: NakoRuntimeResult<NakoRuntimeHttpResponse>) {
            self.responses.lock().unwrap().push_back(response);
        }

        fn requests(&self) -> Vec<NakoRuntimeHttpRequest> {
            self.requests.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl NakoRuntimeTransport for FakeTransport {
        async fn post(
            &self,
            request: NakoRuntimeHttpRequest,
        ) -> NakoRuntimeResult<NakoRuntimeHttpResponse> {
            self.requests.lock().unwrap().push(request);
            self.responses
                .lock()
                .unwrap()
                .pop_front()
                .unwrap_or_else(|| {
                    Err(NakoRuntimeError::Http {
                        message: "fake response queue was empty".to_owned(),
                    })
                })
        }
    }
}
