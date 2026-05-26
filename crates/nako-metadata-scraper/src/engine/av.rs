use serde::{Deserialize, Serialize};

use super::query::{MetadataQuery, QueryExternalId};

pub(crate) const AV_NUMBER_EXTERNAL_ID_PROVIDER: &str = "av_number";

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AvNumberRoute {
    Censored,
    Uncensored,
    Fc2,
    Amateur,
    Western,
    Domestic,
    Unknown,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AvNumberSource {
    ExternalId,
    AvNumber,
    Number,
    FileName,
    Filename,
    Path,
    Title,
    Name,
    QueryTitle,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AvQueryFacts {
    pub number: String,
    pub route: AvNumberRoute,
    pub source: AvNumberSource,
    pub search_terms: Vec<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct AvMetadataFacts {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub actors: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub all_actors: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub directors: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub series: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub studio: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publisher: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wanted_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumb_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trailer_url: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub extrafanart_urls: Vec<String>,
}

impl AvMetadataFacts {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.actors.is_empty()
            && self.all_actors.is_empty()
            && self.directors.is_empty()
            && self.series.is_none()
            && self.studio.is_none()
            && self.publisher.is_none()
            && self.maker.is_none()
            && self.label.is_none()
            && self.wanted_count.is_none()
            && self.thumb_url.is_none()
            && self.trailer_url.is_none()
            && self.extrafanart_urls.is_empty()
    }

    #[must_use]
    pub fn non_empty(self) -> Option<Self> {
        (!self.is_empty()).then_some(self)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ParsedAvNumber {
    number: String,
    route: AvNumberRoute,
}

impl AvQueryFacts {
    fn new(parsed: ParsedAvNumber, source: AvNumberSource) -> Self {
        Self {
            search_terms: search_terms_for(&parsed.number),
            number: parsed.number,
            route: parsed.route,
            source,
        }
    }
}

pub(crate) fn facts_from_payload(payload: &serde_json::Value) -> Option<AvQueryFacts> {
    explicit_external_id_value(payload, AV_NUMBER_EXTERNAL_ID_PROVIDER)
        .and_then(|value| parse_number(&value))
        .map(|parsed| AvQueryFacts::new(parsed, AvNumberSource::ExternalId))
        .or_else(|| facts_from_payload_field(payload, "av_number", AvNumberSource::AvNumber))
        .or_else(|| facts_from_payload_field(payload, "number", AvNumberSource::Number))
        .or_else(|| facts_from_payload_field(payload, "file_name", AvNumberSource::FileName))
        .or_else(|| facts_from_payload_field(payload, "filename", AvNumberSource::Filename))
        .or_else(|| facts_from_payload_field(payload, "path", AvNumberSource::Path))
        .or_else(|| facts_from_payload_field(payload, "title", AvNumberSource::Title))
        .or_else(|| facts_from_payload_field(payload, "name", AvNumberSource::Name))
}

pub(crate) fn facts_from_text(value: &str, source: AvNumberSource) -> Option<AvQueryFacts> {
    parse_number(value).map(|parsed| AvQueryFacts::new(parsed, source))
}

pub(crate) fn facts_from_query(query: &MetadataQuery) -> Option<AvQueryFacts> {
    query
        .external_ids
        .iter()
        .find(|external_id| {
            external_id
                .provider
                .eq_ignore_ascii_case(AV_NUMBER_EXTERNAL_ID_PROVIDER)
        })
        .and_then(|external_id| parse_number(&external_id.value))
        .map(|parsed| AvQueryFacts::new(parsed, AvNumberSource::ExternalId))
        .or_else(|| {
            parse_number(&query.title)
                .map(|parsed| AvQueryFacts::new(parsed, AvNumberSource::QueryTitle))
        })
}

pub(crate) fn push_av_external_id(
    external_ids: &mut Vec<QueryExternalId>,
    av_facts: Option<&AvQueryFacts>,
) {
    normalize_av_external_ids(external_ids);

    let Some(facts) = av_facts else {
        return;
    };

    if external_ids.iter().any(|external_id| {
        external_id
            .provider
            .eq_ignore_ascii_case(AV_NUMBER_EXTERNAL_ID_PROVIDER)
            && external_id.value.eq_ignore_ascii_case(&facts.number)
    }) {
        return;
    }

    external_ids.push(QueryExternalId {
        provider: AV_NUMBER_EXTERNAL_ID_PROVIDER.to_owned(),
        value: facts.number.clone(),
    });
}

pub(crate) fn title_for_query(
    raw_title: Option<String>,
    av_facts: Option<&AvQueryFacts>,
) -> String {
    match (raw_title, av_facts) {
        (Some(title), Some(facts)) if should_replace_title_with_av_number(&title) => {
            facts.number.clone()
        }
        (Some(title), _) if !title.is_empty() => title,
        (_, Some(facts)) => facts.number.clone(),
        _ => "Unknown Title".to_owned(),
    }
}

fn facts_from_payload_field(
    payload: &serde_json::Value,
    field: &str,
    source: AvNumberSource,
) -> Option<AvQueryFacts> {
    payload
        .get(field)
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .and_then(parse_number)
        .map(|parsed| AvQueryFacts::new(parsed, source))
}

fn normalize_av_external_ids(external_ids: &mut Vec<QueryExternalId>) {
    external_ids.retain_mut(|external_id| {
        if !external_id
            .provider
            .eq_ignore_ascii_case(AV_NUMBER_EXTERNAL_ID_PROVIDER)
        {
            return true;
        }

        let Some(parsed) = parse_number(&external_id.value) else {
            return false;
        };
        external_id.provider = AV_NUMBER_EXTERNAL_ID_PROVIDER.to_owned();
        external_id.value = parsed.number;
        true
    });
}

fn explicit_external_id_value(payload: &serde_json::Value, provider: &str) -> Option<String> {
    if let Some(values) = payload
        .get("external_ids")
        .and_then(serde_json::Value::as_object)
        && let Some(value) = values.get(provider)
    {
        return scalar_value(value);
    }

    payload
        .get("external_ids")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .find_map(|value| {
            let value_provider = value.get("provider")?.as_str()?.trim();
            if !value_provider.eq_ignore_ascii_case(provider) {
                return None;
            }
            value
                .get("value")
                .or_else(|| value.get("id"))
                .or_else(|| value.get("external_id"))
                .and_then(scalar_value)
        })
}

fn scalar_value(value: &serde_json::Value) -> Option<String> {
    if let Some(value) = value.as_str() {
        return Some(value.trim().to_owned()).filter(|value| !value.is_empty());
    }

    value.as_i64().map(|value| value.to_string())
}

fn parse_number(value: &str) -> Option<ParsedAvNumber> {
    let candidate = normalize_input(value);
    parse_fc2(&candidate)
        .or_else(|| parse_domestic(&candidate))
        .or_else(|| parse_western_date(&candidate))
        .or_else(|| parse_amateur(&candidate))
        .or_else(|| parse_uncensored(&candidate))
        .or_else(|| parse_censored(&candidate))
}

fn normalize_input(value: &str) -> String {
    let file_name = value.rsplit(['/', '\\']).next().unwrap_or(value).trim();
    let without_extension = strip_known_video_extension(file_name);
    let mut normalized = String::with_capacity(without_extension.len());
    for character in without_extension.chars() {
        if character.is_ascii_alphanumeric() {
            normalized.push(character.to_ascii_uppercase());
        } else if matches!(
            character,
            '-' | '_' | '.' | ' ' | '[' | ']' | '(' | ')' | '{' | '}'
        ) {
            normalized.push(' ');
        } else {
            normalized.push(' ');
        }
    }
    normalized.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn strip_known_video_extension(value: &str) -> &str {
    let Some((stem, extension)) = value.rsplit_once('.') else {
        return value;
    };
    let extension = extension.to_ascii_lowercase();
    if matches!(
        extension.as_str(),
        "avi"
            | "flv"
            | "m2ts"
            | "m4v"
            | "mkv"
            | "mov"
            | "mp4"
            | "mpeg"
            | "mpg"
            | "ts"
            | "webm"
            | "wmv"
    ) {
        stem
    } else {
        value
    }
}

fn parse_fc2(value: &str) -> Option<ParsedAvNumber> {
    let tokens = tokens(value);
    for index in 0..tokens.len() {
        let token = tokens[index];
        if let Some(digits) = token
            .strip_prefix("FC2PPV")
            .or_else(|| token.strip_prefix("FC2"))
            .and_then(leading_digits)
            && digits.len() >= 5
        {
            return Some(ParsedAvNumber {
                number: format!("FC2-{digits}"),
                route: AvNumberRoute::Fc2,
            });
        }
        if token == "FC2" {
            let next_digits = match (tokens.get(index + 1), tokens.get(index + 2)) {
                (Some(&"PPV"), Some(value)) => leading_digits(value),
                (Some(value), _) => leading_digits(value),
                _ => None,
            };
            if let Some(digits) = next_digits.filter(|digits| digits.len() >= 5) {
                return Some(ParsedAvNumber {
                    number: format!("FC2-{digits}"),
                    route: AvNumberRoute::Fc2,
                });
            }
        }
        if token == "FC2PPV"
            && let Some(digits) = tokens
                .get(index + 1)
                .and_then(|value| leading_digits(value))
                .filter(|digits| digits.len() >= 5)
        {
            return Some(ParsedAvNumber {
                number: format!("FC2-{digits}"),
                route: AvNumberRoute::Fc2,
            });
        }
    }

    None
}

fn parse_domestic(value: &str) -> Option<ParsedAvNumber> {
    let tokens = tokens(value);
    for index in 0..tokens.len() {
        let token = tokens[index];
        if token.starts_with("MD") && token.chars().any(|character| character.is_ascii_digit()) {
            return Some(ParsedAvNumber {
                number: join_mixed_number_token(token),
                route: AvNumberRoute::Domestic,
            });
        }
        if token == "MKY"
            && let (Some(series), Some(number)) = (tokens.get(index + 1), tokens.get(index + 2))
            && is_ascii_letters(series)
            && number.chars().all(|character| character.is_ascii_digit())
        {
            return Some(ParsedAvNumber {
                number: format!("MKY-{series}-{number}"),
                route: AvNumberRoute::Domestic,
            });
        }
    }

    None
}

fn parse_western_date(value: &str) -> Option<ParsedAvNumber> {
    let tokens = tokens(value);
    for window in tokens.windows(4) {
        if is_ascii_letters(window[0])
            && window[0].len() >= 3
            && is_two_or_four_digit_year(window[1])
            && is_two_digit_month(window[2])
            && is_two_digit_day(window[3])
        {
            return Some(ParsedAvNumber {
                number: format!("{}.{}.{}.{}", window[0], window[1], window[2], window[3]),
                route: AvNumberRoute::Western,
            });
        }
    }

    None
}

fn parse_amateur(value: &str) -> Option<ParsedAvNumber> {
    const AMATEUR_PREFIXES: &[&str] = &["SIRO", "LUXU", "ARA", "GANA", "MAAN", "MIUM"];
    let tokens = tokens(value);
    for index in 0..tokens.len() {
        let token = tokens[index];
        if AMATEUR_PREFIXES.contains(&token)
            && let Some(number) = tokens.get(index + 1)
            && number.chars().all(|character| character.is_ascii_digit())
            && number.len() >= 3
        {
            return Some(ParsedAvNumber {
                number: format!("{token}-{number}"),
                route: AvNumberRoute::Amateur,
            });
        }
        if AMATEUR_PREFIXES
            .iter()
            .any(|prefix| token.ends_with(prefix))
            && token.chars().any(|character| character.is_ascii_digit())
            && let Some(number) = tokens.get(index + 1)
            && number.chars().all(|character| character.is_ascii_digit())
            && number.len() >= 3
        {
            return Some(ParsedAvNumber {
                number: format!("{token}-{number}"),
                route: AvNumberRoute::Amateur,
            });
        }
    }

    None
}

fn parse_uncensored(value: &str) -> Option<ParsedAvNumber> {
    for token in tokens(value) {
        if token.len() == 5
            && matches!(token.as_bytes()[0], b'C' | b'H' | b'N')
            && token[1..]
                .chars()
                .all(|character| character.is_ascii_digit())
        {
            return Some(ParsedAvNumber {
                number: token.to_owned(),
                route: AvNumberRoute::Uncensored,
            });
        }
    }

    None
}

fn parse_censored(value: &str) -> Option<ParsedAvNumber> {
    let tokens = tokens(value);
    for index in 0..tokens.len() {
        let token = tokens[index];
        if let Some(parsed) = parse_single_censored_token(token) {
            return Some(parsed);
        }
        if is_ascii_letters(token)
            && (2..=10).contains(&token.len())
            && !is_noise_token(token)
            && let Some(number) = tokens.get(index + 1)
            && number.chars().all(|character| character.is_ascii_digit())
            && (2..=7).contains(&number.len())
        {
            return Some(ParsedAvNumber {
                number: format!("{token}-{}", normalize_digits(number)),
                route: AvNumberRoute::Censored,
            });
        }
    }

    None
}

fn parse_single_censored_token(token: &str) -> Option<ParsedAvNumber> {
    let prefix_len = token
        .chars()
        .take_while(|character| character.is_ascii_alphabetic())
        .count();
    let suffix = &token[prefix_len..];
    let prefix = &token[..prefix_len];
    if !(2..=10).contains(&prefix.len())
        || suffix.len() < 2
        || suffix.len() > 7
        || !suffix.chars().all(|character| character.is_ascii_digit())
        || is_noise_token(prefix)
    {
        return None;
    }

    Some(ParsedAvNumber {
        number: format!("{prefix}-{}", normalize_digits(suffix)),
        route: AvNumberRoute::Censored,
    })
}

fn should_replace_title_with_av_number(title: &str) -> bool {
    let normalized = title.to_ascii_lowercase();
    normalized.contains('/')
        || normalized.contains('\\')
        || normalized.contains("1080p")
        || normalized.contains("2160p")
        || normalized.contains("720p")
        || normalized.contains("x264")
        || normalized.contains("x265")
        || normalized.contains("h264")
        || normalized.contains("h265")
        || normalized.contains(".mp4")
        || normalized.contains(".mkv")
        || normalized.contains(".avi")
}

fn search_terms_for(number: &str) -> Vec<String> {
    let compact = number.replace(['-', '.', ' '], "");
    if compact == number {
        vec![number.to_owned()]
    } else {
        vec![number.to_owned(), compact]
    }
}

fn tokens(value: &str) -> Vec<&str> {
    value.split_whitespace().collect()
}

fn leading_digits(value: &str) -> Option<&str> {
    let end = value
        .chars()
        .take_while(|character| character.is_ascii_digit())
        .count();
    (end > 0).then(|| &value[..end])
}

fn is_ascii_letters(value: &str) -> bool {
    value
        .chars()
        .all(|character| character.is_ascii_alphabetic())
}

fn is_two_or_four_digit_year(value: &str) -> bool {
    (value.len() == 2 || value.len() == 4)
        && value.chars().all(|character| character.is_ascii_digit())
}

fn is_two_digit_month(value: &str) -> bool {
    parse_two_digit_range(value, 1, 12)
}

fn is_two_digit_day(value: &str) -> bool {
    parse_two_digit_range(value, 1, 31)
}

fn parse_two_digit_range(value: &str, min: u8, max: u8) -> bool {
    value.len() == 2
        && value
            .parse::<u8>()
            .is_ok_and(|value| (min..=max).contains(&value))
}

fn join_mixed_number_token(token: &str) -> String {
    let prefix_len = token
        .chars()
        .take_while(|character| character.is_ascii_alphabetic())
        .count();
    if prefix_len == 0 || prefix_len == token.len() {
        return token.to_owned();
    }
    format!(
        "{}-{}",
        &token[..prefix_len],
        normalize_digits(&token[prefix_len..])
    )
}

fn normalize_digits(value: &str) -> String {
    if value.len() > 3 {
        let trimmed = value.trim_start_matches('0');
        if !trimmed.is_empty() {
            return trimmed.to_owned();
        }
    }
    value.to_owned()
}

fn is_noise_token(value: &str) -> bool {
    matches!(
        value,
        "AAC" | "AVC" | "BD" | "FHD" | "HD" | "HEVC" | "HDR" | "UHD" | "WEB" | "XVID"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn av_number_normalizes_fc2_ppv_filename() {
        let facts = facts_from_payload(&serde_json::json!({
            "file_name": "FC2PPV-1723984-4K.mp4"
        }))
        .unwrap();

        assert_eq!(facts.number, "FC2-1723984");
        assert_eq!(facts.route, AvNumberRoute::Fc2);
        assert_eq!(facts.source, AvNumberSource::FileName);
        assert_eq!(facts.search_terms, vec!["FC2-1723984", "FC21723984"]);
    }

    #[test]
    fn av_number_normalizes_censored_filename_noise() {
        let facts = facts_from_payload(&serde_json::json!({
            "filename": "[HD] ssni00644 1080p x264.mkv"
        }))
        .unwrap();

        assert_eq!(facts.number, "SSNI-644");
        assert_eq!(facts.route, AvNumberRoute::Censored);
        assert_eq!(facts.source, AvNumberSource::Filename);
    }

    #[test]
    fn av_number_classifies_western_date_like_number() {
        let facts = facts_from_payload(&serde_json::json!({
            "path": "D:/media/BrazzersExxtra.21.02.01.mp4"
        }))
        .unwrap();

        assert_eq!(facts.number, "BRAZZERSEXXTRA.21.02.01");
        assert_eq!(facts.route, AvNumberRoute::Western);
        assert_eq!(facts.source, AvNumberSource::Path);
    }

    #[test]
    fn av_number_prefers_explicit_external_id() {
        let facts = facts_from_payload(&serde_json::json!({
            "external_ids": {"av_number": "abp001"},
            "file_name": "FC2-1723984.mp4"
        }))
        .unwrap();

        assert_eq!(facts.number, "ABP-001");
        assert_eq!(facts.route, AvNumberRoute::Censored);
        assert_eq!(facts.source, AvNumberSource::ExternalId);
    }

    #[test]
    fn av_number_classifies_mgstage_numeric_prefix_amateur_number() {
        let facts = facts_from_payload(&serde_json::json!({
            "file_name": "300MIUM-382.mp4"
        }))
        .unwrap();

        assert_eq!(facts.number, "300MIUM-382");
        assert_eq!(facts.route, AvNumberRoute::Amateur);
        assert_eq!(facts.source, AvNumberSource::FileName);
    }
}
