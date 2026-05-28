use serde::Serialize;

use crate::Config;
use crate::source_policy::SourcePolicy;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProviderCapability {
    ResourceSearch,
    DeterministicFixture,
    ExternalHttpSearch,
    CloudDriveLinks,
    MagnetLinks,
    Refresh,
    MergedLinkResponse,
}

impl ProviderCapability {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ResourceSearch => "resource_search",
            Self::DeterministicFixture => "deterministic_fixture",
            Self::ExternalHttpSearch => "external_http_search",
            Self::CloudDriveLinks => "cloud_drive_links",
            Self::MagnetLinks => "magnet_links",
            Self::Refresh => "refresh",
            Self::MergedLinkResponse => "merged_link_response",
        }
    }
}

pub type ProviderConfigurationSchemaBuilder = fn(&Config) -> ProviderConfigurationSchemaFragment;

#[derive(Clone, Copy, Debug)]
pub struct ProviderDescriptor {
    pub id: &'static str,
    pub display_name: &'static str,
    pub source_policy: SourcePolicy,
    pub default_enabled: bool,
    pub capabilities: &'static [ProviderCapability],
    pub configuration_schema: ProviderConfigurationSchemaBuilder,
}

impl ProviderDescriptor {
    #[must_use]
    pub fn capability_names(self) -> Vec<&'static str> {
        self.capabilities
            .iter()
            .map(|capability| capability.as_str())
            .collect()
    }

    #[must_use]
    pub fn configuration_schema(self, config: &Config) -> ProviderConfigurationSchemaFragment {
        (self.configuration_schema)(config)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProviderConfigurationSchemaFragment {
    pub provider_id: &'static str,
    pub provider_enabled_default: bool,
    pub settings_key: Option<&'static str>,
    pub settings_schema: Option<serde_json::Value>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProviderDiagnostic {
    pub provider_id: &'static str,
    pub display_name: &'static str,
    pub source_policy: &'static str,
    pub default_enabled: bool,
    pub configured: bool,
    pub active: bool,
    pub capabilities: Vec<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safe_message: Option<&'static str>,
}

impl ProviderDiagnostic {
    #[must_use]
    pub fn from_descriptor(
        descriptor: ProviderDescriptor,
        configured: bool,
        active: bool,
        safe_message: Option<&'static str>,
    ) -> Self {
        Self {
            provider_id: descriptor.id,
            display_name: descriptor.display_name,
            source_policy: descriptor.source_policy.as_str(),
            default_enabled: descriptor.default_enabled,
            configured,
            active,
            capabilities: descriptor.capability_names(),
            safe_message,
        }
    }
}
