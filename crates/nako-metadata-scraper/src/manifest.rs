use nako_addon_protocol::{AddonManifest, AddonSecretReferenceFieldDeclaration};
use nako_official_addon_catalog::metadata_scraper;

use crate::{Config, config::ProviderConfig, providers::ProviderRegistry};

pub const ADDON_ID: &str = metadata_scraper::ADDON_ID;
pub const ADDON_NAME: &str = metadata_scraper::ADDON_NAME;
pub const ADDON_VERSION: &str = env!("CARGO_PKG_VERSION");

#[must_use]
pub fn addon_manifest(config: &Config) -> AddonManifest {
    metadata_scraper::manifest_with_version(
        ADDON_VERSION,
        config.base_url.clone(),
        config.preferred_language.clone(),
        provider_toggles(config),
        secret_reference_fields(config),
    )
}

#[must_use]
fn secret_reference_fields(config: &Config) -> Vec<AddonSecretReferenceFieldDeclaration> {
    ProviderRegistry::secret_reference_fields(config)
}

#[must_use]
fn provider_toggles(config: &Config) -> Vec<metadata_scraper::ProviderToggle> {
    config.providers.iter().map(provider_toggle).collect()
}

#[must_use]
fn provider_toggle(provider: &ProviderConfig) -> metadata_scraper::ProviderToggle {
    metadata_scraper::ProviderToggle::new(provider.id.as_str(), provider.enabled)
}

#[cfg(test)]
mod tests {
    use nako_addon_protocol::{AddonScope, validate_manifest};

    use super::*;
    use crate::config::{BangumiProviderConfig, ProviderConfig, ProviderId, TmdbProviderConfig};
    use crate::engine::bulk::{BULK_METADATA_SCRAPE_TASK_ID, BULK_METADATA_SCRAPE_TASK_PATH};

    #[test]
    fn addon_manifest_is_valid() {
        let config = Config::default();
        let manifest = addon_manifest(&config);

        validate_manifest(&manifest).unwrap();
        assert_eq!(manifest.id, ADDON_ID);
        assert_eq!(
            manifest,
            metadata_scraper::manifest_with_version(
                ADDON_VERSION,
                metadata_scraper::DEFAULT_BASE_URL,
                metadata_scraper::DEFAULT_LANGUAGE,
                provider_toggles(&config),
                Vec::new(),
            )
        );
        assert_eq!(
            manifest.resources[0].path,
            metadata_scraper::METADATA_RESOURCE_PATH
        );
        assert_eq!(manifest.tasks.len(), 1);
        assert_eq!(manifest.tasks[0].id, BULK_METADATA_SCRAPE_TASK_ID);
        assert_eq!(manifest.tasks[0].path, BULK_METADATA_SCRAPE_TASK_PATH);
        assert_eq!(
            manifest.tasks[0].required_scopes,
            vec![AddonScope::AutomationRun]
        );
    }

    #[test]
    fn addon_manifest_exposes_bulk_metadata_task() {
        let manifest = addon_manifest(&Config::default());

        assert_eq!(manifest.tasks.len(), 1);
    }

    #[test]
    fn addon_manifest_configuration_schema_declares_only_runtime_supported_providers() {
        let manifest = addon_manifest(&Config::default());
        let schema = &manifest.configuration_schema.unwrap().schema;
        let provider_properties = &schema["properties"]["providers"]["properties"];

        assert_eq!(provider_properties["fixture"]["default"], true);
        assert_eq!(provider_properties["tmdb"]["default"], false);
        assert_eq!(provider_properties["bangumi"]["default"], false);
        assert_eq!(provider_properties["browser_worker"]["default"], false);
        assert_eq!(provider_properties["douban"]["default"], false);
        assert_eq!(provider_properties["javdb"]["default"], false);
        assert_eq!(provider_properties["dmm"]["default"], false);
        assert_eq!(provider_properties["fc2"]["default"], false);
        assert!(manifest.secret_reference_fields.is_empty());
    }

    #[test]
    fn addon_manifest_configuration_schema_reflects_configured_provider_defaults() {
        let config = Config::from_env_lookup(|name| match name {
            "NAKO_METADATA_SCRAPER_PROVIDER_FIXTURE_ENABLED" => Some("false".to_owned()),
            "NAKO_METADATA_SCRAPER_PROVIDER_TMDB_ENABLED" => Some("true".to_owned()),
            "NAKO_METADATA_SCRAPER_PROVIDER_BANGUMI_ENABLED" => Some("true".to_owned()),
            "NAKO_METADATA_SCRAPER_PROVIDER_BROWSER_WORKER_ENABLED" => Some("true".to_owned()),
            "NAKO_METADATA_SCRAPER_PROVIDER_DOUBAN_ENABLED" => Some("true".to_owned()),
            "NAKO_METADATA_SCRAPER_PROVIDER_JAVDB_ENABLED" => Some("true".to_owned()),
            "NAKO_METADATA_SCRAPER_PROVIDER_DMM_ENABLED" => Some("true".to_owned()),
            "NAKO_METADATA_SCRAPER_PROVIDER_FC2_ENABLED" => Some("true".to_owned()),
            _ => None,
        });
        let manifest = addon_manifest(&config);
        let schema = &manifest.configuration_schema.unwrap().schema;

        assert_eq!(
            schema["properties"]["providers"]["properties"]["fixture"]["default"],
            false
        );
        assert_eq!(
            schema["properties"]["providers"]["properties"]["tmdb"]["default"],
            true
        );
        assert_eq!(
            schema["properties"]["providers"]["properties"]["bangumi"]["default"],
            true
        );
        assert_eq!(
            schema["properties"]["providers"]["properties"]["browser_worker"]["default"],
            true
        );
        assert_eq!(
            schema["properties"]["providers"]["properties"]["douban"]["default"],
            true
        );
        assert_eq!(
            schema["properties"]["providers"]["properties"]["javdb"]["default"],
            true
        );
        assert_eq!(
            schema["properties"]["providers"]["properties"]["dmm"]["default"],
            true
        );
        assert_eq!(
            schema["properties"]["providers"]["properties"]["fc2"]["default"],
            true
        );
        assert_eq!(manifest.secret_reference_fields.len(), 2);
        assert_eq!(
            manifest.secret_reference_fields[0].id,
            TmdbProviderConfig::secret_field_id()
        );
        assert!(manifest.secret_reference_fields[0].required);
        assert_eq!(
            manifest.secret_reference_fields[1].id,
            BangumiProviderConfig::secret_field_id()
        );
        assert!(!manifest.secret_reference_fields[1].required);
    }

    #[test]
    fn checked_in_example_manifest_matches_runtime_manifest() {
        let example_manifest: AddonManifest = serde_json::from_str(include_str!(
            "../../../addons/metadata-scraper/manifest.example.json"
        ))
        .unwrap();
        let runtime_manifest = addon_manifest(&Config {
            base_url: "http://nako-metadata-scraper:9100".to_owned(),
            providers: vec![
                ProviderConfig::enabled(ProviderId::Fixture),
                ProviderConfig::disabled(ProviderId::Tmdb),
                ProviderConfig::disabled(ProviderId::Bangumi),
                ProviderConfig::disabled(ProviderId::BrowserWorker),
                ProviderConfig::disabled(ProviderId::Douban),
                ProviderConfig::disabled(ProviderId::Javdb),
                ProviderConfig::disabled(ProviderId::Dmm),
                ProviderConfig::disabled(ProviderId::Fc2),
            ],
            ..Config::default()
        });

        assert_eq!(example_manifest, runtime_manifest);
    }
}
