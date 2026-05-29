use crate::{
    Config,
    config::{ProviderConfig, ProviderId},
    engine::{
        ExternalIdValueKind, ProviderExternalIdCapability, av::AV_NUMBER_EXTERNAL_ID_PROVIDER,
    },
    providers::{
        ProviderBuildStatus, ProviderConfigInput,
        official_uncensored::{
            OfficialUncensoredDetailPath, OfficialUncensoredProviderConfig, OfficialUncensoredSite,
        },
        registry::ProviderCatalogEntry,
    },
};

pub type TenMusumeProviderConfig = OfficialUncensoredProviderConfig;

pub(crate) const TENMUSUME_SITE: OfficialUncensoredSite = OfficialUncensoredSite {
    provider_id: "10musume",
    url_external_id_provider: "10musume_url",
    provider_id_enum: ProviderId::TenMusume,
    default_base_url: "https://www.10musume.com",
    base_url_env_var: "NAKO_METADATA_SCRAPER_10MUSUME_BASE_URL",
    timeout_env_var: "NAKO_METADATA_SCRAPER_10MUSUME_TIMEOUT_MS",
    enabled_env_var: "NAKO_METADATA_SCRAPER_PROVIDER_10MUSUME_ENABLED",
    capabilities: &[
        "metadata_suggestion",
        "av_number_search",
        "10musume_direct_lookup",
        "official_uncensored",
        "browser_worker_rendered_html",
    ],
    field_quality: crate::engine::ProviderFieldQualityDescriptor::new(620, 400, 620, 620),
    detail_path: OfficialUncensoredDetailPath::Movies,
    outcome: crate::engine::ProviderOutcome::TenMusumeRenderedHtmlParsed,
    tagline: "10Musume uncensored AV title",
};

const TENMUSUME_EXTERNAL_ID_CAPABILITIES: &[ProviderExternalIdCapability] = &[
    ProviderExternalIdCapability::new(
        "10musume",
        ExternalIdValueKind::Opaque,
        true,
        true,
        &["10musume_id"],
        false,
    ),
    ProviderExternalIdCapability::new(
        "10musume_url",
        ExternalIdValueKind::Url,
        true,
        true,
        &["10musume_url"],
        false,
    ),
    ProviderExternalIdCapability::new(
        AV_NUMBER_EXTERNAL_ID_PROVIDER,
        ExternalIdValueKind::Opaque,
        true,
        true,
        &[],
        false,
    ),
];

#[must_use]
pub(crate) fn catalog_entry() -> ProviderCatalogEntry {
    crate::providers::official_uncensored::catalog_entry(
        &TENMUSUME_SITE,
        TENMUSUME_EXTERNAL_ID_CAPABILITIES,
        load_config,
        rendered_page_config,
        crate::providers::render_drift::ProviderRenderDriftCaseDescriptor::new(
            140,
            crate::providers::render_drift::RENDER_DRIFT_SAMPLE_10MUSUME_AV_NUMBER_ENV_VAR,
            crate::providers::render_drift::DEFAULT_SAMPLE_10MUSUME_AV_NUMBER,
            render_drift_case_from_config,
        ),
        build_provider,
    )
}

fn rendered_page_config(
    provider: &ProviderConfig,
) -> Option<&crate::providers::rendered_page::RenderedPageSupportConfig> {
    provider
        .tenmusume_config()
        .map(|config| &config.rendered_pages)
}

fn render_drift_case_from_config(
    provider: &ProviderConfig,
    sample: &str,
) -> Option<crate::providers::render_drift::BrowserWorkerRenderDriftCase> {
    provider.tenmusume_config().map(|config| {
        crate::providers::official_uncensored::render_drift_case(&TENMUSUME_SITE, config, sample)
    })
}

fn load_config(input: ProviderConfigInput<'_>) -> ProviderConfig {
    crate::providers::official_uncensored::load_config(
        input,
        &TENMUSUME_SITE,
        ProviderConfig::tenmusume,
    )
}

fn build_provider(config: &Config) -> ProviderBuildStatus {
    crate::providers::official_uncensored::build_provider(
        config,
        &TENMUSUME_SITE,
        ProviderConfig::tenmusume_config,
    )
}
