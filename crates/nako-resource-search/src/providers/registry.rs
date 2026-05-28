use std::sync::Arc;

use crate::Config;

use super::{
    ProviderConfigurationSchemaFragment, ProviderDescriptor, ProviderDiagnostic,
    ResourceSearchProvider,
    fixture::{FIXTURE_DESCRIPTOR, FixtureResourceSearchProvider},
    pansou::{PANSOU_DESCRIPTOR, PansouCompatibleProvider},
};

const PROVIDER_DISABLED: &str = "provider_disabled";
const PANSOU_MISSING_BASE_URL: &str = "pansou_missing_base_url";
const PROVIDER_DESCRIPTORS: &[ProviderDescriptor] = &[FIXTURE_DESCRIPTOR, PANSOU_DESCRIPTOR];

#[derive(Clone)]
pub struct ProviderRegistry {
    registrations: Vec<ProviderRegistration>,
}

impl ProviderRegistry {
    #[must_use]
    pub const fn descriptors() -> &'static [ProviderDescriptor] {
        PROVIDER_DESCRIPTORS
    }

    #[must_use]
    pub fn from_config(config: &Config) -> Self {
        let registrations = vec![fixture_registration(config), pansou_registration(config)];

        Self { registrations }
    }

    #[must_use]
    pub fn enabled_providers(&self) -> Vec<Arc<dyn ResourceSearchProvider>> {
        let mut providers = self
            .registrations
            .iter()
            .filter_map(|registration| registration.provider.clone())
            .collect::<Vec<_>>();
        providers.sort_by_key(|provider| std::cmp::Reverse(provider.priority()));
        providers
    }

    #[must_use]
    pub fn active_provider_count(&self) -> usize {
        self.registrations
            .iter()
            .filter(|registration| registration.active())
            .count()
    }

    #[must_use]
    pub fn active_provider_ids(&self) -> Vec<&'static str> {
        self.enabled_providers()
            .iter()
            .map(|provider| provider.id())
            .collect()
    }

    #[must_use]
    pub fn diagnostics(&self) -> Vec<ProviderDiagnostic> {
        self.registrations
            .iter()
            .map(ProviderRegistration::diagnostic)
            .collect()
    }

    #[must_use]
    pub fn configuration_schema_fragments(
        config: &Config,
    ) -> Vec<ProviderConfigurationSchemaFragment> {
        Self::descriptors()
            .iter()
            .map(|descriptor| descriptor.configuration_schema(config))
            .collect()
    }
}

#[derive(Clone)]
struct ProviderRegistration {
    descriptor: ProviderDescriptor,
    configured: bool,
    safe_message: Option<&'static str>,
    provider: Option<Arc<dyn ResourceSearchProvider>>,
}

impl ProviderRegistration {
    fn enabled(descriptor: ProviderDescriptor, provider: Arc<dyn ResourceSearchProvider>) -> Self {
        Self {
            descriptor,
            configured: true,
            safe_message: None,
            provider: Some(provider),
        }
    }

    const fn inactive(
        descriptor: ProviderDescriptor,
        configured: bool,
        safe_message: &'static str,
    ) -> Self {
        Self {
            descriptor,
            configured,
            safe_message: Some(safe_message),
            provider: None,
        }
    }

    fn active(&self) -> bool {
        self.provider.is_some()
    }

    fn diagnostic(&self) -> ProviderDiagnostic {
        ProviderDiagnostic::from_descriptor(
            self.descriptor,
            self.configured,
            self.active(),
            self.safe_message,
        )
    }
}

fn fixture_registration(config: &Config) -> ProviderRegistration {
    if config.fixture_provider_enabled {
        ProviderRegistration::enabled(FIXTURE_DESCRIPTOR, Arc::new(FixtureResourceSearchProvider))
    } else {
        ProviderRegistration::inactive(FIXTURE_DESCRIPTOR, false, PROVIDER_DISABLED)
    }
}

