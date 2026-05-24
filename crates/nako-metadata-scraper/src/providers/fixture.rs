use async_trait::async_trait;
use nako_addon_protocol::AddonMetadataPatch;

use crate::{Config, providers::registry::ProviderCatalogEntry};
use crate::{
    config::ProviderId,
    engine::{MetadataQuery, ProviderCandidateFacts, ProviderMetadataCandidate},
    providers::{MetadataProvider, ProviderBuildStatus},
};

pub struct FixtureProvider;

#[must_use]
pub(crate) fn catalog_entry() -> ProviderCatalogEntry {
    ProviderCatalogEntry {
        id: ProviderId::Fixture,
        capabilities: &["metadata_suggestion"],
        secret_reference: None,
        build: build_provider,
    }
}

fn build_provider(_config: &Config) -> ProviderBuildStatus {
    ProviderBuildStatus::Ready(Box::new(FixtureProvider))
}

#[async_trait]
impl MetadataProvider for FixtureProvider {
    fn id(&self) -> ProviderId {
        ProviderId::Fixture
    }

    async fn suggest(
        &self,
        query: &MetadataQuery,
    ) -> anyhow::Result<Vec<ProviderMetadataCandidate>> {
        let year_suffix = query
            .year
            .map(|year| format!(" ({year})"))
            .unwrap_or_default();

        Ok(vec![ProviderMetadataCandidate {
            provider: self.id().as_str().to_owned(),
            provider_id: format!("fixture:{}", query.title.to_lowercase().replace(' ', "-")),
            patch: AddonMetadataPatch {
                title: Some(format!("{}{year_suffix}", query.title)),
                original_title: Some(query.title.clone()),
                sort_title: Some(query.title.clone()),
                overview: Some(
                    "Fixture metadata suggestion from the Nako Metadata Scraper skeleton."
                        .to_owned(),
                ),
                release_date: query.year.map(|year| format!("{year}-01-01")),
                runtime_minutes: None,
                tagline: None,
                genres: Some(vec!["Unknown".to_owned()]),
                tags: Some(vec![
                    "nako-metadata-scraper".to_owned(),
                    "fixture".to_owned(),
                ]),
            },
            facts: ProviderCandidateFacts {
                title: Some(query.title.clone()),
                alternate_titles: Vec::new(),
                release_year: query.year,
                language: Some(query.language.clone()),
                community_score_milli: None,
                community_vote_count: None,
                external_ids: Vec::new(),
                provider_note: Some(
                    "Fixture provider echoes normalized title for smoke testing.".to_owned(),
                ),
            },
            artwork_candidates: Vec::new(),
        }])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn fixture_provider_returns_metadata_candidate() {
        let candidates = FixtureProvider
            .suggest(&MetadataQuery {
                title: "The Matrix".to_owned(),
                year: Some(1999),
                language: "en-US".to_owned(),
                external_ids: Vec::new(),
            })
            .await
            .unwrap();

        assert_eq!(
            candidates[0].patch.title.as_deref(),
            Some("The Matrix (1999)")
        );
        assert_eq!(candidates[0].facts.release_year, Some(1999));
    }
}
