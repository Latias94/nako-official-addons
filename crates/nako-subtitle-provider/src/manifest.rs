use nako_addon_protocol::AddonManifest;
use nako_official_addon_catalog::subtitle_provider;

use crate::Config;

pub const ADDON_VERSION: &str = env!("CARGO_PKG_VERSION");

pub(crate) use subtitle_provider::{
    ADDON_ID, ADDON_NAME, DEFAULT_CONTAINER_BASE_URL, DIAGNOSTICS_PATH, SUBTITLE_REQUEST_SCHEMA,
    SUBTITLE_RESOURCE_PATH,
};

#[must_use]
pub fn addon_manifest(config: &Config) -> AddonManifest {
    subtitle_provider::manifest_with_version(
        ADDON_VERSION,
        config.base_url.clone(),
        config.fixture_provider_enabled,
        config.default_language.clone(),
        config.default_limit,
        config.max_limit,
    )
}

#[must_use]
pub fn container_manifest() -> AddonManifest {
    let config = Config {
        base_url: DEFAULT_CONTAINER_BASE_URL.to_owned(),
        ..Config::default()
    };
    addon_manifest(&config)
}

#[cfg(test)]
mod tests {
    use nako_addon_protocol::{
        ADDON_SUBTITLE_RESPONSE_SCHEMA, AddonResource, AddonScope, validate_manifest,
    };

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
            Some(ADDON_SUBTITLE_RESPONSE_SCHEMA)
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
