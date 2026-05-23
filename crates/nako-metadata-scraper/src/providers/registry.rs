use serde::Serialize;

use crate::config::ProviderId;
use crate::{Config, providers::MetadataProvider};

use super::{
    bangumi::BangumiMetadataProvider, browser_worker::BrowserWorkerMetadataProvider,
    douban::DoubanMetadataProvider, fixture, tmdb::TmdbMetadataProvider,
};

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
}

pub struct ProviderRegistry {
    config: Config,
    catalog: Vec<ProviderCatalogEntry>,
}

impl ProviderRegistry {
    #[must_use]
    pub fn from_config(config: Config) -> Self {
        Self::with_catalog(config, default_catalog())
    }

    fn with_catalog(config: Config, catalog: Vec<ProviderCatalogEntry>) -> Self {
        Self { config, catalog }
    }

    #[must_use]
    pub fn providers(&self) -> Vec<Box<dyn MetadataProvider>> {
        self.catalog
            .iter()
            .filter_map(|entry| {
                if !self.config.provider_enabled(entry.id) {
                    return None;
                }
                match (entry.build)(&self.config) {
                    ProviderBuildStatus::Ready(provider) => Some(provider),
                    ProviderBuildStatus::Unavailable => None,
                }
            })
            .collect()
    }

    #[must_use]
    pub fn diagnostics(&self) -> ProviderDiagnostics {
        let supported = self
            .catalog
            .iter()
            .map(|entry| {
                let enabled = self.config.provider_enabled(entry.id);
                let status = match enabled.then(|| (entry.build)(&self.config)) {
                    None => ProviderStatus::Disabled,
                    Some(ProviderBuildStatus::Ready(_)) => ProviderStatus::Ready,
                    Some(ProviderBuildStatus::Unavailable) => ProviderStatus::Unavailable,
                };
                ProviderDescriptor {
                    id: entry.id.as_str(),
                    enabled,
                    available: status == ProviderStatus::Ready,
                    capabilities: entry.capabilities.to_vec(),
                    status,
                }
            })
            .collect::<Vec<_>>();

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

        ProviderDiagnostics {
            supported,
            enabled,
            disabled,
            unavailable,
        }
    }
}

#[derive(Clone)]
struct ProviderCatalogEntry {
    id: ProviderId,
    capabilities: &'static [&'static str],
    build: fn(&Config) -> ProviderBuildStatus,
}

enum ProviderBuildStatus {
    Ready(Box<dyn MetadataProvider>),
    Unavailable,
}

#[must_use]
fn default_catalog() -> Vec<ProviderCatalogEntry> {
    vec![
        ProviderCatalogEntry {
            id: ProviderId::Fixture,
            capabilities: &["metadata_suggestion"],
            build: |_| ProviderBuildStatus::Ready(Box::new(fixture::FixtureProvider)),
        },
        ProviderCatalogEntry {
            id: ProviderId::Tmdb,
            capabilities: &["metadata_suggestion", "movie_search"],
            build: |config| {
                let Some(tmdb_config) = config
                    .provider_config(ProviderId::Tmdb)
                    .and_then(|provider| provider.tmdb.clone())
                else {
                    return ProviderBuildStatus::Unavailable;
                };
                if tmdb_config.read_access_token.is_none() {
                    return ProviderBuildStatus::Unavailable;
                }
                match TmdbMetadataProvider::new(tmdb_config) {
                    Ok(provider) => ProviderBuildStatus::Ready(Box::new(provider)),
                    Err(_) => ProviderBuildStatus::Unavailable,
                }
            },
        },
        ProviderCatalogEntry {
            id: ProviderId::Bangumi,
            capabilities: &["metadata_suggestion", "subject_search", "anime_search"],
            build: |config| {
                let Some(bangumi_config) = config
                    .provider_config(ProviderId::Bangumi)
                    .and_then(|provider| provider.bangumi.clone())
                else {
                    return ProviderBuildStatus::Unavailable;
                };
                match BangumiMetadataProvider::new(bangumi_config) {
                    Ok(provider) => ProviderBuildStatus::Ready(Box::new(provider)),
                    Err(_) => ProviderBuildStatus::Unavailable,
                }
            },
        },
        ProviderCatalogEntry {
            id: ProviderId::BrowserWorker,
            capabilities: &["metadata_suggestion", "rendered_page_extraction"],
            build: |config| {
                let Some(browser_worker_config) = config
                    .provider_config(ProviderId::BrowserWorker)
                    .and_then(|provider| provider.browser_worker.clone())
                else {
                    return ProviderBuildStatus::Unavailable;
                };
                match BrowserWorkerMetadataProvider::new(browser_worker_config) {
                    Ok(provider) => ProviderBuildStatus::Ready(Box::new(provider)),
                    Err(_) => ProviderBuildStatus::Unavailable,
                }
            },
        },
        ProviderCatalogEntry {
            id: ProviderId::Douban,
            capabilities: &[
                "metadata_suggestion",
                "movie_search",
                "browser_worker_rendered_html",
            ],
            build: |config| {
                let Some(douban_config) = config
                    .provider_config(ProviderId::Douban)
                    .and_then(|provider| provider.douban.clone())
                else {
                    return ProviderBuildStatus::Unavailable;
                };
                match DoubanMetadataProvider::new(douban_config) {
                    Ok(provider) => ProviderBuildStatus::Ready(Box::new(provider)),
                    Err(_) => ProviderBuildStatus::Unavailable,
                }
            },
        },
    ]
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

        let registry = ProviderRegistry::with_catalog(
            Config::default(),
            vec![ProviderCatalogEntry {
                id: ProviderId::Fixture,
                capabilities: &["metadata_suggestion"],
                build: unavailable_provider,
            }],
        );

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
