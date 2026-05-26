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

pub type OnePondoProviderConfig = OfficialUncensoredProviderConfig;

pub(crate) const ONEPONDO_SITE: OfficialUncensoredSite = OfficialUncensoredSite {
    provider_id: "1pondo",
    url_external_id_provider: "1pondo_url",
    provider_id_enum: ProviderId::OnePondo,
    default_base_url: "https://www.1pondo.tv",
    base_url_env_var: "NAKO_METADATA_SCRAPER_1PONDO_BASE_URL",
    timeout_env_var: "NAKO_METADATA_SCRAPER_1PONDO_TIMEOUT_MS",
    enabled_env_var: "NAKO_METADATA_SCRAPER_PROVIDER_1PONDO_ENABLED",
    capabilities: &[
        "metadata_suggestion",
        "av_number_search",
        "1pondo_direct_lookup",
        "official_uncensored",
        "browser_worker_rendered_html",
    ],
    field_quality: crate::engine::ProviderFieldQualityDescriptor::new(620, 400, 620, 620),
    detail_path: OfficialUncensoredDetailPath::Movies,
    outcome: crate::engine::ProviderOutcome::OnePondoRenderedHtmlParsed,
    tagline: "1Pondo uncensored AV title",
};

const ONEPONDO_EXTERNAL_ID_CAPABILITIES: &[ProviderExternalIdCapability] = &[
    ProviderExternalIdCapability::new(
        "1pondo",
        ExternalIdValueKind::Opaque,
        true,
        true,
        &["1pondo_id"],
        false,
    ),
    ProviderExternalIdCapability::new(
        "1pondo_url",
        ExternalIdValueKind::Url,
        true,
        true,
        &["1pondo_url"],
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
        &ONEPONDO_SITE,
        ONEPONDO_EXTERNAL_ID_CAPABILITIES,
        load_config,
        build_provider,
    )
}

fn load_config(input: ProviderConfigInput<'_>) -> ProviderConfig {
    crate::providers::official_uncensored::load_config(
        input,
        &ONEPONDO_SITE,
        ProviderConfig::onepondo,
    )
}

fn build_provider(config: &Config) -> ProviderBuildStatus {
    crate::providers::official_uncensored::build_provider(
        config,
        &ONEPONDO_SITE,
        ProviderConfig::onepondo_config,
    )
}
