use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::http_runtime::{
    ProviderHttpResult, ProviderHttpRuntime, ProviderHttpRuntimeConfig, ProviderHttpTransport,
    ReqwestProviderHttpTransport,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RenderedPageSupportConfig {
    pub(crate) base_url: String,
    pub(crate) timeout_ms: u64,
    intent_defaults: RenderedPageIntentDefaults,
}

impl RenderedPageSupportConfig {
    #[must_use]
    pub(crate) fn new(base_url: String, timeout_ms: u64) -> Self {
        Self::new_with_intent_defaults(base_url, timeout_ms, RenderedPageIntentDefaults::default())
    }

    #[must_use]
    pub(crate) fn new_with_intent_defaults(
        base_url: String,
        timeout_ms: u64,
        intent_defaults: RenderedPageIntentDefaults,
    ) -> Self {
        Self {
            base_url,
            timeout_ms,
            intent_defaults,
        }
    }

    #[must_use]
    pub(crate) fn with_env_defaults(self, lookup: impl FnMut(&str) -> Option<String>) -> Self {
        Self::new_with_intent_defaults(
            self.base_url,
            self.timeout_ms,
            RenderedPageIntentDefaults::from_env_lookup(lookup),
        )
    }

    #[must_use]
    pub(crate) fn intent(
        &self,
        path: impl Into<String>,
        url: impl Into<String>,
    ) -> RenderedPageIntent {
        self.intent_defaults
            .apply(RenderedPageIntent::new(path, url))
    }

    #[must_use]
    pub(crate) fn proxy_policy_configured(&self) -> bool {
        self.intent_defaults.proxy_policy.is_some()
    }

    #[must_use]
    pub(crate) fn session_key_configured(&self) -> bool {
        self.intent_defaults.session_key.is_some()
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct RenderedPageIntentDefaults {
    wait_for: Option<RenderedPageWaitFor>,
    proxy_policy: Option<RenderedPageProxyPolicy>,
    session_key: Option<String>,
}

impl RenderedPageIntentDefaults {
    #[must_use]
    pub(crate) fn from_env_lookup(mut lookup: impl FnMut(&str) -> Option<String>) -> Self {
        let wait_state = lookup("NAKO_METADATA_SCRAPER_BROWSER_WORKER_WAIT_STATE")
            .and_then(|value| RenderedPageLoadState::parse(&value));
        let wait_selector = lookup("NAKO_METADATA_SCRAPER_BROWSER_WORKER_WAIT_SELECTOR")
            .and_then(non_empty_trimmed);
        let wait_timeout_ms = lookup("NAKO_METADATA_SCRAPER_BROWSER_WORKER_WAIT_TIMEOUT_MS")
            .and_then(|value| value.trim().parse::<u64>().ok())
            .filter(|value| *value > 0);
        let wait_for =
            if wait_state.is_some() || wait_selector.is_some() || wait_timeout_ms.is_some() {
                let mut wait_for = RenderedPageWaitFor::new(
                    wait_state.unwrap_or(RenderedPageLoadState::NetworkIdle),
                );
                if let Some(selector) = wait_selector {
                    wait_for = wait_for.with_selector(selector);
                }
                if let Some(timeout_ms) = wait_timeout_ms {
                    wait_for = wait_for.with_timeout_ms(timeout_ms);
                }
                Some(wait_for)
            } else {
                None
            };

        Self {
            wait_for,
            proxy_policy: lookup("NAKO_METADATA_SCRAPER_BROWSER_WORKER_PROXY_POLICY")
                .and_then(|value| RenderedPageProxyPolicy::parse(&value)),
            session_key: lookup("NAKO_METADATA_SCRAPER_BROWSER_WORKER_SESSION_KEY")
                .and_then(non_empty_trimmed),
        }
    }

    #[must_use]
    fn apply(&self, mut intent: RenderedPageIntent) -> RenderedPageIntent {
        if let Some(wait_for) = self.wait_for.clone() {
            intent = intent.with_wait_for(wait_for);
        }
        if let Some(proxy_policy) = self.proxy_policy {
            intent = intent.with_proxy_policy(proxy_policy);
        }
        if let Some(session_key) = self.session_key.as_ref() {
            intent = intent.with_session_key(session_key);
        }
        intent
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RenderedPageIntent {
    path: String,
    url: String,
    wait_for: Option<RenderedPageWaitFor>,
    proxy_policy: Option<RenderedPageProxyPolicy>,
    session_key: Option<String>,
    headers: Vec<(String, String)>,
    actions: Vec<RenderedPageAction>,
}

impl RenderedPageIntent {
    #[must_use]
    pub(crate) fn new(path: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            url: url.into(),
            wait_for: None,
            proxy_policy: None,
            session_key: None,
            headers: Vec::new(),
            actions: Vec::new(),
        }
    }

    #[must_use]
    pub(crate) fn with_wait_for(mut self, wait_for: RenderedPageWaitFor) -> Self {
        self.wait_for = Some(wait_for);
        self
    }

    #[must_use]
    pub(crate) fn with_proxy_policy(mut self, proxy_policy: RenderedPageProxyPolicy) -> Self {
        self.proxy_policy = Some(proxy_policy);
        self
    }

    #[must_use]
    pub(crate) fn with_session_key(mut self, session_key: impl Into<String>) -> Self {
        let session_key = session_key.into();
        if !session_key.trim().is_empty() {
            self.session_key = Some(session_key);
        }
        self
    }

    #[must_use]
    pub(crate) fn with_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        let name = name.into();
        let value = value.into();
        if !name.trim().is_empty() && !value.trim().is_empty() {
            self.headers
                .push((name.trim().to_ascii_lowercase(), value.trim().to_owned()));
        }
        self
    }

    #[must_use]
    pub(crate) fn with_action(mut self, action: RenderedPageAction) -> Self {
        self.actions.push(action);
        self
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct RenderedPageWaitFor {
    state: RenderedPageLoadState,
    #[serde(skip_serializing_if = "Option::is_none")]
    selector: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    timeout_ms: Option<u64>,
}

impl RenderedPageWaitFor {
    #[must_use]
    pub(crate) fn new(state: RenderedPageLoadState) -> Self {
        Self {
            state,
            selector: None,
            timeout_ms: None,
        }
    }

    #[must_use]
    pub(crate) fn with_selector(mut self, selector: impl Into<String>) -> Self {
        let selector = selector.into();
        if !selector.trim().is_empty() {
            self.selector = Some(selector);
        }
        self
    }

    #[must_use]
    pub(crate) fn with_timeout_ms(mut self, timeout_ms: u64) -> Self {
        if timeout_ms > 0 {
            self.timeout_ms = Some(timeout_ms);
        }
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum RenderedPageLoadState {
    Load,
    DomContentLoaded,
    NetworkIdle,
}

impl RenderedPageLoadState {
    #[must_use]
    fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "load" => Some(Self::Load),
            "domcontentloaded" | "dom_content_loaded" | "dom-content-loaded" => {
                Some(Self::DomContentLoaded)
            }
            "networkidle" | "network_idle" | "network-idle" => Some(Self::NetworkIdle),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum RenderedPageProxyPolicy {
    Default,
    Direct,
    Required,
}

impl RenderedPageProxyPolicy {
    #[must_use]
    fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "default" => Some(Self::Default),
            "direct" => Some(Self::Direct),
            "required" => Some(Self::Required),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct RenderedPageAction {
    #[serde(rename = "type")]
    action_type: RenderedPageActionType,
    selector: String,
    #[serde(skip_serializing_if = "is_false")]
    optional: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    wait_for: Option<RenderedPageWaitFor>,
}

impl RenderedPageAction {
    #[must_use]
    pub(crate) fn check(selector: impl Into<String>) -> Self {
        Self::new(RenderedPageActionType::Check, selector)
    }

    #[must_use]
    pub(crate) fn click(selector: impl Into<String>) -> Self {
        Self::new(RenderedPageActionType::Click, selector)
    }

    #[must_use]
    fn new(action_type: RenderedPageActionType, selector: impl Into<String>) -> Self {
        Self {
            action_type,
            selector: selector.into(),
            optional: false,
            wait_for: None,
        }
    }

    #[must_use]
    pub(crate) fn optional(mut self) -> Self {
        self.optional = true;
        self
    }

    #[must_use]
    pub(crate) fn with_wait_for(mut self, wait_for: RenderedPageWaitFor) -> Self {
        self.wait_for = Some(wait_for);
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
enum RenderedPageActionType {
    Check,
    Click,
}

fn non_empty_trimmed(value: String) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_owned())
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
        intent: RenderedPageIntent,
    ) -> anyhow::Result<RenderedHtmlPage> {
        let endpoint = self.endpoint(&intent.path);
        let request = RenderedPageRequest::from(intent);
        let response = self
            .runtime
            .post_json(
                provider_id,
                operation,
                endpoint,
                Vec::new(),
                Vec::new(),
                &request,
            )
            .await?;

        RenderedHtmlPage::from_value(response.body)
    }

    pub(crate) async fn extract_text(
        &self,
        provider_id: &'static str,
        operation: &'static str,
        intent: RenderedPageIntent,
    ) -> anyhow::Result<RenderedTextPage> {
        let endpoint = self.endpoint(&intent.path);
        let source_url = intent.url.clone();
        let request = RenderedPageRequest::from(intent);
        let response = self
            .runtime
            .post_json(
                provider_id,
                operation,
                endpoint,
                Vec::new(),
                Vec::new(),
                &request,
            )
            .await?;

        RenderedTextPage::from_value(response.body, &source_url)
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
    #[serde(skip_serializing_if = "Option::is_none")]
    wait_for: Option<RenderedPageWaitFor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    proxy_policy: Option<RenderedPageProxyPolicy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    session_key: Option<String>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    headers: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    actions: Vec<RenderedPageAction>,
}

impl From<RenderedPageIntent> for RenderedPageRequest {
    fn from(intent: RenderedPageIntent) -> Self {
        Self {
            url: intent.url,
            wait_for: intent.wait_for,
            proxy_policy: intent.proxy_policy,
            session_key: intent.session_key,
            headers: intent.headers.into_iter().collect(),
            actions: intent.actions,
        }
    }
}

#[derive(Debug)]
pub(crate) struct RenderedHtmlPage {
    pub(crate) html: String,
}

#[derive(Debug, Deserialize)]
struct RenderedHtmlResponse {
    #[serde(default)]
    status: Option<String>,
    html: Option<String>,
    #[serde(default)]
    safe_error_code: Option<String>,
    #[serde(default)]
    failure_kind: Option<String>,
}

impl RenderedHtmlPage {
    fn from_value(value: serde_json::Value) -> anyhow::Result<Self> {
        let response: RenderedHtmlResponse = serde_json::from_value(value).map_err(|error| {
            anyhow::anyhow!("failed to parse browser worker render response: {error}")
        })?;
        ensure_ok_status(
            response.status,
            "rendered page",
            response.safe_error_code,
            response.failure_kind,
        )?;
        Ok(Self {
            html: response
                .html
                .ok_or_else(|| anyhow::anyhow!("browser worker render response missing html"))?,
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
    #[serde(default)]
    safe_error_code: Option<String>,
    #[serde(default)]
    failure_kind: Option<String>,
}

impl RenderedTextPage {
    fn from_value(value: serde_json::Value, source_url: &str) -> anyhow::Result<Self> {
        let response: RenderedTextResponse = serde_json::from_value(value).map_err(|error| {
            anyhow::anyhow!("failed to parse browser worker extract response: {error}")
        })?;
        ensure_ok_status(
            response.status,
            source_url,
            response.safe_error_code,
            response.failure_kind,
        )?;
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

fn ensure_ok_status(
    status: Option<String>,
    label: &str,
    safe_error_code: Option<String>,
    failure_kind: Option<String>,
) -> anyhow::Result<()> {
    if status.as_deref() != Some("ok") {
        anyhow::bail!(
            "browser worker returned non-ok status for {label}: status={status:?}, safe_error_code={safe_error_code:?}, failure_kind={failure_kind:?}"
        );
    }

    Ok(())
}

fn is_false(value: &bool) -> bool {
    !*value
}

#[cfg(test)]
mod tests {
    use std::{
        collections::VecDeque,
        sync::{Arc, Mutex},
    };

    use async_trait::async_trait;

    use crate::providers::http_runtime::{
        ProviderHttpRequest, ProviderHttpResponse, ProviderHttpRuntime, ProviderHttpRuntimeConfig,
    };

    use super::*;

    #[tokio::test]
    async fn rendered_page_runtime_serializes_render_intent_options() {
        let transport = FakeTransport::default();
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: serde_json::json!({
                "status": "ok",
                "html": "<html><body>rendered</body></html>"
            })
            .to_string()
            .into_bytes(),
        }));
        let runtime = ProviderHttpRuntime::with_transport(
            ProviderHttpRuntimeConfig {
                retry_backoff_ms: 0,
                ..ProviderHttpRuntimeConfig::default()
            },
            transport.clone(),
        );
        let rendered_pages = RenderedPageRuntime::with_runtime(
            RenderedPageSupportConfig::new("http://browser-worker.example".to_owned(), 10_000),
            runtime,
        );

        let intent = RenderedPageIntent::new("/render", "https://javbus.example/SSNI-644")
            .with_wait_for(
                RenderedPageWaitFor::new(RenderedPageLoadState::DomContentLoaded)
                    .with_selector("#movie")
                    .with_timeout_ms(1500),
            )
            .with_proxy_policy(RenderedPageProxyPolicy::Required)
            .with_session_key("javbus:ssni-644")
            .with_header("cookie", "age=verified")
            .with_action(
                RenderedPageAction::check("#ageVerify input[type=\"checkbox\"]").optional(),
            )
            .with_action(
                RenderedPageAction::click("#ageVerify #submit")
                    .optional()
                    .with_wait_for(
                        RenderedPageWaitFor::new(RenderedPageLoadState::DomContentLoaded)
                            .with_timeout_ms(1500),
                    ),
            );

        let page = rendered_pages
            .render_html("javbus", "render page", intent)
            .await
            .unwrap();
        assert!(page.html.contains("rendered"));

        let requests = transport.requests();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].url, "http://browser-worker.example/render");
        let body: serde_json::Value =
            serde_json::from_slice(requests[0].json_body.as_ref().unwrap()).unwrap();
        assert_eq!(
            body,
            serde_json::json!({
                "url": "https://javbus.example/SSNI-644",
                "wait_for": {
                    "state": "domcontentloaded",
                    "selector": "#movie",
                    "timeout_ms": 1500
                },
                "proxy_policy": "required",
                "session_key": "javbus:ssni-644",
                "headers": {
                    "cookie": "age=verified"
                },
                "actions": [
                    {
                        "type": "check",
                        "selector": "#ageVerify input[type=\"checkbox\"]",
                        "optional": true
                    },
                    {
                        "type": "click",
                        "selector": "#ageVerify #submit",
                        "optional": true,
                        "wait_for": {
                            "state": "domcontentloaded",
                            "timeout_ms": 1500
                        }
                    }
                ]
            })
        );
    }

    #[tokio::test]
    async fn rendered_page_runtime_preserves_browser_worker_failure_kind() {
        let transport = FakeTransport::default();
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: serde_json::json!({
                "status": "error",
                "safe_error_code": "proxy_required",
                "failure_kind": "operator_action"
            })
            .to_string()
            .into_bytes(),
        }));
        let runtime = ProviderHttpRuntime::with_transport(
            ProviderHttpRuntimeConfig {
                retry_backoff_ms: 0,
                ..ProviderHttpRuntimeConfig::default()
            },
            transport,
        );
        let rendered_pages = RenderedPageRuntime::with_runtime(
            RenderedPageSupportConfig::new("http://browser-worker.example".to_owned(), 10_000),
            runtime,
        );

        let error = rendered_pages
            .render_html(
                "javbus",
                "render page",
                RenderedPageIntent::new("/render", "https://javbus.example/SSNI-644"),
            )
            .await
            .unwrap_err();

        let message = error.to_string();
        assert!(message.contains("proxy_required"));
        assert!(message.contains("operator_action"));
    }

    #[test]
    fn rendered_page_support_config_applies_env_render_intent_defaults() {
        let config =
            RenderedPageSupportConfig::new("http://browser-worker.example".to_owned(), 10_000)
                .with_env_defaults(|name| match name {
                    "NAKO_METADATA_SCRAPER_BROWSER_WORKER_WAIT_STATE" => Some("load".to_owned()),
                    "NAKO_METADATA_SCRAPER_BROWSER_WORKER_WAIT_SELECTOR" => {
                        Some("#ready".to_owned())
                    }
                    "NAKO_METADATA_SCRAPER_BROWSER_WORKER_WAIT_TIMEOUT_MS" => {
                        Some("2000".to_owned())
                    }
                    "NAKO_METADATA_SCRAPER_BROWSER_WORKER_PROXY_POLICY" => {
                        Some("direct".to_owned())
                    }
                    "NAKO_METADATA_SCRAPER_BROWSER_WORKER_SESSION_KEY" => {
                        Some("operator-session".to_owned())
                    }
                    _ => None,
                });

        let request = RenderedPageRequest::from(config.intent("/render", "https://example.test"));
        let body = serde_json::to_value(request).unwrap();

        assert_eq!(
            body,
            serde_json::json!({
                "url": "https://example.test",
                "wait_for": {
                    "state": "load",
                    "selector": "#ready",
                    "timeout_ms": 2000
                },
                "proxy_policy": "direct",
                "session_key": "operator-session"
            })
        );
    }

    #[derive(Clone, Default)]
    struct FakeTransport {
        responses: Arc<Mutex<VecDeque<ProviderHttpResult<ProviderHttpResponse>>>>,
        requests: Arc<Mutex<Vec<ProviderHttpRequest>>>,
    }

    impl FakeTransport {
        fn push(&self, response: ProviderHttpResult<ProviderHttpResponse>) {
            self.responses.lock().unwrap().push_back(response);
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
                .unwrap_or_else(|| panic!("missing fake response"))
        }
    }
}
