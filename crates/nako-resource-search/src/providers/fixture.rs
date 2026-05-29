use async_trait::async_trait;

use super::descriptor::{
    ProviderCapability, ProviderConfigurationSchemaFragment, ProviderDescriptor,
};
use super::{ProviderSearchBatch, ResourceSearchProvider};
use crate::{
    Config,
    domain::{ResourceSearchQuery, ResourceSearchResult},
    links::resource_link,
    source_policy::SourcePolicy,
};

pub const FIXTURE_PROVIDER_ID: &str = "fixture";

const FIXTURE_CAPABILITIES: &[ProviderCapability] = &[
    ProviderCapability::ResourceSearch,
    ProviderCapability::DeterministicFixture,
    ProviderCapability::CloudDriveLinks,
    ProviderCapability::MagnetLinks,
];

pub const FIXTURE_DESCRIPTOR: ProviderDescriptor = ProviderDescriptor {
    id: FIXTURE_PROVIDER_ID,
    display_name: "Fixture",
    source_policy: SourcePolicy::Official,
    default_enabled: true,
    capabilities: FIXTURE_CAPABILITIES,
    configuration_schema: fixture_configuration_schema,
};

fn fixture_configuration_schema(config: &Config) -> ProviderConfigurationSchemaFragment {
    ProviderConfigurationSchemaFragment {
        provider_id: FIXTURE_PROVIDER_ID,
        provider_enabled_default: config.fixture_provider_enabled,
        settings_key: None,
        settings_schema: None,
    }
}

#[derive(Debug, Default)]
pub struct FixtureResourceSearchProvider;

#[async_trait]
impl ResourceSearchProvider for FixtureResourceSearchProvider {
    fn id(&self) -> &'static str {
        FIXTURE_PROVIDER_ID
    }

    fn priority(&self) -> u16 {
        1000
    }

    async fn search(&self, query: &ResourceSearchQuery) -> anyhow::Result<ProviderSearchBatch> {
        let slug = slugify(&query.query);
        let source = self.id();

        Ok(ProviderSearchBatch::complete(
            source,
            vec![
                ResourceSearchResult {
                    id: format!("fixture:{slug}:pack"),
                    title: format!("{} release pack", query.query),
                    source: source.to_owned(),
                    content: Some("Deterministic fixture resource search result.".to_owned()),
                    links: vec![
                        resource_link(format!("https://pan.quark.cn/s/{slug}"), source)
                            .expect("fixture quark link is valid")
                            .with_password("nako"),
                        resource_link("magnet:?xt=urn:btih:0123456789abcdef", source)
                            .expect("fixture magnet link is valid")
                            .with_note("fixture magnet candidate"),
                    ],
                    tags: vec!["fixture".to_owned(), "pack".to_owned()],
                    images: Vec::new(),
                    score: 900,
                },
                ResourceSearchResult {
                    id: format!("fixture:{slug}:archive"),
                    title: format!("{} 1080p archive", query.query),
                    source: source.to_owned(),
                    content: Some(
                        "Second deterministic fixture result for fusion tests.".to_owned(),
                    ),
                    links: vec![
                        resource_link(format!("https://www.aliyundrive.com/s/{slug}"), source)
                            .expect("fixture aliyun link is valid"),
                        resource_link(format!("https://PAN.QUARK.CN/s/{slug}#duplicate"), source)
                            .expect("fixture duplicate quark link is valid")
                            .with_password("nako"),
                    ],
                    tags: vec!["fixture".to_owned(), "archive".to_owned()],
                    images: Vec::new(),
                    score: 800,
                },
            ],
        ))
    }
}

fn slugify(value: &str) -> String {
    let mut slug = String::new();
    for character in value.trim().chars() {
        if character.is_ascii_alphanumeric() {
            slug.push(character.to_ascii_lowercase());
        } else if !slug.ends_with('-') {
            slug.push('-');
        }
    }

    let slug = slug.trim_matches('-');
    if slug.is_empty() {
        "query".to_owned()
    } else {
        slug.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::ResourceLinkType;

    #[tokio::test]
    async fn fixture_provider_returns_classified_results() {
        let provider = FixtureResourceSearchProvider;
        let batch = provider
            .search(&ResourceSearchQuery::free_text("Demo Movie", 20))
            .await
            .unwrap();
        let results = batch.results;

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].source, "fixture");
        assert_eq!(results[0].links[0].link_type, ResourceLinkType::Quark);
        assert_eq!(results[0].links[1].link_type, ResourceLinkType::Magnet);
        assert_eq!(results[1].links[0].link_type, ResourceLinkType::Aliyun);
    }

    #[test]
    fn slugify_keeps_fixture_urls_stable() {
        assert_eq!(slugify(" Demo Movie 2026 "), "demo-movie-2026");
        assert_eq!(slugify("!!!"), "query");
    }
}
