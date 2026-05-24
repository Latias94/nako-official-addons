use std::collections::HashSet;

use async_trait::async_trait;
use nako_addon_protocol::{AddonArtworkKind, AddonMetadataPatch};
use serde::{Deserialize, Serialize};

use crate::{
    config::{BangumiProviderConfig, ProviderId},
    engine::{
        MetadataQuery, ProviderCandidateFacts, ProviderExternalId, ProviderMetadataCandidate,
        ranking,
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
const BANGUMI_PARTIAL_SEARCH_NOTE: &str =
    "Bangumi provider preserved candidates after partial title-variant search failure.";

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
            proxy_url: config.proxy_url.clone(),
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
        for subject_id in bangumi_query_subject_ids(query) {
            match self.enrich_subject_candidate_by_id(query, subject_id).await {
                Ok(candidate) => return Ok(vec![candidate]),
                Err(error) => {
                    tracing::warn!(provider = BANGUMI_PROVIDER_ID, %error, "Bangumi direct subject lookup failed; falling back to title search");
                }
            }
        }

        let mut search_subjects = Vec::new();
        let mut seen_subject_ids = HashSet::new();
        let mut last_search_error = None;

        for search_title in query.search_title_variants() {
            let search = match self.search_subjects(query, &search_title).await {
                Ok(search) => search,
                Err(error) => {
                    tracing::warn!(provider = BANGUMI_PROVIDER_ID, %error, "Bangumi title-variant search failed");
                    last_search_error = Some(error);
                    continue;
                }
            };
            for subject in search.data {
                if seen_subject_ids.insert(subject.id) {
                    search_subjects.push(subject);
                }
            }
        }
        if search_subjects.is_empty()
            && let Some(error) = last_search_error.take()
        {
            return Err(error);
        }
        let partial_search = last_search_error.is_some();
        search_subjects = bangumi_ranked_enrichment_subjects(query, search_subjects);

        let mut candidates = Vec::new();
        for subject in search_subjects {
            match self.enrich_subject_candidate(query, subject.clone()).await {
                Ok(mut candidate) => {
                    if partial_search {
                        append_provider_note(
                            &mut candidate.facts.provider_note,
                            BANGUMI_PARTIAL_SEARCH_NOTE,
                        );
                    }
                    candidates.push(candidate);
                }
                Err(error) => {
                    tracing::warn!(provider = BANGUMI_PROVIDER_ID, %error, "returning degraded Bangumi candidate after enrichment failure");
                    let mut candidate = subject.into_degraded_candidate(query);
                    if partial_search {
                        append_provider_note(
                            &mut candidate.facts.provider_note,
                            BANGUMI_PARTIAL_SEARCH_NOTE,
                        );
                    }
                    candidates.push(candidate);
                }
            }
        }

        Ok(candidates)
    }
}

