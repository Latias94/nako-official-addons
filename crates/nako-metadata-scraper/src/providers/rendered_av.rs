use scraper::{Html, Selector};

use crate::engine::av::{AvNumberSource, AvQueryFacts, facts_from_text};

pub(crate) fn element_text(document: &Html, selector: &str) -> Option<String> {
    let selector = Selector::parse(selector).ok()?;
    document
        .select(&selector)
        .next()
        .map(|element| normalize_whitespace(&element.text().collect::<Vec<_>>().join(" ")))
        .filter(|value| !value.is_empty())
}

pub(crate) fn attr_value(document: &Html, selector: &str, attr: &str) -> Option<String> {
    let selector = Selector::parse(selector).ok()?;
    document
        .select(&selector)
        .find_map(|element| element.value().attr(attr))
        .map(str::to_owned)
        .filter(|value| !value.trim().is_empty())
}

pub(crate) fn link_texts(document: &Html, selector: &str) -> Vec<String> {
    let Ok(selector) = Selector::parse(selector) else {
        return Vec::new();
    };
    document
        .select(&selector)
        .map(|element| normalize_whitespace(&element.text().collect::<Vec<_>>().join(" ")))
        .filter(|value| !value.is_empty())
        .fold(Vec::new(), |mut values, value| {
            if !values.contains(&value) {
                values.push(value);
            }
            values
        })
}

pub(crate) fn image_urls(document: &Html, selector: &str, base_url: &str) -> Vec<String> {
    let Ok(selector) = Selector::parse(selector) else {
        return Vec::new();
    };
    document
        .select(&selector)
        .filter_map(|element| {
            element
                .value()
                .attr("href")
                .or_else(|| element.value().attr("src"))
                .or_else(|| {
                    element
                        .select(&Selector::parse("img").ok()?)
                        .next()
                        .and_then(|image| image.value().attr("src"))
                })
        })
        .map(|value| normalize_url(absolute_url(base_url, value)))
        .filter(|value| !value.trim().is_empty())
        .fold(Vec::new(), |mut values, value| {
            if !values.contains(&value) {
                values.push(value);
            }
            values
        })
}

pub(crate) fn first_non_empty(values: &[Option<&str>]) -> Option<String> {
    values
        .iter()
        .flatten()
        .map(|value| normalize_whitespace(value))
        .find(|value| !value.is_empty())
}

pub(crate) fn labeled_value(text: &str, labels: &[&str], known_labels: &[&str]) -> Option<String> {
    labels
        .iter()
        .find_map(|label| labeled_value_by_label(text, label, known_labels))
}

pub(crate) fn first_iso_date(text: &str) -> Option<String> {
    for token in text.split_whitespace() {
        if token.len() >= 10 && token.as_bytes().get(4) == Some(&b'-') {
            let value = &token[..10];
            if value
                .chars()
                .enumerate()
                .all(|(index, character)| matches!(index, 4 | 7) || character.is_ascii_digit())
            {
                return Some(value.to_owned());
            }
        }
    }
    None
}

pub(crate) fn first_year(text: &str) -> Option<i32> {
    for token in text.split(|character: char| !character.is_ascii_digit()) {
        if token.len() == 4 {
            let year = token.parse::<i32>().ok()?;
            if (1888..=2100).contains(&year) {
                return Some(year);
            }
        }
    }
    None
}

pub(crate) fn parse_minutes(value: &str) -> Option<u32> {
    value
        .split(|character: char| !character.is_ascii_digit())
        .find(|value| !value.is_empty())
        .and_then(|value| value.parse::<u32>().ok())
}

pub(crate) fn parse_rating_milli(value: &str) -> Option<u16> {
    let rating = value
        .split_whitespace()
        .find_map(|token| token.trim().parse::<f64>().ok())?;
    let scaled = if rating <= 5.0 {
        rating * 200.0
    } else {
        rating * 100.0
    };
    Some(scaled.round().clamp(0.0, 1000.0) as u16)
}

pub(crate) fn first_u32(value: &str) -> Option<u32> {
    value
        .split(|character: char| !character.is_ascii_digit())
        .find(|value| !value.is_empty())
        .and_then(|value| value.parse::<u32>().ok())
}

pub(crate) fn absolute_url(base_url: &str, value: &str) -> String {
    let value = value.trim();
    if value.starts_with("http://") || value.starts_with("https://") {
        return value.to_owned();
    }
    if let Some(value) = value.strip_prefix("//") {
        return format!("https://{value}");
    }
    format!(
        "{}/{}",
        base_url.trim_end_matches('/'),
        value.trim_start_matches('/')
    )
}

pub(crate) fn normalize_url(value: String) -> String {
    if let Some(value) = value.strip_prefix("//") {
        return format!("https://{value}");
    }
    value
}

pub(crate) fn normalize_whitespace(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub(crate) fn percent_encode(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.as_bytes() {
        match *byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(char::from(*byte));
            }
            b' ' => encoded.push_str("%20"),
            byte => encoded.push_str(&format!("%{byte:02X}")),
        }
    }
    encoded
}

pub(crate) fn text_or_url_matches_av(text: &str, url: &str, av: &AvQueryFacts) -> bool {
    [text, url]
        .into_iter()
        .filter_map(|value| facts_from_text(value, AvNumberSource::ExternalId))
        .any(|facts| facts.number.eq_ignore_ascii_case(&av.number))
        || compact(text).contains(&compact(&av.number))
        || compact(url).contains(&compact(&av.number))
}

pub(crate) fn id_query_value(url: &str, key: &str) -> Option<String> {
    let marker = format!("{key}=");
    let start = url.find(&marker)? + marker.len();
    let rest = &url[start..];
    let end = rest.find(['/', '?', '#', '&']).unwrap_or(rest.len());
    let id = &rest[..end];
    (!id.is_empty()).then(|| id.to_owned())
}

fn labeled_value_by_label(text: &str, label: &str, known_labels: &[&str]) -> Option<String> {
    let markers = [format!("{label}:"), format!("{label}：")];
    for marker in markers {
        let Some(start) = text.find(&marker).map(|index| index + marker.len()) else {
            continue;
        };
        let rest = text[start..].trim();
        let end = known_labels
            .iter()
            .flat_map(|known_label| [format!("{known_label}:"), format!("{known_label}：")])
            .filter(|next_marker| next_marker != &marker)
            .filter_map(|next_marker| rest.find(&next_marker))
            .min()
            .unwrap_or(rest.len());
        if let Some(value) =
            Some(normalize_whitespace(&rest[..end])).filter(|value| !value.is_empty())
        {
            return Some(value);
        }
    }

    None
}

fn compact(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .map(|character| character.to_ascii_uppercase())
        .collect()
}
