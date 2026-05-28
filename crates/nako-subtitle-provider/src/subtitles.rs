use serde::{Deserialize, Serialize};

use crate::{Config, manifest::SUBTITLE_RESPONSE_SCHEMA};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SubtitleSearchRequest {
    pub schema: String,
    pub query: String,
    #[serde(default)]
    pub languages: Vec<String>,
    #[serde(default)]
    pub limit: Option<usize>,
    #[serde(default)]
    pub context: serde_json::Value,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SubtitleSearchResponse {
    pub schema: String,
    pub query: String,
    pub total: usize,
    pub subtitles: Vec<SubtitleCandidate>,
    pub provider_executions: Vec<SubtitleProviderExecution>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SubtitleCandidate {
    pub id: String,
    pub title: String,
    pub language: String,
    pub format: SubtitleFormat,
    pub source: String,
    pub release: Option<String>,
    pub score: u16,
    pub delivery: SubtitleDelivery,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SubtitleFormat {
    Vtt,
    Srt,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SubtitleDelivery {
    Inline { text: String },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SubtitleProviderExecution {
    pub provider_id: String,
    pub status: SubtitleProviderStatus,
    pub result_count: usize,
    pub safe_message: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SubtitleProviderStatus {
    Ok,
    Skipped,
}

#[must_use]
pub fn search_subtitles(
    config: &Config,
    request: SubtitleSearchRequest,
) -> Option<SubtitleSearchResponse> {
    let query = normalize_query(&request.query)?;
    let limit = request
        .limit
        .unwrap_or(config.default_limit)
        .clamp(1, config.max_limit);
    let languages = requested_languages(&request.languages, &config.default_language);

    let (subtitles, execution) = if config.fixture_provider_enabled {
        let subtitles = fixture_subtitles(&query, &languages)
            .into_iter()
            .take(limit)
            .collect::<Vec<_>>();
        let result_count = subtitles.len();
        (
            subtitles,
            SubtitleProviderExecution {
                provider_id: "fixture".to_owned(),
                status: SubtitleProviderStatus::Ok,
                result_count,
                safe_message: None,
            },
        )
    } else {
        (
            Vec::new(),
            SubtitleProviderExecution {
                provider_id: "fixture".to_owned(),
                status: SubtitleProviderStatus::Skipped,
                result_count: 0,
                safe_message: Some("provider_disabled".to_owned()),
            },
        )
    };

    Some(SubtitleSearchResponse {
        schema: SUBTITLE_RESPONSE_SCHEMA.to_owned(),
        query,
        total: subtitles.len(),
        subtitles,
        provider_executions: vec![execution],
    })
}

fn fixture_subtitles(query: &str, languages: &[String]) -> Vec<SubtitleCandidate> {
    languages
        .iter()
        .take(2)
        .enumerate()
        .map(|(index, language)| fixture_subtitle(query, language, index))
        .collect()
}

fn fixture_subtitle(query: &str, language: &str, index: usize) -> SubtitleCandidate {
    let slug = slugify(query);
    let format = if index % 2 == 0 {
        SubtitleFormat::Vtt
    } else {
        SubtitleFormat::Srt
    };
    let text = match format {
        SubtitleFormat::Vtt => {
            format!(
                "WEBVTT\n\n00:00:01.000 --> 00:00:04.000\n{query} fixture subtitle ({language})\n"
            )
        }
        SubtitleFormat::Srt => {
            format!("1\n00:00:01,000 --> 00:00:04,000\n{query} fixture subtitle ({language})\n")
        }
    };

    SubtitleCandidate {
        id: format!("fixture:{slug}:{language}:{index}"),
        title: format!("{query} fixture subtitles {language}"),
        language: language.to_owned(),
        format,
        source: "fixture".to_owned(),
        release: Some("WEB-DL".to_owned()),
        score: 900_u16.saturating_sub((index as u16).saturating_mul(20)),
        delivery: SubtitleDelivery::Inline { text },
    }
}

fn requested_languages(languages: &[String], default_language: &str) -> Vec<String> {
    let mut normalized = languages
        .iter()
        .filter_map(|language| normalize_language(language))
        .collect::<Vec<_>>();
    if normalized.is_empty() {
        normalized.push(
            normalize_language(default_language)
                .unwrap_or_else(|| Config::DEFAULT_LANGUAGE.to_owned()),
        );
    }
    normalized.sort();
    normalized.dedup();
    normalized
}

fn normalize_query(query: &str) -> Option<String> {
    let query = query.trim();
    if query.is_empty() {
        None
    } else {
        Some(query.to_owned())
    }
}

fn normalize_language(language: &str) -> Option<String> {
    let language = language.trim();
    if language.is_empty()
        || language.len() > 32
        || !language
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-')
    {
        None
    } else {
        Some(language.to_ascii_lowercase())
    }
}

fn slugify(value: &str) -> String {
    let slug = value
        .chars()
        .filter_map(|ch| {
            if ch.is_ascii_alphanumeric() {
                Some(ch.to_ascii_lowercase())
            } else if ch.is_whitespace() || ch == '-' || ch == '_' {
                Some('-')
            } else {
                None
            }
        })
        .collect::<String>();
    let slug = slug.trim_matches('-');
    if slug.is_empty() {
        "subtitle".to_owned()
    } else {
        slug.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixture_search_returns_inline_read_only_subtitle_candidates() {
        let response = search_subtitles(
            &Config::default(),
            SubtitleSearchRequest {
                schema: "nako.official.subtitle_provider.request.v1".to_owned(),
                query: "Demo Movie".to_owned(),
                languages: vec!["zh-CN".to_owned(), "en".to_owned()],
                limit: Some(10),
                context: serde_json::Value::Null,
            },
        )
        .unwrap();

        assert_eq!(response.total, 2);
        assert_eq!(response.subtitles[0].source, "fixture");
        assert_eq!(response.subtitles[0].format, SubtitleFormat::Vtt);
        assert!(matches!(
            response.subtitles[0].delivery,
            SubtitleDelivery::Inline { .. }
        ));
        assert_eq!(
            response.provider_executions[0].status,
            SubtitleProviderStatus::Ok
        );
    }

    #[test]
    fn disabled_fixture_reports_safe_skipped_execution() {
        let config = Config {
            fixture_provider_enabled: false,
            ..Config::default()
        };
        let response = search_subtitles(
            &config,
            SubtitleSearchRequest {
                schema: "nako.official.subtitle_provider.request.v1".to_owned(),
                query: "Demo Movie".to_owned(),
                languages: Vec::new(),
                limit: None,
                context: serde_json::Value::Null,
            },
        )
        .unwrap();

        assert_eq!(response.total, 0);
        assert_eq!(
            response.provider_executions[0].safe_message.as_deref(),
            Some("provider_disabled")
        );
    }
}
