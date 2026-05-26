mod client;
mod enrichment;
mod mapper;
mod parser;
mod search;
#[cfg(test)]
mod test_support;

use async_trait::async_trait;
use nako_addon_protocol::AddonSecretReferenceFieldDeclaration;

use crate::{
    Config,
    config::{ProviderConfig, ProviderId, non_empty_trimmed, parse_bool},
    engine::{
        ExternalIdValueKind, MetadataQuery, ProviderExternalIdCapability, ProviderMetadataCandidate,
    },
    providers::{
        MetadataProvider, ProviderBuildStatus, ProviderConfigInput,
        http_runtime::{ProviderHttpRuntime, ProviderHttpTransport, ReqwestProviderHttpTransport},
        registry::ProviderCatalogEntry,
    },
};

#[cfg(test)]
use mapper::{BangumiSubjectCandidate, release_year};
#[cfg(test)]
use nako_addon_protocol::AddonArtworkKind;
#[cfg(test)]
use parser::{
    BangumiImages, BangumiInfoboxItem, BangumiRating, BangumiSubject, BangumiSubjectSearchResponse,
    BangumiTag,
};
#[cfg(test)]
use search::{bangumi_air_date_filter, bangumi_query_subject_ids};
#[cfg(test)]
use test_support::FakeTransport;

pub const BANGUMI_PROVIDER_ID: &str = "bangumi";
const BANGUMI_EXTERNAL_ID_CAPABILITIES: &[ProviderExternalIdCapability] =
    &[ProviderExternalIdCapability::new(
        "bangumi",
        ExternalIdValueKind::Numeric,
        true,
        true,
        &["bangumi_id"],
        true,
    )];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BangumiProviderConfig {
    pub access_token: Option<String>,
    pub api_base_url: String,
    pub user_agent: String,
    pub include_nsfw: bool,
    pub subject_types: Vec<u8>,
    pub proxy_url: Option<String>,
}

impl BangumiProviderConfig {
    #[must_use]
    pub fn from_env_lookup(mut lookup: impl FnMut(&str) -> Option<String>) -> Self {
        Self {
            access_token: lookup("NAKO_METADATA_SCRAPER_BANGUMI_ACCESS_TOKEN")
                .and_then(non_empty_trimmed),
            api_base_url: lookup("NAKO_METADATA_SCRAPER_BANGUMI_API_BASE_URL")
                .and_then(non_empty_trimmed)
                .unwrap_or_else(|| "https://api.bgm.tv".to_owned()),
            user_agent: lookup("NAKO_METADATA_SCRAPER_BANGUMI_USER_AGENT")
                .and_then(non_empty_trimmed)
                .unwrap_or_else(Self::default_user_agent),
            include_nsfw: lookup("NAKO_METADATA_SCRAPER_BANGUMI_INCLUDE_NSFW")
                .and_then(|value| parse_bool(&value))
                .unwrap_or(false),
            subject_types: lookup("NAKO_METADATA_SCRAPER_BANGUMI_SUBJECT_TYPES")
                .and_then(|value| parse_bangumi_subject_types(&value))
                .unwrap_or_else(|| vec![2]),
            proxy_url: lookup("NAKO_METADATA_SCRAPER_BANGUMI_PROXY_URL")
                .and_then(non_empty_trimmed),
        }
    }

    #[must_use]
    pub fn default_user_agent() -> String {
        format!(
            "Latias94/nako-official-addons/nako-metadata-scraper/{} (https://github.com/Latias94/nako-official-addons)",
            env!("CARGO_PKG_VERSION")
        )
    }

    #[must_use]
    pub const fn secret_field_id() -> &'static str {
        "bangumi_access_token"
    }
}

fn parse_bangumi_subject_types(value: &str) -> Option<Vec<u8>> {
    let mut subject_types = Vec::new();
    for item in value.split(',') {
        let subject_type = item.trim().parse::<u8>().ok()?;
        if !matches!(subject_type, 1 | 2 | 3 | 4 | 6) {
            return None;
        }
        subject_types.push(subject_type);
    }
    (!subject_types.is_empty()).then_some(subject_types)
}

#[must_use]
pub(crate) fn catalog_entry() -> ProviderCatalogEntry {
    ProviderCatalogEntry {
        id: ProviderId::Bangumi,
        default_enabled: false,
        enabled_env_var: "NAKO_METADATA_SCRAPER_PROVIDER_BANGUMI_ENABLED",
        capabilities: &["metadata_suggestion", "subject_search", "anime_search"],
        field_quality: Default::default(),
        secret_reference: Some(AddonSecretReferenceFieldDeclaration::new(
            BangumiProviderConfig::secret_field_id(),
            "Bangumi Access Token",
            Some(
                "Optional Secret Reference for a Bangumi access token. Public read APIs work without it, but authenticated access may reveal user-permitted sensitive results."
                    .to_owned(),
            ),
            false,
        )),
        external_id_capabilities: BANGUMI_EXTERNAL_ID_CAPABILITIES,
        load_config: load_config,
        proxy_configured: bangumi_proxy_configured,
        network_policy_key: Some("bangumi_proxy_configured"),
        build: build_provider,
    }
}

fn load_config(input: ProviderConfigInput<'_>) -> ProviderConfig {
    let lookup = input.lookup;
    ProviderConfig::bangumi(
        input.enabled,
        BangumiProviderConfig::from_env_lookup(|name| lookup(name)),
    )
}

fn bangumi_proxy_configured(provider: &ProviderConfig) -> bool {
    provider
        .bangumi_config()
        .and_then(|config| config.proxy_url.as_ref())
        .is_some()
}

fn build_provider(config: &Config) -> ProviderBuildStatus {
    let Some(bangumi_config) = config
        .provider_config(ProviderId::Bangumi)
        .and_then(|provider| provider.bangumi_config().cloned())
    else {
        return ProviderBuildStatus::Unavailable;
    };
    match BangumiMetadataProvider::new(bangumi_config) {
        Ok(provider) => ProviderBuildStatus::Ready(Box::new(provider)),
        Err(_) => ProviderBuildStatus::Unavailable,
    }
}

#[derive(Clone, Debug)]
pub struct BangumiMetadataProvider<T = ReqwestProviderHttpTransport>
where
    T: ProviderHttpTransport,
{
    config: BangumiProviderConfig,
    runtime: ProviderHttpRuntime<T>,
}

#[async_trait]
impl<T> MetadataProvider for BangumiMetadataProvider<T>
where
    T: ProviderHttpTransport,
{
    fn id(&self) -> ProviderId {
        ProviderId::Bangumi
    }

    async fn suggest(
        &self,
        query: &MetadataQuery,
    ) -> anyhow::Result<Vec<ProviderMetadataCandidate>> {
        self.suggest_candidates(query).await
    }
}
#[cfg(test)]
mod tests {
    use crate::providers::http_runtime::{
        ProviderHttpMethod, ProviderHttpResponse, ProviderHttpRuntimeConfig,
    };

    use super::*;

    #[test]
    fn bangumi_query_subject_ids_ignores_zero_and_invalid_values() {
        let query = MetadataQuery {
            title: "新世纪福音战士".to_owned(),
            year: Some(1995),
            language: "zh-CN".to_owned(),
            external_ids: vec![
                crate::engine::QueryExternalId {
                    provider: "bangumi".to_owned(),
                    value: "0".to_owned(),
                },
                crate::engine::QueryExternalId {
                    provider: "BANGUMI".to_owned(),
                    value: "265".to_owned(),
                },
                crate::engine::QueryExternalId {
                    provider: "bangumi".to_owned(),
                    value: "265".to_owned(),
                },
                crate::engine::QueryExternalId {
                    provider: "bangumi".to_owned(),
                    value: "not-a-number".to_owned(),
                },
            ],
        };

        let subject_ids = bangumi_query_subject_ids(&query).collect::<Vec<_>>();

        assert_eq!(subject_ids, vec![265]);
    }

    #[test]
    fn bangumi_air_date_filter_ignores_non_positive_years() {
        assert_eq!(bangumi_air_date_filter(Some(0)), None);
        assert_eq!(bangumi_air_date_filter(Some(-1)), None);
        assert_eq!(bangumi_air_date_filter(Some(10000)), None);
        assert_eq!(
            bangumi_air_date_filter(Some(1995)),
            Some([">=1995-01-01".to_owned(), "<1996-01-01".to_owned()])
        );
    }

    #[test]
    fn bangumi_release_year_ignores_zero_year_values() {
        assert_eq!(release_year(Some("0000-10-04")), None);
        assert_eq!(release_year(Some("1995-10-04")), Some(1995));
        assert_eq!(release_year(Some(" 1995-10-04 ")), Some(1995));
        assert_eq!(release_year(Some("10000-10-04")), None);
    }

