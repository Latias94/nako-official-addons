use nako_addon_protocol::{AddonArtworkKind, AddonMetadataPatch};

use crate::engine::{
    MetadataQuery, ProviderArtworkCandidate, ProviderArtworkCandidateFacts, ProviderCandidateFacts,
    ProviderExternalId, ProviderMetadataCandidate, ProviderOutcome,
};

use super::{
    BANGUMI_PROVIDER_ID,
    parser::{BangumiInfoboxItem, BangumiSubject, BangumiTag},
};

const OFFICIAL_SITE_INFOBOX_KEYS: &[&str] = &["官方网站", "官方網站", "官网", "官網"];
const END_DATE_INFOBOX_KEYS: &[&str] = &["播放结束", "播放結束", "放送结束", "放送結束"];
const PRODUCTION_INFOBOX_KEYS: &[&str] = &[
    "动画制作",
    "動畫製作",
    "アニメーション制作",
    "制作",
    "製作",
    "制作公司",
    "制作会社",
];
const INFOBOX_TAG_VALUE_LIMIT: usize = 4;

pub(super) struct BangumiSubjectCandidate {
    pub(super) search: BangumiSubject,
    pub(super) detail: BangumiSubject,
    pub(super) degraded: bool,
}

impl BangumiSubjectCandidate {
    pub(super) fn into_candidate(self, query: &MetadataQuery) -> ProviderMetadataCandidate {
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
        let official_sites = infobox_values(
            &self.detail.infobox,
            &self.search.infobox,
            OFFICIAL_SITE_INFOBOX_KEYS,
            INFOBOX_TAG_VALUE_LIMIT,
        );
        let end_dates = infobox_values(
            &self.detail.infobox,
            &self.search.infobox,
            END_DATE_INFOBOX_KEYS,
            INFOBOX_TAG_VALUE_LIMIT,
        );
        let production_values = infobox_values(
            &self.detail.infobox,
            &self.search.infobox,
            PRODUCTION_INFOBOX_KEYS,
            INFOBOX_TAG_VALUE_LIMIT,
        );
        let summary = non_empty(self.detail.summary).or_else(|| non_empty(self.search.summary));
        let release_date = non_empty(self.detail.date).or_else(|| non_empty(self.search.date));
        let platform = non_empty(self.detail.platform).or_else(|| non_empty(self.search.platform));
        let release_year = release_year(release_date.as_deref());
        let genres = genre_tags(&self.detail.meta_tags, &self.detail.tags)
            .or_else(|| genre_tags(&self.search.meta_tags, &self.search.tags));
        let rating = self.detail.rating.or(self.search.rating);
        let images = self.detail.images.or(self.search.images);
        let nsfw = self.detail.nsfw.or(self.search.nsfw);
        let locked = self.detail.locked.or(self.search.locked);
        let series = self.detail.series.or(self.search.series);
        let volumes = self.detail.volumes.or(self.search.volumes);
        let eps = self.detail.eps.or(self.search.eps);
        let total_episodes = self.detail.total_episodes.or(self.search.total_episodes);
        let air_weekday = self.detail.air_weekday.or(self.search.air_weekday);
        let collection = self.detail.collection.or(self.search.collection);

        let mut tags = vec!["bangumi".to_owned()];
        if self.degraded {
            tags.push("bangumi_degraded".to_owned());
        }
        if nsfw == Some(true) {
            tags.push("bangumi_nsfw".to_owned());
        }
        if locked == Some(true) {
            tags.push("bangumi_locked".to_owned());
        }
        if series == Some(true) {
            tags.push("bangumi_series".to_owned());
        }
        if let Some(subject_type) = subject_type {
            tags.push(format!("bangumi_subject_type:{subject_type}"));
        }
        if let Some(volumes) = volumes.filter(|volumes| *volumes > 0) {
            tags.push(format!("bangumi_volumes:{volumes}"));
        }
        if let Some(eps) = eps {
            tags.push(format!("bangumi_eps:{eps}"));
        }
        if let Some(total_episodes) = total_episodes {
            tags.push(format!("bangumi_total_episodes:{total_episodes}"));
        }
        if let Some(air_weekday) = air_weekday.filter(|weekday| (1..=7).contains(weekday)) {
            tags.push(format!("bangumi_air_weekday:{air_weekday}"));
        }
        if let Some(platform) = &platform {
            push_provider_tag(&mut tags, "platform", platform);
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
        if let Some(collection_total) = collection
            .as_ref()
            .and_then(|collection| collection.total())
        {
            tags.push(format!("bangumi_collection_total:{collection_total}"));
        }
        if !official_sites.is_empty() {
            push_unique_non_empty(&mut tags, "bangumi_official_site".to_owned());
        }
        for end_date in &end_dates {
            push_provider_tag(&mut tags, "end_date", end_date);
        }
        for value in &production_values {
            push_provider_tag(&mut tags, "production", value);
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
                av: None,
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
                provider_outcomes: vec![if self.degraded {
                    ProviderOutcome::BangumiSubjectDegraded
                } else {
                    ProviderOutcome::BangumiSubjectEnriched
                }],
                provider_note: None,
            },
            artwork_candidates,
        }
    }
}

