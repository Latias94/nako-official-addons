use std::{collections::BTreeMap, time::Duration};

use anyhow::Context;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{
    config::PansouProviderConfig,
    domain::{ResourceLink, ResourceLinkType, ResourceSearchQuery, ResourceSearchResult},
    engine::ResourceSearchProvider,
    links::{classify_resource_url, normalize_resource_url},
};

pub const PANSOU_COMPATIBLE_PROVIDER_ID: &str = "pansou_compatible";

#[derive(Clone)]
pub struct PansouCompatibleProvider {
    config: PansouProviderConfig,
    client: reqwest::Client,
}

impl PansouCompatibleProvider {
    #[must_use]
    pub fn new(config: PansouProviderConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_millis(config.timeout_ms))
            .build()
            .expect("reqwest client with timeout builds");
        Self { config, client }
    }
}

#[async_trait]
impl ResourceSearchProvider for PansouCompatibleProvider {
    fn id(&self) -> &'static str {
        PANSOU_COMPATIBLE_PROVIDER_ID
    }

    fn priority(&self) -> u16 {
        900
    }

    async fn search(
        &self,
        query: &ResourceSearchQuery,
    ) -> anyhow::Result<Vec<ResourceSearchResult>> {
        let base_url = self
            .config
            .base_url
            .as_deref()
            .context("pansou compatible base URL is not configured")?;
        let request = build_pansou_request(&self.config, query);
        let request_body = serde_json::to_vec(&request)
            .context("pansou compatible search request did not serialize")?;
        let mut builder = self
            .client
            .post(format!("{base_url}/api/search"))
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(request_body);

        if let Some(token) = self.config.bearer_token.as_deref() {
            builder = builder.bearer_auth(token);
        }

        let response = builder
            .send()
            .await
            .context("pansou compatible search request failed")?
            .error_for_status()
            .context("pansou compatible search returned an HTTP error")?
            .bytes()
            .await
            .context("pansou compatible search response body failed")?;
        let response = serde_json::from_slice::<PansouApiResponse>(&response)
            .context("pansou compatible search response was not valid JSON")?;

        if response.code != 0 {
            anyhow::bail!("pansou compatible search failed: {}", response.message);
        }

        Ok(response
            .data
            .map(|data| map_pansou_response(query, data))
            .unwrap_or_default())
    }
}

#[derive(Debug, Eq, PartialEq, Serialize)]
struct PansouSearchRequest {
    #[serde(rename = "kw")]
    keyword: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    conc: Option<u16>,
    refresh: bool,
    res: &'static str,
    src: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    plugins: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    cloud_types: Vec<String>,
    #[serde(skip_serializing_if = "serde_json::Value::is_null")]
    ext: serde_json::Value,
}

fn build_pansou_request(
    config: &PansouProviderConfig,
    query: &ResourceSearchQuery,
) -> PansouSearchRequest {
    let cloud_types = if query.link_types.is_empty() {
        config
            .cloud_types
            .iter()
            .filter_map(|link_type| pansou_cloud_type(*link_type))
            .map(str::to_owned)
            .collect()
    } else {
        query
            .link_types
            .iter()
            .filter_map(|link_type| pansou_cloud_type(*link_type))
            .map(str::to_owned)
            .collect()
    };

    PansouSearchRequest {
        keyword: query.query.clone(),
        conc: config.concurrency,
        refresh: query.refresh,
        res: "results",
        src: config.source_type.clone(),
        plugins: config.plugins.clone(),
        cloud_types,
        ext: query.ext.clone(),
    }
}

fn pansou_cloud_type(link_type: ResourceLinkType) -> Option<&'static str> {
    match link_type {
        ResourceLinkType::Aliyun
        | ResourceLinkType::Baidu
        | ResourceLinkType::Quark
        | ResourceLinkType::Tianyi
        | ResourceLinkType::Uc
        | ResourceLinkType::Mobile
        | ResourceLinkType::OneOneFive
        | ResourceLinkType::Pikpak
        | ResourceLinkType::Xunlei
        | ResourceLinkType::OneTwoThree
        | ResourceLinkType::Magnet
        | ResourceLinkType::Ed2k => Some(link_type.as_str()),
        ResourceLinkType::Web | ResourceLinkType::Other => None,
    }
}

#[derive(Debug, Deserialize)]
struct PansouApiResponse {
    code: i32,
    message: String,
    #[serde(default)]
    data: Option<PansouSearchResponse>,
}