impl<T> BangumiMetadataProvider<T>
where
    T: ProviderHttpTransport,
{
    async fn search_subjects(
        &self,
        query: &MetadataQuery,
        search_title: &str,
    ) -> anyhow::Result<BangumiSubjectSearchResponse> {
        let search_body = BangumiSubjectSearchRequest {
            keyword: search_title.to_owned(),
            sort: "match",
            filter: BangumiSubjectSearchFilter {
                subject_type: self.config.subject_types.clone(),
                nsfw: self.config.include_nsfw,
                air_date: bangumi_air_date_filter(query.year),
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

        BangumiSubjectSearchResponse::from_value(response.body)
    }

    async fn enrich_subject_candidate(
        &self,
        query: &MetadataQuery,
        search_subject: BangumiSubject,
    ) -> anyhow::Result<ProviderMetadataCandidate> {
        let detail = self.fetch_subject(search_subject.id).await?;
        if detail.id == 0 {
            anyhow::bail!(
                "Bangumi subject detail response returned zero id for requested subject {}",
                search_subject.id
            );
        }
        if detail.id != search_subject.id {
            anyhow::bail!(
                "Bangumi subject detail response id {} did not match requested subject {}",
                detail.id,
                search_subject.id
            );
        }
        Ok(BangumiSubjectCandidate {
            search: search_subject,
            detail,
            degraded: false,
        }
        .into_candidate(query))
    }

    async fn enrich_subject_candidate_by_id(
        &self,
        query: &MetadataQuery,
        subject_id: u64,
    ) -> anyhow::Result<ProviderMetadataCandidate> {
        let detail = self.fetch_subject(subject_id).await?;
        if detail.id == 0 {
            anyhow::bail!(
                "Bangumi subject detail response returned zero id for requested subject {subject_id}"
            );
        }
        if detail.id != subject_id {
            anyhow::bail!(
                "Bangumi subject detail response id {} did not match requested subject {subject_id}",
                detail.id
            );
        }
        Ok(BangumiSubjectCandidate {
            search: BangumiSubject::default(),
            detail,
            degraded: false,
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

fn bangumi_ranked_enrichment_subjects(
    query: &MetadataQuery,
    subjects: Vec<BangumiSubject>,
) -> Vec<BangumiSubject> {
    ranking::select_ranked_provider_inputs(
        query,
        subjects,
        BANGUMI_DETAIL_ENRICHMENT_LIMIT,
        |subject| subject.clone().into_degraded_candidate(query),
    )
}

fn bangumi_query_subject_ids(query: &MetadataQuery) -> impl Iterator<Item = u64> + '_ {
    let mut seen = HashSet::new();
    query
        .external_ids
        .iter()
        .filter(|external_id| {
            external_id
                .provider
                .eq_ignore_ascii_case(BANGUMI_PROVIDER_ID)
        })
        .filter_map(|external_id| external_id.value.trim().parse().ok())
        .filter(|subject_id| *subject_id > 0)
        .filter(move |subject_id| seen.insert(*subject_id))
}

fn bangumi_air_date_filter(year: Option<i32>) -> Option<[String; 2]> {
    let year = year?;
    if !(1..=9999).contains(&year) {
        return None;
    }
    Some([
        format!(">={year:04}-01-01"),
        format!("<{:04}-01-01", year.saturating_add(1)),
    ])
}

fn append_provider_note(note: &mut Option<String>, fragment: &str) {
    match note {
        Some(value) => {
            if !value.ends_with(' ') {
                value.push(' ');
            }
            value.push_str(fragment);
        }
        None => *note = Some(fragment.to_owned()),
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
    #[serde(skip_serializing_if = "Option::is_none")]
    air_date: Option<[String; 2]>,
}

#[derive(Debug, Deserialize)]
struct BangumiSubjectSearchResponse {
    #[serde(default)]
    data: Vec<BangumiSubject>,
}

impl BangumiSubjectSearchResponse {
    fn from_value(value: serde_json::Value) -> anyhow::Result<Self> {
        let items = value
            .get("data")
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "failed to parse Bangumi subject search response: missing field `data`"
                )
            })?
            .as_array()
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "failed to parse Bangumi subject search response: `data` must be an array"
                )
            })?;
        let mut skipped_count = 0usize;
        let data = items
            .iter()
            .filter_map(|item| match serde_json::from_value::<BangumiSubject>(item.clone()) {
                Ok(subject) if subject.id > 0 => Some(subject),
                Ok(_) => {
                    skipped_count += 1;
                    tracing::warn!(
                        provider = BANGUMI_PROVIDER_ID,
                        "skipping Bangumi search subject item with zero id"
                    );
                    None
                }
                Err(error) => {
                    skipped_count += 1;
                    tracing::warn!(provider = BANGUMI_PROVIDER_ID, %error, "skipping malformed Bangumi search subject item");
                    None
                }
            })
            .collect();
        if !items.is_empty() && skipped_count == items.len() {
            anyhow::bail!("all Bangumi search subject items were malformed");
        }
        Ok(Self { data })
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
    infobox: Vec<BangumiInfoboxItem>,
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

    fn into_degraded_candidate(self, query: &MetadataQuery) -> ProviderMetadataCandidate {
        BangumiSubjectCandidate {
            search: self.clone(),
            detail: self,
            degraded: true,
        }
        .into_candidate(query)
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

#[derive(Clone, Debug, Deserialize)]
struct BangumiInfoboxItem {
    key: Option<String>,
    #[serde(default)]
    value: serde_json::Value,
}

struct BangumiSubjectCandidate {
    search: BangumiSubject,
    detail: BangumiSubject,
    degraded: bool,
}

impl BangumiSubjectCandidate {
    fn into_candidate(self, query: &MetadataQuery) -> ProviderMetadataCandidate {
        let subject_id = self.detail.id;
        let subject_type = self.detail.subject_type.or(self.search.subject_type);
        let search_name = self.search.name.clone();
        let search_name_cn = self.search.name_cn.clone();
        let original_title = non_empty(self.detail.name).or_else(|| non_empty(self.search.name));
        let localized_title =
            non_empty(self.detail.name_cn).or_else(|| non_empty(self.search.name_cn));
        let title = selected_title(query, localized_title.as_deref(), original_title.as_deref());
        let alternate_titles = bangumi_alternate_titles(
            title.as_deref(),
            [
                original_title.as_deref(),
                localized_title.as_deref(),
                search_name.as_deref(),
                search_name_cn.as_deref(),
            ],
            &self.detail.infobox,
            &self.search.infobox,
        );
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
        if self.degraded {
            tags.push("bangumi_degraded".to_owned());
        }
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
        let mut artwork_candidates = Vec::new();
        if let Some(images) = images {
            push_bangumi_artwork_candidate(
                &mut artwork_candidates,
                subject_id,
                "large",
                images.large,
            );
            push_bangumi_artwork_candidate(
                &mut artwork_candidates,
                subject_id,
                "common",
                images.common,
            );
            push_bangumi_artwork_candidate(
                &mut artwork_candidates,
                subject_id,
                "medium",
                images.medium,
            );
            push_bangumi_artwork_candidate(
                &mut artwork_candidates,
                subject_id,
                "small",
                images.small,
            );
            push_bangumi_artwork_candidate(
                &mut artwork_candidates,
                subject_id,
                "grid",
                images.grid,
            );
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
                alternate_titles,
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
                    if self.degraded {
                        "Bangumi subject candidate degraded from search response after enrichment failure."
                    } else {
                        "Bangumi subject candidate enriched with search and detail responses."
                    }
                    .to_owned(),
                ),
            },
            artwork_candidates,
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
        .find_map(|value| normalize_non_empty(value))
}

fn non_empty(value: Option<String>) -> Option<String> {
    value.and_then(|value| normalize_non_empty(&value))
}

fn bangumi_alternate_titles<const N: usize>(
    selected_title: Option<&str>,
    known_titles: [Option<&str>; N],
    detail_infobox: &[BangumiInfoboxItem],
    search_infobox: &[BangumiInfoboxItem],
) -> Vec<String> {
    let mut titles = Vec::new();
    for title in known_titles.into_iter().flatten() {
        push_unique_title(&mut titles, selected_title, title);
    }
    push_infobox_titles(&mut titles, selected_title, detail_infobox);
    push_infobox_titles(&mut titles, selected_title, search_infobox);
    titles
}

fn push_infobox_titles(
    values: &mut Vec<String>,
    selected_title: Option<&str>,
    infobox: &[BangumiInfoboxItem],
) {
    for item in infobox
        .iter()
        .filter(|item| is_title_like_key(item.key.as_deref()))
    {
        push_infobox_value_titles(values, selected_title, &item.value);
    }
}

fn push_infobox_value_titles(
    values: &mut Vec<String>,
    selected_title: Option<&str>,
    value: &serde_json::Value,
) {
    match value {
        serde_json::Value::String(value) => push_unique_title(values, selected_title, value),
        serde_json::Value::Array(items) => {
            for item in items {
                push_infobox_value_titles(values, selected_title, item);
            }
        }
        serde_json::Value::Object(object) => {
            if let Some(value) = object.get("v").and_then(serde_json::Value::as_str) {
                push_unique_title(values, selected_title, value);
            }
        }
        _ => {}
    }
}

fn is_title_like_key(key: Option<&str>) -> bool {
    let Some(key) = key.map(str::trim).filter(|key| !key.is_empty()) else {
        return false;
    };
    matches!(
        key,
        "别名"
            | "中文名"
            | "英文名"
            | "日文名"
            | "简体中文名"
            | "繁体中文名"
            | "原名"
            | "原作名"
    ) || key.eq_ignore_ascii_case("alias")
        || key.eq_ignore_ascii_case("aliases")
        || key.eq_ignore_ascii_case("title")
        || key.eq_ignore_ascii_case("original title")
        || key.eq_ignore_ascii_case("english title")
}

fn push_unique_title(values: &mut Vec<String>, selected_title: Option<&str>, title: &str) {
    let title = title.trim();
    if title.is_empty()
        || selected_title.is_some_and(|selected| selected == title)
        || values.iter().any(|value| value == title)
    {
        return;
    }
    values.push(title.to_owned());
}

fn release_year(value: Option<&str>) -> Option<u16> {
    let value = value?.trim();
    if value
        .as_bytes()
        .get(4)
        .is_some_and(|value| value.is_ascii_digit())
    {
        return None;
    }
    let year = value.get(0..4)?;
    year.parse::<u16>().ok().filter(|year| *year > 0)
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
    let Some(value) = normalize_non_empty(&value) else {
        return;
    };
    if values.iter().any(|existing| existing == &value) {
        return;
    };
    values.push(value);
}

fn normalize_non_empty(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_owned())
}

fn push_bangumi_artwork_candidate(
    candidates: &mut Vec<crate::engine::ProviderArtworkCandidate>,
    subject_id: u64,
    variant: &str,
    value: Option<String>,
) {
    if let Some(value) = non_empty(value) {
        candidates.push(crate::engine::ProviderArtworkCandidate {
            provider: BANGUMI_PROVIDER_ID.to_owned(),
            provider_id: format!("bangumi:subject:{subject_id}:image:{variant}"),
            facts: crate::engine::ProviderArtworkCandidateFacts {
                kind: AddonArtworkKind::Poster,
                source_url: value,
                language: None,
                width: None,
                height: None,
            },
        });
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
                eps: Some(26),
                total_episodes: Some(26),
                rating: Some(BangumiRating {
                    rank: Some(10),
                    total: Some(12000),
                    score: Some(8.8),
                }),
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
                eps: Some(26),
                total_episodes: Some(26),
                rating: Some(BangumiRating {
                    rank: Some(10),
                    total: Some(12000),
                    score: Some(8.8),
                }),
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
        let provider_note = candidates[0].facts.provider_note.as_deref().unwrap();
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
        assert!(
            candidates[0]
                .facts
                .provider_note
                .as_deref()
                .is_some_and(|note| note.contains("degraded"))
        );
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
