use scraper::{Html, Selector};

use crate::engine::av::{AvNumberSource, AvQueryFacts, facts_from_text};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct DmmSearchResult {
    pub(super) cid: String,
    pub(super) url: String,
    pub(super) title: String,
    pub(super) number: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct DmmDetailFacts {
    pub(super) cid: String,
    pub(super) url: String,
    pub(super) av: AvQueryFacts,
    pub(super) title: String,
    pub(super) overview: Option<String>,
    pub(super) release_date: Option<String>,
    pub(super) release_year: Option<i32>,
    pub(super) runtime_minutes: Option<u32>,
    pub(super) actors: Vec<String>,
    pub(super) tags: Vec<String>,
    pub(super) maker: Option<String>,
    pub(super) label: Option<String>,
    pub(super) series: Option<String>,
    pub(super) director: Option<String>,
    pub(super) rating_milli: Option<u16>,
    pub(super) poster_url: Option<String>,
    pub(super) backdrop_urls: Vec<String>,
}

pub(super) fn parse_search_results(html: &str, av: &AvQueryFacts) -> Vec<DmmSearchResult> {
    let document = Html::parse_document(html);
    let Ok(link_selector) = Selector::parse("a[href*=\"cid=\"]") else {
        return Vec::new();
    };
    let mut results = Vec::new();

    for link in document.select(&link_selector) {
        let Some(href) = link.value().attr("href") else {
            continue;
        };
        let Some(cid) = cid_from_url(href) else {
            continue;
        };
        let text = normalize_whitespace(&link.text().collect::<Vec<_>>().join(" "));
        if text.is_empty() || !text_or_cid_matches_av(&text, &cid, av) {
            continue;
        }
        results.push(DmmSearchResult {
            cid,
            url: href.to_owned(),
            title: text,
            number: av.number.clone(),
        });
    }

    results
}

pub(super) fn parse_detail_page(
    html: &str,
    search_result: &DmmSearchResult,
    detail_url: &str,
    av: Option<AvQueryFacts>,
) -> Option<DmmDetailFacts> {
    let document = Html::parse_document(html);
    let body_text = element_text(&document, "body").unwrap_or_default();
    let info_text = element_text(&document, ".product-info, #mu, table, body")
        .unwrap_or_else(|| body_text.clone());
    let title = first_non_empty(&[
        element_text(&document, "h1#title, h1.product-title, h1").as_deref(),
        attr_value(&document, "meta[property=\"og:title\"]", "content").as_deref(),
        Some(search_result.title.as_str()),
    ])?;
    let overview = first_non_empty(&[
        element_text(&document, ".story").as_deref(),
        element_text(&document, ".mg-b20.lh4").as_deref(),
        element_text(&document, "#comment").as_deref(),
        element_text(&document, ".comment").as_deref(),
    ]);
    let release_date = labeled_value(&info_text, &["発売日", "配信開始日", "Release Date"])
        .or_else(|| first_iso_date(&body_text));
    let release_year = release_date.as_deref().and_then(first_year);
    let runtime_minutes =
        labeled_value(&info_text, &["収録時間", "再生時間", "Duration", "Runtime"])
            .and_then(|value| parse_minutes(&value));
    let maker = labeled_value(&info_text, &["メーカー", "Maker", "Studio"]);
    let label = labeled_value(&info_text, &["レーベル", "Label", "Publisher"]);
    let series = labeled_value(&info_text, &["シリーズ", "Series"]);
    let director = labeled_value(&info_text, &["監督", "Director"]);
    let product_number = labeled_value(&info_text, &["品番", "商品番号", "Product ID", "Number"])
        .or_else(|| {
            facts_from_text(&body_text, AvNumberSource::ExternalId).map(|facts| facts.number)
        });
    let actors = link_texts(
        &document,
        "a[href*=\"/actress/\"], a[href*=\"article=actress\"], a[href*=\"article=actor\"]",
    );
    let tags = link_texts(
        &document,
        "a[href*=\"article=keyword\"], a[href*=\"article=genre\"], a[href*=\"/genre/\"], a[href*=\"/keyword/\"]",
    );
    let rating_milli = element_text(
        &document,
        ".review-average, .d-review__average, [class*=\"review\"]",
    )
    .and_then(|value| parse_rating_milli(&value));
    let poster_url = attr_value(&document, "meta[property=\"og:image\"]", "content")
        .or_else(|| attr_value(&document, "#package-image img, .package-image img", "src"))
        .map(normalize_url);
    let backdrop_urls = image_urls(
        &document,
        ".sample-image-block img, a[name*=\"sample\"] img, .sample-image img",
    );

    let av = product_number
        .as_deref()
        .and_then(|value| facts_from_text(value, AvNumberSource::ExternalId))
        .or_else(|| facts_from_text(&search_result.number, AvNumberSource::ExternalId))
        .or(av)?;

    Some(DmmDetailFacts {
        cid: cid_from_url(detail_url).unwrap_or_else(|| search_result.cid.clone()),
        url: detail_url.to_owned(),
        av,
        title,
        overview,
        release_date,
        release_year,
        runtime_minutes,
        actors,
        tags,
        maker,
        label,
        series,
        director,
        rating_milli,
        poster_url,
        backdrop_urls: backdrop_urls.into_iter().map(normalize_url).collect(),
    })
}

pub(super) fn cid_from_url(url: &str) -> Option<String> {
    let marker = "cid=";
    let start = url.find(marker)? + marker.len();
    let rest = &url[start..];
    let end = rest.find(['/', '?', '#', '&']).unwrap_or(rest.len());
    let cid = &rest[..end];
    (!cid.is_empty()).then(|| cid.to_owned())
}

fn text_or_cid_matches_av(text: &str, cid: &str, av: &AvQueryFacts) -> bool {
    [text, cid]
        .into_iter()
        .filter_map(|value| facts_from_text(value, AvNumberSource::ExternalId))
        .any(|facts| facts.number.eq_ignore_ascii_case(&av.number))
        || comparable_number(text).contains(&comparable_number(&av.number))
        || comparable_number(cid).contains(&comparable_number(&av.number))
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
    let markers = [format!("{label}:"), format!("{label}：")];
    for marker in markers {
        let Some(start) = text.find(&marker).map(|index| index + marker.len()) else {
            continue;
        };
        let rest = text[start..].trim();
        let end = [
            "品番:",
            "品番：",
            "商品番号:",
            "商品番号：",
            "Product ID:",
            "Number:",
            "発売日:",
            "発売日：",
            "配信開始日:",
            "配信開始日：",
            "Release Date:",
            "収録時間:",
            "収録時間：",
            "再生時間:",
            "再生時間：",
            "Duration:",
            "Runtime:",
            "メーカー:",
            "メーカー：",
            "Maker:",
            "Studio:",
            "レーベル:",
            "レーベル：",
            "Label:",
            "Publisher:",
            "シリーズ:",
            "シリーズ：",
            "Series:",
            "監督:",
            "監督：",
            "Director:",
        ]
        .into_iter()
        .filter(|next_marker| *next_marker != marker)
        .filter_map(|next_marker| rest.find(next_marker))
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

fn parse_rating_milli(value: &str) -> Option<u16> {
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
