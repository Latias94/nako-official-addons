use nako_addon_protocol::{
    ADDON_PROTOCOL_VERSION, AddonAuth, AddonEventSubscriptionDeclaration,
    AddonHostedPageDeclaration, AddonManifest, AddonResource, AddonResourceDeclaration, AddonScope,
};

use crate::Config;

pub const ADDON_ID: &str = "nako.official.notification-bridge";
pub const ADDON_NAME: &str = "Nako Notification Bridge";
pub const ADDON_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const DEFAULT_CONTAINER_BASE_URL: &str = "http://nako-notification-bridge:9110";
pub const DESCRIPTION: &str = "Official Nako notification bridge sidecar. The first proof acknowledges scheduled Addon Events without provider fan-out.";
pub const LIBRARY_SCANNED_EVENT_SUBSCRIPTION_ID: &str = "library-scanned-notification";
pub const LIBRARY_SCANNED_EVENT_KIND: &str = "library.scanned";
pub const LIBRARY_SCANNED_EVENT_PATH: &str = "/events/library-scanned";
pub const WEBHOOK_RESOURCE_PATH: &str = "/events/library-scanned";
pub const WEBHOOK_REQUEST_SCHEMA: &str = "nako.addon.event.library-scanned.request.v1";
pub const WEBHOOK_RESPONSE_SCHEMA: &str =
    "nako.official.notification-bridge.library-scanned.event.v1";
pub const DIAGNOSTICS_HOSTED_PAGE_ID: &str = "diagnostics";
pub const DIAGNOSTICS_LABEL: &str = "Notification Bridge Diagnostics";
pub const DIAGNOSTICS_PATH: &str = "/ui/diagnostics";
pub const DEFAULT_TIMEOUT_MS: u64 = 10_000;
pub const DEFAULT_MAX_ATTEMPTS: u32 = 2;

#[must_use]
pub fn addon_manifest(config: &Config) -> AddonManifest {
    manifest_with_version(ADDON_VERSION, config.base_url.clone())
}

#[must_use]
pub fn container_manifest() -> AddonManifest {
    manifest_with_version(ADDON_VERSION, DEFAULT_CONTAINER_BASE_URL)
}

#[must_use]
pub fn manifest_with_version(
    version: impl Into<String>,
    base_url: impl Into<String>,
) -> AddonManifest {
    AddonManifest {
        id: ADDON_ID.to_owned(),
        name: ADDON_NAME.to_owned(),
        version: version.into(),
        protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
        base_url: base_url.into(),
        description: Some(DESCRIPTION.to_owned()),
        resources: vec![AddonResourceDeclaration {
            kind: AddonResource::Webhook,
            path: WEBHOOK_RESOURCE_PATH.to_owned(),
            input_schema: Some(WEBHOOK_REQUEST_SCHEMA.to_owned()),
            output_schema: Some(WEBHOOK_RESPONSE_SCHEMA.to_owned()),
            required_scopes: vec![AddonScope::WebhookEventRead],
            timeout_ms: Some(DEFAULT_TIMEOUT_MS),
            max_attempts: Some(DEFAULT_MAX_ATTEMPTS),
        }],
        entry_points: Vec::new(),
        hosted_pages: vec![AddonHostedPageDeclaration {
            id: DIAGNOSTICS_HOSTED_PAGE_ID.to_owned(),
            title: DIAGNOSTICS_LABEL.to_owned(),
            path: DIAGNOSTICS_PATH.to_owned(),
            required_scopes: vec![AddonScope::WebhookEventRead],
        }],
        configuration_schema: None,
        secret_reference_fields: Vec::new(),
        event_subscriptions: vec![AddonEventSubscriptionDeclaration::new(
            LIBRARY_SCANNED_EVENT_SUBSCRIPTION_ID,
            LIBRARY_SCANNED_EVENT_KIND,
            LIBRARY_SCANNED_EVENT_PATH,
            vec![AddonScope::WebhookEventRead],
            serde_json::Value::Null,
        )],
        tasks: Vec::new(),
        auth: AddonAuth::None,
        default_timeout_ms: Some(DEFAULT_TIMEOUT_MS),
        default_max_attempts: Some(DEFAULT_MAX_ATTEMPTS),
        scopes: vec![AddonScope::WebhookEventRead],
    }
}

#[cfg(test)]
mod tests {
    use nako_addon_protocol::validate_manifest;

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
        let runtime_manifest = manifest_with_version(ADDON_VERSION, DEFAULT_CONTAINER_BASE_URL);

        assert_eq!(example_manifest, runtime_manifest);
    }
}
