use serde::Serialize;

use crate::{
    Config,
    config::ProviderId,
    providers::{
        airav, avsox, caribbean, dmm, douban, fc2, fc2ppvdb, javbus, javdb, javlibrary, mgstage,
        official_uncensored, onepondo,
        rendered_page::{RenderedPageProxyPolicy, RenderedPageSupportConfig},
        rendered_search_av, tenmusume, xcity,
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

const DEFAULT_SAMPLE_AV_NUMBER: &str = "SSNI-644";
const DEFAULT_SAMPLE_FC2_AV_NUMBER: &str = "FC2-1723984";
const DEFAULT_SAMPLE_UNCENSORED_AV_NUMBER: &str = "010116-001";
const DEFAULT_SAMPLE_DOUBAN_TITLE: &str = "千与千寻";
const DEFAULT_SAMPLE_MGSTAGE_AV_NUMBER: &str = "300MIUM-382";

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
    let sample_av_number = non_empty(lookup(RENDER_DRIFT_SAMPLE_AV_NUMBER_ENV_VAR));
    let sample_douban_title = first_non_empty(
        lookup(RENDER_DRIFT_SAMPLE_DOUBAN_TITLE_ENV_VAR),
        DEFAULT_SAMPLE_DOUBAN_TITLE,
    );
    let sample_javbus_av_number = sample_av_number_with_default(
        lookup(RENDER_DRIFT_SAMPLE_JAVBUS_AV_NUMBER_ENV_VAR),
        sample_av_number.as_deref(),
        DEFAULT_SAMPLE_AV_NUMBER,
    );
    let sample_javlibrary_av_number = sample_av_number_with_default(
        lookup(RENDER_DRIFT_SAMPLE_JAVLIBRARY_AV_NUMBER_ENV_VAR),
        sample_av_number.as_deref(),
        DEFAULT_SAMPLE_AV_NUMBER,
    );
    let sample_dmm_av_number = sample_av_number_with_default(
        lookup(RENDER_DRIFT_SAMPLE_DMM_AV_NUMBER_ENV_VAR),
        sample_av_number.as_deref(),
        DEFAULT_SAMPLE_AV_NUMBER,
    );
    let sample_mgstage_av_number = sample_av_number_with_default(
        lookup(RENDER_DRIFT_SAMPLE_MGSTAGE_AV_NUMBER_ENV_VAR),
        sample_av_number.as_deref(),
        DEFAULT_SAMPLE_MGSTAGE_AV_NUMBER,
    );
    let sample_xcity_av_number = sample_av_number_with_default(
        lookup(RENDER_DRIFT_SAMPLE_XCITY_AV_NUMBER_ENV_VAR),
        sample_av_number.as_deref(),
        DEFAULT_SAMPLE_AV_NUMBER,
    );
    let sample_airav_av_number = sample_av_number_with_default(
        lookup(RENDER_DRIFT_SAMPLE_AIRAV_AV_NUMBER_ENV_VAR),
        sample_av_number.as_deref(),
        DEFAULT_SAMPLE_AV_NUMBER,
    );
    let sample_avsox_av_number = sample_av_number_with_default(
        lookup(RENDER_DRIFT_SAMPLE_AVSOX_AV_NUMBER_ENV_VAR),
        sample_av_number.as_deref(),
        DEFAULT_SAMPLE_AV_NUMBER,
    );
    let sample_javdb_av_number = sample_av_number_with_default(
        lookup(RENDER_DRIFT_SAMPLE_JAVDB_AV_NUMBER_ENV_VAR),
        sample_av_number.as_deref(),
        DEFAULT_SAMPLE_AV_NUMBER,
    );
    let sample_fc2_av_number = sample_av_number_with_default(
        lookup(RENDER_DRIFT_SAMPLE_FC2_AV_NUMBER_ENV_VAR),
        None,
        DEFAULT_SAMPLE_FC2_AV_NUMBER,
    );
    let sample_fc2ppvdb_av_number = sample_av_number_with_default(
        lookup(RENDER_DRIFT_SAMPLE_FC2PPVDB_AV_NUMBER_ENV_VAR),
        Some(&sample_fc2_av_number),
        DEFAULT_SAMPLE_FC2_AV_NUMBER,
    );
    let sample_caribbean_av_number = sample_av_number_with_default(
        lookup(RENDER_DRIFT_SAMPLE_CARIBBEAN_AV_NUMBER_ENV_VAR),
        None,
        DEFAULT_SAMPLE_UNCENSORED_AV_NUMBER,
    );
    let sample_1pondo_av_number = sample_av_number_with_default(
        lookup(RENDER_DRIFT_SAMPLE_1PONDO_AV_NUMBER_ENV_VAR),
        Some(&sample_caribbean_av_number),
        DEFAULT_SAMPLE_UNCENSORED_AV_NUMBER,
    );
    let sample_10musume_av_number = sample_av_number_with_default(
        lookup(RENDER_DRIFT_SAMPLE_10MUSUME_AV_NUMBER_ENV_VAR),
        Some(&sample_caribbean_av_number),
        DEFAULT_SAMPLE_UNCENSORED_AV_NUMBER,
    );

    let mut cases = Vec::new();
    if config.provider_enabled(ProviderId::Douban)
        && let Some(provider) = config
            .provider_config(ProviderId::Douban)
            .and_then(|provider| provider.douban_config())
    {
        cases.push(douban::render_drift_case(provider, &sample_douban_title));
    }
    if config.provider_enabled(ProviderId::Dmm)
        && let Some(provider) = config
            .provider_config(ProviderId::Dmm)
            .and_then(|provider| provider.dmm_config())
    {
        cases.push(dmm::render_drift_case(provider, &sample_dmm_av_number));
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
    if config.provider_enabled(ProviderId::Xcity)
        && let Some(provider) = config
            .provider_config(ProviderId::Xcity)
            .and_then(|provider| provider.xcity_config())
    {
        cases.push(rendered_search_av::render_drift_case(
            &xcity::XCITY_SITE,
            provider,
            &sample_xcity_av_number,
        ));
    }
    if config.provider_enabled(ProviderId::Airav)
        && let Some(provider) = config
            .provider_config(ProviderId::Airav)
            .and_then(|provider| provider.airav_config())
    {
        cases.push(rendered_search_av::render_drift_case(
            &airav::AIRAV_SITE,
            provider,
            &sample_airav_av_number,
        ));
    }
    if config.provider_enabled(ProviderId::Avsox)
        && let Some(provider) = config
            .provider_config(ProviderId::Avsox)
            .and_then(|provider| provider.avsox_config())
    {
        cases.push(rendered_search_av::render_drift_case(
            &avsox::AVSOX_SITE,
            provider,
            &sample_avsox_av_number,
        ));
    }
    if config.provider_enabled(ProviderId::Mgstage)
        && let Some(provider) = config
            .provider_config(ProviderId::Mgstage)
            .and_then(|provider| provider.mgstage_config())
    {
        cases.push(mgstage::render_drift_case(
            provider,
            &sample_mgstage_av_number,
        ));
    }
    if config.provider_enabled(ProviderId::Javdb)
        && let Some(provider) = config
            .provider_config(ProviderId::Javdb)
            .and_then(|provider| provider.javdb_config())
    {
        cases.push(javdb::render_drift_case(provider, &sample_javdb_av_number));
    }
    if config.provider_enabled(ProviderId::Fc2)
        && let Some(provider) = config
            .provider_config(ProviderId::Fc2)
            .and_then(|provider| provider.fc2_config())
    {
        cases.push(fc2::render_drift_case(provider, &sample_fc2_av_number));
    }
    if config.provider_enabled(ProviderId::Fc2ppvdb)
        && let Some(provider) = config
            .provider_config(ProviderId::Fc2ppvdb)
            .and_then(|provider| provider.fc2ppvdb_config())
    {
        cases.push(fc2ppvdb::render_drift_case(
            provider,
            &sample_fc2ppvdb_av_number,
        ));
    }
    if config.provider_enabled(ProviderId::Caribbean)
        && let Some(provider) = config
            .provider_config(ProviderId::Caribbean)
            .and_then(|provider| provider.caribbean_config())
    {
        cases.push(official_uncensored::render_drift_case(
            &caribbean::CARIBBEAN_SITE,
            provider,
            &sample_caribbean_av_number,
        ));
    }
    if config.provider_enabled(ProviderId::OnePondo)
        && let Some(provider) = config
            .provider_config(ProviderId::OnePondo)
            .and_then(|provider| provider.onepondo_config())
    {
        cases.push(official_uncensored::render_drift_case(
            &onepondo::ONEPONDO_SITE,
            provider,
            &sample_1pondo_av_number,
        ));
    }
    if config.provider_enabled(ProviderId::TenMusume)
        && let Some(provider) = config
            .provider_config(ProviderId::TenMusume)
            .and_then(|provider| provider.tenmusume_config())
    {
        cases.push(official_uncensored::render_drift_case(
            &tenmusume::TENMUSUME_SITE,
            provider,
            &sample_10musume_av_number,
        ));
    }

    cases
}

fn sample_av_number_with_default(
    provider_value: Option<String>,
    generic_value: Option<&str>,
    fallback: &str,
) -> String {
    non_empty(provider_value)
        .or_else(|| generic_value.map(str::to_owned))
        .unwrap_or_else(|| fallback.to_owned())
}

fn non_empty(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

fn first_non_empty(value: Option<String>, fallback: &str) -> String {
    non_empty(value).unwrap_or_else(|| fallback.to_owned())
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
                    "proxy_policy": "direct",
                    "render_timeout_ms": 10000,
                    "min_text_bytes": 100,
                    "min_html_bytes": 500
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
                    "proxy_policy": "required",
                    "render_timeout_ms": 10000,
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
                    "proxy_policy": "required",
                    "render_timeout_ms": 10000,
                    "min_text_bytes": 100,
                    "min_html_bytes": 500
                },
                {
                    "id": "1pondo-detail",
                    "url": "https://1pondo.example/movies/010116_002/index.html",
                    "selector": "article, main, .movie-info, .detail, .info, h1, h2",
                    "proxy_policy": "required",
                    "render_timeout_ms": 10000,
                    "min_text_bytes": 100,
                    "min_html_bytes": 500
                },
                {
                    "id": "10musume-detail",
                    "url": "https://10musume.example/movies/010116_03/index.html",
                    "selector": "article, main, .movie-info, .detail, .info, h1, h2",
                    "proxy_policy": "required",
                    "render_timeout_ms": 10000,
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
            "https://caribbean.example/moviepages/010116-001/index.html"
        );
        assert_eq!(
            cases[2].url,
            "https://1pondo.example/movies/010116_001/index.html"
        );
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
