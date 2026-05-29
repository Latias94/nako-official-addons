use scraper::{Html, Selector};

use crate::engine::MetadataQuery;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct DoubanSearchResult {
    pub(super) subject_id: String,
    pub(super) url: String,
    pub(super) title: String,
    pub(super) year: Option<i32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct DoubanDetailFacts {
    pub(super) subject_id: String,
    pub(super) url: String,
    pub(super) title: String,
    pub(super) original_title: Option<String>,
    pub(super) summary: Option<String>,
    pub(super) release_date: Option<String>,
    pub(super) release_year: Option<i32>,
    pub(super) runtime_minutes: Option<u32>,
    pub(super) genres: Vec<String>,
    pub(super) rating_milli: Option<u16>,
    pub(super) vote_count: Option<u32>,
    pub(super) poster_url: Option<String>,
}

pub(super) fn parse_search_results(html: &str) -> Vec<DoubanSearchResult> {
    let document = Html::parse_document(html);
    let Ok(link_selector) = Selector::parse("a[href*=\"/subject/\"]") else {
        return Vec::new();
    };
    let mut results = Vec::new();

    for link in document.select(&link_selector) {
        let Some(href) = link.value().attr("href") else {
            continue;
        };
        let Some(subject_id) = subject_id_from_url(href) else {
            continue;
        };
        let title = normalize_whitespace(&link.text().collect::<Vec<_>>().join(" "));
        if title.is_empty() {
            continue;
        }
        let container_text = normalize_whitespace(&link.text().collect::<Vec<_>>().join(" "));
        results.push(DoubanSearchResult {
            subject_id,
            url: href.to_owned(),
            title,
            year: first_year(&container_text),
        });
    }

    results
}

pub(super) fn parse_detail_page(
    html: &str,
    search_result: &DoubanSearchResult,
    query: &MetadataQuery,
) -> Option<DoubanDetailFacts> {
    let document = Html::parse_document(html);
    let body_text = element_text(&document, "body").unwrap_or_default();
    let info_text = element_text(&document, "#info").unwrap_or_default();
    let title = first_non_empty(&[
        element_text(&document, "span[property=\"v:itemreviewed\"]").as_deref(),
        element_text(&document, "h1").as_deref(),
        Some(search_result.title.as_str()),
    ])?;
    let release_date = attr_value(
        &document,
        "span[property=\"v:initialReleaseDate\"]",
        "content",
    )
    .or_else(|| first_iso_date(&body_text));
    let release_year = release_date
        .as_deref()
        .and_then(first_year)
        .or(search_result.year)
        .or(query.year);
    let rating_milli = element_text(&document, "strong[property=\"v:average\"], .rating_num")
        .and_then(|value| parse_rating_milli(&value));
    let vote_count = element_text(&document, "span[property=\"v:votes\"]")
        .and_then(|value| parse_vote_count(&value));
    let summary = element_text(
        &document,
        ".short, span[property=\"v:summary\"], #link-report span",
    )
    .or_else(|| first_non_empty(&[RenderedSummary::from_text(&body_text).as_deref()]));
    let runtime_minutes = labeled_value(&info_text, "片长").and_then(|value| parse_minutes(&value));
    let original_title = labeled_value(&info_text, "又名");
    let genres = labeled_value(&info_text, "类型")
        .map(|value| split_slash_values(&value))
        .unwrap_or_default();
    let poster_url = attr_value(&document, "meta[property=\"og:image\"]", "content")
        .or_else(|| attr_value(&document, "img[rel=\"v:image\"]", "src"));

    Some(DoubanDetailFacts {
        subject_id: search_result.subject_id.clone(),
        url: search_result.url.clone(),
        title: strip_year_suffix(&title),
        original_title,
        summary,
        release_date,
        release_year,
        runtime_minutes,
        genres,
        rating_milli,
        vote_count,
        poster_url,
    })
}

struct RenderedSummary;

impl RenderedSummary {
    fn from_text(text: &str) -> Option<String> {
        text.lines()
            .map(normalize_whitespace)
            .find(|line| line.len() > 16 && !line.contains("豆瓣"))
    }
}

fn element_text(document: &Html, selector: &str) -> Option<String> {
    let selector = Selector::parse(selector).ok()?;
    document
        .select(&selector)
        .next()
        .map(|element| normalize_whitespace(&element.text().collect::<Vec<_>>().join(" ")))
        .filter(|value| !value.is_empty())
}

fn attr_value(document: &Html, selector: &str, attr: &str) -> Option<String> {
    let selector = Selector::parse(selector).ok()?;
    document
        .select(&selector)
        .find_map(|element| element.value().attr(attr))
        .map(str::to_owned)
        .filter(|value| !value.trim().is_empty())
}

fn labeled_value(text: &str, label: &str) -> Option<String> {
    let marker = format!("{label}:");
    let start = text.find(&marker)? + marker.len();
    let rest = text[start..].trim();
    let end = [
        "又名:",
        "片长:",
        "类型:",
        "上映日期:",
        "导演:",
        "编剧:",
        "主演:",
    ]
    .into_iter()
    .filter(|next_marker| *next_marker != marker)
    .filter_map(|next_marker| rest.find(next_marker))
    .min()
    .unwrap_or(rest.len());
    Some(normalize_whitespace(&rest[..end])).filter(|value| !value.is_empty())
}

fn subject_id_from_url(url: &str) -> Option<String> {
    let marker = "/subject/";
    let start = url.find(marker)? + marker.len();
    let rest = &url[start..];
    let end = rest.find('/').unwrap_or(rest.len());
    let subject_id = &rest[..end];
    (!subject_id.is_empty()
        && subject_id
            .chars()
            .all(|character| character.is_ascii_digit()))
    .then(|| subject_id.to_owned())
}

fn first_non_empty(values: &[Option<&str>]) -> Option<String> {
    values
        .iter()
        .flatten()
        .map(|value| normalize_whitespace(value))
        .find(|value| !value.is_empty())
}

fn first_iso_date(text: &str) -> Option<String> {
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

fn first_year(text: &str) -> Option<i32> {
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

fn parse_rating_milli(value: &str) -> Option<u16> {
    let rating = value.trim().parse::<f64>().ok()?;
    Some((rating * 100.0).round().clamp(0.0, 1000.0) as u16)
}

fn parse_vote_count(value: &str) -> Option<u32> {
    value
        .chars()
        .filter(|character| character.is_ascii_digit())
        .collect::<String>()
        .parse()
        .ok()
}

fn parse_minutes(value: &str) -> Option<u32> {
    value
        .chars()
        .take_while(|character| character.is_ascii_digit())
        .collect::<String>()
        .parse()
        .ok()
}

fn split_slash_values(value: &str) -> Vec<String> {
    value
        .split('/')
        .map(normalize_whitespace)
        .filter(|value| !value.is_empty())
        .collect()
}

fn strip_year_suffix(value: &str) -> String {
    let value = normalize_whitespace(value);
    if let Some(index) = value.rfind('(') {
        let suffix = &value[index..];
        if suffix.ends_with(')') && first_year(suffix).is_some() {
            return value[..index].trim().to_owned();
        }
    }
    value
}

fn normalize_whitespace(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}
