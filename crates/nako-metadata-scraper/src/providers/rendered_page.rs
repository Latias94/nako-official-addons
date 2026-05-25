use serde::{Deserialize, Serialize};

use super::http_runtime::{
    ProviderHttpResult, ProviderHttpRuntime, ProviderHttpRuntimeConfig, ProviderHttpTransport,
    ReqwestProviderHttpTransport,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RenderedPageSupportConfig {
    pub(crate) base_url: String,
    pub(crate) timeout_ms: u64,
}

impl RenderedPageSupportConfig {
    #[must_use]
    pub(crate) fn new(base_url: String, timeout_ms: u64) -> Self {
        Self {
            base_url,
            timeout_ms,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct RenderedPageRuntime<T = ReqwestProviderHttpTransport>
where
    T: ProviderHttpTransport,
{
    base_url: String,
    runtime: ProviderHttpRuntime<T>,
}

impl RenderedPageRuntime<ReqwestProviderHttpTransport> {
    pub(crate) fn new(config: RenderedPageSupportConfig) -> ProviderHttpResult<Self> {
        let runtime = ProviderHttpRuntime::new(ProviderHttpRuntimeConfig {
            timeout_ms: config.timeout_ms,
            ..ProviderHttpRuntimeConfig::default()
        })?;
        Ok(Self {
            base_url: config.base_url,
            runtime,
        })
    }
}

impl<T> RenderedPageRuntime<T>
where
    T: ProviderHttpTransport,
{
    #[must_use]
    pub(crate) fn with_runtime(
        config: RenderedPageSupportConfig,
        runtime: ProviderHttpRuntime<T>,
    ) -> Self {
        Self {
            base_url: config.base_url,
            runtime,
        }
    }

    pub(crate) async fn render_html(
        &self,
        provider_id: &'static str,
        operation: &'static str,
        path: &str,
        url: String,
    ) -> anyhow::Result<RenderedHtmlPage> {
        let response = self
            .runtime
            .post_json(
                provider_id,
                operation,
                self.endpoint(path),
                Vec::new(),
                Vec::new(),
                &RenderedPageRequest { url },
            )
            .await?;

        RenderedHtmlPage::from_value(response.body)
    }

    pub(crate) async fn extract_text(
        &self,
        provider_id: &'static str,
        operation: &'static str,
        path: &str,
        source_url: &str,
    ) -> anyhow::Result<RenderedTextPage> {
        let response = self
            .runtime
            .post_json(
                provider_id,
                operation,
                self.endpoint(path),
                Vec::new(),
                Vec::new(),
                &RenderedPageRequest {
                    url: source_url.to_owned(),
                },
            )
            .await?;

        RenderedTextPage::from_value(response.body, source_url)
    }

    fn endpoint(&self, path: impl AsRef<str>) -> String {
        let path = path.as_ref();
        format!(
            "{}/{}",
            self.base_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }
}

#[derive(Debug, Serialize)]
struct RenderedPageRequest {
    url: String,
}

#[derive(Debug)]
pub(crate) struct RenderedHtmlPage {
    pub(crate) html: String,
}

#[derive(Debug, Deserialize)]
struct RenderedHtmlResponse {
    #[serde(default)]
    status: Option<String>,
    html: String,
}

impl RenderedHtmlPage {
    fn from_value(value: serde_json::Value) -> anyhow::Result<Self> {
        let response: RenderedHtmlResponse = serde_json::from_value(value).map_err(|error| {
            anyhow::anyhow!("failed to parse browser worker render response: {error}")
        })?;
        ensure_ok_status(response.status, "rendered page")?;
        Ok(Self {
            html: response.html,
        })
    }
}

#[derive(Debug)]
pub(crate) struct RenderedTextPage {
    pub(crate) final_url: String,
    pub(crate) title: Option<String>,
    pub(crate) rendered_text: Option<String>,
    pub(crate) excerpt: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RenderedTextResponse {
    #[serde(default)]
    status: Option<String>,
    url: Option<String>,
    title: Option<String>,
    rendered_text: Option<String>,
    excerpt: Option<String>,
}

impl RenderedTextPage {
    fn from_value(value: serde_json::Value, source_url: &str) -> anyhow::Result<Self> {
        let response: RenderedTextResponse = serde_json::from_value(value).map_err(|error| {
            anyhow::anyhow!("failed to parse browser worker extract response: {error}")
        })?;
        ensure_ok_status(response.status, source_url)?;
        Ok(Self {
            final_url: response
                .url
                .filter(|value| !value.trim().is_empty())
                .unwrap_or_else(|| source_url.to_owned()),
            title: response.title,
            rendered_text: response.rendered_text,
            excerpt: response.excerpt,
        })
    }
}

fn ensure_ok_status(status: Option<String>, label: &str) -> anyhow::Result<()> {
    if status.as_deref() != Some("ok") {
        anyhow::bail!("browser worker returned non-ok status for {label}: {status:?}");
    }

    Ok(())
}
