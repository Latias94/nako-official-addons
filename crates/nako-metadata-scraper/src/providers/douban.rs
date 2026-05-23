use async_trait::async_trait;
use nako_addon_protocol::{AddonArtworkKind, AddonMetadataPatch};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

use crate::{
    config::{DoubanProviderConfig, ProviderId},
    engine::{
        MetadataQuery, ProviderCandidateFacts, ProviderExternalId, ProviderMetadataCandidate,
    },
    providers::{
        MetadataProvider,
        http_runtime::{
            ProviderHttpResult, ProviderHttpRuntime, ProviderHttpRuntimeConfig,
            ProviderHttpTransport, ReqwestProviderHttpTransport,
        },
    },
};

pub const DOUBAN_PROVIDER_ID: &str = "douban";
const DOUBAN_DETAIL_ENRICHMENT_LIMIT: usize = 1;

#[derive(Clone, Debug)]
pub struct DoubanMetadataProvider<T = ReqwestProviderHttpTransport>
where
    T: ProviderHttpTransport,
{
    config: DoubanProviderConfig,
    runtime: ProviderHttpRuntime<T>,
}

impl DoubanMetadataProvider<ReqwestProviderHttpTransport> {
    pub fn new(config: DoubanProviderConfig) -> ProviderHttpResult<Self> {
        let runtime = ProviderHttpRuntime::new(ProviderHttpRuntimeConfig {
            timeout_ms: config.timeout_ms,
            ..ProviderHttpRuntimeConfig::default()
        })?;
        Ok(Self { config, runtime })
    }
}

impl<T> DoubanMetadataProvider<T>
where
    T: ProviderHttpTransport,
{
    #[must_use]
    pub fn with_runtime(config: DoubanProviderConfig, runtime: ProviderHttpRuntime<T>) -> Self {
        Self { config, runtime }
    }

    fn render_endpoint(&self) -> String {
        format!(
            "{}/{}",
            self.config.browser_worker_base_url.trim_end_matches('/'),
            self.config.render_path.trim_start_matches('/')
        )
    }

    fn search_url(&self, query: &MetadataQuery) -> String {
        format!(
            "{}?search_text={}",
            self.config.search_base_url.trim_end_matches('?'),
            percent_encode_query(&query.title)
        )
    }

    async fn render(&self, url: String) -> anyhow::Result<RenderedPage> {
        let response = self
            .runtime
            .post_json(
                DOUBAN_PROVIDER_ID,
                "render page",
                self.render_endpoint(),
                Vec::new(),
                Vec::new(),
                &RenderPageRequest { url },
            )
            .await?;
        RenderedPage::from_value(response.body)
    }
}

#[async_trait]
impl<T> MetadataProvider for DoubanMetadataProvider<T>
where
    T: ProviderHttpTransport,
{
    fn id(&self) -> ProviderId {
        ProviderId::Douban
    }

    async fn suggest(
        &self,
        query: &MetadataQuery,
    ) -> anyhow::Result<Vec<ProviderMetadataCandidate>> {
        let search = self.render(self.search_url(query)).await?;
        let search_results = parse_search_results(&search.html);
        let mut candidates = Vec::new();

        for result in search_results
            .into_iter()
            .take(DOUBAN_DETAIL_ENRICHMENT_LIMIT)
        {
            let detail = self.render(result.url.clone()).await?;
            if let Some(detail) = parse_detail_page(&detail.html, &result, query) {
                candidates.push(detail.into_candidate(query));
            }
        }

        Ok(candidates)
    }
}

#[derive(Debug, Serialize)]
struct RenderPageRequest {
    url: String,
}

#[derive(Debug, Deserialize)]
struct RenderedPage {
    #[serde(default)]
    status: Option<String>,
    #[serde(rename = "url")]
    _url: String,
    #[serde(rename = "title")]
    _title: Option<String>,
    html: String,
    #[serde(rename = "text")]
    _text: Option<String>,
    #[serde(rename = "excerpt")]
    _excerpt: Option<String>,
}

