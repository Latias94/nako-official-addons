use nako_addon_protocol::{
    ADDON_PROTOCOL_VERSION, AddonAuth, AddonConfigurationSchema, AddonEntryPointDeclaration,
    AddonEntryPointKind, AddonHostedPageDeclaration, AddonManifest, AddonResource,
    AddonResourceDeclaration, AddonScope,
};

pub const ADDON_ID: &str = "nako.official.metadata-scraper";
pub const ADDON_NAME: &str = "Nako Metadata Scraper";
pub const ADDON_VERSION: &str = env!("CARGO_PKG_VERSION");

#[must_use]
pub fn addon_manifest(base_url: impl Into<String>) -> AddonManifest {
    AddonManifest {
        id: ADDON_ID.to_owned(),
        name: ADDON_NAME.to_owned(),
        version: ADDON_VERSION.to_owned(),
        protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
        base_url: base_url.into(),
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
        configuration_schema: Some(AddonConfigurationSchema {
            schema_id: "nako.official.metadata-scraper.config.v1".to_owned(),
            schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "preferred_language": {
                        "type": "string",
                        "default": "en-US"
                    },
                    "providers": {
                        "type": "object",
                        "properties": {
                            "fixture": { "type": "boolean", "default": true },
                            "tmdb": { "type": "boolean", "default": false },
                            "bangumi": { "type": "boolean", "default": false },
                            "douban": { "type": "boolean", "default": false }
                        },
                        "additionalProperties": false
                    }
                },
                "additionalProperties": false
            }),
        }),
        secret_reference_fields: vec![],
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

#[cfg(test)]
mod tests {
    use nako_addon_protocol::validate_manifest;

    use super::*;

    #[test]
    fn addon_manifest_is_valid() {
        let manifest = addon_manifest("http://127.0.0.1:9100");

        validate_manifest(&manifest).unwrap();
        assert_eq!(manifest.id, ADDON_ID);
        assert_eq!(manifest.resources[0].path, "/metadata");
    }
}