fn pansou_registration(config: &Config) -> ProviderRegistration {
    if config.pansou.is_active() {
        ProviderRegistration::enabled(
            PANSOU_DESCRIPTOR,
            Arc::new(PansouCompatibleProvider::new(config.pansou.clone())),
        )
    } else if config.pansou.enabled {
        ProviderRegistration::inactive(PANSOU_DESCRIPTOR, true, PANSOU_MISSING_BASE_URL)
    } else {
        ProviderRegistration::inactive(PANSOU_DESCRIPTOR, false, PROVIDER_DISABLED)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        config::PansouProviderConfig,
        providers::{
            ProviderCapability, fixture::FIXTURE_PROVIDER_ID, pansou::PANSOU_COMPATIBLE_PROVIDER_ID,
        },
        source_policy::SourcePolicy,
    };

    use super::*;

    #[test]
    fn default_registry_enables_only_official_fixture_without_network() {
        let registry = ProviderRegistry::from_config(&Config::default());

        assert_eq!(registry.active_provider_count(), 1);
        assert_eq!(registry.active_provider_ids(), vec![FIXTURE_PROVIDER_ID]);

        let diagnostics = registry.diagnostics();
        assert_eq!(diagnostics.len(), 2);
        assert_provider(
            &diagnostics,
            FIXTURE_PROVIDER_ID,
            SourcePolicy::Official,
            true,
            true,
            true,
            None,
        );
        assert_provider(
            &diagnostics,
            PANSOU_COMPATIBLE_PROVIDER_ID,
            SourcePolicy::ExternalService,
            false,
            false,
            false,
            Some(PROVIDER_DISABLED),
        );
    }

    #[test]
    fn pansou_provider_requires_endpoint_before_activation() {
        let config = Config {
            pansou: PansouProviderConfig {
                enabled: true,
                ..PansouProviderConfig::default()
            },
            ..Config::default()
        };

        let registry = ProviderRegistry::from_config(&config);

        assert_eq!(registry.active_provider_ids(), vec![FIXTURE_PROVIDER_ID]);
        assert_provider(
            &registry.diagnostics(),
            PANSOU_COMPATIBLE_PROVIDER_ID,
            SourcePolicy::ExternalService,
            false,
            true,
            false,
            Some(PANSOU_MISSING_BASE_URL),
        );
    }

    #[test]
    fn pansou_provider_activates_when_endpoint_is_configured() {
        let config = Config {
            pansou: PansouProviderConfig {
                enabled: true,
                base_url: Some("http://127.0.0.1:8888".to_owned()),
                ..PansouProviderConfig::default()
            },
            ..Config::default()
        };

        let registry = ProviderRegistry::from_config(&config);

        assert_eq!(
            registry.active_provider_ids(),
            vec![FIXTURE_PROVIDER_ID, PANSOU_COMPATIBLE_PROVIDER_ID]
        );
        assert_provider(
            &registry.diagnostics(),
            PANSOU_COMPATIBLE_PROVIDER_ID,
            SourcePolicy::ExternalService,
            false,
            true,
            true,
            None,
        );
    }

    #[test]
    fn provider_schema_fragments_keep_provider_defaults() {
        let fragments = ProviderRegistry::configuration_schema_fragments(&Config::default());

        assert_eq!(fragments.len(), 2);
        assert_eq!(fragments[0].provider_id, FIXTURE_PROVIDER_ID);
        assert!(fragments[0].provider_enabled_default);
        assert!(fragments[0].settings_schema.is_none());
        assert_eq!(fragments[1].provider_id, PANSOU_COMPATIBLE_PROVIDER_ID);
        assert!(!fragments[1].provider_enabled_default);
        assert_eq!(fragments[1].settings_key, Some("pansou"));
        assert_eq!(
            fragments[1].settings_schema.as_ref().unwrap()["properties"]["timeout_ms"]["default"],
            PansouProviderConfig::DEFAULT_TIMEOUT_MS
        );
    }

    fn assert_provider(
        diagnostics: &[ProviderDiagnostic],
        provider_id: &str,
        source_policy: SourcePolicy,
        default_enabled: bool,
        configured: bool,
        active: bool,
        safe_message: Option<&'static str>,
    ) {
        let diagnostic = diagnostics
            .iter()
            .find(|diagnostic| diagnostic.provider_id == provider_id)
            .unwrap();

        assert_eq!(diagnostic.source_policy, source_policy.as_str());
        assert_eq!(diagnostic.default_enabled, default_enabled);
        assert_eq!(diagnostic.configured, configured);
        assert_eq!(diagnostic.active, active);
        assert_eq!(diagnostic.safe_message, safe_message);
        assert!(
            diagnostic
                .capabilities
                .contains(&ProviderCapability::ResourceSearch.as_str())
        );
    }
}
