use nako_addon_protocol::AddonManifest;
use nako_official_addon_catalog::chromecast_renderer;

use crate::Config;

pub use chromecast_renderer::{
    ADDON_ID, ADDON_NAME, ADDON_VERSION, DEFAULT_CONTAINER_BASE_URL, DEFAULT_RECEIVER_APP_ID,
    DESCRIPTION, DIAGNOSTICS_LABEL, DIAGNOSTICS_PATH, RENDERER_ADAPTER_RESOURCE_PATH,
};

#[must_use]
pub fn addon_manifest(config: &Config) -> AddonManifest {
    chromecast_renderer::manifest(config.base_url.clone(), config.receiver_app_id.clone())
}

#[must_use]
pub fn container_manifest() -> AddonManifest {
    chromecast_renderer::container_manifest()
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
        assert_eq!(
            manifest.resources[0].path,
            chromecast_renderer::RENDERER_ADAPTER_RESOURCE_PATH
        );
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
            "../../../addons/chromecast-renderer/manifest.example.json"
        ))
        .unwrap();
        let runtime_manifest = container_manifest();

        assert_eq!(example_manifest, runtime_manifest);
    }
}
