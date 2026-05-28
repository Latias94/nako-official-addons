use nako_addon_protocol::{
    ADDON_PROTOCOL_VERSION, AddonAuth, AddonConfigurationSchema, AddonEntryPointDeclaration,
    AddonEntryPointKind, AddonHostedPageDeclaration, AddonManifest, AddonResource,
    AddonResourceDeclaration, AddonScope,
};

use crate::Config;

pub const ADDON_ID: &str = "nako.official.subtitle-provider";
pub const ADDON_NAME: &str = "Nako Subtitle Provider";
pub const ADDON_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const DEFAULT_CONTAINER_BASE_URL: &str = "http://nako-subtitle-provider:9140";
pub const DESCRIPTION: &str =
    "Official Nako subtitle provider sidecar for read-only subtitle candidate discovery.";
pub const CONFIG_SCHEMA_ID: &str = "nako.official.subtitle-provider.config.v1";
pub const SUBTITLE_RESOURCE_PATH: &str = "/subtitle";
pub const SUBTITLE_REQUEST_SCHEMA: &str = "nako.official.subtitle_provider.request.v1";
pub const SUBTITLE_RESPONSE_SCHEMA: &str = "nako.official.subtitle_provider.response.v1";
pub const DIAGNOSTICS_ENTRY_POINT_ID: &str = "subtitle-provider-diagnostics";
pub const DIAGNOSTICS_HOSTED_PAGE_ID: &str = "diagnostics";
pub const DIAGNOSTICS_LABEL: &str = "Subtitle Provider Diagnostics";
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
            kind: AddonResource::Subtitle,
            path: SUBTITLE_RESOURCE_PATH.to_owned(),
            input_schema: Some(SUBTITLE_REQUEST_SCHEMA.to_owned()),
            output_schema: Some(SUBTITLE_RESPONSE_SCHEMA.to_owned()),
            required_scopes: vec![AddonScope::SubtitleRead],
            timeout_ms: Some(DEFAULT_TIMEOUT_MS),
            max_attempts: Some(DEFAULT_MAX_ATTEMPTS),
        }],
        entry_points: vec![AddonEntryPointDeclaration::hosted_page(
            DIAGNOSTICS_ENTRY_POINT_ID,
            AddonEntryPointKind::Diagnostics,
            DIAGNOSTICS_LABEL,
            DIAGNOSTICS_PATH,
            DIAGNOSTICS_HOSTED_PAGE_ID,
            vec![AddonScope::SubtitleRead],
        )],
        hosted_pages: vec![AddonHostedPageDeclaration::new(
            DIAGNOSTICS_HOSTED_PAGE_ID,
            DIAGNOSTICS_LABEL,
            DIAGNOSTICS_PATH,
            vec![AddonScope::SubtitleRead],
        )],
        configuration_schema: Some(configuration_schema(config)),
        secret_reference_fields: Vec::new(),
        event_subscriptions: Vec::new(),
        tasks: Vec::new(),
        auth: AddonAuth::None,
        default_timeout_ms: Some(DEFAULT_TIMEOUT_MS),
        default_max_attempts: Some(DEFAULT_MAX_ATTEMPTS),
        scopes: vec![AddonScope::SubtitleRead],
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
                        }
                    },
                    "additionalProperties": false
                },
                "default_language": {
                    "type": "string",
                    "default": config.default_language
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
                    "maximum": 200
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
    fn addon_manifest_is_valid_subtitle_manifest() {
        let manifest = addon_manifest(&Config::default());

        validate_manifest(&manifest).unwrap();
        assert_eq!(manifest.id, ADDON_ID);
        assert_eq!(manifest.name, ADDON_NAME);
        assert_eq!(manifest.version, ADDON_VERSION);
        assert_eq!(manifest.resources.len(), 1);
        assert_eq!(manifest.resources[0].kind, AddonResource::Subtitle);
        assert_eq!(manifest.resources[0].path, SUBTITLE_RESOURCE_PATH);
        assert_eq!(
            manifest.resources[0].input_schema.as_deref(),
            Some(SUBTITLE_REQUEST_SCHEMA)
        );
        assert_eq!(
            manifest.resources[0].output_schema.as_deref(),
            Some(SUBTITLE_RESPONSE_SCHEMA)
        );
        assert_eq!(manifest.scopes, vec![AddonScope::SubtitleRead]);
        assert_eq!(manifest.hosted_pages.len(), 1);
        assert_eq!(manifest.entry_points.len(), 1);
        assert!(manifest.tasks.is_empty());
        assert!(manifest.event_subscriptions.is_empty());
        assert!(manifest.secret_reference_fields.is_empty());
    }

    #[test]
    fn checked_in_example_manifest_matches_runtime_manifest() {
        let example_manifest: AddonManifest = serde_json::from_str(include_str!(
            "../../../addons/subtitle-provider/manifest.example.json"
        ))
        .unwrap();
        let runtime_manifest = container_manifest();

        assert_eq!(example_manifest, runtime_manifest);
    }
}