pub(super) fn release_year(value: Option<&str>) -> Option<u16> {
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

fn infobox_values(
    detail_infobox: &[BangumiInfoboxItem],
    search_infobox: &[BangumiInfoboxItem],
    keys: &[&str],
    limit: usize,
) -> Vec<String> {
    let mut values = Vec::new();
    for item in detail_infobox.iter().chain(search_infobox) {
        push_infobox_item_values(&mut values, item, keys, limit);
        if values.len() >= limit {
            break;
        }
    }
    values
}

fn push_infobox_item_values(
    values: &mut Vec<String>,
    item: &BangumiInfoboxItem,
    keys: &[&str],
    limit: usize,
) {
    if infobox_key_matches_any(item.key.as_deref(), keys) {
        push_infobox_text_values(values, &item.value, limit);
        return;
    }

    push_infobox_keyed_values(values, &item.value, keys, limit);
}

fn push_infobox_text_values(values: &mut Vec<String>, value: &serde_json::Value, limit: usize) {
    if values.len() >= limit {
        return;
    }

    match value {
        serde_json::Value::String(value) => push_unique_limited_value(values, value, limit),
        serde_json::Value::Array(items) => {
            for item in items {
                push_infobox_text_values(values, item, limit);
                if values.len() >= limit {
                    break;
                }
            }
        }
        serde_json::Value::Object(object) => {
            if let Some(value) = object.get("v").and_then(serde_json::Value::as_str) {
                push_unique_limited_value(values, value, limit);
            }
        }
        _ => {}
    }
}

fn push_infobox_keyed_values(
    values: &mut Vec<String>,
    value: &serde_json::Value,
    keys: &[&str],
    limit: usize,
) {
    if values.len() >= limit {
        return;
    }

    match value {
        serde_json::Value::Array(items) => {
            for item in items {
                push_infobox_keyed_values(values, item, keys, limit);
                if values.len() >= limit {
                    break;
                }
            }
        }
        serde_json::Value::Object(object) => {
            if infobox_key_matches_any(object.get("k").and_then(serde_json::Value::as_str), keys)
                && let Some(value) = object.get("v").and_then(serde_json::Value::as_str)
            {
                push_unique_limited_value(values, value, limit);
            }
        }
        _ => {}
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

fn infobox_key_matches_any(key: Option<&str>, expected: &[&str]) -> bool {
    let Some(key) = key.map(str::trim).filter(|key| !key.is_empty()) else {
        return false;
    };
    expected
        .iter()
        .any(|expected| key.eq_ignore_ascii_case(expected.trim()))
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

fn push_unique_limited_value(values: &mut Vec<String>, value: &str, limit: usize) {
    if values.len() >= limit {
        return;
    }
    let Some(value) = normalize_tag_value(value) else {
        return;
    };
    if values.iter().any(|existing| existing == &value) {
        return;
    }
    values.push(value);
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

fn push_provider_tag(tags: &mut Vec<String>, key: &str, value: &str) {
    let Some(value) = normalize_tag_value(value) else {
        return;
    };
    push_unique_non_empty(tags, format!("bangumi_{key}:{value}"));
}

fn normalize_tag_value(value: &str) -> Option<String> {
    let value = value.split_whitespace().collect::<Vec<_>>().join(" ");
    (!value.is_empty()).then_some(value)
}

fn normalize_non_empty(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_owned())
}

fn push_bangumi_artwork_candidate(
    candidates: &mut Vec<ProviderArtworkCandidate>,
    subject_id: u64,
    variant: &str,
    value: Option<String>,
) {
    if let Some(value) = non_empty(value) {
        candidates.push(ProviderArtworkCandidate {
            provider: BANGUMI_PROVIDER_ID.to_owned(),
            provider_id: format!("bangumi:subject:{subject_id}:image:{variant}"),
            facts: ProviderArtworkCandidateFacts {
                kind: AddonArtworkKind::Poster,
                source_url: value,
                language: None,
                width: None,
                height: None,
            },
        });
    }
}
