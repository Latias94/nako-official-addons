use async_trait::async_trait;
use nako_addon_protocol::AddonMetadataPatch;
use serde::{Deserialize, Serialize};

use crate::{
    config::{BangumiProviderConfig, ProviderId},
    engine::{
        MetadataQuery, ProviderCandidateFacts, ProviderExternalId, ProviderMetadataCandidate,
    },
    providers::{
        MetadataProvider,
        http_runtime::{
            ProviderHttpResult, ProviderHttpRuntime, ProviderHttpRuntimeConfig,
            ProviderHttpTransport, ReqwestProviderHttpTransport,
        },
    },
};

pub const BANGUMI_PROVIDER_ID: &str = "bangumi";
const BANGUMI_DETAIL_ENRICHMENT_LIMIT: usize = 3;

#[derive(Clone, Debug)]
pub struct BangumiMetadataProvider<T = ReqwestProviderHttpTransport>
where
    T: ProviderHttpTransport,
{
    config: BangumiProviderConfig,
    runtime: ProviderHttpRuntime<T>,
}

impl BangumiMetadataProvider<ReqwestProviderHttpTransport> {
    pub fn new(config: BangumiProviderConfig) -> ProviderHttpResult<Self> {
        let runtime = ProviderHttpRuntime::new(ProviderHttpRuntimeConfig {
            user_agent: config.user_agent.clone(),
            ..ProviderHttpRuntimeConfig::default()
        })?;
        Ok(Self { config, runtime })
    }
}

impl<T> BangumiMetadataProvider<T>
where
    T: ProviderHttpTransport,
{
    #[must_use]
    pub fn with_runtime(config: BangumiProviderConfig, runtime: ProviderHttpRuntime<T>) -> Self {
        Self { config, runtime }
    }

    fn endpoint(&self, path: impl AsRef<str>) -> String {
        let path = path.as_ref();
        format!(
            "{}/{}",
            self.config.api_base_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }

    fn bearer_headers(&self) -> Vec<(String, String)> {
        self.config
            .access_token
            .as_ref()
            .map(|token| vec![("authorization".to_owned(), format!("Bearer {token}"))])
            .unwrap_or_default()
    }
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
        let search_body = BangumiSubjectSearchRequest {
            keyword: query.title.clone(),
            sort: "match",
            filter: BangumiSubjectSearchFilter {
                subject_type: self.config.subject_types.clone(),
                nsfw: self.config.include_nsfw,
            },
        };
        let response = self
            .runtime
            .post_json(
                BANGUMI_PROVIDER_ID,
                "search subjects",
                self.endpoint("v0/search/subjects"),
                vec![
                    (
                        "limit".to_owned(),
                        BANGUMI_DETAIL_ENRICHMENT_LIMIT.to_string(),
                    ),
                    ("offset".to_owned(), "0".to_owned()),
                ],
                self.bearer_headers(),
                &search_body,
            )
            .await?;
        let search = BangumiSubjectSearchResponse::from_value(response.body)?;
        let mut candidates = Vec::new();

        for subject in search
            .data
            .into_iter()
            .take(BANGUMI_DETAIL_ENRICHMENT_LIMIT)
        {
            candidates.push(self.enrich_subject_candidate(query, subject).await?);
        }

        Ok(candidates)
    }
}

impl<T> BangumiMetadataProvider<T>
where
    T: ProviderHttpTransport,
{
    async fn enrich_subject_candidate(
        &self,
        query: &MetadataQuery,
        search_subject: BangumiSubject,
    ) -> anyhow::Result<ProviderMetadataCandidate> {
        let detail = self.fetch_subject(search_subject.id).await?;
        Ok(BangumiSubjectCandidate {
            search: search_subject,
            detail,
        }
        .into_candidate(query))
    }

    async fn fetch_subject(&self, subject_id: u64) -> anyhow::Result<BangumiSubject> {
        let response = self
            .runtime
            .get_json(
                BANGUMI_PROVIDER_ID,
                "subject detail",
                self.endpoint(format!("v0/subjects/{subject_id}")),
                Vec::new(),
                self.bearer_headers(),
            )
            .await?;

        BangumiSubject::from_value(response.body)
    }
}

#[derive(Debug, Serialize)]
struct BangumiSubjectSearchRequest {
    keyword: String,
    sort: &'static str,
    filter: BangumiSubjectSearchFilter,
}

#[derive(Debug, Serialize)]
struct BangumiSubjectSearchFilter {
    #[serde(rename = "type")]
    subject_type: Vec<u8>,
    nsfw: bool,
}

#[derive(Debug, Deserialize)]
struct BangumiSubjectSearchResponse {
    #[serde(default)]
    data: Vec<BangumiSubject>,
}

