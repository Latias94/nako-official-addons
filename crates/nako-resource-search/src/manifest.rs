use nako_addon_protocol::{AddonConfigurationSchema, AddonManifest};
use nako_official_addon_catalog::resource_search;

use crate::{Config, providers::ProviderRegistry};

pub const ADDON_VERSION: &str = env!("CARGO_PKG_VERSION");

pub use resource_search::{
    ADDON_ID, ADDON_NAME, CONFIG_SCHEMA_ID, DEFAULT_CONTAINER_BASE_URL, DEFAULT_MAX_ATTEMPTS,
    DESCRIPTION, DIAGNOSTICS_ENTRY_POINT_ID, DIAGNOSTICS_HOSTED_PAGE_ID, DIAGNOSTICS_LABEL,
    DIAGNOSTICS_PATH, RESOURCE_LINK_CHECK_RESOURCE_PATH, RESOURCE_SEARCH_RESOURCE_PATH,
};

#[must_use]
pub fn addon_manifest(config: &Config) -> AddonManifest {
    manifest_with_base_url(config.base_url.clone(), config)
}

#[must_use]
pub fn container_manifest() -> AddonManifest {
    let config = Config {
        base_url: DEFAULT_CONTAINER_BASE_URL.to_owned(),
        ..Config::default()
    };
    manifest_with_base_url(DEFAULT_CONTAINER_BASE_URL, &config)
}

fn manifest_with_base_url(base_url: impl Into<String>, config: &Config) -> AddonManifest {
    let mut manifest = resource_search::manifest_with_version(
        ADDON_VERSION,
        base_url,
        provider_toggles(config),
        config.default_limit,
        config.max_limit,
        config.search_timeout_ms,
    );
    manifest.configuration_schema = Some(configuration_schema(config));
    manifest
}

fn provider_toggles(config: &Config) -> Vec<resource_search::ProviderToggle> {
    ProviderRegistry::configuration_schema_fragments(config)
        .into_iter()
        .map(|fragment| {
            resource_search::ProviderToggle::new(
                fragment.provider_id,
                fragment.provider_enabled_default,
            )
        })
        .collect()
}

fn configuration_schema(config: &Config) -> AddonConfigurationSchema {
    let provider_fragments = ProviderRegistry::configuration_schema_fragments(config);
    let mut provider_properties = serde_json::Map::new();
    let mut root_properties = serde_json::Map::new();

    for fragment in &provider_fragments {
        provider_properties.insert(
            fragment.provider_id.to_owned(),
            serde_json::json!({
                "type": "boolean",
                "default": fragment.provider_enabled_default
            }),
        );
    }

    root_properties.insert(
        "providers".to_owned(),
        serde_json::json!({
            "type": "object",
            "properties": provider_properties,
            "additionalProperties": false
        }),
    );

    for fragment in provider_fragments {
        if let (Some(settings_key), Some(settings_schema)) =
            (fragment.settings_key, fragment.settings_schema)
        {
            root_properties.insert(settings_key.to_owned(), settings_schema);
        }
    }

    root_properties.insert(
        "default_limit".to_owned(),
        serde_json::json!({
            "type": "integer",
            "default": config.default_limit,
            "minimum": 1,
            "maximum": config.max_limit
        }),
    );
    root_properties.insert(
        "max_limit".to_owned(),
        serde_json::json!({
            "type": "integer",
            "default": config.max_limit,
            "minimum": 1,
            "maximum": 500
        }),
    );
    root_properties.insert(
        "search_timeout_ms".to_owned(),
        serde_json::json!({
            "type": "integer",
            "default": config.search_timeout_ms,
            "minimum": 250,
            "maximum": 60000
        }),
    );

    AddonConfigurationSchema::new(
        CONFIG_SCHEMA_ID,
        serde_json::json!({
            "type": "object",
            "properties": root_properties,
            "additionalProperties": false
        }),
    )
}

#[cfg(test)]
mod tests {
    use nako_addon_protocol::{
        ADDON_RESOURCE_LINK_CHECK_REQUEST_SCHEMA, ADDON_RESOURCE_LINK_CHECK_RESPONSE_SCHEMA,
        ADDON_RESOURCE_SEARCH_REQUEST_SCHEMA, ADDON_RESOURCE_SEARCH_RESPONSE_SCHEMA, AddonResource,
        AddonScope, validate_manifest,
    };

    use super::*;

    #[test]
    fn addon_manifest_is_valid_resource_search_manifest() {
        let manifest = addon_manifest(&Config::default());

        validate_manifest(&manifest).unwrap();
        assert_eq!(manifest.id, ADDON_ID);
        assert_eq!(manifest.name, ADDON_NAME);
        assert_eq!(manifest.version, ADDON_VERSION);
        assert_eq!(manifest.resources.len(), 2);
        assert_eq!(manifest.resources[0].kind, AddonResource::ResourceSearch);
        assert_eq!(manifest.resources[0].path, RESOURCE_SEARCH_RESOURCE_PATH);
        assert_eq!(
            manifest.resources[0].input_schema.as_deref(),
            Some(ADDON_RESOURCE_SEARCH_REQUEST_SCHEMA)
        );
        assert_eq!(
            manifest.resources[0].output_schema.as_deref(),
            Some(ADDON_RESOURCE_SEARCH_RESPONSE_SCHEMA)
        );
        assert_eq!(manifest.resources[1].kind, AddonResource::ResourceLinkCheck);
        assert_eq!(
            manifest.resources[1].path,
            RESOURCE_LINK_CHECK_RESOURCE_PATH
        );
        assert_eq!(
            manifest.resources[1].input_schema.as_deref(),
            Some(ADDON_RESOURCE_LINK_CHECK_REQUEST_SCHEMA)
        );
        assert_eq!(
            manifest.resources[1].output_schema.as_deref(),
            Some(ADDON_RESOURCE_LINK_CHECK_RESPONSE_SCHEMA)
        );
        assert_eq!(
            manifest.scopes,
            vec![
                AddonScope::AcquisitionSearchRead,
                AddonScope::AcquisitionLinkCheckRead
            ]
        );
        assert_eq!(manifest.hosted_pages.len(), 1);
        assert_eq!(manifest.entry_points.len(), 1);
        let schema = &manifest.configuration_schema.as_ref().unwrap().schema;
        assert_eq!(
            schema["properties"]["providers"]["properties"]["fixture"]["default"],
            true
        );
        assert_eq!(
            schema["properties"]["providers"]["properties"]["pansou_compatible"]["default"],
            false
        );
        assert_eq!(schema["properties"]["pansou"]["type"], "object");
        assert!(manifest.tasks.is_empty());
        assert!(manifest.event_subscriptions.is_empty());
        assert!(manifest.secret_reference_fields.is_empty());
    }

    #[test]
    fn container_manifest_uses_container_base_url() {
        let manifest = container_manifest();

        validate_manifest(&manifest).unwrap();
        assert_eq!(manifest.base_url, DEFAULT_CONTAINER_BASE_URL);
    }

    #[test]
    fn checked_in_example_manifest_matches_runtime_manifest() {
        let example_manifest: AddonManifest = serde_json::from_str(include_str!(
            "../../../addons/resource-search/manifest.example.json"
        ))
        .unwrap();
        let runtime_manifest = container_manifest();

        assert_eq!(example_manifest, runtime_manifest);
    }
}
