use nako_addon_protocol::{
    ADDON_PROTOCOL_VERSION, AddonAuth, AddonConfigurationSchema, AddonEntryPointDeclaration,
    AddonEntryPointKind, AddonHostedPageDeclaration, AddonManifest, AddonResource,
    AddonResourceDeclaration, AddonScope,
};

use crate::{
    Config,
    domain::{RESOURCE_SEARCH_REQUEST_SCHEMA, RESOURCE_SEARCH_RESPONSE_SCHEMA},
};

pub const ADDON_ID: &str = "nako.official.resource-search";
pub const ADDON_NAME: &str = "Nako Resource Search";
pub const ADDON_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const DEFAULT_CONTAINER_BASE_URL: &str = "http://nako-resource-search:9130";
pub const DESCRIPTION: &str = "Official Nako resource search sidecar. Alpha contract for external resource discovery, link classification, and result fusion while the final Nako resource_search protocol surface is designed.";
pub const CONFIG_SCHEMA_ID: &str = "nako.official.resource-search.config.v1";
pub const RESOURCE_SEARCH_RESOURCE_PATH: &str = "/resource-search";
pub const DIAGNOSTICS_ENTRY_POINT_ID: &str = "resource-search-diagnostics";
pub const DIAGNOSTICS_HOSTED_PAGE_ID: &str = "diagnostics";
pub const DIAGNOSTICS_LABEL: &str = "Resource Search Diagnostics";
pub const DIAGNOSTICS_PATH: &str = "/ui/diagnostics";
pub const DEFAULT_MAX_ATTEMPTS: u32 = 1;

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
    AddonManifest {
        id: ADDON_ID.to_owned(),
        name: ADDON_NAME.to_owned(),
        version: ADDON_VERSION.to_owned(),
        protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
        base_url: base_url.into(),
        description: Some(DESCRIPTION.to_owned()),
        resources: vec![AddonResourceDeclaration {
            kind: AddonResource::Automation,
            path: RESOURCE_SEARCH_RESOURCE_PATH.to_owned(),
            input_schema: Some(RESOURCE_SEARCH_REQUEST_SCHEMA.to_owned()),
            output_schema: Some(RESOURCE_SEARCH_RESPONSE_SCHEMA.to_owned()),
            required_scopes: vec![AddonScope::AutomationRun],
            timeout_ms: Some(config.search_timeout_ms),
            max_attempts: Some(DEFAULT_MAX_ATTEMPTS),
        }],
        entry_points: vec![AddonEntryPointDeclaration::hosted_page(
            DIAGNOSTICS_ENTRY_POINT_ID,
            AddonEntryPointKind::Diagnostics,
            DIAGNOSTICS_LABEL,
            DIAGNOSTICS_PATH,
            DIAGNOSTICS_HOSTED_PAGE_ID,
            vec![AddonScope::AutomationRun],
        )],
        hosted_pages: vec![AddonHostedPageDeclaration::new(
            DIAGNOSTICS_HOSTED_PAGE_ID,
            DIAGNOSTICS_LABEL,
            DIAGNOSTICS_PATH,
            vec![AddonScope::AutomationRun],
        )],
        configuration_schema: Some(configuration_schema(config)),
        secret_reference_fields: Vec::new(),
        event_subscriptions: Vec::new(),
        tasks: Vec::new(),
        auth: AddonAuth::None,
        default_timeout_ms: Some(config.search_timeout_ms),
        default_max_attempts: Some(DEFAULT_MAX_ATTEMPTS),
        scopes: vec![AddonScope::AutomationRun],
    }
}

fn configuration_schema(config: &Config) -> AddonConfigurationSchema {
    AddonConfigurationSchema::new(
        CONFIG_SCHEMA_ID,
        serde_json::json!({
            "type": "object",
            "properties": {
                "providers": {
                    "type": "object",
                    "properties": {
                        "fixture": {
                            "type": "boolean",
                            "default": config.fixture_provider_enabled
                        },
                        "pansou_compatible": {
                            "type": "boolean",
                            "default": config.pansou.enabled
                        }
                    },
                    "additionalProperties": false
                },
                "pansou": {
                    "type": "object",
                    "properties": {
                        "base_url": {
                            "type": "string",
                            "default": config.pansou.base_url.clone().unwrap_or_default()
                        },
                        "source_type": {
                            "type": "string",
                            "default": config.pansou.source_type
                        },
                        "plugins": {
                            "type": "array",
                            "items": { "type": "string" },
                            "default": config.pansou.plugins
                        },
                        "cloud_types": {
                            "type": "array",
                            "items": { "type": "string" },
                            "default": config.pansou.cloud_types.iter().map(|link_type| link_type.as_str()).collect::<Vec<_>>()
                        },
                        "concurrency": {
                            "type": ["integer", "null"],
                            "default": config.pansou.concurrency,
                            "minimum": 1
                        },
                        "timeout_ms": {
                            "type": "integer",
                            "default": config.pansou.timeout_ms,
                            "minimum": 250,
                            "maximum": 60000
                        }
                    },
                    "additionalProperties": false
                },
                "default_limit": {
                    "type": "integer",
                    "default": config.default_limit,
                    "minimum": 1,
                    "maximum": config.max_limit
                },
                "max_limit": {
                    "type": "integer",
                    "default": config.max_limit,
                    "minimum": 1,
                    "maximum": 500
                },
                "search_timeout_ms": {
                    "type": "integer",
                    "default": config.search_timeout_ms,
                    "minimum": 250,
                    "maximum": 60000
                }
            },
            "additionalProperties": false
        }),
    )
}

#[cfg(test)]
mod tests {
    use nako_addon_protocol::{AddonResource, AddonScope, validate_manifest};

    use super::*;

    #[test]
    fn addon_manifest_is_valid_alpha_resource_search_manifest() {
        let manifest = addon_manifest(&Config::default());

        validate_manifest(&manifest).unwrap();
        assert_eq!(manifest.id, ADDON_ID);
        assert_eq!(manifest.name, ADDON_NAME);
        assert_eq!(manifest.version, ADDON_VERSION);
        assert_eq!(manifest.resources.len(), 1);
        assert_eq!(manifest.resources[0].kind, AddonResource::Automation);
        assert_eq!(manifest.resources[0].path, RESOURCE_SEARCH_RESOURCE_PATH);
        assert_eq!(
            manifest.resources[0].input_schema.as_deref(),
            Some(RESOURCE_SEARCH_REQUEST_SCHEMA)
        );
        assert_eq!(
            manifest.resources[0].output_schema.as_deref(),
            Some(RESOURCE_SEARCH_RESPONSE_SCHEMA)
        );
        assert_eq!(manifest.scopes, vec![AddonScope::AutomationRun]);
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
