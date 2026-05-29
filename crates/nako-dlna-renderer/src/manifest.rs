use nako_addon_protocol::{
    ADDON_PROTOCOL_VERSION, AddonAuth, AddonConfigurationSchema, AddonEntryPointDeclaration,
    AddonEntryPointKind, AddonHostedPageDeclaration, AddonManifest, AddonResource,
    AddonResourceDeclaration, AddonScope,
};

use crate::Config;

pub const ADDON_ID: &str = "nako.official.dlna-renderer";
pub const ADDON_NAME: &str = "Nako DLNA Renderer";
pub const ADDON_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const DEFAULT_CONTAINER_BASE_URL: &str = "http://nako-dlna-renderer:9150";
pub const DESCRIPTION: &str = "Official Nako DLNA renderer adapter sidecar. The foundation release validates host-owned renderer command envelopes and returns plan-only results.";
pub const CONFIG_SCHEMA_ID: &str = "nako.official.dlna-renderer.config.v1";
pub const RENDERER_ADAPTER_RESOURCE_PATH: &str = "/renderer-adapter";
pub const RENDERER_ADAPTER_REQUEST_SCHEMA: &str = "nako.renderer-adapter.request.v1";
pub const RENDERER_ADAPTER_RESPONSE_SCHEMA: &str = "nako.renderer-adapter.response.v1";
pub const DIAGNOSTICS_ENTRY_POINT_ID: &str = "dlna-renderer-diagnostics";
pub const DIAGNOSTICS_HOSTED_PAGE_ID: &str = "diagnostics";
pub const DIAGNOSTICS_LABEL: &str = "DLNA Renderer Diagnostics";
pub const DIAGNOSTICS_PATH: &str = "/ui/diagnostics";
pub const DEFAULT_TIMEOUT_MS: u64 = 10_000;
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
            kind: AddonResource::RendererAdapter,
            path: RENDERER_ADAPTER_RESOURCE_PATH.to_owned(),
            input_schema: Some(RENDERER_ADAPTER_REQUEST_SCHEMA.to_owned()),
            output_schema: Some(RENDERER_ADAPTER_RESPONSE_SCHEMA.to_owned()),
            required_scopes: vec![
                AddonScope::RendererAdapterRead,
                AddonScope::RendererAdapterControl,
            ],
            timeout_ms: Some(DEFAULT_TIMEOUT_MS),
            max_attempts: Some(DEFAULT_MAX_ATTEMPTS),
        }],
        entry_points: vec![AddonEntryPointDeclaration::hosted_page(
            DIAGNOSTICS_ENTRY_POINT_ID,
            AddonEntryPointKind::Diagnostics,
            DIAGNOSTICS_LABEL,
            DIAGNOSTICS_PATH,
            DIAGNOSTICS_HOSTED_PAGE_ID,
            vec![AddonScope::RendererAdapterRead],
        )],
        hosted_pages: vec![AddonHostedPageDeclaration::new(
            DIAGNOSTICS_HOSTED_PAGE_ID,
            DIAGNOSTICS_LABEL,
            DIAGNOSTICS_PATH,
            vec![AddonScope::RendererAdapterRead],
        )],
        configuration_schema: Some(configuration_schema(config)),
        secret_reference_fields: Vec::new(),
        event_subscriptions: Vec::new(),
        tasks: Vec::new(),
        auth: AddonAuth::None,
        default_timeout_ms: Some(DEFAULT_TIMEOUT_MS),
        default_max_attempts: Some(DEFAULT_MAX_ATTEMPTS),
        scopes: vec![
            AddonScope::RendererAdapterRead,
            AddonScope::RendererAdapterControl,
        ],
    }
}

fn configuration_schema(_config: &Config) -> AddonConfigurationSchema {
    AddonConfigurationSchema::new(
        CONFIG_SCHEMA_ID,
        serde_json::json!({
            "type": "object",
            "properties": {
                "manual_devices": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "required": ["stable_device_id", "display_name", "host"],
                        "properties": {
                            "stable_device_id": { "type": "string" },
                            "display_name": { "type": "string" },
                            "host": { "type": "string" },
                            "port": {
                                "type": "integer",
                                "default": 8200,
                                "minimum": 1,
                                "maximum": 65535
                            },
                            "model": { "type": "string" }
                        },
                        "additionalProperties": false
                    },
                    "default": []
                },
                "plan_only": {
                    "type": "boolean",
                    "default": true,
                    "description": "Foundation release validates commands but does not perform live DLNA control."
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
    fn addon_manifest_is_valid_renderer_adapter_manifest() {
        let manifest = addon_manifest(&Config::default());

        validate_manifest(&manifest).unwrap();
        assert_eq!(manifest.id, ADDON_ID);
        assert_eq!(manifest.name, ADDON_NAME);
        assert_eq!(manifest.version, ADDON_VERSION);
        assert_eq!(manifest.resources.len(), 1);
        assert_eq!(manifest.resources[0].kind, AddonResource::RendererAdapter);
        assert_eq!(manifest.resources[0].path, RENDERER_ADAPTER_RESOURCE_PATH);
        assert_eq!(
            manifest.scopes,
            vec![
                AddonScope::RendererAdapterRead,
                AddonScope::RendererAdapterControl,
            ]
        );
        assert_eq!(manifest.hosted_pages.len(), 1);
        assert_eq!(manifest.entry_points.len(), 1);
        assert!(manifest.tasks.is_empty());
        assert!(manifest.event_subscriptions.is_empty());
        assert!(manifest.secret_reference_fields.is_empty());
    }

    #[test]
    fn checked_in_example_manifest_matches_runtime_manifest() {
        let example_manifest: AddonManifest = serde_json::from_str(include_str!(
            "../../../addons/dlna-renderer/manifest.example.json"
        ))
        .unwrap();
        let runtime_manifest = container_manifest();

        assert_eq!(example_manifest, runtime_manifest);
    }
}
