use nako_addon_protocol::{AddonArtworkKind, AddonMetadataPatch};

use crate::engine::{
    MetadataQuery, ProviderArtworkCandidate, ProviderArtworkCandidateFacts, ProviderCandidateFacts,
    ProviderExternalId, ProviderMetadataCandidate, ProviderOutcome,
};

use super::{
    BANGUMI_PROVIDER_ID,
    parser::{BangumiInfoboxItem, BangumiSubject, BangumiTag},
};

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