    #[test]
    fn bangumi_search_response_skips_zero_id_items() {
        let response = BangumiSubjectSearchResponse::from_value(serde_json::json!({
            "data": [
                {"id": 0, "type": 2},
                {"id": 265, "type": 2}
            ]
        }))
        .unwrap();

        assert_eq!(response.data.len(), 1);
        assert_eq!(response.data[0].id, 265);
    }

    #[tokio::test]
    async fn bangumi_provider_rejects_mismatched_detail_id_for_direct_lookup() {
        let transport = FakeTransport::default();
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{"id": 999, "type": 2}"#.to_vec(),
        }));
        let runtime = ProviderHttpRuntime::with_transport(
            ProviderHttpRuntimeConfig {
                retry_backoff_ms: 0,
                user_agent: "Latias94/test-addon/0.1.0".to_owned(),
                ..ProviderHttpRuntimeConfig::default()
            },
            transport.clone(),
        );
        let provider = BangumiMetadataProvider::with_runtime(
            BangumiProviderConfig {
                access_token: Some("bangumi-token".to_owned()),
                api_base_url: "https://bangumi.example".to_owned(),
                user_agent: "Latias94/test-addon/0.1.0".to_owned(),
                include_nsfw: false,
                subject_types: vec![2],
                proxy_url: None,
            },
            runtime,
        );

        let error = provider
            .enrich_subject_candidate_by_id(
                &MetadataQuery {
                    title: "新世纪福音战士".to_owned(),
                    year: Some(1995),
                    language: "zh-CN".to_owned(),
                    external_ids: Vec::new(),
                },
                265,
            )
            .await
            .unwrap_err();

        assert!(
            error
                .to_string()
                .contains("did not match requested subject 265")
        );
        assert_eq!(transport.requests().len(), 1);
    }

    #[tokio::test]
    async fn bangumi_provider_uses_http_runtime_and_maps_subject_candidates() {
        let transport = FakeTransport::default();
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: r#"{
                "data": [{
                    "id": 265,
                    "type": 2,
                    "name": "新世紀エヴァンゲリオン",
                    "name_cn": "新世纪福音战士",
                    "summary": "Search summary.",
                    "date": "1995-10-04",
                    "platform": "TV",
                    "images": {
                        "large": "https://lain.bgm.tv/pic/cover/l/example.jpg"
                    },
                    "eps": 26,
                    "total_episodes": 26,
                    "rating": {"rank": 12, "total": 10000, "score": 8.7},
                    "infobox": [
                        {"key": "别名", "value": [
                            {"v": "EVA"},
                            {"v": "Neon Genesis Evangelion"}
                        ]}
                    ],
                    "meta_tags": ["科幻"],
                    "tags": [{"name": "庵野秀明", "count": 500}]
                }]
            }"#
            .as_bytes()
            .to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: r#"{
                "id": 265,
                "type": 2,
                "name": "新世紀エヴァンゲリオン",
                "name_cn": "新世纪福音战士",
                "summary": "Detail summary.",
                "date": "1995-10-04",
                "platform": "TV",
                "images": {
                    "large": "https://lain.bgm.tv/pic/cover/l/detail.jpg",
                    "common": "https://lain.bgm.tv/pic/cover/c/detail.jpg"
                },
                "eps": 26,
                "total_episodes": 26,
                "rating": {"rank": 10, "total": 12000, "score": 8.8},
                "infobox": [
                    {"key": "别名", "value": [
                        {"v": "EVA"},
                        {"v": "Neon Genesis Evangelion"},
                        "NGE"
                    ]},
                    {"key": "中文名", "value": "新世纪福音战士"}
                ],
                "meta_tags": ["科幻", "机战"],
                "tags": [
                    {"name": "庵野秀明", "count": 500},
                    {"name": "GAINAX", "count": 300}
                ]
            }"#
            .as_bytes()
            .to_vec(),
        }));
        let runtime = ProviderHttpRuntime::with_transport(
            ProviderHttpRuntimeConfig {
                retry_backoff_ms: 0,
                user_agent: "Latias94/test-addon/0.1.0".to_owned(),
                ..ProviderHttpRuntimeConfig::default()
            },
            transport.clone(),
        );
        let provider = BangumiMetadataProvider::with_runtime(
            BangumiProviderConfig {
                access_token: Some("bangumi-token".to_owned()),
                api_base_url: "https://bangumi.example".to_owned(),
                user_agent: "Latias94/test-addon/0.1.0".to_owned(),
                include_nsfw: false,
                subject_types: vec![2],
                proxy_url: None,
            },
            runtime,
        );

        let candidates = provider
            .suggest(&MetadataQuery {
                title: "新世纪福音战士".to_owned(),
                year: Some(1995),
                language: "zh-CN".to_owned(),
                external_ids: Vec::new(),
            })
            .await
            .unwrap();

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].provider, "bangumi");
        assert_eq!(candidates[0].provider_id, "bangumi:subject:265");
        assert_eq!(candidates[0].patch.title.as_deref(), Some("新世纪福音战士"));
        assert_eq!(
            candidates[0].patch.original_title.as_deref(),
            Some("新世紀エヴァンゲリオン")
        );
        assert_eq!(
            candidates[0].patch.overview.as_deref(),
            Some("Detail summary.")
        );
        assert_eq!(
            candidates[0].patch.release_date.as_deref(),
            Some("1995-10-04")
        );
        assert_eq!(candidates[0].patch.tagline.as_deref(), Some("TV"));
        assert_eq!(
            candidates[0].patch.genres.as_ref().unwrap(),
            &vec![
                "科幻".to_owned(),
                "机战".to_owned(),
                "庵野秀明".to_owned(),
                "GAINAX".to_owned()
            ]
        );
        assert_eq!(candidates[0].artwork_candidates.len(), 2);
        assert_eq!(
            candidates[0].artwork_candidates[0].facts.kind,
            AddonArtworkKind::Poster
        );
        assert_eq!(
            candidates[0].artwork_candidates[0].facts.source_url,
            "https://lain.bgm.tv/pic/cover/l/detail.jpg"
        );
        assert_eq!(
            candidates[0].artwork_candidates[1].facts.kind,
            AddonArtworkKind::Poster
        );
        assert_eq!(
            candidates[0].artwork_candidates[1].facts.source_url,
            "https://lain.bgm.tv/pic/cover/c/detail.jpg"
        );
        assert_eq!(candidates[0].facts.title.as_deref(), Some("新世纪福音战士"));
        assert!(
            candidates[0]
                .facts
                .alternate_titles
                .iter()
                .any(|title| title == "Neon Genesis Evangelion")
        );
        assert!(
            candidates[0]
                .facts
                .alternate_titles
                .iter()
                .any(|title| title == "NGE")
        );
        assert_eq!(candidates[0].facts.release_year, Some(1995));
        assert_eq!(candidates[0].facts.language.as_deref(), Some("zh-CN"));
        assert_eq!(candidates[0].facts.community_score_milli, Some(880));
        assert_eq!(candidates[0].facts.community_vote_count, Some(12000));
        assert_eq!(candidates[0].facts.external_ids[0].provider, "bangumi");
        assert_eq!(candidates[0].facts.external_ids[0].value, "265");

        let requests = transport.requests();
        let configs = transport.configs();
        assert_eq!(configs[0].user_agent, "Latias94/test-addon/0.1.0");
        assert!(configs[0].proxy_url.is_none());
        assert_eq!(requests[0].method, ProviderHttpMethod::Post);
        assert_eq!(
            requests[0].url,
            "https://bangumi.example/v0/search/subjects"
        );
        assert_eq!(
            requests[0].query,
            vec![
                ("limit".to_owned(), "3".to_owned()),
                ("offset".to_owned(), "0".to_owned())
            ]
        );
        assert_eq!(
            requests[0].headers,
            vec![(
                "authorization".to_owned(),
                "Bearer bangumi-token".to_owned()
            )]
        );
        let body: serde_json::Value =
            serde_json::from_slice(requests[0].json_body.as_ref().unwrap()).unwrap();
        assert_eq!(body["keyword"], "新世纪福音战士");
        assert_eq!(body["sort"], "match");
        assert_eq!(body["filter"]["type"], serde_json::json!([2]));
        assert_eq!(body["filter"]["nsfw"], false);
        assert_eq!(
            body["filter"]["air_date"],
            serde_json::json!([">=1995-01-01", "<1996-01-01"])
        );
        assert_eq!(requests[1].method, ProviderHttpMethod::Get);
        assert_eq!(requests[1].url, "https://bangumi.example/v0/subjects/265");
    }

    #[tokio::test]
    async fn bangumi_provider_new_uses_proxy_url_from_config() {
        let provider = BangumiMetadataProvider::new(BangumiProviderConfig {
            access_token: Some("bangumi-token".to_owned()),
            api_base_url: "https://bangumi.example".to_owned(),
            user_agent: "Latias94/test-addon/0.1.0".to_owned(),
            include_nsfw: false,
            subject_types: vec![2],
            proxy_url: Some("http://proxy.example:8080".to_owned()),
        })
        .unwrap();

        assert_eq!(
            provider.runtime.config().proxy_url.as_deref(),
            Some("http://proxy.example:8080")
        );
    }

    #[test]
    fn bangumi_candidate_mapping_trims_provider_text_boundaries() {
        let candidate = BangumiSubjectCandidate {
            search: BangumiSubject {
                id: 265,
                subject_type: Some(2),
                name: Some(" Search Original ".to_owned()),
                name_cn: Some(" 搜索标题 ".to_owned()),
                summary: Some(" Search summary. ".to_owned()),
                date: Some(" 1995-10-03 ".to_owned()),
                platform: Some(" TV ".to_owned()),
                images: Some(BangumiImages {
                    large: Some(" https://lain.bgm.tv/pic/cover/l/search.jpg ".to_owned()),
                    common: None,
                    medium: None,
                    small: None,
                    grid: None,
                }),
                nsfw: None,
                locked: None,
                series: None,
                volumes: None,
                eps: Some(26),
                total_episodes: Some(26),
                air_weekday: None,
                rating: Some(BangumiRating {
                    rank: Some(10),
                    total: Some(12000),
                    score: Some(8.8),
                }),
                collection: None,
                infobox: vec![BangumiInfoboxItem {
                    key: Some(" 别名 ".to_owned()),
                    value: serde_json::json!([{"v": " Search Alias "}]),
                }],
                meta_tags: vec![" 科幻 ".to_owned(), "   ".to_owned()],
                tags: vec![
                    BangumiTag {
                        name: Some(" GAINAX ".to_owned()),
                        count: Some(300),
                    },
                    BangumiTag {
                        name: Some("   ".to_owned()),
                        count: Some(500),
                    },
                ],
            },
            detail: BangumiSubject {
                id: 265,
                subject_type: Some(2),
                name: Some(" 新世紀エヴァンゲリオン ".to_owned()),
                name_cn: Some(" 新世纪福音战士 ".to_owned()),
                summary: Some(" Detail summary. ".to_owned()),
                date: Some(" 1995-10-04 ".to_owned()),
                platform: Some(" TV ".to_owned()),
                images: Some(BangumiImages {
                    large: Some(" https://lain.bgm.tv/pic/cover/l/detail.jpg ".to_owned()),
                    common: Some("   ".to_owned()),
                    medium: None,
                    small: None,
                    grid: None,
                }),
                nsfw: None,
                locked: None,
                series: None,
                volumes: None,
                eps: Some(26),
                total_episodes: Some(26),
                air_weekday: None,
                rating: Some(BangumiRating {
                    rank: Some(10),
                    total: Some(12000),
                    score: Some(8.8),
                }),
                collection: None,
                infobox: vec![
                    BangumiInfoboxItem {
                        key: Some(" 别名 ".to_owned()),
                        value: serde_json::json!([{"v": " EVA "}, " NGE "]),
                    },
                    BangumiInfoboxItem {
                        key: Some(" 中文名 ".to_owned()),
                        value: serde_json::json!(" 新世纪福音战士 "),
                    },
                ],
                meta_tags: vec![" 科幻 ".to_owned(), " 机战 ".to_owned()],
                tags: vec![BangumiTag {
                    name: Some(" 庵野秀明 ".to_owned()),
                    count: Some(500),
                }],
            },
            degraded: false,
        }
        .into_candidate(&MetadataQuery {
            title: "新世纪福音战士".to_owned(),
            year: Some(1995),
            language: "zh-CN".to_owned(),
            external_ids: Vec::new(),
        });

        assert_eq!(candidate.patch.title.as_deref(), Some("新世纪福音战士"));
        assert_eq!(
            candidate.patch.original_title.as_deref(),
            Some("新世紀エヴァンゲリオン")
        );
        assert_eq!(candidate.patch.overview.as_deref(), Some("Detail summary."));
        assert_eq!(candidate.patch.release_date.as_deref(), Some("1995-10-04"));
        assert_eq!(candidate.patch.tagline.as_deref(), Some("TV"));
        assert_eq!(
            candidate.patch.genres.as_ref().unwrap(),
            &vec!["科幻".to_owned(), "机战".to_owned(), "庵野秀明".to_owned()]
        );
        assert!(
            candidate
                .facts
                .alternate_titles
                .iter()
                .any(|title| title == "EVA")
        );
        assert!(
            candidate
                .facts
                .alternate_titles
                .iter()
                .any(|title| title == "NGE")
        );
        assert_eq!(candidate.facts.release_year, Some(1995));
        assert_eq!(candidate.facts.language.as_deref(), Some("zh-CN"));
        assert_eq!(candidate.artwork_candidates.len(), 1);
        assert_eq!(
            candidate.artwork_candidates[0].facts.source_url,
            "https://lain.bgm.tv/pic/cover/l/detail.jpg"
        );
    }

    #[test]
    fn bangumi_candidate_mapping_preserves_official_subject_facts_as_tags() {
        let subject = BangumiSubject::from_value(serde_json::json!({
            "id": 265,
            "type": 2,
            "name": "新世紀エヴァンゲリオン",
            "name_cn": "新世纪福音战士",
            "summary": "Detail summary.",
            "date": "1995-10-04",
            "platform": " TV ",
            "images": {},
            "nsfw": true,
            "locked": true,
            "series": true,
            "volumes": 0,
            "eps": 26,
            "total_episodes": 26,
            "air_weekday": 3,
            "rating": {"rank": 10, "total": 12000, "score": 8.8},
            "collection": {
                "wish": 1000,
                "collect": 9000,
                "doing": 500,
                "on_hold": 300,
                "dropped": 200
            },
            "infobox": [
                {"key": "官方网站", "value": " https://www.evangelion.co.jp/ "},
                {"key": "播放结束", "value": "1996年03月27日"},
                {"key": "制作", "value": [
                    {"v": " GAINAX "},
                    {"k": "动画制作", "v": " タツノコプロ "},
                    {"v": "GAINAX"}
                ]}
            ],
            "meta_tags": [],
            "tags": []
        }))
        .unwrap();

        let candidate = BangumiSubjectCandidate {
            search: BangumiSubject::default(),
            detail: subject,
            degraded: false,
        }
        .into_candidate(&MetadataQuery {
            title: "新世纪福音战士".to_owned(),
            year: Some(1995),
            language: "zh-CN".to_owned(),
            external_ids: Vec::new(),
        });

        let tags = candidate.patch.tags.as_ref().unwrap();
        assert!(tags.contains(&"bangumi_nsfw".to_owned()));
        assert!(tags.contains(&"bangumi_locked".to_owned()));
        assert!(tags.contains(&"bangumi_series".to_owned()));
        assert!(tags.contains(&"bangumi_subject_type:2".to_owned()));
        assert!(tags.contains(&"bangumi_eps:26".to_owned()));
        assert!(tags.contains(&"bangumi_total_episodes:26".to_owned()));
        assert!(tags.contains(&"bangumi_air_weekday:3".to_owned()));
        assert!(tags.contains(&"bangumi_platform:TV".to_owned()));
        assert!(tags.contains(&"bangumi_collection_total:11000".to_owned()));
        assert!(tags.contains(&"bangumi_official_site".to_owned()));
        assert!(!tags.iter().any(|tag| tag.contains("evangelion.co.jp")));
        assert!(tags.contains(&"bangumi_end_date:1996年03月27日".to_owned()));
        assert!(tags.contains(&"bangumi_production:GAINAX".to_owned()));
        assert!(tags.contains(&"bangumi_production:タツノコプロ".to_owned()));
        assert!(!tags.contains(&"bangumi_volumes:0".to_owned()));
        assert_eq!(
            tags.iter()
                .filter(|tag| tag.as_str() == "bangumi_production:GAINAX")
                .count(),
            1
        );
    }

    #[tokio::test]
    async fn bangumi_provider_omits_air_date_search_filter_when_query_year_is_missing() {
        let transport = FakeTransport::default();
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: r#"{
                "data": [{
                    "id": 265,
                    "type": 2,
                    "name": "新世紀エヴァンゲリオン",
                    "name_cn": "新世纪福音战士",
                    "summary": "Search summary.",
                    "date": "1995-10-04",
                    "platform": "TV",
                    "images": {},
                    "eps": 26,
                    "total_episodes": 26,
                    "rating": {"rank": 10, "total": 12000, "score": 8.8},
                    "infobox": [],
                    "meta_tags": [],
                    "tags": []
                }]
            }"#
            .as_bytes()
            .to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: r#"{
                "id": 265,
                "type": 2,
                "name": "新世紀エヴァンゲリオン",
                "name_cn": "新世纪福音战士",
                "summary": "Detail summary.",
                "date": "1995-10-04",
                "platform": "TV",
                "images": {},
                "eps": 26,
                "total_episodes": 26,
                "rating": {"rank": 10, "total": 12000, "score": 8.8},
                "infobox": [],
                "meta_tags": [],
                "tags": []
            }"#
            .as_bytes()
            .to_vec(),
        }));
        let runtime = ProviderHttpRuntime::with_transport(
            ProviderHttpRuntimeConfig {
                retry_backoff_ms: 0,
                user_agent: "Latias94/test-addon/0.1.0".to_owned(),
                ..ProviderHttpRuntimeConfig::default()
            },
            transport.clone(),
        );
        let provider = BangumiMetadataProvider::with_runtime(
            BangumiProviderConfig {
                access_token: None,
                api_base_url: "https://bangumi.example".to_owned(),
                user_agent: "Latias94/test-addon/0.1.0".to_owned(),
                include_nsfw: false,
                subject_types: vec![2],
                proxy_url: None,
            },
            runtime,
        );

        let candidates = provider
            .suggest(&MetadataQuery {
                title: "新世纪福音战士".to_owned(),
                year: None,
                language: "zh-CN".to_owned(),
                external_ids: Vec::new(),
            })
            .await
            .unwrap();

        assert_eq!(candidates.len(), 1);
        let requests = transport.requests();
        let body: serde_json::Value =
            serde_json::from_slice(requests[0].json_body.as_ref().unwrap()).unwrap();
        assert_eq!(body["filter"]["type"], serde_json::json!([2]));
        assert_eq!(body["filter"]["nsfw"], false);
        assert!(body["filter"].get("air_date").is_none());
    }

    #[tokio::test]
    async fn bangumi_provider_uses_query_external_id_for_direct_subject_lookup() {
        let transport = FakeTransport::default();
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: r#"{
                "id": 265,
                "type": 2,
                "name": "新世紀エヴァンゲリオン",
                "name_cn": "新世纪福音战士",
                "summary": "Direct detail summary.",
                "date": "1995-10-04",
                "platform": "TV",
                "images": {
                    "large": "https://lain.bgm.tv/pic/cover/l/detail.jpg"
                },
                "eps": 26,
                "total_episodes": 26,
                "rating": {"rank": 10, "total": 12000, "score": 8.8},
                "infobox": [
                    {"key": "别名", "value": [
                        {"v": "EVA"},
                        "NGE"
                    ]}
                ],
                "meta_tags": ["科幻"],
                "tags": [{"name": "GAINAX", "count": 300}]
            }"#
            .as_bytes()
            .to_vec(),
        }));
        let runtime = ProviderHttpRuntime::with_transport(
            ProviderHttpRuntimeConfig {
                retry_backoff_ms: 0,
                user_agent: "Latias94/test-addon/0.1.0".to_owned(),
                ..ProviderHttpRuntimeConfig::default()
            },
            transport.clone(),
        );
        let provider = BangumiMetadataProvider::with_runtime(
            BangumiProviderConfig {
                access_token: None,
                api_base_url: "https://bangumi.example".to_owned(),
                user_agent: "Latias94/test-addon/0.1.0".to_owned(),
                include_nsfw: false,
                subject_types: vec![2],
                proxy_url: None,
            },
            runtime,
        );

        let candidates = provider
            .suggest(&MetadataQuery {
                title: "Wrong Local Title".to_owned(),
                year: Some(1995),
                language: "zh-CN".to_owned(),
                external_ids: vec![crate::engine::QueryExternalId {
                    provider: "bangumi".to_owned(),
                    value: "265".to_owned(),
                }],
            })
            .await
            .unwrap();

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].provider_id, "bangumi:subject:265");
        assert_eq!(candidates[0].patch.title.as_deref(), Some("新世纪福音战士"));
        assert_eq!(
            candidates[0].patch.overview.as_deref(),
            Some("Direct detail summary.")
        );
        assert!(
            candidates[0]
                .facts
                .external_ids
                .iter()
                .any(|id| id.provider == "bangumi" && id.value == "265")
        );
        assert!(
            candidates[0]
                .facts
                .alternate_titles
                .iter()
                .any(|title| title == "NGE")
        );

        let requests = transport.requests();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].method, ProviderHttpMethod::Get);
        assert_eq!(requests[0].url, "https://bangumi.example/v0/subjects/265");
    }

    #[tokio::test]
    async fn bangumi_provider_falls_back_to_search_when_query_external_id_is_invalid() {
        let transport = FakeTransport::default();
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: r#"{
                "data": [{
                    "id": 265,
                    "type": 2,
                    "name": "新世紀エヴァンゲリオン",
                    "name_cn": "新世纪福音战士",
                    "summary": "Search summary.",
                    "date": "1995-10-04",
                    "platform": "TV",
                    "images": {},
                    "eps": 26,
                    "total_episodes": 26,
                    "rating": {"rank": 12, "total": 10000, "score": 8.7},
                    "infobox": [],
                    "meta_tags": [],
                    "tags": []
                }]
            }"#
            .as_bytes()
            .to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: r#"{
                "id": 265,
                "type": 2,
                "name": "新世紀エヴァンゲリオン",
                "name_cn": "新世纪福音战士",
                "summary": "Detail summary.",
                "date": "1995-10-04",
                "platform": "TV",
                "images": {},
                "eps": 26,
                "total_episodes": 26,
                "rating": {"rank": 10, "total": 12000, "score": 8.8},
                "infobox": [],
                "meta_tags": [],
                "tags": []
            }"#
            .as_bytes()
            .to_vec(),
        }));
        let runtime = ProviderHttpRuntime::with_transport(
            ProviderHttpRuntimeConfig {
                retry_backoff_ms: 0,
                user_agent: "Latias94/test-addon/0.1.0".to_owned(),
                ..ProviderHttpRuntimeConfig::default()
            },
            transport.clone(),
        );
        let provider = BangumiMetadataProvider::with_runtime(
            BangumiProviderConfig {
                access_token: None,
                api_base_url: "https://bangumi.example".to_owned(),
                user_agent: "Latias94/test-addon/0.1.0".to_owned(),
                include_nsfw: false,
                subject_types: vec![2],
                proxy_url: None,
            },
            runtime,
        );

        let candidates = provider
            .suggest(&MetadataQuery {
                title: "新世纪福音战士".to_owned(),
                year: Some(1995),
                language: "zh-CN".to_owned(),
                external_ids: vec![crate::engine::QueryExternalId {
                    provider: "bangumi".to_owned(),
                    value: "not-a-number".to_owned(),
                }],
            })
            .await
            .unwrap();

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].provider_id, "bangumi:subject:265");
        let requests = transport.requests();
        assert_eq!(requests[0].method, ProviderHttpMethod::Post);
        assert_eq!(
            requests[0].url,
            "https://bangumi.example/v0/search/subjects"
        );
        assert_eq!(requests[1].method, ProviderHttpMethod::Get);
        assert_eq!(requests[1].url, "https://bangumi.example/v0/subjects/265");
    }

    #[tokio::test]
    async fn bangumi_provider_uses_later_valid_query_external_id_when_first_is_invalid() {
        let transport = FakeTransport::default();
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: r#"{
                "id": 265,
                "type": 2,
                "name": "新世紀エヴァンゲリオン",
                "name_cn": "新世纪福音战士",
                "summary": "Direct detail summary.",
                "date": "1995-10-04",
                "platform": "TV",
                "images": {},
                "eps": 26,
                "total_episodes": 26,
                "rating": {"rank": 10, "total": 12000, "score": 8.8},
                "infobox": [],
                "meta_tags": [],
                "tags": []
            }"#
            .as_bytes()
            .to_vec(),
        }));
        let runtime = ProviderHttpRuntime::with_transport(
            ProviderHttpRuntimeConfig {
                retry_backoff_ms: 0,
                user_agent: "Latias94/test-addon/0.1.0".to_owned(),
                ..ProviderHttpRuntimeConfig::default()
            },
            transport.clone(),
        );
        let provider = BangumiMetadataProvider::with_runtime(
            BangumiProviderConfig {
                access_token: None,
                api_base_url: "https://bangumi.example".to_owned(),
                user_agent: "Latias94/test-addon/0.1.0".to_owned(),
                include_nsfw: false,
                subject_types: vec![2],
                proxy_url: None,
            },
            runtime,
        );

        let candidates = provider
            .suggest(&MetadataQuery {
                title: "Wrong Local Title".to_owned(),
                year: Some(1995),
                language: "zh-CN".to_owned(),
                external_ids: vec![
                    crate::engine::QueryExternalId {
                        provider: "bangumi".to_owned(),
                        value: "not-a-number".to_owned(),
                    },
                    crate::engine::QueryExternalId {
                        provider: "BANGUMI".to_owned(),
                        value: "265".to_owned(),
                    },
                ],
            })
            .await
            .unwrap();

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].provider_id, "bangumi:subject:265");
        let requests = transport.requests();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].method, ProviderHttpMethod::Get);
        assert_eq!(requests[0].url, "https://bangumi.example/v0/subjects/265");
    }

    #[tokio::test]
    async fn bangumi_provider_falls_back_to_search_when_direct_subject_lookup_fails() {
        let transport = FakeTransport::default();
        transport.push(Ok(ProviderHttpResponse {
            status: 404,
            body: br#"not found"#.to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: r#"{
                "data": [{
                    "id": 265,
                    "type": 2,
                    "name": "新世紀エヴァンゲリオン",
                    "name_cn": "新世纪福音战士",
                    "summary": "Search summary.",
                    "date": "1995-10-04",
                    "platform": "TV",
                    "images": {},
                    "eps": 26,
                    "total_episodes": 26,
                    "rating": {"rank": 12, "total": 10000, "score": 8.7},
                    "infobox": [],
                    "meta_tags": [],
                    "tags": []
                }]
            }"#
            .as_bytes()
            .to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: r#"{
                "id": 265,
                "type": 2,
                "name": "新世紀エヴァンゲリオン",
                "name_cn": "新世纪福音战士",
                "summary": "Recovered detail summary.",
                "date": "1995-10-04",
                "platform": "TV",
                "images": {},
                "eps": 26,
                "total_episodes": 26,
                "rating": {"rank": 10, "total": 12000, "score": 8.8},
                "infobox": [],
                "meta_tags": [],
                "tags": []
            }"#
            .as_bytes()
            .to_vec(),
        }));
        let runtime = ProviderHttpRuntime::with_transport(
            ProviderHttpRuntimeConfig {
                max_attempts: 1,
                retry_backoff_ms: 0,
                user_agent: "Latias94/test-addon/0.1.0".to_owned(),
                ..ProviderHttpRuntimeConfig::default()
            },
            transport.clone(),
        );
        let provider = BangumiMetadataProvider::with_runtime(
            BangumiProviderConfig {
                access_token: None,
                api_base_url: "https://bangumi.example".to_owned(),
                user_agent: "Latias94/test-addon/0.1.0".to_owned(),
                include_nsfw: false,
                subject_types: vec![2],
                proxy_url: None,
            },
            runtime,
        );

        let candidates = provider
            .suggest(&MetadataQuery {
                title: "新世纪福音战士".to_owned(),
                year: Some(1995),
                language: "zh-CN".to_owned(),
                external_ids: vec![crate::engine::QueryExternalId {
                    provider: "bangumi".to_owned(),
                    value: "999999".to_owned(),
                }],
            })
            .await
            .unwrap();

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].provider_id, "bangumi:subject:265");
        assert_eq!(
            candidates[0].patch.overview.as_deref(),
            Some("Recovered detail summary.")
        );
        let requests = transport.requests();
        assert_eq!(requests[0].method, ProviderHttpMethod::Get);
        assert_eq!(
            requests[0].url,
            "https://bangumi.example/v0/subjects/999999"
        );
        assert_eq!(requests[1].method, ProviderHttpMethod::Post);
        assert_eq!(
            requests[1].url,
            "https://bangumi.example/v0/search/subjects"
        );
        assert_eq!(requests[2].method, ProviderHttpMethod::Get);
        assert_eq!(requests[2].url, "https://bangumi.example/v0/subjects/265");
    }

    #[tokio::test]
    async fn bangumi_provider_uses_later_valid_query_external_id_when_first_lookup_fails() {
        let transport = FakeTransport::default();
        transport.push(Ok(ProviderHttpResponse {
            status: 404,
            body: br#"not found"#.to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: r#"{
                "id": 265,
                "type": 2,
                "name": "新世紀エヴァンゲリオン",
                "name_cn": "新世纪福音战士",
                "summary": "Direct detail summary.",
                "date": "1995-10-04",
                "platform": "TV",
                "images": {},
                "eps": 26,
                "total_episodes": 26,
                "rating": {"rank": 10, "total": 12000, "score": 8.8},
                "infobox": [],
                "meta_tags": [],
                "tags": []
            }"#
            .as_bytes()
            .to_vec(),
        }));
        let runtime = ProviderHttpRuntime::with_transport(
            ProviderHttpRuntimeConfig {
                max_attempts: 1,
                retry_backoff_ms: 0,
                user_agent: "Latias94/test-addon/0.1.0".to_owned(),
                ..ProviderHttpRuntimeConfig::default()
            },
            transport.clone(),
        );
        let provider = BangumiMetadataProvider::with_runtime(
            BangumiProviderConfig {
                access_token: None,
                api_base_url: "https://bangumi.example".to_owned(),
                user_agent: "Latias94/test-addon/0.1.0".to_owned(),
                include_nsfw: false,
                subject_types: vec![2],
                proxy_url: None,
            },
            runtime,
        );

        let candidates = provider
            .suggest(&MetadataQuery {
                title: "Wrong Local Title".to_owned(),
                year: Some(1995),
                language: "zh-CN".to_owned(),
                external_ids: vec![
                    crate::engine::QueryExternalId {
                        provider: "bangumi".to_owned(),
                        value: "999999".to_owned(),
                    },
                    crate::engine::QueryExternalId {
                        provider: "bangumi".to_owned(),
                        value: "265".to_owned(),
                    },
                ],
            })
            .await
            .unwrap();

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].provider_id, "bangumi:subject:265");
        let requests = transport.requests();
        assert_eq!(requests[0].method, ProviderHttpMethod::Get);
        assert_eq!(
            requests[0].url,
            "https://bangumi.example/v0/subjects/999999"
        );
        assert_eq!(requests[1].method, ProviderHttpMethod::Get);
        assert_eq!(requests[1].url, "https://bangumi.example/v0/subjects/265");
        assert!(
            requests
                .iter()
                .all(|request| request.url != "https://bangumi.example/v0/search/subjects")
        );
    }

    #[tokio::test]
    async fn bangumi_provider_deduplicates_query_external_ids_before_direct_lookup() {
        let transport = FakeTransport::default();
        transport.push(Ok(ProviderHttpResponse {
            status: 404,
            body: br#"not found"#.to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: r#"{
                "id": 265,
                "type": 2,
                "name": "新世紀エヴァンゲリオン",
                "name_cn": "新世纪福音战士",
                "summary": "Direct detail summary.",
                "date": "1995-10-04",
                "platform": "TV",
                "images": {},
                "eps": 26,
                "total_episodes": 26,
                "rating": {"rank": 10, "total": 12000, "score": 8.8},
                "infobox": [],
                "meta_tags": [],
                "tags": []
            }"#
            .as_bytes()
            .to_vec(),
        }));
        let runtime = ProviderHttpRuntime::with_transport(
            ProviderHttpRuntimeConfig {
                max_attempts: 1,
                retry_backoff_ms: 0,
                user_agent: "Latias94/test-addon/0.1.0".to_owned(),
                ..ProviderHttpRuntimeConfig::default()
            },
            transport.clone(),
        );
        let provider = BangumiMetadataProvider::with_runtime(
            BangumiProviderConfig {
                access_token: None,
                api_base_url: "https://bangumi.example".to_owned(),
                user_agent: "Latias94/test-addon/0.1.0".to_owned(),
                include_nsfw: false,
                subject_types: vec![2],
                proxy_url: None,
            },
            runtime,
        );

        let candidates = provider
            .suggest(&MetadataQuery {
                title: "Wrong Local Title".to_owned(),
                year: Some(1995),
                language: "zh-CN".to_owned(),
                external_ids: vec![
                    crate::engine::QueryExternalId {
                        provider: "bangumi".to_owned(),
                        value: "999999".to_owned(),
                    },
                    crate::engine::QueryExternalId {
                        provider: "BANGUMI".to_owned(),
                        value: "999999".to_owned(),
                    },
                    crate::engine::QueryExternalId {
                        provider: "bangumi".to_owned(),
                        value: "265".to_owned(),
                    },
                ],
            })
            .await
            .unwrap();

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].provider_id, "bangumi:subject:265");
        let requests = transport.requests();
        let failed_lookup_count = requests
            .iter()
            .filter(|request| request.url == "https://bangumi.example/v0/subjects/999999")
            .count();
        assert_eq!(failed_lookup_count, 1);
        assert_eq!(requests[1].url, "https://bangumi.example/v0/subjects/265");
    }

    #[tokio::test]
    async fn bangumi_provider_skips_malformed_search_subject_items() {
        let transport = FakeTransport::default();
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: r#"{
                "data": [
                    {
                        "type": 2,
                        "name": "Malformed Subject",
                        "name_cn": "损坏条目",
                        "summary": "Missing ID should not poison the response.",
                        "date": "1995-10-04",
                        "platform": "TV",
                        "images": {},
                        "eps": 26,
                        "total_episodes": 26,
                        "rating": {"rank": 12, "total": 10000, "score": 8.7},
                        "infobox": [],
                        "meta_tags": [],
                        "tags": []
                    },
                    {
                        "id": 265,
                        "type": 2,
                        "name": "新世紀エヴァンゲリオン",
                        "name_cn": "新世纪福音战士",
                        "summary": "Search summary.",
                        "date": "1995-10-04",
                        "platform": "TV",
                        "images": {},
                        "eps": 26,
                        "total_episodes": 26,
                        "rating": {"rank": 12, "total": 10000, "score": 8.7},
                        "infobox": [],
                        "meta_tags": [],
                        "tags": []
                    }
                ]
            }"#
            .as_bytes()
            .to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: r#"{
                "id": 265,
                "type": 2,
                "name": "新世紀エヴァンゲリオン",
                "name_cn": "新世纪福音战士",
                "summary": "Detail summary.",
                "date": "1995-10-04",
                "platform": "TV",
                "images": {},
                "eps": 26,
                "total_episodes": 26,
                "rating": {"rank": 10, "total": 12000, "score": 8.8},
                "infobox": [],
                "meta_tags": [],
                "tags": []
            }"#
            .as_bytes()
            .to_vec(),
        }));
        let runtime = ProviderHttpRuntime::with_transport(
            ProviderHttpRuntimeConfig {
                retry_backoff_ms: 0,
                user_agent: "Latias94/test-addon/0.1.0".to_owned(),
                ..ProviderHttpRuntimeConfig::default()
            },
            transport.clone(),
        );
        let provider = BangumiMetadataProvider::with_runtime(
            BangumiProviderConfig {
                access_token: None,
                api_base_url: "https://bangumi.example".to_owned(),
                user_agent: "Latias94/test-addon/0.1.0".to_owned(),
                include_nsfw: false,
                subject_types: vec![2],
                proxy_url: None,
            },
            runtime,
        );

        let candidates = provider
            .suggest(&MetadataQuery {
                title: "新世纪福音战士".to_owned(),
                year: Some(1995),
                language: "zh-CN".to_owned(),
                external_ids: Vec::new(),
            })
            .await
            .unwrap();

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].provider_id, "bangumi:subject:265");
        assert_eq!(candidates[0].patch.title.as_deref(), Some("新世纪福音战士"));
        let requests = transport.requests();
        assert_eq!(requests[0].method, ProviderHttpMethod::Post);
        assert_eq!(
            requests[0].url,
            "https://bangumi.example/v0/search/subjects"
        );
        assert_eq!(requests[1].method, ProviderHttpMethod::Get);
        assert_eq!(requests[1].url, "https://bangumi.example/v0/subjects/265");
    }

    #[tokio::test]
    async fn bangumi_provider_reports_error_when_all_search_subject_items_are_malformed() {
        let transport = FakeTransport::default();
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: r#"{
                "data": [
                    {
                        "type": 2,
                        "name": "Malformed Subject One",
                        "date": "1995-10-04"
                    },
                    {
                        "type": 2,
                        "name": "Malformed Subject Two",
                        "date": "1995-10-04"
                    }
                ]
            }"#
            .as_bytes()
            .to_vec(),
        }));
        let runtime = ProviderHttpRuntime::with_transport(
            ProviderHttpRuntimeConfig {
                retry_backoff_ms: 0,
                user_agent: "Latias94/test-addon/0.1.0".to_owned(),
                ..ProviderHttpRuntimeConfig::default()
            },
            transport,
        );
        let provider = BangumiMetadataProvider::with_runtime(
            BangumiProviderConfig {
                access_token: None,
                api_base_url: "https://bangumi.example".to_owned(),
                user_agent: "Latias94/test-addon/0.1.0".to_owned(),
                include_nsfw: false,
                subject_types: vec![2],
                proxy_url: None,
            },
            runtime,
        );

        let error = provider
            .suggest(&MetadataQuery {
                title: "新世纪福音战士".to_owned(),
                year: Some(1995),
                language: "zh-CN".to_owned(),
                external_ids: Vec::new(),
            })
            .await
            .unwrap_err();

        assert!(
            error
                .to_string()
                .contains("all Bangumi search subject items were malformed")
        );
    }

    #[tokio::test]
    async fn bangumi_provider_falls_back_to_normalized_search_title_when_raw_search_is_empty() {
        let transport = FakeTransport::default();
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{"data": []}"#.to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: r#"{
                "data": [{
                    "id": 265,
                    "type": 2,
                    "name": "新世紀エヴァンゲリオン",
                    "name_cn": "新世纪福音战士",
                    "summary": "Search summary.",
                    "date": "1995-10-04",
                    "platform": "TV",
                    "images": {},
                    "eps": 26,
                    "total_episodes": 26,
                    "rating": {"rank": 12, "total": 10000, "score": 8.7},
                    "infobox": [],
                    "meta_tags": [],
                    "tags": []
                }]
            }"#
            .as_bytes()
            .to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: r#"{
                "id": 265,
                "type": 2,
                "name": "新世紀エヴァンゲリオン",
                "name_cn": "新世纪福音战士",
                "summary": "Detail summary.",
                "date": "1995-10-04",
                "platform": "TV",
                "images": {},
                "eps": 26,
                "total_episodes": 26,
                "rating": {"rank": 10, "total": 12000, "score": 8.8},
                "infobox": [],
                "meta_tags": [],
                "tags": []
            }"#
            .as_bytes()
            .to_vec(),
        }));
        let runtime = ProviderHttpRuntime::with_transport(
            ProviderHttpRuntimeConfig {
                retry_backoff_ms: 0,
                user_agent: "Latias94/test-addon/0.1.0".to_owned(),
                ..ProviderHttpRuntimeConfig::default()
            },
            transport.clone(),
        );
        let provider = BangumiMetadataProvider::with_runtime(
            BangumiProviderConfig {
                access_token: None,
                api_base_url: "https://bangumi.example".to_owned(),
                user_agent: "Latias94/test-addon/0.1.0".to_owned(),
                include_nsfw: false,
                subject_types: vec![2],
                proxy_url: None,
            },
            runtime,
        );

        let candidates = provider
            .suggest(&MetadataQuery {
                title: "Neon Genesis Evangelion: TV".to_owned(),
                year: Some(1995),
                language: "zh-CN".to_owned(),
                external_ids: Vec::new(),
            })
            .await
            .unwrap();

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].provider_id, "bangumi:subject:265");

        let requests = transport.requests();
        assert_eq!(requests[0].method, ProviderHttpMethod::Post);
        let raw_body: serde_json::Value =
            serde_json::from_slice(requests[0].json_body.as_ref().unwrap()).unwrap();
        assert_eq!(raw_body["keyword"], "Neon Genesis Evangelion: TV");
        assert_eq!(requests[1].method, ProviderHttpMethod::Post);
        let fallback_body: serde_json::Value =
            serde_json::from_slice(requests[1].json_body.as_ref().unwrap()).unwrap();
        assert_eq!(fallback_body["keyword"], "neon genesis evangelion tv");
        assert_eq!(requests[2].method, ProviderHttpMethod::Get);
        assert_eq!(requests[2].url, "https://bangumi.example/v0/subjects/265");
    }

    #[tokio::test]
    async fn bangumi_provider_merges_search_title_variants_with_deduped_enrichment_budget() {
        let transport = FakeTransport::default();
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: r#"{
                "data": [
                    {
                        "id": 1,
                        "type": 2,
                        "name": "Subject One",
                        "name_cn": "条目一",
                        "summary": "Raw result one.",
                        "date": "2021-01-01",
                        "platform": "TV",
                        "images": {},
                        "eps": 12,
                        "total_episodes": 12,
                        "rating": {"rank": 10, "total": 1000, "score": 8.0},
                        "infobox": [],
                        "meta_tags": [],
                        "tags": []
                    },
                    {
                        "id": 2,
                        "type": 2,
                        "name": "Subject Two",
                        "name_cn": "条目二",
                        "summary": "Raw result two.",
                        "date": "2021-01-02",
                        "platform": "TV",
                        "images": {},
                        "eps": 12,
                        "total_episodes": 12,
                        "rating": {"rank": 20, "total": 900, "score": 7.9},
                        "infobox": [],
                        "meta_tags": [],
                        "tags": []
                    }
                ]
            }"#
            .as_bytes()
            .to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: r#"{
                "data": [
                    {
                        "id": 2,
                        "type": 2,
                        "name": "Subject Two",
                        "name_cn": "条目二",
                        "summary": "Duplicate normalized result.",
                        "date": "2021-01-02",
                        "platform": "TV",
                        "images": {},
                        "eps": 12,
                        "total_episodes": 12,
                        "rating": {"rank": 20, "total": 900, "score": 7.9},
                        "infobox": [],
                        "meta_tags": [],
                        "tags": []
                    },
                    {
                        "id": 3,
                        "type": 2,
                        "name": "Subject Three",
                        "name_cn": "条目三",
                        "summary": "Normalized-only result.",
                        "date": "2021-01-03",
                        "platform": "TV",
                        "images": {},
                        "eps": 12,
                        "total_episodes": 12,
                        "rating": {"rank": 30, "total": 800, "score": 7.8},
                        "infobox": [],
                        "meta_tags": [],
                        "tags": []
                    }
                ]
            }"#
            .as_bytes()
            .to_vec(),
        }));
        for subject_id in [1, 2, 3] {
            transport.push(Ok(ProviderHttpResponse {
                status: 200,
                body: format!(
                    r#"{{
                        "id": {subject_id},
                        "type": 2,
                        "name": "Subject {subject_id}",
                        "name_cn": "条目{subject_id}",
                        "summary": "Detail {subject_id}.",
                        "date": "2021-01-0{subject_id}",
                        "platform": "TV",
                        "images": {{}},
                        "eps": 12,
                        "total_episodes": 12,
                        "rating": {{"rank": {subject_id}, "total": 1000, "score": 8.0}},
                        "infobox": [],
                        "meta_tags": [],
                        "tags": []
                    }}"#
                )
                .into_bytes(),
            }));
        }
        let runtime = ProviderHttpRuntime::with_transport(
            ProviderHttpRuntimeConfig {
                retry_backoff_ms: 0,
                user_agent: "Latias94/test-addon/0.1.0".to_owned(),
                ..ProviderHttpRuntimeConfig::default()
            },
            transport.clone(),
        );
        let provider = BangumiMetadataProvider::with_runtime(
            BangumiProviderConfig {
                access_token: None,
                api_base_url: "https://bangumi.example".to_owned(),
                user_agent: "Latias94/test-addon/0.1.0".to_owned(),
                include_nsfw: false,
                subject_types: vec![2],
                proxy_url: None,
            },
            runtime,
        );

        let candidates = provider
            .suggest(&MetadataQuery {
                title: "Subject: Merge".to_owned(),
                year: Some(2021),
                language: "zh-CN".to_owned(),
                external_ids: Vec::new(),
            })
            .await
            .unwrap();

        assert_eq!(
            candidates
                .iter()
                .map(|candidate| candidate.provider_id.as_str())
                .collect::<Vec<_>>(),
            vec![
                "bangumi:subject:1",
                "bangumi:subject:2",
                "bangumi:subject:3"
            ]
        );

        let requests = transport.requests();
        assert_eq!(requests[0].method, ProviderHttpMethod::Post);
        assert_eq!(requests[1].method, ProviderHttpMethod::Post);
        let fallback_body: serde_json::Value =
            serde_json::from_slice(requests[1].json_body.as_ref().unwrap()).unwrap();
        assert_eq!(fallback_body["keyword"], "subject merge");
        let detail_urls = requests
            .iter()
            .filter(|request| request.method == ProviderHttpMethod::Get)
            .map(|request| request.url.as_str())
            .collect::<Vec<_>>();
        assert_eq!(
            detail_urls,
            vec![
                "https://bangumi.example/v0/subjects/1",
                "https://bangumi.example/v0/subjects/2",
                "https://bangumi.example/v0/subjects/3"
            ]
        );
    }

    #[tokio::test]
    async fn bangumi_provider_prioritizes_more_relevant_merged_search_results_for_enrichment() {
        let transport = FakeTransport::default();
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: r#"{
                "data": [
                    {
                        "id": 1,
                        "type": 2,
                        "name": "Subject Adjacent One",
                        "name_cn": "相邻条目一",
                        "summary": "Weak raw result one.",
                        "date": "2021-01-01",
                        "platform": "TV",
                        "images": {},
                        "eps": 12,
                        "total_episodes": 12,
                        "rating": {"rank": 10, "total": 500, "score": 7.0},
                        "infobox": [],
                        "meta_tags": [],
                        "tags": []
                    },
                    {
                        "id": 2,
                        "type": 2,
                        "name": "Subject Adjacent Two",
                        "name_cn": "相邻条目二",
                        "summary": "Weak raw result two.",
                        "date": "2021-01-02",
                        "platform": "TV",
                        "images": {},
                        "eps": 12,
                        "total_episodes": 12,
                        "rating": {"rank": 20, "total": 500, "score": 7.0},
                        "infobox": [],
                        "meta_tags": [],
                        "tags": []
                    }
                ]
            }"#
            .as_bytes()
            .to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: r#"{
                "data": [
                    {
                        "id": 3,
                        "type": 2,
                        "name": "Subject Merge",
                        "name_cn": "条目合并",
                        "summary": "Strong normalized result.",
                        "date": "2021-01-03",
                        "platform": "TV",
                        "images": {},
                        "eps": 12,
                        "total_episodes": 12,
                        "rating": {"rank": 3, "total": 1500, "score": 8.5},
                        "infobox": [],
                        "meta_tags": [],
                        "tags": []
                    },
                    {
                        "id": 4,
                        "type": 2,
                        "name": "Subject Merge",
                        "name_cn": "条目合并 第二版",
                        "summary": "Second strong normalized result.",
                        "date": "2021-01-04",
                        "platform": "TV",
                        "images": {},
                        "eps": 12,
                        "total_episodes": 12,
                        "rating": {"rank": 4, "total": 1400, "score": 8.4},
                        "infobox": [],
                        "meta_tags": [],
                        "tags": []
                    }
                ]
            }"#
            .as_bytes()
            .to_vec(),
        }));
        for subject_id in [3, 4, 1] {
            transport.push(Ok(ProviderHttpResponse {
                status: 200,
                body: format!(
                    r#"{{
                        "id": {subject_id},
                        "type": 2,
                        "name": "Subject {subject_id}",
                        "name_cn": "条目{subject_id}",
                        "summary": "Detail {subject_id}.",
                        "date": "2021-01-0{subject_id}",
                        "platform": "TV",
                        "images": {{}},
                        "eps": 12,
                        "total_episodes": 12,
                        "rating": {{"rank": {subject_id}, "total": 1000, "score": 8.0}},
                        "infobox": [],
                        "meta_tags": [],
                        "tags": []
                    }}"#
                )
                .into_bytes(),
            }));
        }
        let runtime = ProviderHttpRuntime::with_transport(
            ProviderHttpRuntimeConfig {
                retry_backoff_ms: 0,
                user_agent: "Latias94/test-addon/0.1.0".to_owned(),
                ..ProviderHttpRuntimeConfig::default()
            },
            transport.clone(),
        );
        let provider = BangumiMetadataProvider::with_runtime(
            BangumiProviderConfig {
                access_token: None,
                api_base_url: "https://bangumi.example".to_owned(),
                user_agent: "Latias94/test-addon/0.1.0".to_owned(),
                include_nsfw: false,
                subject_types: vec![2],
                proxy_url: None,
            },
            runtime,
        );

        let candidates = provider
            .suggest(&MetadataQuery {
                title: "Subject: Merge".to_owned(),
                year: Some(2021),
                language: "zh-CN".to_owned(),
                external_ids: Vec::new(),
            })
            .await
            .unwrap();

        assert_eq!(
            candidates
                .iter()
                .map(|candidate| candidate.provider_id.as_str())
                .collect::<Vec<_>>(),
            vec![
                "bangumi:subject:3",
                "bangumi:subject:4",
                "bangumi:subject:1"
            ]
        );

        let requests = transport.requests();
        let detail_urls = requests
            .iter()
            .filter(|request| request.method == ProviderHttpMethod::Get)
            .map(|request| request.url.as_str())
            .collect::<Vec<_>>();
        assert_eq!(
            detail_urls,
            vec![
                "https://bangumi.example/v0/subjects/3",
                "https://bangumi.example/v0/subjects/4",
                "https://bangumi.example/v0/subjects/1"
            ]
        );
    }

    #[tokio::test]
    async fn bangumi_provider_preserves_search_results_when_later_title_variant_search_fails() {
        let transport = FakeTransport::default();
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: r#"{
                "data": [{
                    "id": 1,
                    "type": 2,
                    "name": "Subject Merge",
                    "name_cn": "条目合并",
                    "summary": "Raw search result.",
                    "date": "2021-01-01",
                    "platform": "TV",
                    "images": {},
                    "eps": 12,
                    "total_episodes": 12,
                    "rating": {"rank": 1, "total": 1000, "score": 8.0},
                    "infobox": [],
                    "meta_tags": [],
                    "tags": []
                }]
            }"#
            .as_bytes()
            .to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 503,
            body: br#"temporarily unavailable"#.to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: r#"{
                "id": 1,
                "type": 2,
                "name": "Subject Merge",
                "name_cn": "条目合并",
                "summary": "Detail result.",
                "date": "2021-01-01",
                "platform": "TV",
                "images": {},
                "eps": 12,
                "total_episodes": 12,
                "rating": {"rank": 1, "total": 1000, "score": 8.0},
                "infobox": [],
                "meta_tags": [],
                "tags": []
            }"#
            .as_bytes()
            .to_vec(),
        }));
        let runtime = ProviderHttpRuntime::with_transport(
            ProviderHttpRuntimeConfig {
                max_attempts: 1,
                retry_backoff_ms: 0,
                user_agent: "Latias94/test-addon/0.1.0".to_owned(),
                ..ProviderHttpRuntimeConfig::default()
            },
            transport.clone(),
        );
        let provider = BangumiMetadataProvider::with_runtime(
            BangumiProviderConfig {
                access_token: None,
                api_base_url: "https://bangumi.example".to_owned(),
                user_agent: "Latias94/test-addon/0.1.0".to_owned(),
                include_nsfw: false,
                subject_types: vec![2],
                proxy_url: None,
            },
            runtime,
        );

        let candidates = provider
            .suggest(&MetadataQuery {
                title: "Subject: Merge".to_owned(),
                year: Some(2021),
                language: "zh-CN".to_owned(),
                external_ids: Vec::new(),
            })
            .await
            .unwrap();

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].provider_id, "bangumi:subject:1");
        assert_eq!(
            candidates[0].patch.overview.as_deref(),
            Some("Detail result.")
        );
        let provider_note = crate::engine::render_provider_note(
            &candidates[0].facts.provider_outcomes,
            candidates[0].facts.provider_note.as_deref(),
        )
        .unwrap();
        assert!(provider_note.contains("partial title-variant search failure"));
        assert!(!provider_note.contains("503"));
        assert!(!provider_note.contains("temporarily unavailable"));
        assert!(!provider_note.contains("https://"));

        let requests = transport.requests();
        assert_eq!(requests[0].method, ProviderHttpMethod::Post);
        assert_eq!(requests[1].method, ProviderHttpMethod::Post);
        assert_eq!(requests[2].method, ProviderHttpMethod::Get);
        assert_eq!(requests[2].url, "https://bangumi.example/v0/subjects/1");
    }

    #[tokio::test]
    async fn bangumi_provider_propagates_error_when_all_title_variant_searches_fail() {
        let transport = FakeTransport::default();
        transport.push(Ok(ProviderHttpResponse {
            status: 503,
            body: br#"raw unavailable"#.to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 503,
            body: br#"normalized unavailable"#.to_vec(),
        }));
        let runtime = ProviderHttpRuntime::with_transport(
            ProviderHttpRuntimeConfig {
                max_attempts: 1,
                retry_backoff_ms: 0,
                user_agent: "Latias94/test-addon/0.1.0".to_owned(),
                ..ProviderHttpRuntimeConfig::default()
            },
            transport.clone(),
        );
        let provider = BangumiMetadataProvider::with_runtime(
            BangumiProviderConfig {
                access_token: None,
                api_base_url: "https://bangumi.example".to_owned(),
                user_agent: "Latias94/test-addon/0.1.0".to_owned(),
                include_nsfw: false,
                subject_types: vec![2],
                proxy_url: None,
            },
            runtime,
        );

        let error = provider
            .suggest(&MetadataQuery {
                title: "Subject: Merge".to_owned(),
                year: Some(2021),
                language: "zh-CN".to_owned(),
                external_ids: Vec::new(),
            })
            .await
            .unwrap_err();

        assert!(error.to_string().contains("HTTP 503"));
        let requests = transport.requests();
        assert_eq!(requests.len(), 2);
        assert_eq!(requests[0].method, ProviderHttpMethod::Post);
        assert_eq!(requests[1].method, ProviderHttpMethod::Post);
    }

    #[tokio::test]
    async fn bangumi_provider_returns_degraded_candidate_after_failed_enrichment() {
        let transport = FakeTransport::default();
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: r#"{
                "data": [
                    {
                        "id": 1,
                        "type": 2,
                        "name": "Broken Subject",
                        "name_cn": "失败条目",
                        "summary": "Search result one.",
                        "date": "2021-01-01",
                        "platform": "TV",
                        "images": {
                            "large": "https://lain.bgm.tv/pic/cover/l/broken.jpg"
                        },
                        "eps": 12,
                        "total_episodes": 12,
                        "rating": {"rank": 10, "total": 1000, "score": 8.0},
                        "infobox": [
                            {"key": "别名", "value": [{"v": "Broken Alias"}]}
                        ],
                        "meta_tags": ["科幻"],
                        "tags": [{"name": "测试", "count": 10}]
                    },
                    {
                        "id": 2,
                        "type": 2,
                        "name": "Usable Subject",
                        "name_cn": "可用条目",
                        "summary": "Search result two.",
                        "date": "2021-01-02",
                        "platform": "TV",
                        "images": {},
                        "eps": 12,
                        "total_episodes": 12,
                        "rating": {"rank": 20, "total": 900, "score": 7.9},
                        "infobox": [],
                        "meta_tags": [],
                        "tags": []
                    }
                ]
            }"#
            .as_bytes()
            .to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 503,
            body: br#"temporarily unavailable"#.to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: r#"{
                "id": 2,
                "type": 2,
                "name": "Usable Subject",
                "name_cn": "可用条目",
                "summary": "Detail two.",
                "date": "2021-01-02",
                "platform": "TV",
                "images": {},
                "eps": 12,
                "total_episodes": 12,
                "rating": {"rank": 20, "total": 900, "score": 7.9},
                "infobox": [],
                "meta_tags": [],
                "tags": []
            }"#
            .as_bytes()
            .to_vec(),
        }));
        let runtime = ProviderHttpRuntime::with_transport(
            ProviderHttpRuntimeConfig {
                max_attempts: 1,
                retry_backoff_ms: 0,
                user_agent: "Latias94/test-addon/0.1.0".to_owned(),
                ..ProviderHttpRuntimeConfig::default()
            },
            transport.clone(),
        );
        let provider = BangumiMetadataProvider::with_runtime(
            BangumiProviderConfig {
                access_token: None,
                api_base_url: "https://bangumi.example".to_owned(),
                user_agent: "Latias94/test-addon/0.1.0".to_owned(),
                include_nsfw: false,
                subject_types: vec![2],
                proxy_url: None,
            },
            runtime,
        );

        let candidates = provider
            .suggest(&MetadataQuery {
                title: "Subject".to_owned(),
                year: Some(2021),
                language: "zh-CN".to_owned(),
                external_ids: Vec::new(),
            })
            .await
            .unwrap();

        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[0].provider_id, "bangumi:subject:1");
        assert_eq!(candidates[0].patch.title.as_deref(), Some("失败条目"));
        assert_eq!(
            candidates[0].patch.original_title.as_deref(),
            Some("Broken Subject")
        );
        assert_eq!(
            candidates[0].patch.overview.as_deref(),
            Some("Search result one.")
        );
        assert_eq!(
            candidates[0].patch.release_date.as_deref(),
            Some("2021-01-01")
        );
        assert_eq!(candidates[0].patch.tagline.as_deref(), Some("TV"));
        assert_eq!(
            candidates[0].patch.genres.as_ref().unwrap(),
            &vec!["科幻".to_owned(), "测试".to_owned()]
        );
        assert!(
            candidates[0]
                .facts
                .alternate_titles
                .iter()
                .any(|title| title == "Broken Alias")
        );
        assert_eq!(candidates[0].facts.release_year, Some(2021));
        assert_eq!(candidates[0].facts.language.as_deref(), Some("zh-CN"));
        assert_eq!(candidates[0].facts.community_score_milli, Some(800));
        assert_eq!(candidates[0].facts.community_vote_count, Some(1000));
        assert!(
            candidates[0]
                .facts
                .external_ids
                .iter()
                .any(|id| id.provider == "bangumi" && id.value == "1")
        );
        let provider_note = crate::engine::render_provider_note(
            &candidates[0].facts.provider_outcomes,
            candidates[0].facts.provider_note.as_deref(),
        );
        assert!(provider_note.is_some_and(|note| note.contains("degraded")));
        assert!(
            candidates[0]
                .artwork_candidates
                .iter()
                .any(|candidate| candidate.facts.source_url
                    == "https://lain.bgm.tv/pic/cover/l/broken.jpg")
        );
        assert_eq!(candidates[1].provider_id, "bangumi:subject:2");
        let requests = transport.requests();
        assert_eq!(requests[1].url, "https://bangumi.example/v0/subjects/1");
        assert_eq!(requests[2].url, "https://bangumi.example/v0/subjects/2");
    }
}
