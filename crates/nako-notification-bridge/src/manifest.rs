use nako_addon_protocol::AddonManifest;
use nako_official_addon_catalog::notification_bridge;

use crate::Config;

pub const ADDON_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const PROVIDER_TEST_SEND_PATH: &str = "/providers/test-send";
pub const PROVIDER_TEST_SEND_RESPONSE_SCHEMA: &str =
    "nako.official.notification-bridge.provider-test-send.v1";

pub use notification_bridge::{
    ADDON_ID, ADDON_NAME, DEFAULT_CONTAINER_BASE_URL, DEFAULT_MAX_ATTEMPTS, DEFAULT_TIMEOUT_MS,
    DESCRIPTION, DIAGNOSTICS_HOSTED_PAGE_ID, DIAGNOSTICS_LABEL, DIAGNOSTICS_PATH,
    LIBRARY_SCANNED_EVENT_KIND, LIBRARY_SCANNED_EVENT_PATH, LIBRARY_SCANNED_EVENT_SUBSCRIPTION_ID,
    WEBHOOK_REQUEST_SCHEMA, WEBHOOK_RESOURCE_PATH, WEBHOOK_RESPONSE_SCHEMA,
};

#[must_use]
pub fn addon_manifest(config: &Config) -> AddonManifest {
    notification_bridge::manifest_with_version(ADDON_VERSION, config.base_url.clone())
}

#[must_use]
pub fn container_manifest() -> AddonManifest {
    notification_bridge::manifest_with_version(ADDON_VERSION, DEFAULT_CONTAINER_BASE_URL)
}

#[cfg(test)]
mod tests {
    use nako_addon_protocol::{AddonResource, AddonScope, validate_manifest};

    use super::*;

    #[test]
    fn addon_manifest_is_valid() {
        let manifest = addon_manifest(&Config::default());

        validate_manifest(&manifest).unwrap();
        assert_eq!(manifest.id, ADDON_ID);
        assert_eq!(manifest.version, ADDON_VERSION);
        assert_eq!(manifest.scopes, vec![AddonScope::WebhookEventRead]);
        assert_eq!(manifest.resources.len(), 1);
        assert_eq!(manifest.resources[0].kind, AddonResource::Webhook);
        assert_eq!(manifest.resources[0].path, WEBHOOK_RESOURCE_PATH);
        assert!(manifest.tasks.is_empty());
        assert!(manifest.secret_reference_fields.is_empty());
        assert_eq!(manifest.event_subscriptions.len(), 1);
        assert_eq!(
            manifest.event_subscriptions[0].id,
            LIBRARY_SCANNED_EVENT_SUBSCRIPTION_ID
        );
        assert_eq!(
            manifest.event_subscriptions[0].event_kind,
            LIBRARY_SCANNED_EVENT_KIND
        );
        assert_eq!(
            manifest.event_subscriptions[0].path,
            LIBRARY_SCANNED_EVENT_PATH
        );
    }

    #[test]
    fn checked_in_example_manifest_matches_runtime_manifest() {
        let example_manifest: AddonManifest = serde_json::from_str(include_str!(
            "../../../addons/notification-bridge/manifest.example.json"
        ))
        .unwrap();
        let runtime_manifest = container_manifest();

        assert_eq!(example_manifest, runtime_manifest);
    }
}
