use serde::Serialize;

use crate::{
    Config,
    config::ProviderId,
    providers::{
        douban, javbus, javlibrary,
        rendered_page::{RenderedPageProxyPolicy, RenderedPageSupportConfig},
    },
};

pub const RENDER_DRIFT_SAMPLE_AV_NUMBER_ENV_VAR: &str =
    "NAKO_METADATA_SCRAPER_RENDER_DRIFT_SAMPLE_AV_NUMBER";
pub const RENDER_DRIFT_SAMPLE_DOUBAN_TITLE_ENV_VAR: &str =
    "NAKO_METADATA_SCRAPER_RENDER_DRIFT_SAMPLE_DOUBAN_TITLE";
pub const RENDER_DRIFT_SAMPLE_JAVBUS_AV_NUMBER_ENV_VAR: &str =
    "NAKO_METADATA_SCRAPER_RENDER_DRIFT_SAMPLE_JAVBUS_AV_NUMBER";
pub const RENDER_DRIFT_SAMPLE_JAVLIBRARY_AV_NUMBER_ENV_VAR: &str =
    "NAKO_METADATA_SCRAPER_RENDER_DRIFT_SAMPLE_JAVLIBRARY_AV_NUMBER";

const DEFAULT_SAMPLE_AV_NUMBER: &str = "SSNI-644";
const DEFAULT_SAMPLE_DOUBAN_TITLE: &str = "千与千寻";

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct BrowserWorkerRenderDriftCase {
    pub id: String,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy_policy: Option<BrowserWorkerRenderDriftProxyPolicy>,
    pub render_timeout_ms: u64,
    pub min_text_bytes: usize,
    pub min_html_bytes: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub actions: Vec<BrowserWorkerRenderDriftAction>,
}