#[derive(Debug, Default, Deserialize)]
struct PansouSearchResponse {
    #[serde(default)]
    results: Vec<PansouSearchResult>,
    #[serde(default)]
    merged_by_type: BTreeMap<String, Vec<PansouMergedLink>>,
}

#[derive(Debug, Deserialize)]
struct PansouSearchResult {
    #[serde(default)]
    message_id: String,
    #[serde(default)]
    unique_id: String,
    #[serde(default)]
    channel: String,
    #[serde(default)]
    title: String,
    #[serde(default)]
    content: String,
    #[serde(default)]
    links: Vec<PansouLink>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    images: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct PansouLink {
    #[serde(default, rename = "type")]
    link_type: String,
    #[serde(default)]
    url: String,
    #[serde(default)]
    password: String,
    #[serde(default)]
    work_title: String,
}

#[derive(Debug, Deserialize)]
struct PansouMergedLink {
    #[serde(default)]
    url: String,
    #[serde(default)]
    password: String,
    #[serde(default)]
    note: String,
    #[serde(default)]
    source: String,
    #[serde(default)]
    images: Vec<String>,
}

fn map_pansou_response(
    query: &ResourceSearchQuery,
    response: PansouSearchResponse,
) -> Vec<ResourceSearchResult> {
    if !response.results.is_empty() {
        return response
            .results
            .into_iter()
            .enumerate()
            .filter_map(|(index, result)| map_pansou_result(index, result))
            .collect();
    }

    map_pansou_merged_response(query, response.merged_by_type)
}

fn map_pansou_result(index: usize, result: PansouSearchResult) -> Option<ResourceSearchResult> {
    let source = source_from_channel(&result.channel);
    let links = result
        .links
        .into_iter()
        .filter_map(|link| map_pansou_link(link, &source))
        .collect::<Vec<_>>();
    if links.is_empty() {
        return None;
    }

    let id = first_non_empty(&[&result.unique_id, &result.message_id])
        .map(str::to_owned)
        .unwrap_or_else(|| format!("pansou:{index}"));
    let title =
        non_empty_trimmed(&result.title).unwrap_or_else(|| format!("PanSou result {index}"));

    Some(ResourceSearchResult {
        id,
        title,
        source,
        content: non_empty_trimmed(&result.content),
        links,
        tags: result.tags,
        images: result.images,
        score: 700,
    })
}

fn map_pansou_merged_response(
    query: &ResourceSearchQuery,
    merged_by_type: BTreeMap<String, Vec<PansouMergedLink>>,
) -> Vec<ResourceSearchResult> {
    let mut results = Vec::new();
    for (link_type, links) in merged_by_type {
        for (index, merged_link) in links.into_iter().enumerate() {
            let source = non_empty_trimmed(&merged_link.source)
                .unwrap_or_else(|| PANSOU_COMPATIBLE_PROVIDER_ID.to_owned());
            let Some(link) = map_pansou_merged_link(&link_type, &source, &merged_link) else {
                continue;
            };
            let title = format!("{} {} resource", query.query, link.link_type.as_str());
            results.push(ResourceSearchResult {
                id: format!("pansou:merged:{link_type}:{index}"),
                title,
                source,
                content: non_empty_trimmed(&merged_link.note),
                links: vec![link],
                tags: Vec::new(),
                images: merged_link.images,
                score: 650,
            });
        }
    }

    results
}

fn map_pansou_link(link: PansouLink, source: &str) -> Option<ResourceLink> {
    let mut mapped = resource_link_with_type(&link.url, &link.link_type, source)?;
    if let Some(password) = non_empty_trimmed(&link.password) {
        mapped = mapped.with_password(password);
    }
    if let Some(work_title) = non_empty_trimmed(&link.work_title) {
        mapped = mapped.with_note(work_title);
    }

    Some(mapped)
}

fn map_pansou_merged_link(
    link_type: &str,
    source: &str,
    link: &PansouMergedLink,
) -> Option<ResourceLink> {
    let mut mapped = resource_link_with_type(&link.url, link_type, source)?;
    if let Some(password) = non_empty_trimmed(&link.password) {
        mapped = mapped.with_password(password);
    }
    if let Some(note) = non_empty_trimmed(&link.note) {
        mapped = mapped.with_note(note);
    }

    Some(mapped)
}

fn resource_link_with_type(
    url: &str,
    pansou_link_type: &str,
    source: &str,
) -> Option<ResourceLink> {
    let normalized_url = normalize_resource_url(url)?;
    let link_type = pansou_link_type
        .parse::<ResourceLinkType>()
        .unwrap_or_else(|_| classify_resource_url(url));

    Some(ResourceLink {
        url: url.trim().to_owned(),
        normalized_url,
        link_type,
        source: source.to_owned(),
        password: None,
        note: None,
    })
}

fn source_from_channel(channel: &str) -> String {
    non_empty_trimmed(channel)
        .map(|channel| format!("pansou:{channel}"))
        .unwrap_or_else(|| PANSOU_COMPATIBLE_PROVIDER_ID.to_owned())
}

fn first_non_empty<'a>(values: &[&'a str]) -> Option<&'a str> {
    values
        .iter()
        .copied()
        .find(|value| !value.trim().is_empty())
}

