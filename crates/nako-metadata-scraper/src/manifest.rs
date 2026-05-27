use nako_addon_protocol::{AddonManifest, AddonSecretReferenceFieldDeclaration};
use nako_official_addon_catalog::metadata_scraper;

use crate::{
    Config,
    config::{AvFieldPolicyPreset, AvProviderPreset, ProviderConfig},
    providers::ProviderRegistry,
};

pub const ADDON_ID: &str = metadata_scraper::ADDON_ID;
pub const ADDON_NAME: &str = metadata_scraper::ADDON_NAME;
pub const ADDON_VERSION: &str = env!("CARGO_PKG_VERSION");

#[must_use]
pub fn addon_manifest(config: &Config) -> AddonManifest {
    let mut manifest = metadata_scraper::manifest_with_version(
        ADDON_VERSION,
        config.base_url.clone(),
        config.preferred_language.clone(),
        provider_toggles(config),
        secret_reference_fields(config),
    );
    add_av_provider_preset_schema(&mut manifest, config.av_provider_preset);
    add_av_field_policy_preset_schema(&mut manifest, config.av_field_policy_preset);
    manifest
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

fn add_av_provider_preset_schema(manifest: &mut AddonManifest, preset: AvProviderPreset) {
    add_string_enum_schema(
        manifest,
        "av_provider_preset",
        AvProviderPreset::SCHEMA_VALUES,
        preset.as_str(),
    );
}

fn add_av_field_policy_preset_schema(manifest: &mut AddonManifest, preset: AvFieldPolicyPreset) {
    add_string_enum_schema(
        manifest,
        "av_field_policy_preset",
        AvFieldPolicyPreset::SCHEMA_VALUES,
        preset.as_str(),
    );
}

fn add_string_enum_schema(
    manifest: &mut AddonManifest,
    field: &str,
    values: &[&str],
    default: &str,
) {
    let Some(configuration_schema) = manifest.configuration_schema.as_mut() else {
        return;
    };
    let Some(properties) = configuration_schema
        .schema
        .get_mut("properties")
        .and_then(|value| value.as_object_mut())
    else {
        return;
    };

    properties.insert(
        field.to_owned(),
        serde_json::json!({
            "type": "string",
            "enum": values,
            "default": default,
        }),
    );
}

#[cfg(test)]
mod tests {
    use nako_addon_protocol::{AddonScope, validate_manifest};

    use super::*;
    use crate::config::{
        AniListProviderConfig, AvFieldPolicyPreset, AvProviderPreset, BangumiProviderConfig,
        ProviderConfig, ProviderId, TmdbProviderConfig,
    };
    use crate::engine::bulk::{BULK_METADATA_SCRAPE_TASK_ID, BULK_METADATA_SCRAPE_TASK_PATH};
    use crate::providers::{javbus::JavbusProviderConfig, theporndb::ThePornDbProviderConfig};

    #[test]
    fn addon_manifest_is_valid() {
        let config = Config::default();
        let manifest = addon_manifest(&config);
        let mut expected_manifest = metadata_scraper::manifest_with_version(
            ADDON_VERSION,
            metadata_scraper::DEFAULT_BASE_URL,
            metadata_scraper::DEFAULT_LANGUAGE,
            provider_toggles(&config),
            Vec::new(),
        );
        add_av_provider_preset_schema(&mut expected_manifest, AvProviderPreset::default());
        add_av_field_policy_preset_schema(&mut expected_manifest, AvFieldPolicyPreset::default());

        validate_manifest(&manifest).unwrap();
        assert_eq!(manifest.id, ADDON_ID);
        assert_eq!(manifest, expected_manifest);
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

        assert_eq!(
            schema["properties"]["av_provider_preset"]["default"],
            "manual"
        );
        assert_eq!(
            schema["properties"]["av_provider_preset"]["enum"],
            serde_json::json!([
                "manual",
                "fast_safe",
                "official_only",
                "community_first",
                "fc2_enhanced",
                "uncensored_official"
            ])
        );
        assert_eq!(
            schema["properties"]["av_field_policy_preset"]["default"],
            "default"
        );
        assert_eq!(
            schema["properties"]["av_field_policy_preset"]["enum"],
            serde_json::json!(["default", "quality_scores", "none"])
        );
        assert_eq!(provider_properties["fixture"]["default"], true);
        assert_eq!(provider_properties["tmdb"]["default"], false);
        assert_eq!(provider_properties["bangumi"]["default"], false);
        assert_eq!(provider_properties["browser_worker"]["default"], false);
        assert_eq!(provider_properties["douban"]["default"], false);
        assert_eq!(provider_properties["javdb"]["default"], false);
        assert_eq!(provider_properties["dmm"]["default"], false);
        assert_eq!(provider_properties["xcity"]["default"], false);
        assert_eq!(provider_properties["fc2"]["default"], false);
        assert_eq!(provider_properties["fc2ppvdb"]["default"], false);
        assert_eq!(provider_properties["caribbean"]["default"], false);
        assert_eq!(provider_properties["1pondo"]["default"], false);
        assert_eq!(provider_properties["10musume"]["default"], false);
        assert_eq!(provider_properties["jav321"]["default"], false);
        assert_eq!(provider_properties["javbus"]["default"], false);
        assert_eq!(provider_properties["javlibrary"]["default"], false);
        assert_eq!(provider_properties["airav"]["default"], false);
        assert_eq!(provider_properties["avsox"]["default"], false);
        assert_eq!(provider_properties["mgstage"]["default"], false);
        assert_eq!(provider_properties["prestige"]["default"], false);
        assert_eq!(provider_properties["theporndb"]["default"], false);
        assert_eq!(provider_properties["anilist"]["default"], false);
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
            "NAKO_METADATA_SCRAPER_PROVIDER_XCITY_ENABLED" => Some("true".to_owned()),
            "NAKO_METADATA_SCRAPER_PROVIDER_FC2_ENABLED" => Some("true".to_owned()),
            "NAKO_METADATA_SCRAPER_PROVIDER_FC2PPVDB_ENABLED" => Some("true".to_owned()),
            "NAKO_METADATA_SCRAPER_PROVIDER_CARIBBEAN_ENABLED" => Some("true".to_owned()),
            "NAKO_METADATA_SCRAPER_PROVIDER_1PONDO_ENABLED" => Some("true".to_owned()),
            "NAKO_METADATA_SCRAPER_PROVIDER_10MUSUME_ENABLED" => Some("true".to_owned()),
            "NAKO_METADATA_SCRAPER_PROVIDER_JAV321_ENABLED" => Some("true".to_owned()),
            "NAKO_METADATA_SCRAPER_PROVIDER_JAVBUS_ENABLED" => Some("true".to_owned()),
            "NAKO_METADATA_SCRAPER_PROVIDER_JAVLIBRARY_ENABLED" => Some("true".to_owned()),
            "NAKO_METADATA_SCRAPER_PROVIDER_AIRAV_ENABLED" => Some("true".to_owned()),
            "NAKO_METADATA_SCRAPER_PROVIDER_AVSOX_ENABLED" => Some("true".to_owned()),
            "NAKO_METADATA_SCRAPER_PROVIDER_MGSTAGE_ENABLED" => Some("true".to_owned()),
            "NAKO_METADATA_SCRAPER_PROVIDER_PRESTIGE_ENABLED" => Some("true".to_owned()),
            "NAKO_METADATA_SCRAPER_PROVIDER_THEPORNDB_ENABLED" => Some("true".to_owned()),
            "NAKO_METADATA_SCRAPER_PROVIDER_ANILIST_ENABLED" => Some("true".to_owned()),
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
            schema["properties"]["providers"]["properties"]["xcity"]["default"],
            true
        );
        assert_eq!(
            schema["properties"]["providers"]["properties"]["fc2"]["default"],
            true
        );
        assert_eq!(
            schema["properties"]["providers"]["properties"]["fc2ppvdb"]["default"],
            true
        );
        assert_eq!(
            schema["properties"]["providers"]["properties"]["caribbean"]["default"],
            true
        );
        assert_eq!(
            schema["properties"]["providers"]["properties"]["1pondo"]["default"],
            true
        );
        assert_eq!(
            schema["properties"]["providers"]["properties"]["10musume"]["default"],
            true
        );
        assert_eq!(
            schema["properties"]["providers"]["properties"]["jav321"]["default"],
            true
        );
        assert_eq!(
            schema["properties"]["providers"]["properties"]["javbus"]["default"],
            true
        );
        assert_eq!(
            schema["properties"]["providers"]["properties"]["javlibrary"]["default"],
            true
        );
        assert_eq!(
            schema["properties"]["providers"]["properties"]["airav"]["default"],
            true
        );
        assert_eq!(
            schema["properties"]["providers"]["properties"]["avsox"]["default"],
            true
        );
        assert_eq!(
            schema["properties"]["providers"]["properties"]["mgstage"]["default"],
            true
        );
        assert_eq!(
            schema["properties"]["providers"]["properties"]["prestige"]["default"],
            true
        );
        assert_eq!(
            schema["properties"]["providers"]["properties"]["theporndb"]["default"],
            true
        );
        assert_eq!(
            schema["properties"]["providers"]["properties"]["anilist"]["default"],
            true
        );
        assert_eq!(manifest.secret_reference_fields.len(), 5);
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
        assert_eq!(
            manifest.secret_reference_fields[2].id,
            JavbusProviderConfig::secret_field_id()
        );
        assert!(!manifest.secret_reference_fields[2].required);
        assert_eq!(
            manifest.secret_reference_fields[3].id,
            ThePornDbProviderConfig::secret_field_id()
        );
        assert!(manifest.secret_reference_fields[3].required);
        assert_eq!(
            manifest.secret_reference_fields[4].id,
            AniListProviderConfig::secret_field_id()
        );
        assert!(!manifest.secret_reference_fields[4].required);
    }

    #[test]
    fn checked_in_example_manifest_matches_runtime_manifest() {
        let example_manifest: AddonManifest = serde_json::from_str(include_str!(
            "../../../addons/metadata-scraper/manifest.example.json"
        ))
        .unwrap();
        let runtime_manifest = addon_manifest(&Config {
            base_url: "http://nako-metadata-scraper:9100".to_owned(),
            av_provider_preset: AvProviderPreset::default(),
            providers: vec![
                ProviderConfig::enabled(ProviderId::Fixture),
                ProviderConfig::disabled(ProviderId::Tmdb),
                ProviderConfig::disabled(ProviderId::Bangumi),
                ProviderConfig::disabled(ProviderId::BrowserWorker),
                ProviderConfig::disabled(ProviderId::Douban),
                ProviderConfig::disabled(ProviderId::Javdb),
                ProviderConfig::disabled(ProviderId::Dmm),
                ProviderConfig::disabled(ProviderId::Xcity),
                ProviderConfig::disabled(ProviderId::Fc2),
                ProviderConfig::disabled(ProviderId::Fc2ppvdb),
                ProviderConfig::disabled(ProviderId::Caribbean),
                ProviderConfig::disabled(ProviderId::OnePondo),
                ProviderConfig::disabled(ProviderId::TenMusume),
                ProviderConfig::disabled(ProviderId::Jav321),
                ProviderConfig::disabled(ProviderId::Javbus),
                ProviderConfig::disabled(ProviderId::Javlibrary),
                ProviderConfig::disabled(ProviderId::Airav),
                ProviderConfig::disabled(ProviderId::Avsox),
                ProviderConfig::disabled(ProviderId::Mgstage),
                ProviderConfig::disabled(ProviderId::Prestige),
                ProviderConfig::disabled(ProviderId::ThePornDb),
                ProviderConfig::disabled(ProviderId::AniList),
            ],
            ..Config::default()
        });

        assert_eq!(example_manifest, runtime_manifest);
    }
}