impl BrowserWorkerRenderDriftCase {
    #[must_use]
    pub(crate) fn new(id: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            url: url.into(),
            selector: None,
            proxy_policy: None,
            render_timeout_ms: 10_000,
            min_text_bytes: 1,
            min_html_bytes: 1,
            actions: Vec::new(),
        }
    }

    #[must_use]
    pub(crate) fn with_selector(mut self, selector: impl Into<String>) -> Self {
        self.selector = Some(selector.into());
        self
    }

    #[must_use]
    pub(crate) fn with_render_timeout_ms(mut self, timeout_ms: u64) -> Self {
        if timeout_ms > 0 {
            self.render_timeout_ms = timeout_ms;
        }
        self
    }

    #[must_use]
    pub(crate) fn with_rendered_page_defaults(
        mut self,
        config: &RenderedPageSupportConfig,
    ) -> Self {
        if let Some(proxy_policy) = config.proxy_policy() {
            self.proxy_policy = Some(proxy_policy.into());
        }
        self
    }

    #[must_use]
    pub(crate) fn with_min_text_bytes(mut self, min_text_bytes: usize) -> Self {
        self.min_text_bytes = min_text_bytes;
        self
    }

    #[must_use]
    pub(crate) fn with_min_html_bytes(mut self, min_html_bytes: usize) -> Self {
        self.min_html_bytes = min_html_bytes;
        self
    }

    #[must_use]
    pub(crate) fn with_action(mut self, action: BrowserWorkerRenderDriftAction) -> Self {
        self.actions.push(action);
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum BrowserWorkerRenderDriftProxyPolicy {
    Default,
    Direct,
    Required,
}

impl From<RenderedPageProxyPolicy> for BrowserWorkerRenderDriftProxyPolicy {
    fn from(value: RenderedPageProxyPolicy) -> Self {
        match value {
            RenderedPageProxyPolicy::Default => Self::Default,
            RenderedPageProxyPolicy::Direct => Self::Direct,
            RenderedPageProxyPolicy::Required => Self::Required,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct BrowserWorkerRenderDriftAction {
    #[serde(rename = "type")]
    action_type: BrowserWorkerRenderDriftActionType,
    selector: String,
    #[serde(skip_serializing_if = "is_false")]
    optional: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    wait_for: Option<BrowserWorkerRenderDriftWaitFor>,
}

impl BrowserWorkerRenderDriftAction {
    #[must_use]
    pub(crate) fn check(selector: impl Into<String>) -> Self {
        Self::new(BrowserWorkerRenderDriftActionType::Check, selector)
    }

    #[must_use]
    pub(crate) fn click(selector: impl Into<String>) -> Self {
        Self::new(BrowserWorkerRenderDriftActionType::Click, selector)
    }

    #[must_use]
    fn new(action_type: BrowserWorkerRenderDriftActionType, selector: impl Into<String>) -> Self {
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
    pub(crate) fn with_wait_for(mut self, wait_for: BrowserWorkerRenderDriftWaitFor) -> Self {
        self.wait_for = Some(wait_for);
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
enum BrowserWorkerRenderDriftActionType {
    Check,
    Click,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct BrowserWorkerRenderDriftWaitFor {
    state: BrowserWorkerRenderDriftLoadState,
    #[serde(skip_serializing_if = "Option::is_none")]
    timeout_ms: Option<u64>,
}

impl BrowserWorkerRenderDriftWaitFor {
    #[must_use]
    pub(crate) fn domcontentloaded() -> Self {
        Self {
            state: BrowserWorkerRenderDriftLoadState::DomContentLoaded,
            timeout_ms: None,
        }
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
enum BrowserWorkerRenderDriftLoadState {
    DomContentLoaded,
}

#[must_use]
pub fn browser_worker_render_drift_cases_from_env(
    config: &Config,
) -> Vec<BrowserWorkerRenderDriftCase> {
    browser_worker_render_drift_cases_from_lookup(config, |name| std::env::var(name).ok())
}

#[must_use]
pub fn browser_worker_render_drift_cases_from_lookup(
    config: &Config,
    mut lookup: impl FnMut(&str) -> Option<String>,
) -> Vec<BrowserWorkerRenderDriftCase> {
    let sample_av_number = first_non_empty(
        lookup(RENDER_DRIFT_SAMPLE_AV_NUMBER_ENV_VAR),
        DEFAULT_SAMPLE_AV_NUMBER,
    );
    let sample_douban_title = first_non_empty(
        lookup(RENDER_DRIFT_SAMPLE_DOUBAN_TITLE_ENV_VAR),
        DEFAULT_SAMPLE_DOUBAN_TITLE,
    );
    let sample_javbus_av_number = first_non_empty(
        lookup(RENDER_DRIFT_SAMPLE_JAVBUS_AV_NUMBER_ENV_VAR),
        &sample_av_number,
    );
    let sample_javlibrary_av_number = first_non_empty(
        lookup(RENDER_DRIFT_SAMPLE_JAVLIBRARY_AV_NUMBER_ENV_VAR),
        &sample_av_number,
    );

    let mut cases = Vec::new();
    if config.provider_enabled(ProviderId::Douban)
        && let Some(provider) = config
            .provider_config(ProviderId::Douban)
            .and_then(|provider| provider.douban_config())
    {
        cases.push(douban::render_drift_case(provider, &sample_douban_title));
    }
    if config.provider_enabled(ProviderId::Javbus)
        && let Some(provider) = config
            .provider_config(ProviderId::Javbus)
            .and_then(|provider| provider.javbus_config())
    {
        cases.push(javbus::render_drift_case(
            provider,
            &sample_javbus_av_number,
        ));
    }
    if config.provider_enabled(ProviderId::Javlibrary)
        && let Some(provider) = config
            .provider_config(ProviderId::Javlibrary)
            .and_then(|provider| provider.javlibrary_config())
    {
        cases.push(javlibrary::render_drift_case(
            provider,
            &sample_javlibrary_av_number,
        ));
    }

    cases
}

fn first_non_empty(value: Option<String>, fallback: &str) -> String {
    value
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| fallback.to_owned())
}

fn is_false(value: &bool) -> bool {
    !*value
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::config::{AV_PROVIDER_PRESET_ENV_VAR, Config};

    #[test]
    fn render_drift_cases_include_enabled_provider_owned_presets() {
        let config = Config::from_env_lookup(|name| match name {
            AV_PROVIDER_PRESET_ENV_VAR => Some("manual".to_owned()),
            "NAKO_METADATA_SCRAPER_PROVIDER_DOUBAN_ENABLED" => Some("true".to_owned()),
            "NAKO_METADATA_SCRAPER_PROVIDER_JAVBUS_ENABLED" => Some("true".to_owned()),
            "NAKO_METADATA_SCRAPER_PROVIDER_JAVLIBRARY_ENABLED" => Some("true".to_owned()),
            "NAKO_METADATA_SCRAPER_DOUBAN_SEARCH_BASE_URL" => {
                Some("https://douban.example/subject_search".to_owned())
            }
            "NAKO_METADATA_SCRAPER_JAVBUS_BASE_URL" => Some("https://javbus.example".to_owned()),
            "NAKO_METADATA_SCRAPER_JAVLIBRARY_BASE_URL" => {
                Some("https://javlibrary.example".to_owned())
            }
            "NAKO_METADATA_SCRAPER_JAVLIBRARY_LANGUAGE" => Some("ja".to_owned()),
            "NAKO_METADATA_SCRAPER_JAVBUS_COOKIE" => Some("age=verified".to_owned()),
            "NAKO_METADATA_SCRAPER_BROWSER_WORKER_PROXY_POLICY" => Some("required".to_owned()),
            "NAKO_METADATA_SCRAPER_BROWSER_WORKER_SESSION_KEY" => {
                Some("session-key-should-not-emit".to_owned())
            }
            _ => None,
        });

        let cases = browser_worker_render_drift_cases_from_lookup(&config, |name| match name {
            RENDER_DRIFT_SAMPLE_DOUBAN_TITLE_ENV_VAR => Some("新世纪福音战士".to_owned()),
            RENDER_DRIFT_SAMPLE_AV_NUMBER_ENV_VAR => Some("ABP-123".to_owned()),
            RENDER_DRIFT_SAMPLE_JAVLIBRARY_AV_NUMBER_ENV_VAR => Some("ABP-456".to_owned()),
            _ => None,
        });

        assert_eq!(cases.len(), 3);
        assert_eq!(
            serde_json::to_value(&cases).unwrap(),
            json!([
                {
                    "id": "douban-search",
                    "url": "https://douban.example/subject_search?search_text=%E6%96%B0%E4%B8%96%E7%BA%AA%E7%A6%8F%E9%9F%B3%E6%88%98%E5%A3%AB",
                    "selector": "a[href*=\"/subject/\"]",
                    "proxy_policy": "required",
                    "render_timeout_ms": 10000,
                    "min_text_bytes": 100,
                    "min_html_bytes": 500
                },
                {
                    "id": "javbus-detail",
                    "url": "https://javbus.example/ABP-123",
                    "selector": "h3, .info, #movie, .movie",
                    "proxy_policy": "required",
                    "render_timeout_ms": 10000,
                    "min_text_bytes": 100,
                    "min_html_bytes": 500,
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
                                "timeout_ms": 10000
                            }
                        }
                    ]
                },
                {
                    "id": "javlibrary-search",
                    "url": "https://javlibrary.example/ja/vl_searchbyid.php?keyword=ABP-456",
                    "selector": "a[href*=\"?v=\"], .video a[href], .videothumblist a[href]",
                    "proxy_policy": "required",
                    "render_timeout_ms": 10000,
                    "min_text_bytes": 100,
                    "min_html_bytes": 500
                }
            ])
        );
        let rendered = serde_json::to_string(&cases).unwrap();
        assert!(!rendered.contains("age=verified"));
        assert!(!rendered.contains("session-key-should-not-emit"));
    }

    #[test]
    fn render_drift_cases_skip_disabled_providers() {
        let config = Config::default();

        let cases = browser_worker_render_drift_cases_from_lookup(&config, |_| None);

        assert!(cases.is_empty());
    }
}
