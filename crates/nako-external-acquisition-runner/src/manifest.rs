use nako_addon_protocol::AddonManifest;
use nako_official_addon_catalog::external_acquisition_runner;

use crate::Config;

pub const ADDON_VERSION: &str = env!("CARGO_PKG_VERSION");

pub use external_acquisition_runner::{
    ACTION_REQUEST_SCHEMA, ACTION_RESPONSE_SCHEMA, ACTION_TASK_ID, ACTION_TASK_PATH, ADDON_ID,
    ADDON_NAME, DEFAULT_CONTAINER_BASE_URL, DEFAULT_RUNNER_PROFILE_ID, DIAGNOSTICS_PATH,
};

#[must_use]
pub fn addon_manifest(config: &Config) -> AddonManifest {
    external_acquisition_runner::manifest_with_version(
        ADDON_VERSION,
        config.base_url.clone(),
        config.default_runner_profile_id.clone(),
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
        ADDON_EXTERNAL_ACQUISITION_ACTION_REQUEST_SCHEMA,
        ADDON_EXTERNAL_ACQUISITION_ACTION_RESPONSE_SCHEMA, AddonScope, validate_manifest,
    };

    use super::*;

    #[test]
    fn addon_manifest_is_valid_external_acquisition_runner_manifest() {
        let manifest = addon_manifest(&Config::default());

        validate_manifest(&manifest).unwrap();
        assert_eq!(manifest.id, ADDON_ID);
        assert_eq!(manifest.name, ADDON_NAME);
        assert_eq!(manifest.version, ADDON_VERSION);
        assert!(manifest.resources.is_empty());
        assert_eq!(manifest.tasks.len(), 1);
        assert_eq!(manifest.tasks[0].id, ACTION_TASK_ID);
        assert_eq!(manifest.tasks[0].path, ACTION_TASK_PATH);
        assert_eq!(
            manifest.tasks[0].input_schema.as_deref(),
            Some(ADDON_EXTERNAL_ACQUISITION_ACTION_REQUEST_SCHEMA)
        );
        assert_eq!(
            manifest.tasks[0].output_schema.as_deref(),
            Some(ADDON_EXTERNAL_ACQUISITION_ACTION_RESPONSE_SCHEMA)
        );
        assert_eq!(
            manifest.tasks[0].required_scopes,
            vec![AddonScope::AcquisitionActionRun]
        );
        assert_eq!(manifest.hosted_pages.len(), 1);
        assert_eq!(manifest.entry_points.len(), 1);
        assert!(manifest.event_subscriptions.is_empty());
        assert!(manifest.secret_reference_fields.is_empty());
    }

    #[test]
    fn checked_in_example_manifest_matches_runtime_manifest() {
        let example_manifest: AddonManifest = serde_json::from_str(include_str!(
            "../../../addons/external-acquisition-runner/manifest.example.json"
        ))
        .unwrap();
        let runtime_manifest = container_manifest();

        assert_eq!(example_manifest, runtime_manifest);
    }
}
