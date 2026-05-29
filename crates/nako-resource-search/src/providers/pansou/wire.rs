use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{
    config::PansouProviderConfig,
    domain::{ResourceLinkType, ResourceSearchQuery},
};

#[derive(Debug, Eq, PartialEq, Serialize)]
pub(super) struct PansouSearchRequest {
    #[serde(rename = "kw")]
    pub(super) keyword: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) conc: Option<u16>,
    pub(super) refresh: bool,
    pub(super) res: &'static str,
    pub(super) src: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(super) plugins: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(super) cloud_types: Vec<String>,
    #[serde(skip_serializing_if = "serde_json::Value::is_null")]
    pub(super) ext: serde_json::Value,
}

pub(super) fn build_pansou_request(
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
pub(super) struct PansouApiResponse {
    code: i32,
    message: String,
    #[serde(default)]
    data: Option<PansouSearchResponse>,
}

impl PansouApiResponse {
    pub(super) fn into_success_data(self) -> anyhow::Result<Option<PansouSearchResponse>> {
        if self.code != 0 {
            anyhow::bail!("pansou compatible search failed: {}", self.message);
        }

        Ok(self.data)
    }
}

#[derive(Debug, Default, Deserialize)]
pub(super) struct PansouSearchResponse {
    #[serde(default)]
    pub(super) results: Vec<PansouSearchResult>,
    #[serde(default)]
    pub(super) merged_by_type: BTreeMap<String, Vec<PansouMergedLink>>,
}

#[derive(Debug, Deserialize)]
pub(super) struct PansouSearchResult {
    #[serde(default)]
    pub(super) message_id: String,
    #[serde(default)]
    pub(super) unique_id: String,
    #[serde(default)]
    pub(super) channel: String,
    #[serde(default)]
    pub(super) title: String,
    #[serde(default)]
    pub(super) content: String,
    #[serde(default)]
    pub(super) links: Vec<PansouLink>,
    #[serde(default)]
    pub(super) tags: Vec<String>,
    #[serde(default)]
    pub(super) images: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub(super) struct PansouLink {
    #[serde(default, rename = "type")]
    pub(super) link_type: String,
    #[serde(default)]
    pub(super) url: String,
    #[serde(default)]
    pub(super) password: String,
    #[serde(default)]
    pub(super) work_title: String,
}

#[derive(Debug, Deserialize)]
pub(super) struct PansouMergedLink {
    #[serde(default)]
    pub(super) url: String,
    #[serde(default)]
    pub(super) password: String,
    #[serde(default)]
    pub(super) note: String,
    #[serde(default)]
    pub(super) source: String,
    #[serde(default)]
    pub(super) images: Vec<String>,
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
        let mut query = ResourceSearchQuery::free_text("Demo Movie", 10);
        query.link_types = vec![
            ResourceLinkType::Quark,
            ResourceLinkType::Magnet,
            ResourceLinkType::Web,
        ];
        query.refresh = true;
        query.ext = serde_json::json!({ "season": 1 });

        let request = build_pansou_request(&config, &query);

        assert_eq!(request.keyword, "Demo Movie");
        assert_eq!(request.conc, Some(4));
        assert!(request.refresh);
        assert_eq!(request.res, "results");
        assert_eq!(request.src, "plugin");
        assert_eq!(request.plugins, vec!["jikepan"]);
        assert_eq!(request.cloud_types, vec!["quark", "magnet"]);
        assert_eq!(request.ext, serde_json::json!({ "season": 1 }));
    }
}
