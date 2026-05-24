use serde::{Deserialize, Serialize};

use crate::{
    config::DoubanProviderConfig,
    providers::http_runtime::{
        ProviderHttpResult, ProviderHttpRuntime, ProviderHttpRuntimeConfig, ProviderHttpTransport,
        ReqwestProviderHttpTransport,
    },
};

use super::{DOUBAN_PROVIDER_ID, DoubanMetadataProvider};

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

    pub(super) fn search_url(&self, title: &str) -> String {
        format!(
            "{}?search_text={}",
            self.config.search_base_url.trim_end_matches('?'),
            percent_encode_query(title)
        )
    }

    pub(super) async fn render(&self, url: String) -> anyhow::Result<RenderedPage> {
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

#[derive(Debug, Serialize)]
struct RenderPageRequest {
    url: String,
}

#[derive(Debug, Deserialize)]
pub(super) struct RenderedPage {
    #[serde(default)]
    status: Option<String>,
    #[serde(rename = "url")]
    _url: String,
    #[serde(rename = "title")]
    _title: Option<String>,
    pub(super) html: String,
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