fn non_empty_trimmed(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_shape_maps_nako_filters_to_pansou_cloud_types() {
        let config = PansouProviderConfig {
            enabled: true,
            base_url: Some("http://127.0.0.1:8888".to_owned()),
            bearer_token: Some("secret-token".to_owned()),
            source_type: "plugin".to_owned(),
            plugins: vec!["jikepan".to_owned()],
            cloud_types: vec![ResourceLinkType::Aliyun],
            concurrency: Some(4),
            timeout_ms: 500,
        };
        let request = build_pansou_request(
            &config,
            &ResourceSearchQuery {
                query: "Demo Movie".to_owned(),
                limit: 10,
                sources: Vec::new(),
                link_types: vec![
                    ResourceLinkType::Quark,
                    ResourceLinkType::Magnet,
                    ResourceLinkType::Web,
                ],
                refresh: true,
                ext: serde_json::json!({ "season": 1 }),
            },
        );

        assert_eq!(request.keyword, "Demo Movie");
        assert_eq!(request.conc, Some(4));
        assert!(request.refresh);
        assert_eq!(request.res, "results");
        assert_eq!(request.src, "plugin");
        assert_eq!(request.plugins, vec!["jikepan"]);
        assert_eq!(request.cloud_types, vec!["quark", "magnet"]);
        assert_eq!(request.ext, serde_json::json!({ "season": 1 }));
    }

    #[test]
    fn maps_pansou_results_into_resource_search_results() {
        let response = serde_json::from_value::<PansouSearchResponse>(serde_json::json!({
            "results": [{
                "message_id": "m1",
                "unique_id": "u1",
                "channel": "movies",
                "title": "Demo Movie",
                "content": "content",
                "links": [
                    { "type": "quark", "url": "https://pan.quark.cn/s/demo", "password": "1234", "work_title": "disc 1" },
                    { "type": "magnet", "url": "magnet:?xt=urn:btih:ABCDEF" }
                ],
                "tags": ["tag1"],
                "images": ["https://example.test/image.jpg"]
            }]
        }))
        .unwrap();

        let results = map_pansou_response(&query(), response);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "u1");
        assert_eq!(results[0].source, "pansou:movies");
        assert_eq!(results[0].links[0].link_type, ResourceLinkType::Quark);
        assert_eq!(results[0].links[0].password.as_deref(), Some("1234"));
        assert_eq!(results[0].links[0].note.as_deref(), Some("disc 1"));
        assert_eq!(results[0].links[1].link_type, ResourceLinkType::Magnet);
    }

    #[test]
    fn maps_merged_only_pansou_response_into_synthetic_results() {
        let response = serde_json::from_value::<PansouSearchResponse>(serde_json::json!({
            "merged_by_type": {
                "quark": [{
                    "url": "https://pan.quark.cn/s/demo",
                    "password": "abcd",
                    "note": "merged",
                    "source": "plugin:jikepan",
                    "images": ["https://example.test/image.jpg"]
                }]
            }
        }))
        .unwrap();

        let results = map_pansou_response(&query(), response);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "pansou:merged:quark:0");
        assert_eq!(results[0].source, "plugin:jikepan");
        assert_eq!(results[0].links[0].link_type, ResourceLinkType::Quark);
        assert_eq!(results[0].links[0].password.as_deref(), Some("abcd"));
        assert_eq!(results[0].images, vec!["https://example.test/image.jpg"]);
    }

    fn query() -> ResourceSearchQuery {
        ResourceSearchQuery {
            query: "Demo Movie".to_owned(),
            limit: 10,
            sources: Vec::new(),
            link_types: Vec::new(),
            refresh: false,
            ext: serde_json::Value::Null,
        }
    }
}
