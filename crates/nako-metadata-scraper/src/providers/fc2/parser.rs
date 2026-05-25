use scraper::{Html, Selector};

use crate::engine::av::AvQueryFacts;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct Fc2DetailFacts {
    pub(super) article_id: String,
    pub(super) url: String,
    pub(super) av: AvQueryFacts,
    pub(super) title: String,
    pub(super) overview: Option<String>,
    pub(super) release_date: Option<String>,
    pub(super) release_year: Option<i32>,
    pub(super) runtime_minutes: Option<u32>,
    pub(super) seller: Option<String>,
    pub(super) tags: Vec<String>,
    pub(super) poster_url: Option<String>,
}

pub(super) fn parse_detail_page(
    html: &str,
    detail_url: &str,
    av: AvQueryFacts,
) -> Option<Fc2DetailFacts> {
    let document = Html::parse_document(html);
    let body_text = element_text(&document, "body").unwrap_or_default();
    let info_text = element_text(&document, ".items_article_info, .items_article_HeadInfo")
        .unwrap_or_else(|| body_text.clone());
    let title = first_non_empty(&[
        element_text(&document, "h1").as_deref(),
        element_text(&document, "meta[property=\"og:title\"]").as_deref(),
        Some(av.number.as_str()),
    ])?;
    let release_date = labeled_value(&info_text, &["販売日", "配信開始日", "Release Date"])
        .or_else(|| first_iso_date(&body_text));
    let release_year = release_date.as_deref().and_then(first_year);
    let runtime_minutes = labeled_value(&info_text, &["収録時間", "再生時間", "Runtime"])
        .and_then(|value| parse_minutes(&value));
    let seller = labeled_value(&info_text, &["販売者", "Seller", "メーカー"]);
    let overview = element_text(
        &document,
        ".items_article_Comment, .items_article_description, .comment",
    );
    let tags = link_texts(&document, "a[href*=\"/genre/\"], a[href*=\"/tag/\"]");
    let poster_url = attr_value(&document, "meta[property=\"og:image\"]", "content")
        .or_else(|| {
            attr_value(
                &document,
                ".items_article_MainitemThumb, .items_article_Mainitem img",
                "src",
            )
        })
        .map(normalize_url);

    Some(Fc2DetailFacts {
        article_id: article_id_from_av_number(&av.number)?,
        url: detail_url.to_owned(),
        av,
        title,
        overview,
        release_date,
        release_year,
        runtime_minutes,
        seller,
        tags,
        poster_url,
    })
}

pub(super) fn article_id_from_av_number(number: &str) -> Option<String> {
    number
        .strip_prefix("FC2-")
        .filter(|value| value.chars().all(|character| character.is_ascii_digit()))
        .map(str::to_owned)
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

fn link_texts(document: &Html, selector: &str) -> Vec<String> {
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

fn labeled_value(text: &str, labels: &[&str]) -> Option<String> {
    labels
        .iter()
        .find_map(|label| labeled_value_by_label(text, label))
}

fn labeled_value_by_label(text: &str, label: &str) -> Option<String> {
    let marker = format!("{label}:");
    let start = text.find(&marker)? + marker.len();
    let rest = text[start..].trim();
    let end = [
        "販売日:",
        "配信開始日:",
        "Release Date:",
        "収録時間:",
        "再生時間:",
        "Runtime:",
        "販売者:",
        "Seller:",
        "メーカー:",
    ]
    .into_iter()
    .filter(|next_marker| *next_marker != marker)
    .filter_map(|next_marker| rest.find(next_marker))
    .min()
    .unwrap_or(rest.len());
    Some(normalize_whitespace(&rest[..end])).filter(|value| !value.is_empty())
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

fn parse_minutes(value: &str) -> Option<u32> {
    value
        .split(|character: char| !character.is_ascii_digit())
        .find(|value| !value.is_empty())
        .and_then(|value| value.parse::<u32>().ok())
}

fn normalize_url(value: String) -> String {
    if let Some(value) = value.strip_prefix("//") {
        return format!("https://{value}");
    }
    value
}

fn normalize_whitespace(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}
