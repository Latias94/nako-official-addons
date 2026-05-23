use nako_addon_protocol::{
    ADDON_PROTOCOL_VERSION, AddonAuth, AddonConfigurationSchema, AddonEntryPointDeclaration,
    AddonEntryPointKind, AddonHostedPageDeclaration, AddonManifest, AddonResource,
    AddonResourceDeclaration, AddonScope, AddonSecretReferenceFieldDeclaration,
};
use serde_json::json;

use crate::{
    Config,
    config::{BangumiProviderConfig, ProviderId, TmdbProviderConfig},
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
            "Official Nako metadata scraper sidecar. It returns metadata suggestions and does not write media libraries directly."
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
        tasks: vec![],
        auth: AddonAuth::None,
        default_timeout_ms: Some(10_000),
        default_max_attempts: Some(2),
        scopes: vec![
            AddonScope::ItemMetadataRead,
            AddonScope::ItemMetadataSuggest,
        ],
    }
}

#[must_use]
fn secret_reference_fields(config: &Config) -> Vec<AddonSecretReferenceFieldDeclaration> {
    let mut fields = Vec::new();
    if config.provider_enabled(ProviderId::Tmdb) {
        fields.push(AddonSecretReferenceFieldDeclaration::new(
            TmdbProviderConfig::secret_field_id(),
            "TMDB Read Access Token",
            Some(
                "Secret Reference for a TMDB API Read Access Token. The sidecar resolves it from NAKO_METADATA_SCRAPER_TMDB_READ_ACCESS_TOKEN."
                    .to_owned(),
            ),
            true,
        ));
    }
    if config.provider_enabled(ProviderId::Bangumi) {
        fields.push(AddonSecretReferenceFieldDeclaration::new(
            BangumiProviderConfig::secret_field_id(),
            "Bangumi Access Token",
            Some(
                "Optional Secret Reference for a Bangumi access token. Public read APIs work without it, but authenticated access may reveal user-permitted sensitive results."
                    .to_owned(),
            ),
            false,
        ));
    }

    fields
}

#[must_use]
fn configuration_schema(config: &Config) -> AddonConfigurationSchema {
    let mut provider_properties = serde_json::Map::new();
    for provider in &config.providers {
        provider_properties.insert(
            provider.id.as_str().to_owned(),
            json!({
                "type": "boolean",
                "default": provider.enabled
            }),
        );
    }

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
    use crate::config::ProviderConfig;

    #[test]
    fn addon_manifest_is_valid() {
        let manifest = addon_manifest(&Config::default());

        validate_manifest(&manifest).unwrap();
        assert_eq!(manifest.id, ADDON_ID);
        assert_eq!(manifest.resources[0].path, "/metadata");
    }

    #[test]
    fn addon_manifest_configuration_schema_declares_only_runtime_supported_providers() {
        let manifest = addon_manifest(&Config::default());
        let schema = &manifest.configuration_schema.unwrap().schema;
        let provider_properties = &schema["properties"]["providers"]["properties"];

        assert_eq!(provider_properties["fixture"]["default"], true);
        assert_eq!(provider_properties["tmdb"]["default"], false);
        assert_eq!(provider_properties["bangumi"]["default"], false);
        assert!(provider_properties.get("douban").is_none());
        assert!(manifest.secret_reference_fields.is_empty());
    }

    #[test]
    fn addon_manifest_configuration_schema_reflects_configured_provider_defaults() {
        let config = Config::from_env_lookup(|name| match name {
            "NAKO_METADATA_SCRAPER_PROVIDER_FIXTURE_ENABLED" => Some("false".to_owned()),
            "NAKO_METADATA_SCRAPER_PROVIDER_TMDB_ENABLED" => Some("true".to_owned()),
            "NAKO_METADATA_SCRAPER_PROVIDER_BANGUMI_ENABLED" => Some("true".to_owned()),
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
            ],
            ..Config::default()
        });

        assert_eq!(example_manifest, runtime_manifest);
    }
}