impl RenderedPage {
    fn from_value(value: serde_json::Value) -> anyhow::Result<Self> {
        let page: Self = serde_json::from_value(value).map_err(|error| {
            anyhow::anyhow!("failed to parse browser worker render response: {error}")
        })?;
        if page.status.as_deref() != Some("ok") {
            anyhow::bail!(
                "browser worker returned non-ok status for rendered page: {:?}",
                page.status
            );
        }
        Ok(page)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct DoubanSearchResult {
    subject_id: String,
    url: String,
    title: String,
    year: Option<i32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct DoubanDetailFacts {
    subject_id: String,
    url: String,
    title: String,
    original_title: Option<String>,
    summary: Option<String>,
    release_date: Option<String>,
    release_year: Option<i32>,
    runtime_minutes: Option<u32>,
    genres: Vec<String>,
    rating_milli: Option<u16>,
    vote_count: Option<u32>,
    poster_url: Option<String>,
}

impl DoubanDetailFacts {
    fn into_candidate(self, query: &MetadataQuery) -> ProviderMetadataCandidate {
        let mut tags = vec![DOUBAN_PROVIDER_ID.to_owned()];
        if let Some(rating) = self.rating_milli {
            tags.push(format!("douban_rating:{:.1}", f64::from(rating) / 100.0));
        }
        if let Some(votes) = self.vote_count {
            tags.push(format!("douban_votes:{votes}"));
        }
        let mut artwork_candidates = Vec::new();
        if let Some(poster_url) = self.poster_url.clone() {
            artwork_candidates.push(crate::engine::ProviderArtworkCandidate {
                provider: DOUBAN_PROVIDER_ID.to_owned(),
                provider_id: format!("douban:subject:{}:poster", self.subject_id),
                facts: crate::engine::ProviderArtworkCandidateFacts {
                    kind: AddonArtworkKind::Poster,
                    source_url: poster_url,
                    language: None,
                    width: None,
                    height: None,
                },
            });
        }

        ProviderMetadataCandidate {
            provider: DOUBAN_PROVIDER_ID.to_owned(),
            provider_id: format!("douban:subject:{}", self.subject_id),
            patch: AddonMetadataPatch {
                title: Some(self.title.clone()),
                original_title: self
                    .original_title
                    .clone()
                    .filter(|original_title| original_title != &self.title),
                sort_title: Some(self.title.clone()),
                overview: self.summary.clone(),
                release_date: self.release_date.clone(),
                runtime_minutes: self.runtime_minutes,
                tagline: Some("Douban movie subject".to_owned()),
                genres: Some(self.genres.clone()).filter(|genres| !genres.is_empty()),
                tags: Some(tags).filter(|tags| !tags.is_empty()),
            },
            facts: ProviderCandidateFacts {
                title: Some(self.title),
                release_year: self.release_year,
                language: Some(query.language.clone()),
                community_score_milli: self.rating_milli,
                community_vote_count: self.vote_count,
                external_ids: vec![
                    ProviderExternalId {
                        provider: DOUBAN_PROVIDER_ID.to_owned(),
                        value: self.subject_id,
                    },
                    ProviderExternalId {
                        provider: "douban_url".to_owned(),
                        value: self.url,
                    },
                ],
                provider_note: Some(
                    "Douban candidate parsed from browser-worker rendered HTML.".to_owned(),
                ),
            },
            artwork_candidates,
        }
    }
}

fn parse_search_results(html: &str) -> Vec<DoubanSearchResult> {
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

fn parse_detail_page(
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

fn percent_encode_query(value: &str) -> String {
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

#[cfg(test)]
mod tests {
    use std::{
        collections::VecDeque,
        sync::{Arc, Mutex},
    };

    use async_trait::async_trait;

    use crate::{
        config::DoubanProviderConfig,
        providers::http_runtime::{
            ProviderHttpRequest, ProviderHttpResponse, ProviderHttpResult, ProviderHttpRuntime,
            ProviderHttpRuntimeConfig, ProviderHttpTransport,
        },
    };

    use super::*;

    #[tokio::test]
    async fn douban_provider_uses_browser_worker_render_contract_for_search_and_detail() {
        let transport = FakeTransport::default();
        transport.push_rendered_html(
            "https://movie.douban.com/subject_search?search_text=%E5%8D%83%E4%B8%8E%E5%8D%83%E5%AF%BB",
            "Douban Search",
            r#"
<!doctype html>
<html>
<body>
  <div class="result">
    <a class="title" href="https://movie.douban.com/subject/1291561/">千与千寻</a>
    <span class="year">2001</span>
  </div>
</body>
</html>"#,
        );
        transport.push_rendered_html(
            "https://movie.douban.com/subject/1291561/",
            "千与千寻 (豆瓣)",
            r#"
<!doctype html>
<html>
<head>
  <meta property="og:image" content="https://img1.doubanio.com/view/photo/s_ratio_poster/public/p123.jpg">
</head>
<body>
  <h1>
    <span property="v:itemreviewed">千与千寻</span>
    <span class="year">(2001)</span>
  </h1>
  <div id="info">
    <span class="pl">又名:</span> 神隐少女 / Spirited Away
    <span class="pl">片长:</span> 125分钟
    <span class="pl">类型:</span> 剧情 / 动画 / 奇幻
  </div>
  <span property="v:initialReleaseDate" content="2001-07-20">2001-07-20</span>
  <strong class="ll rating_num" property="v:average">9.4</strong>
  <span property="v:votes">2345678</span>
  <span class="short">少女误入神灵世界。</span>
</body>
</html>"#,
        );
        let runtime = ProviderHttpRuntime::with_transport(
            ProviderHttpRuntimeConfig {
                retry_backoff_ms: 0,
                ..ProviderHttpRuntimeConfig::default()
            },
            transport.clone(),
        );
        let provider = DoubanMetadataProvider::with_runtime(
            DoubanProviderConfig {
                search_base_url: "https://movie.douban.com/subject_search".to_owned(),
                browser_worker_base_url: "http://browser-worker.example".to_owned(),
                render_path: "/render".to_owned(),
                timeout_ms: 10_000,
            },
            runtime,
        );

        let candidates = provider
            .suggest(&MetadataQuery {
                title: "千与千寻".to_owned(),
                year: Some(2001),
                language: "zh-CN".to_owned(),
                external_ids: Vec::new(),
            })
            .await
            .unwrap();

        assert_eq!(candidates.len(), 1);
        let candidate = &candidates[0];
        assert_eq!(candidate.provider, "douban");
        assert_eq!(candidate.provider_id, "douban:subject:1291561");
        assert_eq!(candidate.patch.title.as_deref(), Some("千与千寻"));
        assert_eq!(
            candidate.patch.original_title.as_deref(),
            Some("神隐少女 / Spirited Away")
        );
        assert_eq!(candidate.patch.release_date.as_deref(), Some("2001-07-20"));
        assert_eq!(candidate.patch.runtime_minutes, Some(125));
        assert_eq!(
            candidate.patch.overview.as_deref(),
            Some("少女误入神灵世界。")
        );
        assert_eq!(
            candidate.patch.genres.as_ref().unwrap(),
            &vec!["剧情".to_owned(), "动画".to_owned(), "奇幻".to_owned()]
        );
        assert_eq!(candidate.facts.title.as_deref(), Some("千与千寻"));
        assert_eq!(candidate.facts.release_year, Some(2001));
        assert_eq!(candidate.facts.language.as_deref(), Some("zh-CN"));
        assert_eq!(candidate.facts.community_score_milli, Some(940));
        assert_eq!(candidate.facts.community_vote_count, Some(2_345_678));
        assert!(
            candidate
                .facts
                .external_ids
                .iter()
                .any(|id| id.provider == "douban" && id.value == "1291561")
        );
        assert_eq!(candidate.artwork_candidates.len(), 1);
        assert_eq!(
            candidate.artwork_candidates[0].facts.kind,
            AddonArtworkKind::Poster
        );
        assert_eq!(
            candidate.artwork_candidates[0].facts.source_url,
            "https://img1.doubanio.com/view/photo/s_ratio_poster/public/p123.jpg"
        );

        let requests = transport.requests();
        assert_eq!(requests.len(), 2);
        assert_eq!(requests[0].url, "http://browser-worker.example/render");
        assert_eq!(requests[1].url, "http://browser-worker.example/render");
        let search_body: serde_json::Value =
            serde_json::from_slice(requests[0].json_body.as_ref().unwrap()).unwrap();
        assert_eq!(
            search_body["url"],
            "https://movie.douban.com/subject_search?search_text=%E5%8D%83%E4%B8%8E%E5%8D%83%E5%AF%BB"
        );
        let detail_body: serde_json::Value =
            serde_json::from_slice(requests[1].json_body.as_ref().unwrap()).unwrap();
        assert_eq!(
            detail_body["url"],
            "https://movie.douban.com/subject/1291561/"
        );
    }

    #[derive(Clone, Default)]
    struct FakeTransport {
        responses: Arc<Mutex<VecDeque<ProviderHttpResult<ProviderHttpResponse>>>>,
        requests: Arc<Mutex<Vec<ProviderHttpRequest>>>,
    }

    impl FakeTransport {
        fn push_rendered_html(&self, url: &str, title: &str, html: &str) {
            self.responses
                .lock()
                .unwrap()
                .push_back(Ok(ProviderHttpResponse {
                    status: 200,
                    body: serde_json::json!({
                        "status": "ok",
                        "url": url,
                        "title": title,
                        "html": html,
                        "text": html,
                        "excerpt": html.chars().take(240).collect::<String>()
                    })
                    .to_string()
                    .into_bytes(),
                }));
        }

        fn requests(&self) -> Vec<ProviderHttpRequest> {
            self.requests.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl ProviderHttpTransport for FakeTransport {
        async fn send(
            &self,
            request: ProviderHttpRequest,
            _config: ProviderHttpRuntimeConfig,
        ) -> ProviderHttpResult<ProviderHttpResponse> {
            self.requests.lock().unwrap().push(request);
            self.responses
                .lock()
                .unwrap()
                .pop_front()
                .unwrap_or_else(|| {
                    Err(
                        crate::providers::http_runtime::ProviderHttpError::Transport {
                            provider_id: DOUBAN_PROVIDER_ID,
                            operation: "fake",
                            message: "fake transport response queue was empty".to_owned(),
                            attempts: 0,
                        },
                    )
                })
        }
    }
}
