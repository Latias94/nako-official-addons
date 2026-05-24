use nako_addon_protocol::{
    ADDON_PROTOCOL_VERSION, AddonAuth, AddonConfigurationSchema, AddonEntryPointDeclaration,
    AddonEntryPointKind, AddonHostedPageDeclaration, AddonManifest, AddonResource,
    AddonResourceDeclaration, AddonScope, AddonSecretReferenceFieldDeclaration,
    AddonTaskDeclaration,
};
use serde_json::json;

use crate::{
    Config,
    engine::bulk::{
        BULK_METADATA_SCRAPE_TASK_DESCRIPTION, BULK_METADATA_SCRAPE_TASK_ID,
        BULK_METADATA_SCRAPE_TASK_NAME, BULK_METADATA_SCRAPE_TASK_PATH,
    },
    providers::ProviderRegistry,
};

pub const ADDON_ID: &str = "nako.official.metadata-scraper";
pub const ADDON_NAME: &str = "Nako Metadata Scraper";
pub const ADDON_VERSION: &str = env!("CARGO_PKG_VERSION");

#[must_use]
pub fn addon_manifest(config: &Config) -> AddonManifest {
    AddonManifest {
        id: ADDON_ID.to_owned(),
        name: ADDON_NAME.to_owned(),
        version: ADDON_VERSION.to_owned(),
        protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
        base_url: config.base_url.clone(),
        description: Some(
            "Official Nako metadata scraper sidecar. It returns metadata suggestions and can submit explicit Nako-owned metadata/artwork side effects when configured."
                .to_owned(),
        ),
        resources: vec![AddonResourceDeclaration {
            kind: AddonResource::Metadata,
            path: "/metadata".to_owned(),
            input_schema: Some("nako.metadata.request.v1".to_owned()),
            output_schema: Some("nako.metadata.response.v1".to_owned()),
            required_scopes: vec![
                AddonScope::ItemMetadataRead,
                AddonScope::ItemMetadataSuggest,
            ],
            timeout_ms: Some(10_000),
            max_attempts: Some(2),
        }],
        entry_points: vec![AddonEntryPointDeclaration::hosted_page(
            "metadata-diagnostics",
            AddonEntryPointKind::Diagnostics,
            "Metadata Scraper Diagnostics",
            "/ui/diagnostics",
            "diagnostics",
            vec![AddonScope::ItemMetadataRead],
        )],
        hosted_pages: vec![AddonHostedPageDeclaration {
            id: "diagnostics".to_owned(),
            title: "Metadata Scraper Diagnostics".to_owned(),
            path: "/ui/diagnostics".to_owned(),
            required_scopes: vec![AddonScope::ItemMetadataRead],
        }],
        configuration_schema: Some(configuration_schema(config)),
        secret_reference_fields: secret_reference_fields(config),
        event_subscriptions: vec![],
        tasks: vec![AddonTaskDeclaration::new(
            BULK_METADATA_SCRAPE_TASK_ID,
            BULK_METADATA_SCRAPE_TASK_NAME,
            BULK_METADATA_SCRAPE_TASK_PATH,
            vec![AddonScope::AutomationRun],
        )
        .with_description(BULK_METADATA_SCRAPE_TASK_DESCRIPTION)
        .with_execution_bounds(Some(30_000), Some(2))],
        auth: AddonAuth::None,
        default_timeout_ms: Some(10_000),
        default_max_attempts: Some(2),
        scopes: vec![
            AddonScope::ItemMetadataRead,
            AddonScope::ItemMetadataSuggest,
            AddonScope::AutomationRun,
        ],
    }
}

#[must_use]
fn secret_reference_fields(config: &Config) -> Vec<AddonSecretReferenceFieldDeclaration> {
    ProviderRegistry::secret_reference_fields(config)
}

#[must_use]
fn configuration_schema(config: &Config) -> AddonConfigurationSchema {
    let provider_properties = ProviderRegistry::provider_schema_properties(config);

    AddonConfigurationSchema {
        schema_id: "nako.official.metadata-scraper.config.v1".to_owned(),
        schema: json!({
            "type": "object",
            "properties": {
                "preferred_language": {
                    "type": "string",
                    "default": config.preferred_language
                },
                "providers": {
                    "type": "object",
                    "properties": provider_properties,
                    "additionalProperties": false
                }
            },
            "additionalProperties": false
        }),
    }
}

#[cfg(test)]
mod tests {
    use nako_addon_protocol::validate_manifest;

    use super::*;
    use crate::config::{BangumiProviderConfig, ProviderConfig, ProviderId, TmdbProviderConfig};

    #[test]
    fn addon_manifest_is_valid() {
        let manifest = addon_manifest(&Config::default());

        validate_manifest(&manifest).unwrap();
        assert_eq!(manifest.id, ADDON_ID);
        assert_eq!(manifest.resources[0].path, "/metadata");
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
            ],
            ..Config::default()
        });

        assert_eq!(example_manifest, runtime_manifest);
    }
}
