use std::collections::BTreeMap;

use serde::Serialize;

use nako_addon_protocol::AddonSecretReferenceFieldDeclaration;

use crate::config::{ProviderConfig, ProviderId};
use crate::{Config, providers::MetadataProvider};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProviderDescriptor {
    pub id: &'static str,
    pub enabled: bool,
    pub available: bool,
    pub capabilities: Vec<&'static str>,
    pub status: ProviderStatus,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderStatus {
    Ready,
    Disabled,
    Unavailable,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProviderDiagnostics {
    pub supported: Vec<ProviderDescriptor>,
    pub enabled: Vec<&'static str>,
    pub disabled: Vec<&'static str>,
    pub unavailable: Vec<&'static str>,
    pub network_policy: BTreeMap<&'static str, bool>,
}

pub struct ProviderRegistry {
    config: Config,
    catalog: Vec<ProviderCatalogEntry>,
}

pub struct ProviderAssembly {
    pub providers: Vec<Box<dyn MetadataProvider>>,
    pub diagnostics: ProviderDiagnostics,
}

impl ProviderRegistry {
    #[must_use]
    pub fn from_config(config: Config) -> Self {
        Self::with_catalog(config, super::provider_catalog())
    }

    fn with_catalog(config: Config, catalog: Vec<ProviderCatalogEntry>) -> Self {
        Self { config, catalog }
    }

    #[must_use]
    pub fn catalog() -> Vec<ProviderCatalogEntry> {
        super::provider_catalog()
    }

    #[must_use]
    pub fn provider_schema_properties(
        config: &Config,
    ) -> serde_json::Map<String, serde_json::Value> {
        Self::catalog()
            .into_iter()
            .map(|entry| {
                (
                    entry.id.as_str().to_owned(),
                    serde_json::json!({
                        "type": "boolean",
                        "default": config.provider_enabled(entry.id)
                    }),
                )
            })
            .collect()
    }

    #[must_use]
    pub fn secret_reference_fields(config: &Config) -> Vec<AddonSecretReferenceFieldDeclaration> {
        Self::catalog()
            .into_iter()
            .filter(|entry| config.provider_enabled(entry.id))
            .filter_map(|entry| entry.secret_reference)
            .collect()
    }

    #[must_use]
    pub fn providers(&self) -> Vec<Box<dyn MetadataProvider>> {
        self.assemble().providers
    }

    #[must_use]
    pub fn diagnostics(&self) -> ProviderDiagnostics {
        self.assemble().diagnostics
    }

    #[must_use]
    pub fn assemble(&self) -> ProviderAssembly {
        let mut providers = Vec::new();
        let mut supported = Vec::new();
        let mut network_policy = BTreeMap::new();

        for entry in &self.catalog {
            if let Some(key) = entry.network_policy_key {
                let configured = self
                    .config
                    .provider_config(entry.id)
                    .is_some_and(|provider| (entry.proxy_configured)(provider));
                network_policy.insert(key, configured);
            }

            let enabled = self.config.provider_enabled(entry.id);
            let status = if enabled {
                match (entry.build)(&self.config) {
                    ProviderBuildStatus::Ready(provider) => {
                        providers.push(provider);
                        ProviderStatus::Ready
                    }
                    ProviderBuildStatus::Unavailable => ProviderStatus::Unavailable,
                }
            } else {
                ProviderStatus::Disabled
            };

            supported.push(ProviderDescriptor {
                id: entry.id.as_str(),
                enabled,
                available: status == ProviderStatus::Ready,
                capabilities: entry.capabilities.to_vec(),
                status,
            });
        }

        let enabled = supported
            .iter()
            .filter(|provider| provider.enabled && provider.available)
            .map(|provider| provider.id)
            .collect();
        let disabled = supported
            .iter()
            .filter(|provider| provider.status == ProviderStatus::Disabled)
            .map(|provider| provider.id)
            .collect();
        let unavailable = supported
            .iter()
            .filter(|provider| provider.status == ProviderStatus::Unavailable)
            .map(|provider| provider.id)
            .collect();

        let diagnostics = ProviderDiagnostics {
            supported,
            enabled,
            disabled,
            unavailable,
            network_policy,
        };

        ProviderAssembly {
            providers,
            diagnostics,
        }
    }
}

#[derive(Clone)]
pub struct ProviderCatalogEntry {
    pub(crate) id: ProviderId,
    pub(crate) default_enabled: bool,
    pub(crate) enabled_env_var: &'static str,
    pub(crate) capabilities: &'static [&'static str],
    pub(crate) secret_reference: Option<AddonSecretReferenceFieldDeclaration>,
    pub(crate) load_config: for<'a> fn(ProviderConfigInput<'a>) -> ProviderConfig,
    pub(crate) proxy_configured: fn(&ProviderConfig) -> bool,
    pub(crate) network_policy_key: Option<&'static str>,
    pub(crate) build: fn(&Config) -> ProviderBuildStatus,
}

