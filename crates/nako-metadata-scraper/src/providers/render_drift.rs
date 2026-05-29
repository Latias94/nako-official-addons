use std::collections::BTreeMap;

use serde::Serialize;

use crate::{
    Config,
    config::ProviderConfig,
    providers::{
        ProviderRegistry,
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
pub const RENDER_DRIFT_SAMPLE_DMM_AV_NUMBER_ENV_VAR: &str =
    "NAKO_METADATA_SCRAPER_RENDER_DRIFT_SAMPLE_DMM_AV_NUMBER";
pub const RENDER_DRIFT_SAMPLE_MGSTAGE_AV_NUMBER_ENV_VAR: &str =
    "NAKO_METADATA_SCRAPER_RENDER_DRIFT_SAMPLE_MGSTAGE_AV_NUMBER";
pub const RENDER_DRIFT_SAMPLE_XCITY_AV_NUMBER_ENV_VAR: &str =
    "NAKO_METADATA_SCRAPER_RENDER_DRIFT_SAMPLE_XCITY_AV_NUMBER";
pub const RENDER_DRIFT_SAMPLE_AIRAV_AV_NUMBER_ENV_VAR: &str =
    "NAKO_METADATA_SCRAPER_RENDER_DRIFT_SAMPLE_AIRAV_AV_NUMBER";
pub const RENDER_DRIFT_SAMPLE_AVSOX_AV_NUMBER_ENV_VAR: &str =
    "NAKO_METADATA_SCRAPER_RENDER_DRIFT_SAMPLE_AVSOX_AV_NUMBER";
pub const RENDER_DRIFT_SAMPLE_JAVDB_AV_NUMBER_ENV_VAR: &str =
    "NAKO_METADATA_SCRAPER_RENDER_DRIFT_SAMPLE_JAVDB_AV_NUMBER";
pub const RENDER_DRIFT_SAMPLE_FC2_AV_NUMBER_ENV_VAR: &str =
    "NAKO_METADATA_SCRAPER_RENDER_DRIFT_SAMPLE_FC2_AV_NUMBER";
pub const RENDER_DRIFT_SAMPLE_FC2PPVDB_AV_NUMBER_ENV_VAR: &str =
    "NAKO_METADATA_SCRAPER_RENDER_DRIFT_SAMPLE_FC2PPVDB_AV_NUMBER";
pub const RENDER_DRIFT_SAMPLE_CARIBBEAN_AV_NUMBER_ENV_VAR: &str =
    "NAKO_METADATA_SCRAPER_RENDER_DRIFT_SAMPLE_CARIBBEAN_AV_NUMBER";
pub const RENDER_DRIFT_SAMPLE_1PONDO_AV_NUMBER_ENV_VAR: &str =
    "NAKO_METADATA_SCRAPER_RENDER_DRIFT_SAMPLE_1PONDO_AV_NUMBER";
pub const RENDER_DRIFT_SAMPLE_10MUSUME_AV_NUMBER_ENV_VAR: &str =
    "NAKO_METADATA_SCRAPER_RENDER_DRIFT_SAMPLE_10MUSUME_AV_NUMBER";

pub(crate) const DEFAULT_SAMPLE_AV_NUMBER: &str = "SSNI-644";
pub(crate) const DEFAULT_SAMPLE_FC2_AV_NUMBER: &str = "FC2-1723984";
pub(crate) const DEFAULT_SAMPLE_CARIBBEAN_AV_NUMBER: &str = "052226-001";
pub(crate) const DEFAULT_SAMPLE_1PONDO_AV_NUMBER: &str = "080616_355";
pub(crate) const DEFAULT_SAMPLE_10MUSUME_AV_NUMBER: &str = "010116_001";
pub(crate) const DEFAULT_SAMPLE_DOUBAN_TITLE: &str = "千与千寻";
pub(crate) const DEFAULT_SAMPLE_MGSTAGE_AV_NUMBER: &str = "300MIUM-382";
pub(crate) const SLOW_LIVE_RENDER_DRIFT_TIMEOUT_MS: u64 = 30_000;
pub(crate) const SLOW_LIVE_RENDER_DRIFT_SELECTOR_TIMEOUT_MS: u64 = 15_000;

#[derive(Clone, Copy)]
pub(crate) struct ProviderRenderDriftCaseDescriptor {
    order: u16,
    sample_env_var: &'static str,
    fallback_env_var: Option<&'static str>,
    generic_env_var: Option<&'static str>,
    fallback: &'static str,
    build: fn(&ProviderConfig, &str) -> Option<BrowserWorkerRenderDriftCase>,
}

impl ProviderRenderDriftCaseDescriptor {
    #[must_use]
    pub(crate) const fn new(
        order: u16,
        sample_env_var: &'static str,
        fallback: &'static str,
        build: fn(&ProviderConfig, &str) -> Option<BrowserWorkerRenderDriftCase>,
    ) -> Self {
        Self {
            order,
            sample_env_var,
            fallback_env_var: None,
            generic_env_var: None,
            fallback,
            build,
        }
    }

    #[must_use]
    pub(crate) const fn with_fallback_env_var(mut self, env_var: &'static str) -> Self {
        self.fallback_env_var = Some(env_var);
        self
    }

    #[must_use]
    pub(crate) const fn with_generic_av_sample(mut self) -> Self {
        self.generic_env_var = Some(RENDER_DRIFT_SAMPLE_AV_NUMBER_ENV_VAR);
        self
    }

    #[must_use]
    fn order(self) -> u16 {
        self.order
    }

    #[must_use]
    fn sample(self, lookup: &mut impl FnMut(&str) -> Option<String>) -> String {
        non_empty(lookup(self.sample_env_var))
            .or_else(|| {
                self.fallback_env_var
                    .and_then(|env_var| non_empty(lookup(env_var)))
            })
            .or_else(|| {
                self.generic_env_var
                    .and_then(|env_var| non_empty(lookup(env_var)))
            })
            .unwrap_or_else(|| self.fallback.to_owned())
    }

    fn build(
        self,
        provider_config: &ProviderConfig,
        sample: &str,
    ) -> Option<BrowserWorkerRenderDriftCase> {
        (self.build)(provider_config, sample)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct BrowserWorkerRenderDriftCase {
    pub id: String,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector_timeout_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy_policy: Option<BrowserWorkerRenderDriftProxyPolicy>,
    pub render_timeout_ms: u64,
    pub min_text_bytes: usize,
    pub min_html_bytes: usize,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub headers_from_env: BTreeMap<String, String>,
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
            selector_timeout_ms: None,
            proxy_policy: None,
            render_timeout_ms: 10_000,
            min_text_bytes: 1,
            min_html_bytes: 1,
            headers_from_env: BTreeMap::new(),
            actions: Vec::new(),
        }
    }

    #[must_use]
    pub(crate) fn with_selector(mut self, selector: impl Into<String>) -> Self {
        self.selector = Some(selector.into());
        self
    }

    #[must_use]
    pub(crate) fn with_selector_timeout_ms(mut self, timeout_ms: u64) -> Self {
        if timeout_ms > 0 {
            self.selector_timeout_ms = Some(timeout_ms);
        }
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
    pub(crate) fn with_header_from_env(
        mut self,
        name: impl Into<String>,
        env_var: impl Into<String>,
    ) -> Self {
        let name = name.into();
        let env_var = env_var.into();
        if !name.trim().is_empty() && !env_var.trim().is_empty() {
            self.headers_from_env
                .insert(name.trim().to_ascii_lowercase(), env_var.trim().to_owned());
        }
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
    let mut cases = ProviderRegistry::catalog()
        .into_iter()
        .filter(|entry| config.provider_enabled(entry.id))
        .filter_map(|entry| {
            let descriptor = entry.render_drift_case?;
            let provider_config = config.provider_config(entry.id)?;
            let sample = descriptor.sample(&mut lookup);
            descriptor
                .build(provider_config, &sample)
                .map(|case| (descriptor.order(), case))
        })
        .collect::<Vec<_>>();
    cases.sort_by_key(|(order, _)| *order);
    cases.into_iter().map(|(_, case)| case).collect()
}

fn non_empty(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
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
                    "selector_timeout_ms": 30000,
                    "proxy_policy": "required",
                    "render_timeout_ms": 30000,
                    "min_text_bytes": 100,
                    "min_html_bytes": 500,
                    "headers_from_env": {
                        "cookie": "NAKO_METADATA_SCRAPER_JAVBUS_COOKIE"
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
                                "timeout_ms": 30000
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
    fn render_drift_cases_include_wave2_rendered_av_presets() {
        let config = Config::from_env_lookup(|name| match name {
            AV_PROVIDER_PRESET_ENV_VAR => Some("manual".to_owned()),
            "NAKO_METADATA_SCRAPER_PROVIDER_DMM_ENABLED" => Some("true".to_owned()),
            "NAKO_METADATA_SCRAPER_PROVIDER_XCITY_ENABLED" => Some("true".to_owned()),
            "NAKO_METADATA_SCRAPER_PROVIDER_AIRAV_ENABLED" => Some("true".to_owned()),
            "NAKO_METADATA_SCRAPER_PROVIDER_AVSOX_ENABLED" => Some("true".to_owned()),
            "NAKO_METADATA_SCRAPER_PROVIDER_MGSTAGE_ENABLED" => Some("true".to_owned()),
            "NAKO_METADATA_SCRAPER_DMM_BASE_URL" => Some("https://dmm.example".to_owned()),
            "NAKO_METADATA_SCRAPER_XCITY_BASE_URL" => Some("https://xcity.example".to_owned()),
            "NAKO_METADATA_SCRAPER_AIRAV_BASE_URL" => Some("https://airav.example".to_owned()),
            "NAKO_METADATA_SCRAPER_AVSOX_BASE_URL" => Some("https://avsox.example".to_owned()),
            "NAKO_METADATA_SCRAPER_MGSTAGE_BASE_URL" => Some("https://mgstage.example".to_owned()),
            "NAKO_METADATA_SCRAPER_BROWSER_WORKER_PROXY_POLICY" => Some("direct".to_owned()),
            _ => None,
        });

        let cases = browser_worker_render_drift_cases_from_lookup(&config, |name| match name {
            RENDER_DRIFT_SAMPLE_AV_NUMBER_ENV_VAR => Some("ABP-123".to_owned()),
            RENDER_DRIFT_SAMPLE_MGSTAGE_AV_NUMBER_ENV_VAR => Some("300MIUM-382".to_owned()),
            RENDER_DRIFT_SAMPLE_AVSOX_AV_NUMBER_ENV_VAR => Some("FC2-1723984".to_owned()),
            _ => None,
        });

        assert_eq!(cases.len(), 5);
        assert_eq!(
            serde_json::to_value(&cases).unwrap(),
            json!([
                {
                    "id": "dmm-search",
                    "url": "https://dmm.example/search/=/searchstr=ABP-123/",
                    "selector": "a[href*=\"cid=\"]",
                    "selector_timeout_ms": 30000,
                    "proxy_policy": "direct",
                    "render_timeout_ms": 30000,
                    "min_text_bytes": 100,
                    "min_html_bytes": 500,
                    "headers_from_env": {
                        "cookie": "NAKO_METADATA_SCRAPER_DMM_COOKIE"
                    }
                },
                {
                    "id": "xcity-search",
                    "url": "https://xcity.example/result_published/?q=ABP123",
                    "selector": "a[href], .item a[href], .video-item a[href], table a[href]",
                    "proxy_policy": "direct",
                    "render_timeout_ms": 10000,
                    "min_text_bytes": 100,
                    "min_html_bytes": 500
                },
                {
                    "id": "airav-search",
                    "url": "https://airav.example/?search=ABP-123",
                    "selector": "a[href], .item a[href], .video-item a[href], table a[href]",
                    "proxy_policy": "direct",
                    "render_timeout_ms": 10000,
                    "min_text_bytes": 100,
                    "min_html_bytes": 500
                },
                {
                    "id": "avsox-search",
                    "url": "https://avsox.example/cn/search/FC2-1723984",
                    "selector": "a[href], .item a[href], .video-item a[href], table a[href]",
                    "proxy_policy": "direct",
                    "render_timeout_ms": 10000,
                    "min_text_bytes": 100,
                    "min_html_bytes": 500
                },
                {
                    "id": "mgstage-detail",
                    "url": "https://mgstage.example/product/product_detail/300MIUM-382/",
                    "selector": "h1, .product_title, .detail_title, .detail, .product_detail",
                    "proxy_policy": "direct",
                    "render_timeout_ms": 10000,
                    "min_text_bytes": 100,
                    "min_html_bytes": 500
                }
            ])
        );
    }

    #[test]
    fn render_drift_cases_use_provider_specific_mgstage_default() {
        let config = Config::from_env_lookup(|name| match name {
            AV_PROVIDER_PRESET_ENV_VAR => Some("manual".to_owned()),
            "NAKO_METADATA_SCRAPER_PROVIDER_MGSTAGE_ENABLED" => Some("true".to_owned()),
            "NAKO_METADATA_SCRAPER_MGSTAGE_BASE_URL" => Some("https://mgstage.example".to_owned()),
            _ => None,
        });

        let cases = browser_worker_render_drift_cases_from_lookup(&config, |_| None);

        assert_eq!(cases.len(), 1);
        assert_eq!(
            cases[0].url,
            "https://mgstage.example/product/product_detail/300MIUM-382/"
        );
    }

    #[test]
    fn render_drift_cases_include_wave3_remaining_rendered_av_presets() {
        let config = Config::from_env_lookup(|name| match name {
            AV_PROVIDER_PRESET_ENV_VAR => Some("manual".to_owned()),
            "NAKO_METADATA_SCRAPER_PROVIDER_JAVDB_ENABLED" => Some("true".to_owned()),
            "NAKO_METADATA_SCRAPER_PROVIDER_FC2_ENABLED" => Some("true".to_owned()),
            "NAKO_METADATA_SCRAPER_PROVIDER_FC2PPVDB_ENABLED" => Some("true".to_owned()),
            "NAKO_METADATA_SCRAPER_PROVIDER_CARIBBEAN_ENABLED" => Some("true".to_owned()),
            "NAKO_METADATA_SCRAPER_PROVIDER_1PONDO_ENABLED" => Some("true".to_owned()),
            "NAKO_METADATA_SCRAPER_PROVIDER_10MUSUME_ENABLED" => Some("true".to_owned()),
            "NAKO_METADATA_SCRAPER_JAVDB_BASE_URL" => Some("https://javdb.example".to_owned()),
            "NAKO_METADATA_SCRAPER_FC2_BASE_URL" => Some("https://fc2.example".to_owned()),
            "NAKO_METADATA_SCRAPER_FC2PPVDB_BASE_URL" => {
                Some("https://fc2ppvdb.example".to_owned())
            }
            "NAKO_METADATA_SCRAPER_CARIBBEAN_BASE_URL" => {
                Some("https://caribbean.example".to_owned())
            }
            "NAKO_METADATA_SCRAPER_1PONDO_BASE_URL" => Some("https://1pondo.example".to_owned()),
            "NAKO_METADATA_SCRAPER_10MUSUME_BASE_URL" => {
                Some("https://10musume.example".to_owned())
            }
            "NAKO_METADATA_SCRAPER_BROWSER_WORKER_PROXY_POLICY" => Some("required".to_owned()),
            "NAKO_METADATA_SCRAPER_BROWSER_WORKER_SESSION_KEY" => {
                Some("session-key-should-not-emit".to_owned())
            }
            _ => None,
        });

        let cases = browser_worker_render_drift_cases_from_lookup(&config, |name| match name {
            RENDER_DRIFT_SAMPLE_JAVDB_AV_NUMBER_ENV_VAR => Some("MIDE-900".to_owned()),
            RENDER_DRIFT_SAMPLE_FC2_AV_NUMBER_ENV_VAR => Some("FC2-1723984".to_owned()),
            RENDER_DRIFT_SAMPLE_FC2PPVDB_AV_NUMBER_ENV_VAR => Some("FC2-2392657".to_owned()),
            RENDER_DRIFT_SAMPLE_CARIBBEAN_AV_NUMBER_ENV_VAR => Some("010116-001".to_owned()),
            RENDER_DRIFT_SAMPLE_1PONDO_AV_NUMBER_ENV_VAR => Some("010116-002".to_owned()),
            RENDER_DRIFT_SAMPLE_10MUSUME_AV_NUMBER_ENV_VAR => Some("010116-03".to_owned()),
            _ => None,
        });

        assert_eq!(cases.len(), 6);
        assert_eq!(
            serde_json::to_value(&cases).unwrap(),
            json!([
                {
                    "id": "javdb-search",
                    "url": "https://javdb.example/search?q=MIDE-900&locale=zh",
                    "selector": "a.box[href], a[href*=\"/v/\"]",
                    "proxy_policy": "required",
                    "render_timeout_ms": 10000,
                    "min_text_bytes": 100,
                    "min_html_bytes": 500
                },
                {
                    "id": "fc2-detail",
                    "url": "https://fc2.example/article/1723984/",
                    "selector": "h1, .items_article_info, .items_article_HeadInfo",
                    "selector_timeout_ms": 60000,
                    "proxy_policy": "required",
                    "render_timeout_ms": 60000,
                    "min_text_bytes": 100,
                    "min_html_bytes": 500
                },
                {
                    "id": "fc2ppvdb-detail",
                    "url": "https://fc2ppvdb.example/articles/2392657",
                    "selector": "article, main, .details, h1, h2",
                    "proxy_policy": "required",
                    "render_timeout_ms": 10000,
                    "min_text_bytes": 100,
                    "min_html_bytes": 500
                },
                {
                    "id": "caribbean-detail",
                    "url": "https://caribbean.example/moviepages/010116-001/index.html",
                    "selector": "article, main, .movie-info, .detail, .info, h1, h2",
                    "selector_timeout_ms": 60000,
                    "proxy_policy": "required",
                    "render_timeout_ms": 60000,
                    "min_text_bytes": 100,
                    "min_html_bytes": 500
                },
                {
                    "id": "1pondo-detail",
                    "url": "https://1pondo.example/movies/010116_002/",
                    "selector": "article, main, .movie-info, .detail, .info, h1, h2",
                    "selector_timeout_ms": 60000,
                    "proxy_policy": "required",
                    "render_timeout_ms": 60000,
                    "min_text_bytes": 100,
                    "min_html_bytes": 500
                },
                {
                    "id": "10musume-detail",
                    "url": "https://10musume.example/movies/010116_03/index.html",
                    "selector": "article, main, .movie-info, .detail, .info, h1, h2",
                    "selector_timeout_ms": 60000,
                    "proxy_policy": "required",
                    "render_timeout_ms": 60000,
                    "min_text_bytes": 100,
                    "min_html_bytes": 500
                }
            ])
        );
        let rendered = serde_json::to_string(&cases).unwrap();
        assert!(!rendered.contains("session-key-should-not-emit"));
    }

    #[test]
    fn render_drift_cases_use_route_specific_wave3_defaults() {
        let config = Config::from_env_lookup(|name| match name {
            AV_PROVIDER_PRESET_ENV_VAR => Some("manual".to_owned()),
            "NAKO_METADATA_SCRAPER_PROVIDER_FC2_ENABLED" => Some("true".to_owned()),
            "NAKO_METADATA_SCRAPER_PROVIDER_CARIBBEAN_ENABLED" => Some("true".to_owned()),
            "NAKO_METADATA_SCRAPER_PROVIDER_1PONDO_ENABLED" => Some("true".to_owned()),
            "NAKO_METADATA_SCRAPER_PROVIDER_10MUSUME_ENABLED" => Some("true".to_owned()),
            "NAKO_METADATA_SCRAPER_FC2_BASE_URL" => Some("https://fc2.example".to_owned()),
            "NAKO_METADATA_SCRAPER_CARIBBEAN_BASE_URL" => {
                Some("https://caribbean.example".to_owned())
            }
            "NAKO_METADATA_SCRAPER_1PONDO_BASE_URL" => Some("https://1pondo.example".to_owned()),
            "NAKO_METADATA_SCRAPER_10MUSUME_BASE_URL" => {
                Some("https://10musume.example".to_owned())
            }
            _ => None,
        });

        let cases = browser_worker_render_drift_cases_from_lookup(&config, |_| None);

        assert_eq!(cases.len(), 4);
        assert_eq!(cases[0].url, "https://fc2.example/article/1723984/");
        assert_eq!(
            cases[1].url,
            "https://caribbean.example/moviepages/052226-001/index.html"
        );
        assert_eq!(cases[2].url, "https://1pondo.example/movies/080616_355/");
        assert_eq!(
            cases[3].url,
            "https://10musume.example/movies/010116_001/index.html"
        );
    }

    #[test]
    fn render_drift_cases_skip_disabled_providers() {
        let config = Config::default();

        let cases = browser_worker_render_drift_cases_from_lookup(&config, |_| None);

        assert!(cases.is_empty());
    }
}
