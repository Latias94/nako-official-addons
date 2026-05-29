use scraper::{Html, Selector};

use super::rendered_av;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RenderedMetadataExtraction {
    pub(crate) recipe_id: &'static str,
    pub(crate) title: Option<String>,
    pub(crate) overview: Option<String>,
    pub(crate) release_date: Option<String>,
    pub(crate) release_year: Option<i32>,
    pub(crate) runtime_minutes: Option<u32>,
    pub(crate) genres: Vec<String>,
    pub(crate) tags: Vec<String>,
    pub(crate) poster_urls: Vec<String>,
    pub(crate) score_milli: Option<u16>,
    pub(crate) vote_count: Option<u32>,
    pub(crate) canonical_url: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RenderedMetadataRecipe {
    id: &'static str,
    title: &'static [RenderedSelector],
    overview: &'static [RenderedSelector],
    release_date: &'static [RenderedSelector],
    runtime: &'static [RenderedSelector],
    genres: &'static [RenderedSelector],
    tags: &'static [RenderedSelector],
    poster_urls: &'static [RenderedSelector],
    score: &'static [RenderedSelector],
    vote_count: &'static [RenderedSelector],
    canonical_url: &'static [RenderedSelector],
}

impl RenderedMetadataRecipe {
    #[must_use]
    pub(crate) const fn generic_metadata() -> Self {
        Self {
            id: "generic_metadata",
            title: TITLE_SELECTORS,
            overview: OVERVIEW_SELECTORS,
            release_date: RELEASE_DATE_SELECTORS,
            runtime: RUNTIME_SELECTORS,
            genres: GENRE_SELECTORS,
            tags: TAG_SELECTORS,
            poster_urls: POSTER_URL_SELECTORS,
            score: SCORE_SELECTORS,
            vote_count: VOTE_COUNT_SELECTORS,
            canonical_url: CANONICAL_URL_SELECTORS,
        }
    }

    #[must_use]
    pub(crate) fn extract(&self, html: &str, source_url: &str) -> RenderedMetadataExtraction {
        let document = Html::parse_document(html);
        let body_text = rendered_av::element_text(&document, "body").unwrap_or_default();
        let release_date = first_value(&document, self.release_date).or_else(|| {
            rendered_av::structured_or_labeled_value(
                &document,
                "tr, li, p, .meta, .info, .detail",
                &body_text,
                &[
                    "Release Date",
                    "Released",
                    "Air Date",
                    "Published",
                    "Date",
                    "上映日期",
                    "放送开始",
                    "播放开始",
                    "播出日期",
                ],
                KNOWN_METADATA_LABELS,
            )
            .and_then(|value| rendered_av::first_iso_date(&value).or(Some(value)))
        });
        let runtime_minutes = first_value(&document, self.runtime)
            .or_else(|| {
                rendered_av::structured_or_labeled_value(
                    &document,
                    "tr, li, p, .meta, .info, .detail",
                    &body_text,
                    &["Runtime", "Duration", "Length", "时长", "片长"],
                    KNOWN_METADATA_LABELS,
                )
            })
            .and_then(|value| rendered_av::parse_minutes(&value));
        let score_milli = first_value(&document, self.score)
            .or_else(|| {
                rendered_av::structured_or_labeled_value(
                    &document,
                    "tr, li, p, .meta, .info, .detail",
                    &body_text,
                    &["Rating", "Score", "评分", "評価"],
                    KNOWN_METADATA_LABELS,
                )
            })
            .and_then(|value| rendered_av::parse_rating_milli(&value));
        let vote_count = first_value(&document, self.vote_count)
            .or_else(|| {
                rendered_av::structured_or_labeled_value(
                    &document,
                    "tr, li, p, .meta, .info, .detail",
                    &body_text,
                    &["Votes", "Vote Count", "Reviews", "评分人数", "评价人数"],
                    KNOWN_METADATA_LABELS,
                )
            })
            .and_then(|value| rendered_av::first_u32(&value));

        RenderedMetadataExtraction {
            recipe_id: self.id,
            title: first_value(&document, self.title),
            overview: first_value(&document, self.overview),
            release_year: release_date
                .as_deref()
                .and_then(rendered_av::first_year)
                .or_else(|| rendered_av::first_year(&body_text)),
            release_date: release_date.and_then(|value| {
                rendered_av::first_iso_date(&value)
                    .or_else(|| (!value.trim().is_empty()).then(|| value.trim().to_owned()))
            }),
            runtime_minutes,
            genres: selector_values(&document, self.genres, source_url),
            tags: selector_values(&document, self.tags, source_url),
            poster_urls: selector_values(&document, self.poster_urls, source_url),
            score_milli,
            vote_count,
            canonical_url: first_value(&document, self.canonical_url)
                .map(|value| absolute_rendered_url(source_url, &value)),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct RenderedSelector {
    selector: &'static str,
    attr: Option<&'static str>,
}

impl RenderedSelector {
    const fn text(selector: &'static str) -> Self {
        Self {
            selector,
            attr: None,
        }
    }

    const fn attr(selector: &'static str, attr: &'static str) -> Self {
        Self {
            selector,
            attr: Some(attr),
        }
    }
}

const TITLE_SELECTORS: &[RenderedSelector] = &[
    RenderedSelector::attr("meta[property=\"og:title\"]", "content"),
    RenderedSelector::attr("meta[name=\"twitter:title\"]", "content"),
    RenderedSelector::text("h1"),
    RenderedSelector::text(".title, .movie-title, .video-title, [itemprop=\"name\"]"),
    RenderedSelector::text("title"),
];

const OVERVIEW_SELECTORS: &[RenderedSelector] = &[
    RenderedSelector::attr("meta[name=\"description\"]", "content"),
    RenderedSelector::attr("meta[property=\"og:description\"]", "content"),
    RenderedSelector::text(
        "[itemprop=\"description\"], .summary, .overview, .description, .synopsis",
    ),
];

const RELEASE_DATE_SELECTORS: &[RenderedSelector] = &[
    RenderedSelector::attr("time[datetime]", "datetime"),
    RenderedSelector::attr("[itemprop=\"datePublished\"]", "content"),
    RenderedSelector::text(".release-date, .air-date, .published-date"),
];

const RUNTIME_SELECTORS: &[RenderedSelector] = &[
    RenderedSelector::attr("[itemprop=\"duration\"]", "content"),
    RenderedSelector::text("[itemprop=\"duration\"], .runtime, .duration"),
];

const GENRE_SELECTORS: &[RenderedSelector] = &[
    RenderedSelector::text(".genres a, .genre a, a[href*=\"genre\"]"),
    RenderedSelector::text(".genres, .genre, [itemprop=\"genre\"]"),
];

const TAG_SELECTORS: &[RenderedSelector] = &[
    RenderedSelector::text(".tags a, .tag, a[href*=\"tag\"], [rel=\"tag\"]"),
    RenderedSelector::text(".keyword, .keywords a"),
];

const POSTER_URL_SELECTORS: &[RenderedSelector] = &[
    RenderedSelector::attr("meta[property=\"og:image\"]", "content"),
    RenderedSelector::attr("meta[name=\"twitter:image\"]", "content"),
    RenderedSelector::attr("[itemprop=\"image\"]", "content"),
    RenderedSelector::attr("img.poster, .poster img, .cover img", "src"),
];

const SCORE_SELECTORS: &[RenderedSelector] = &[
    RenderedSelector::attr("[itemprop=\"ratingValue\"]", "content"),
    RenderedSelector::text("[itemprop=\"ratingValue\"], .score, .rating-value"),
];

const VOTE_COUNT_SELECTORS: &[RenderedSelector] = &[
    RenderedSelector::attr("[itemprop=\"ratingCount\"]", "content"),
    RenderedSelector::text("[itemprop=\"ratingCount\"], .vote-count, .votes"),
];

const CANONICAL_URL_SELECTORS: &[RenderedSelector] = &[
    RenderedSelector::attr("link[rel=\"canonical\"]", "href"),
    RenderedSelector::attr("meta[property=\"og:url\"]", "content"),
];

const KNOWN_METADATA_LABELS: &[&str] = &[
    "Release Date",
    "Released",
    "Air Date",
    "Published",
    "Date",
    "Runtime",
    "Duration",
    "Length",
    "Rating",
    "Score",
    "Votes",
    "Vote Count",
    "Reviews",
    "上映日期",
    "放送开始",
    "播放开始",
    "播出日期",
    "时长",
    "片长",
    "评分",
    "評価",
    "评分人数",
    "评价人数",
];

fn first_value(document: &Html, selectors: &[RenderedSelector]) -> Option<String> {
    selectors.iter().find_map(|selector| {
        selector_values(document, &[*selector], "")
            .into_iter()
            .next()
    })
}

fn selector_values(document: &Html, selectors: &[RenderedSelector], base_url: &str) -> Vec<String> {
    let mut values = Vec::new();
    for selector in selectors {
        let Ok(parsed) = Selector::parse(selector.selector) else {
            continue;
        };
        for element in document.select(&parsed) {
            let value = if let Some(attr) = selector.attr {
                element.value().attr(attr).map(str::to_owned)
            } else {
                Some(rendered_av::normalize_whitespace(
                    &element.text().collect::<Vec<_>>().join(" "),
                ))
            };
            let Some(value) = value else {
                continue;
            };
            push_selector_value(&mut values, &value, selector.attr.is_some(), base_url);
        }
    }
    values
}

fn push_selector_value(values: &mut Vec<String>, value: &str, is_url: bool, base_url: &str) {
    let parts = if is_url {
        let value = rendered_av::normalize_whitespace(value);
        (!value.is_empty())
            .then_some(vec![value])
            .unwrap_or_default()
    } else {
        split_selector_value(value)
    };
    for part in parts {
        let value = if is_url && !base_url.is_empty() {
            absolute_rendered_url(base_url, &part)
        } else {
            part
        };
        if !value.is_empty() && !values.iter().any(|existing| existing == &value) {
            values.push(value);
        }
    }
}

fn split_selector_value(value: &str) -> Vec<String> {
    let normalized = rendered_av::normalize_whitespace(value);
    normalized
        .split([',', '/', '|', '、'])
        .map(rendered_av::normalize_whitespace)
        .filter(|value| !value.is_empty())
        .collect()
}

fn absolute_rendered_url(base_url: &str, value: &str) -> String {
    let value = value.trim();
    if value.starts_with('/')
        && !value.starts_with("//")
        && let Some(origin) = url_origin(base_url)
    {
        return format!("{origin}{value}");
    }
    rendered_av::absolute_url(base_url, value)
}

fn url_origin(value: &str) -> Option<&str> {
    let scheme_end = value.find("://")? + 3;
    let path_start = value[scheme_end..]
        .find('/')
        .map(|index| scheme_end + index)
        .unwrap_or(value.len());
    Some(&value[..path_start])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generic_rendered_recipe_extracts_common_metadata_fields() {
        let extraction = RenderedMetadataRecipe::generic_metadata().extract(
            r#"
<html>
  <head>
    <meta property="og:title" content="Recipe Title">
    <meta name="description" content="Recipe overview.">
    <meta property="og:image" content="/poster.jpg">
    <link rel="canonical" href="/movie/123">
  </head>
  <body>
    <main>
      <time datetime="2021-01-02">ignored</time>
      <span itemprop="duration" content="PT121M"></span>
      <span itemprop="ratingValue" content="8.5"></span>
      <span itemprop="ratingCount" content="1200"></span>
      <a href="/genre/drama">Drama</a>
      <a href="/tag/favorite" rel="tag">Favorite</a>
    </main>
  </body>
</html>
"#,
            "https://example.test/detail",
        );

        assert_eq!(extraction.title.as_deref(), Some("Recipe Title"));
        assert_eq!(extraction.overview.as_deref(), Some("Recipe overview."));
        assert_eq!(extraction.release_date.as_deref(), Some("2021-01-02"));
        assert_eq!(extraction.release_year, Some(2021));
        assert_eq!(extraction.runtime_minutes, Some(121));
        assert_eq!(extraction.score_milli, Some(850));
        assert_eq!(extraction.vote_count, Some(1200));
        assert_eq!(
            extraction.poster_urls,
            vec!["https://example.test/poster.jpg".to_owned()]
        );
        assert_eq!(
            extraction.canonical_url.as_deref(),
            Some("https://example.test/movie/123")
        );
        assert!(extraction.genres.contains(&"Drama".to_owned()));
        assert!(extraction.tags.contains(&"Favorite".to_owned()));
    }
}