pub enum ProviderBuildStatus {
    Ready(Box<dyn MetadataProvider>),
    Unavailable,
}

pub struct ProviderConfigInput<'a> {
    pub(crate) enabled: bool,
    pub(crate) lookup: &'a mut dyn FnMut(&str) -> Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{
        BangumiProviderConfig, BrowserWorkerProviderConfig, ProviderConfig, TmdbProviderConfig,
    };

    #[test]
    fn registry_builds_enabled_available_providers() {
        let registry = ProviderRegistry::from_config(Config::default());

        let providers = registry.providers();

        assert_eq!(providers.len(), 1);
        assert_eq!(providers[0].id(), ProviderId::Fixture);
    }

    #[test]
    fn registry_does_not_build_disabled_providers() {
        let registry = ProviderRegistry::from_config(Config {
            providers: vec![ProviderConfig::disabled(ProviderId::Fixture)],
            ..Config::default()
        });

        assert!(registry.providers().is_empty());
    }

    #[test]
    fn registry_reports_redaction_safe_provider_diagnostics() {
        let registry = ProviderRegistry::from_config(Config::default());

        let diagnostics = registry.diagnostics();

        assert_eq!(diagnostics.enabled, vec!["fixture"]);
        assert_eq!(
            diagnostics.disabled,
            vec!["tmdb", "bangumi", "browser_worker", "douban"]
        );
        assert!(diagnostics.unavailable.is_empty());
        assert_eq!(
            diagnostics.supported[0],
            ProviderDescriptor {
                id: "fixture",
                enabled: true,
                available: true,
                capabilities: vec!["metadata_suggestion"],
                status: ProviderStatus::Ready,
            }
        );
        assert_eq!(
            diagnostics.supported[1],
            ProviderDescriptor {
                id: "tmdb",
                enabled: false,
                available: false,
                capabilities: vec!["metadata_suggestion", "movie_search"],
                status: ProviderStatus::Disabled,
            }
        );
        assert_eq!(
            diagnostics.supported[2],
            ProviderDescriptor {
                id: "bangumi",
                enabled: false,
                available: false,
                capabilities: vec!["metadata_suggestion", "subject_search", "anime_search"],
                status: ProviderStatus::Disabled,
            }
        );
        assert_eq!(
            diagnostics.supported[3],
            ProviderDescriptor {
                id: "browser_worker",
                enabled: false,
                available: false,
                capabilities: vec!["metadata_suggestion", "rendered_page_extraction"],
                status: ProviderStatus::Disabled,
            }
        );
        assert_eq!(
            diagnostics.supported[4],
            ProviderDescriptor {
                id: "douban",
                enabled: false,
                available: false,
                capabilities: vec![
                    "metadata_suggestion",
                    "movie_search",
                    "browser_worker_rendered_html"
                ],
                status: ProviderStatus::Disabled,
            }
        );
    }

    #[test]
    fn registry_reports_disabled_provider_diagnostics() {
        let registry = ProviderRegistry::from_config(Config {
            providers: vec![ProviderConfig::disabled(ProviderId::Fixture)],
            ..Config::default()
        });

        let diagnostics = registry.diagnostics();

        assert!(diagnostics.enabled.is_empty());
        assert_eq!(
            diagnostics.disabled,
            vec!["fixture", "tmdb", "bangumi", "browser_worker", "douban"]
        );
        assert!(diagnostics.unavailable.is_empty());
        assert_eq!(diagnostics.supported[0].status, ProviderStatus::Disabled);
    }

    #[test]
    fn registry_reports_unavailable_provider_diagnostics_without_building_it() {
        fn unavailable_provider(_config: &Config) -> ProviderBuildStatus {
            ProviderBuildStatus::Unavailable
        }

        let mut entry = crate::providers::fixture::catalog_entry();
        entry.build = unavailable_provider;

        let registry = ProviderRegistry::with_catalog(Config::default(), vec![entry]);

        let diagnostics = registry.diagnostics();

        assert!(registry.providers().is_empty());
        assert!(diagnostics.enabled.is_empty());
        assert!(diagnostics.disabled.is_empty());
        assert_eq!(diagnostics.unavailable, vec!["fixture"]);
        assert_eq!(diagnostics.supported[0].status, ProviderStatus::Unavailable);
    }

    #[test]
    fn registry_reports_enabled_tmdb_without_token_as_unavailable() {
        let registry = ProviderRegistry::from_config(Config {
            providers: vec![
                ProviderConfig::disabled(ProviderId::Fixture),
                ProviderConfig {
                    id: ProviderId::Tmdb,
                    enabled: true,
                    tmdb: Some(TmdbProviderConfig::from_env_lookup(|_| None)),
                    bangumi: None,
                    browser_worker: None,
                    douban: None,
                },
            ],
            ..Config::default()
        });

        let diagnostics = registry.diagnostics();

        assert!(registry.providers().is_empty());
        assert!(diagnostics.enabled.is_empty());
        assert_eq!(
            diagnostics.disabled,
            vec!["fixture", "bangumi", "browser_worker", "douban"]
        );
        assert_eq!(diagnostics.unavailable, vec!["tmdb"]);
        assert_eq!(diagnostics.supported[1].status, ProviderStatus::Unavailable);
    }

    #[test]
    fn registry_builds_enabled_tmdb_when_token_is_configured() {
        let registry = ProviderRegistry::from_config(Config {
            providers: vec![
                ProviderConfig::disabled(ProviderId::Fixture),
                ProviderConfig {
                    id: ProviderId::Tmdb,
                    enabled: true,
                    tmdb: Some(TmdbProviderConfig {
                        read_access_token: Some("tmdb-token".to_owned()),
                        api_base_url: "https://tmdb.example/3".to_owned(),
                        language: "en-US".to_owned(),
                        include_adult: false,
                        proxy_url: None,
                    }),
                    bangumi: None,
                    browser_worker: None,
                    douban: None,
                },
            ],
            ..Config::default()
        });

        let providers = registry.providers();
        let diagnostics = registry.diagnostics();

        assert_eq!(providers.len(), 1);
        assert_eq!(providers[0].id(), ProviderId::Tmdb);
        assert_eq!(diagnostics.enabled, vec!["tmdb"]);
        assert!(diagnostics.unavailable.is_empty());
    }

    #[test]
    fn registry_builds_enabled_bangumi_without_token() {
        let registry = ProviderRegistry::from_config(Config {
            providers: vec![
                ProviderConfig::disabled(ProviderId::Fixture),
                ProviderConfig::disabled(ProviderId::Tmdb),
                ProviderConfig {
                    id: ProviderId::Bangumi,
                    enabled: true,
                    bangumi: Some(BangumiProviderConfig::from_env_lookup(|_| None)),
                    tmdb: None,
                    browser_worker: None,
                    douban: None,
                },
            ],
            ..Config::default()
        });

        let providers = registry.providers();
        let diagnostics = registry.diagnostics();

        assert_eq!(providers.len(), 1);
        assert_eq!(providers[0].id(), ProviderId::Bangumi);
        assert_eq!(diagnostics.enabled, vec!["bangumi"]);
        assert!(diagnostics.unavailable.is_empty());
    }

    #[test]
    fn registry_reports_enabled_browser_worker_without_base_url_as_unavailable() {
        let registry = ProviderRegistry::from_config(Config {
            providers: vec![
                ProviderConfig::disabled(ProviderId::Fixture),
                ProviderConfig::disabled(ProviderId::Tmdb),
                ProviderConfig::disabled(ProviderId::Bangumi),
                ProviderConfig {
                    id: ProviderId::BrowserWorker,
                    enabled: true,
                    tmdb: None,
                    bangumi: None,
                    browser_worker: Some(BrowserWorkerProviderConfig::from_env_lookup(|_| None)),
                    douban: None,
                },
            ],
            ..Config::default()
        });

        let providers = registry.providers();
        let diagnostics = registry.diagnostics();

        assert_eq!(providers.len(), 1);
        assert_eq!(providers[0].id(), ProviderId::BrowserWorker);
        assert_eq!(diagnostics.enabled, vec!["browser_worker"]);
        assert!(diagnostics.unavailable.is_empty());
    }
}
