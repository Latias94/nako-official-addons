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

pub type CaribbeanProviderConfig = OfficialUncensoredProviderConfig;

pub(crate) const CARIBBEAN_SITE: OfficialUncensoredSite = OfficialUncensoredSite {
    provider_id: "caribbean",
    url_external_id_provider: "caribbean_url",
    provider_id_enum: ProviderId::Caribbean,
    default_base_url: "https://www.caribbeancom.com",
    base_url_env_var: "NAKO_METADATA_SCRAPER_CARIBBEAN_BASE_URL",
    timeout_env_var: "NAKO_METADATA_SCRAPER_CARIBBEAN_TIMEOUT_MS",
    enabled_env_var: "NAKO_METADATA_SCRAPER_PROVIDER_CARIBBEAN_ENABLED",
    capabilities: &[
        "metadata_suggestion",
        "av_number_search",
        "caribbean_direct_lookup",
        "official_uncensored",
        "browser_worker_rendered_html",
    ],
    field_quality: crate::engine::ProviderFieldQualityDescriptor::new(620, 400, 620, 620),
    detail_path: OfficialUncensoredDetailPath::CaribbeanMoviepages,
    outcome: crate::engine::ProviderOutcome::CaribbeanRenderedHtmlParsed,
    tagline: "Caribbeancom uncensored AV title",
};

const CARIBBEAN_EXTERNAL_ID_CAPABILITIES: &[ProviderExternalIdCapability] = &[
    ProviderExternalIdCapability::new(
        "caribbean",
        ExternalIdValueKind::Opaque,
        true,
        true,
        &["caribbean_id"],
        false,
    ),
    ProviderExternalIdCapability::new(
        "caribbean_url",
        ExternalIdValueKind::Url,
        true,
        true,
        &["caribbean_url"],
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
        &CARIBBEAN_SITE,
        CARIBBEAN_EXTERNAL_ID_CAPABILITIES,
        load_config,
        build_provider,
    )
}

fn load_config(input: ProviderConfigInput<'_>) -> ProviderConfig {
    crate::providers::official_uncensored::load_config(
        input,
        &CARIBBEAN_SITE,
        ProviderConfig::caribbean,
    )
}

fn build_provider(config: &Config) -> ProviderBuildStatus {
    crate::providers::official_uncensored::build_provider(
        config,
        &CARIBBEAN_SITE,
        ProviderConfig::caribbean_config,
    )
}
