use std::time::Duration;

use anyhow::Context;
use async_trait::async_trait;

use super::descriptor::{
    ProviderCapability, ProviderConfigurationSchemaFragment, ProviderDescriptor,
};
use super::{ProviderSearchBatch, ResourceSearchProvider};
use crate::{
    Config, config::PansouProviderConfig, domain::ResourceSearchQuery, source_policy::SourcePolicy,
};

mod mapper;
mod wire;

use mapper::map_pansou_response;
use wire::{PansouApiResponse, build_pansou_request};

pub const PANSOU_COMPATIBLE_PROVIDER_ID: &str = "pansou_compatible";

const PANSOU_CAPABILITIES: &[ProviderCapability] = &[
    ProviderCapability::ResourceSearch,
    ProviderCapability::ExternalHttpSearch,
    ProviderCapability::CloudDriveLinks,
    ProviderCapability::MagnetLinks,
    ProviderCapability::Refresh,
    ProviderCapability::MergedLinkResponse,
];

pub const PANSOU_DESCRIPTOR: ProviderDescriptor = ProviderDescriptor {
    id: PANSOU_COMPATIBLE_PROVIDER_ID,
    display_name: "PanSou Compatible",
    source_policy: SourcePolicy::ExternalService,
    default_enabled: false,
    capabilities: PANSOU_CAPABILITIES,
    configuration_schema: pansou_configuration_schema,
};

fn pansou_configuration_schema(config: &Config) -> ProviderConfigurationSchemaFragment {
    ProviderConfigurationSchemaFragment {
        provider_id: PANSOU_COMPATIBLE_PROVIDER_ID,
        provider_enabled_default: config.pansou.enabled,
        settings_key: Some("pansou"),
        settings_schema: Some(serde_json::json!({
            "type": "object",
            "properties": {
                "base_url": {
                    "type": "string",
                    "default": config.pansou.base_url.clone().unwrap_or_default()
                },
                "source_type": {
                    "type": "string",
                    "default": config.pansou.source_type
                },
                "plugins": {
                    "type": "array",
                    "items": { "type": "string" },
                    "default": config.pansou.plugins
                },
                "cloud_types": {
                    "type": "array",
                    "items": { "type": "string" },
                    "default": config.pansou.cloud_types.iter().map(|link_type| link_type.as_str()).collect::<Vec<_>>()
                },
                "concurrency": {
                    "type": ["integer", "null"],
                    "default": config.pansou.concurrency,
                    "minimum": 1
                },
                "timeout_ms": {
                    "type": "integer",
                    "default": config.pansou.timeout_ms,
                    "minimum": 250,
                    "maximum": 60000
                }
            },
            "additionalProperties": false
        })),
    }
}

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

    async fn search(&self, query: &ResourceSearchQuery) -> anyhow::Result<ProviderSearchBatch> {
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

        let results = response
            .into_success_data()?
            .map(|data| map_pansou_response(query, data))
            .unwrap_or_default();

        Ok(ProviderSearchBatch::complete(self.id(), results))
    }
}