impl BangumiSubjectSearchResponse {
    fn from_value(value: serde_json::Value) -> anyhow::Result<Self> {
        serde_json::from_value(value).map_err(|error| {
            anyhow::anyhow!("failed to parse Bangumi subject search response: {error}")
        })
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
struct BangumiSubject {
    id: u64,
    #[serde(rename = "type")]
    subject_type: Option<u8>,
    name: Option<String>,
    name_cn: Option<String>,
    summary: Option<String>,
    date: Option<String>,
    platform: Option<String>,
    images: Option<BangumiImages>,
    eps: Option<u32>,
    total_episodes: Option<u32>,
    rating: Option<BangumiRating>,
    #[serde(default)]
    meta_tags: Vec<String>,
    #[serde(default)]
    tags: Vec<BangumiTag>,
}

impl BangumiSubject {
    fn from_value(value: serde_json::Value) -> anyhow::Result<Self> {
        serde_json::from_value(value)
            .map_err(|error| anyhow::anyhow!("failed to parse Bangumi subject response: {error}"))
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
struct BangumiImages {
    large: Option<String>,
    common: Option<String>,
    medium: Option<String>,
    small: Option<String>,
    grid: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
struct BangumiRating {
    rank: Option<u32>,
    total: Option<u32>,
    score: Option<f64>,
}

#[derive(Clone, Debug, Deserialize)]
struct BangumiTag {
    name: Option<String>,
    count: Option<u32>,
}

struct BangumiSubjectCandidate {
    search: BangumiSubject,
    detail: BangumiSubject,
}

impl BangumiSubjectCandidate {
    fn into_candidate(self, query: &MetadataQuery) -> ProviderMetadataCandidate {
        let subject_id = self.detail.id;
        let subject_type = self.detail.subject_type.or(self.search.subject_type);
        let original_title = non_empty(self.detail.name).or_else(|| non_empty(self.search.name));
        let localized_title =
            non_empty(self.detail.name_cn).or_else(|| non_empty(self.search.name_cn));
        let title = selected_title(query, localized_title.as_deref(), original_title.as_deref());
        let title_language = localized_title
            .as_ref()
            .filter(|localized| Some(localized.as_str()) == title.as_deref())
            .map(|_| "zh-CN".to_owned());
        let summary = non_empty(self.detail.summary).or_else(|| non_empty(self.search.summary));
        let release_date = non_empty(self.detail.date).or_else(|| non_empty(self.search.date));
        let platform = non_empty(self.detail.platform).or_else(|| non_empty(self.search.platform));
        let release_year = release_year(release_date.as_deref());
        let genres = genre_tags(&self.detail.meta_tags, &self.detail.tags)
            .or_else(|| genre_tags(&self.search.meta_tags, &self.search.tags));
        let rating = self.detail.rating.or(self.search.rating);
        let images = self.detail.images.or(self.search.images);
        let eps = self.detail.eps.or(self.search.eps);
        let total_episodes = self.detail.total_episodes.or(self.search.total_episodes);

        let mut tags = vec!["bangumi".to_owned()];
        if let Some(subject_type) = subject_type {
            tags.push(format!("bangumi_subject_type:{subject_type}"));
        }
        if let Some(eps) = eps {
            tags.push(format!("bangumi_eps:{eps}"));
        }
        if let Some(total_episodes) = total_episodes {
            tags.push(format!("bangumi_total_episodes:{total_episodes}"));
        }
        if let Some(platform) = &platform {
            tags.push(format!("bangumi_platform:{platform}"));
        }
        if let Some(rating) = &rating {
            if let Some(rank) = rating.rank {
                tags.push(format!("bangumi_rank:{rank}"));
            }
            if let Some(total) = rating.total {
                tags.push(format!("bangumi_rating_total:{total}"));
            }
            if let Some(score) = rating.score {
                tags.push(format!("bangumi_score:{score:.1}"));
            }
        }
        if let Some(images) = images {
            push_image_tag(&mut tags, "large", images.large);
            push_image_tag(&mut tags, "common", images.common);
            push_image_tag(&mut tags, "medium", images.medium);
            push_image_tag(&mut tags, "small", images.small);
            push_image_tag(&mut tags, "grid", images.grid);
        }

        ProviderMetadataCandidate {
            provider: BANGUMI_PROVIDER_ID.to_owned(),
            provider_id: format!("bangumi:subject:{subject_id}"),
            patch: AddonMetadataPatch {
                title: title.clone(),
                original_title: original_title
                    .clone()
                    .filter(|original| Some(original) != title.as_ref()),
                sort_title: title.clone(),
                overview: summary,
                release_date,
                runtime_minutes: None,
                tagline: platform,
                genres,
                tags: Some(tags).filter(|tags| !tags.is_empty()),
            },
            facts: ProviderCandidateFacts {
                title: title.or(original_title).or(localized_title),
                release_year: release_year.map(i32::from),
                language: title_language,
                community_score_milli: rating.as_ref().and_then(|rating| {
                    rating
                        .score
                        .map(|score| (score * 100.0).round().clamp(0.0, 1000.0) as u16)
                }),
                community_vote_count: rating.as_ref().and_then(|rating| rating.total),
                external_ids: vec![ProviderExternalId {
                    provider: BANGUMI_PROVIDER_ID.to_owned(),
                    value: subject_id.to_string(),
                }],
                provider_note: Some(
                    "Bangumi subject candidate enriched with search and detail responses."
                        .to_owned(),
                ),
            },
        }
    }
}

fn selected_title(
    query: &MetadataQuery,
    localized: Option<&str>,
    original: Option<&str>,
) -> Option<String> {
    if title_matches(&query.title, localized) {
        return localized.map(str::to_owned);
    }
    if title_matches(&query.title, original) {
        return original.map(str::to_owned);
    }
    if query.language.to_ascii_lowercase().starts_with("zh") {
        first_non_empty(&[localized, original])
    } else {
        first_non_empty(&[original, localized])
    }
}

fn title_matches(query_title: &str, candidate_title: Option<&str>) -> bool {
    let Some(candidate_title) = candidate_title.filter(|value| !value.trim().is_empty()) else {
        return false;
    };
    query_title == candidate_title
        || normalize_title(query_title) == normalize_title(candidate_title)
}

fn normalize_title(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn first_non_empty(values: &[Option<&str>]) -> Option<String> {
    values
        .iter()
        .flatten()
        .find(|value| !value.trim().is_empty())
        .map(|value| (*value).to_owned())
}

fn non_empty(value: Option<String>) -> Option<String> {
    value.filter(|value| !value.trim().is_empty())
}

fn release_year(value: Option<&str>) -> Option<u16> {
    let year = value?.get(0..4)?;
    year.parse().ok()
}

fn genre_tags(meta_tags: &[String], tags: &[BangumiTag]) -> Option<Vec<String>> {
    let mut values = Vec::new();
    for tag in meta_tags {
        push_unique_non_empty(&mut values, tag.clone());
    }
    let mut provider_tags = tags.iter().collect::<Vec<_>>();
    provider_tags.sort_by_key(|tag| std::cmp::Reverse(tag.count.unwrap_or_default()));
    for tag in provider_tags.into_iter().take(8) {
        if let Some(name) = tag.name.clone() {
            push_unique_non_empty(&mut values, name);
        }
    }

    (!values.is_empty()).then_some(values)
}

fn push_unique_non_empty(values: &mut Vec<String>, value: String) {
    if value.trim().is_empty() || values.iter().any(|existing| existing == &value) {
        return;
    }
    values.push(value);
}

fn push_image_tag(tags: &mut Vec<String>, kind: &str, value: Option<String>) {
    if let Some(value) = non_empty(value) {
        tags.push(format!("bangumi_image_{kind}:{value}"));
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::VecDeque,
        sync::{Arc, Mutex},
    };

    use crate::providers::http_runtime::{
        ProviderHttpMethod, ProviderHttpRequest, ProviderHttpResponse, ProviderHttpRuntimeConfig,
    };

    use super::*;

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
        assert!(candidates[0].patch.tags.as_ref().unwrap().contains(
            &"bangumi_image_large:https://lain.bgm.tv/pic/cover/l/detail.jpg".to_owned()
        ));
        assert_eq!(candidates[0].facts.title.as_deref(), Some("新世纪福音战士"));
        assert_eq!(candidates[0].facts.release_year, Some(1995));
        assert_eq!(candidates[0].facts.language.as_deref(), Some("zh-CN"));
        assert_eq!(candidates[0].facts.community_score_milli, Some(880));
        assert_eq!(candidates[0].facts.community_vote_count, Some(12000));
        assert_eq!(candidates[0].facts.external_ids[0].provider, "bangumi");
        assert_eq!(candidates[0].facts.external_ids[0].value, "265");

        let requests = transport.requests();
        let configs = transport.configs();
        assert_eq!(configs[0].user_agent, "Latias94/test-addon/0.1.0");
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
        assert_eq!(requests[1].method, ProviderHttpMethod::Get);
        assert_eq!(requests[1].url, "https://bangumi.example/v0/subjects/265");
    }

    #[derive(Clone, Default)]
    struct FakeTransport {
        responses: Arc<Mutex<VecDeque<ProviderHttpResult<ProviderHttpResponse>>>>,
        requests: Arc<Mutex<Vec<ProviderHttpRequest>>>,
        configs: Arc<Mutex<Vec<ProviderHttpRuntimeConfig>>>,
    }

    impl FakeTransport {
        fn push(&self, response: ProviderHttpResult<ProviderHttpResponse>) {
            self.responses.lock().unwrap().push_back(response);
        }

        fn requests(&self) -> Vec<ProviderHttpRequest> {
            self.requests.lock().unwrap().clone()
        }

        fn configs(&self) -> Vec<ProviderHttpRuntimeConfig> {
            self.configs.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl ProviderHttpTransport for FakeTransport {
        async fn send(
            &self,
            request: ProviderHttpRequest,
            config: ProviderHttpRuntimeConfig,
        ) -> ProviderHttpResult<ProviderHttpResponse> {
            self.requests.lock().unwrap().push(request);
            self.configs.lock().unwrap().push(config);
            self.responses
                .lock()
                .unwrap()
                .pop_front()
                .unwrap_or_else(|| {
                    Err(
                        crate::providers::http_runtime::ProviderHttpError::Transport {
                            provider_id: BANGUMI_PROVIDER_ID,
                            operation: "fake",
                            message: "fake transport response queue was empty".to_owned(),
                            attempts: 0,
                        },
                    )
                })
        }
    }
}
