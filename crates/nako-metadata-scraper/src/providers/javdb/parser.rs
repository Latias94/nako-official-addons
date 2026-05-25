use scraper::{Html, Selector};

use crate::engine::av::AvQueryFacts;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct JavdbSearchResult {
    pub(super) movie_id: String,
    pub(super) url: String,
    pub(super) title: String,
    pub(super) number: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct JavdbDetailFacts {
    pub(super) movie_id: String,
    pub(super) url: String,
    pub(super) av: AvQueryFacts,
    pub(super) title: String,
    pub(super) release_date: Option<String>,
    pub(super) release_year: Option<i32>,
    pub(super) runtime_minutes: Option<u32>,
    pub(super) actors: Vec<String>,
    pub(super) tags: Vec<String>,
    pub(super) studio: Option<String>,
    pub(super) publisher: Option<String>,
    pub(super) series: Option<String>,
    pub(super) director: Option<String>,
    pub(super) rating_milli: Option<u16>,
    pub(super) wanted_count: Option<u32>,
    pub(super) poster_url: Option<String>,
    pub(super) backdrop_urls: Vec<String>,
}

pub(super) fn parse_search_results(html: &str, query_number: &str) -> Vec<JavdbSearchResult> {
    let document = Html::parse_document(html);
    let Ok(link_selector) = Selector::parse("a.box[href], a[href*=\"/v/\"]") else {
        return Vec::new();
    };
    let normalized_query = comparable_number(query_number);
    let mut results = Vec::new();

    for link in document.select(&link_selector) {
        let Some(href) = link.value().attr("href") else {
            continue;
        };
        let Some(movie_id) = movie_id_from_url(href) else {
            continue;
        };
        let text = normalize_whitespace(&link.text().collect::<Vec<_>>().join(" "));
        if text.is_empty() || !comparable_number(&text).contains(&normalized_query) {
            continue;
        }
        results.push(JavdbSearchResult {
            movie_id,
            url: href.to_owned(),
            title: text.clone(),
            number: query_number.to_owned(),
        });
    }

    results
}

pub(super) fn parse_detail_page(
    html: &str,
    search_result: &JavdbSearchResult,
    detail_url: &str,
    av: AvQueryFacts,
) -> Option<JavdbDetailFacts> {
    let document = Html::parse_document(html);
    let body_text = element_text(&document, "body").unwrap_or_default();
    let info_text = element_text(&document, ".movie-panel-info, .video-meta-panel")
        .unwrap_or_else(|| element_text(&document, "body").unwrap_or_default());
    let title = first_non_empty(&[
        element_text(&document, "strong.current-title").as_deref(),
        element_text(&document, "h2.title").as_deref(),
        Some(search_result.title.as_str()),
    ])?;
    let release_date = labeled_value(&info_text, &["日期", "發行日期", "発売日", "Release Date"])
        .or_else(|| first_iso_date(&body_text));
    let release_year = release_date.as_deref().and_then(first_year);
    let runtime_minutes = labeled_value(&info_text, &["時長", "片長", "収録時間", "Runtime"])
        .and_then(|value| parse_minutes(&value));
    let studio = labeled_value(&info_text, &["片商", "製作商", "Studio", "Maker"]);
    let publisher = labeled_value(&info_text, &["發行", "发行", "Publisher", "Label"]);
    let series = labeled_value(&info_text, &["系列", "Series"]);
    let director = labeled_value(&info_text, &["導演", "导演", "Director"]);
    let actors = link_texts(&document, "a[href*=\"/actors/\"]");
    let tags = link_texts(&document, "a[href*=\"/tags/\"]");
    let rating_milli = element_text(&document, ".score, .rating, strong.score")
        .and_then(|value| parse_rating_milli(&value));
    let wanted_count = element_text(&document, ".wanted, .want")
        .or_else(|| labeled_value(&body_text, &["想看", "Wanted"]))
        .and_then(|value| parse_first_u32(&value));
    let poster_url = attr_value(&document, "meta[property=\"og:image\"]", "content")
        .or_else(|| attr_value(&document, ".cover img, .movie-cover img", "src"));
    let backdrop_urls = image_urls(
        &document,
        ".preview-images img, .tile-images img, a.tile-item img",
    );

    Some(JavdbDetailFacts {
        movie_id: search_result.movie_id.clone(),
        url: detail_url.to_owned(),
        av,
        title,
        release_date,
        release_year,
        runtime_minutes,
        actors,
        tags,
        studio,
        publisher,
        series,
        director,
        rating_milli,
        wanted_count,
        poster_url: poster_url.map(normalize_url),
        backdrop_urls: backdrop_urls.into_iter().map(normalize_url).collect(),
    })
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

fn image_urls(document: &Html, selector: &str) -> Vec<String> {
    let Ok(selector) = Selector::parse(selector) else {
        return Vec::new();
    };
    document
        .select(&selector)
        .filter_map(|element| element.value().attr("src"))
        .map(str::to_owned)
        .filter(|value| !value.trim().is_empty())
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
        "日期:",
        "發行日期:",
        "発売日:",
        "Release Date:",
        "時長:",
        "片長:",
        "収録時間:",
        "Runtime:",
        "片商:",
        "製作商:",
        "Studio:",
        "Maker:",
        "發行:",
        "发行:",
        "Publisher:",
        "Label:",
        "系列:",
        "Series:",
        "導演:",
        "导演:",
        "Director:",
        "想看:",
        "Wanted:",
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
    parse_first_u32(value)
}

fn parse_first_u32(value: &str) -> Option<u32> {
    value
        .split(|character: char| !character.is_ascii_digit())
        .find(|value| !value.is_empty())
        .and_then(|value| value.parse::<u32>().ok())
}

fn parse_rating_milli(value: &str) -> Option<u16> {
    let rating = value.trim().parse::<f64>().ok()?;
    let scaled = if rating <= 5.0 {
        rating * 200.0
    } else {
        rating * 100.0
    };
    Some(scaled.round().clamp(0.0, 1000.0) as u16)
}

fn movie_id_from_url(url: &str) -> Option<String> {
    let marker = "/v/";
    let start = url.find(marker)? + marker.len();
    let rest = &url[start..];
    let end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let movie_id = &rest[..end];
    (!movie_id.is_empty()).then(|| movie_id.to_owned())
}

fn comparable_number(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .map(|character| character.to_ascii_uppercase())
        .collect()
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
