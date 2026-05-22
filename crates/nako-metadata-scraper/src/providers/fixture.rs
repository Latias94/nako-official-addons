use async_trait::async_trait;
use nako_addon_protocol::AddonMetadataPatch;

use crate::{
    engine::{MetadataCandidate, MetadataQuery},
    providers::{MetadataProvider, evidence},
};

pub struct FixtureProvider;

#[async_trait]
impl MetadataProvider for FixtureProvider {
    fn id(&self) -> &'static str {
        "fixture"
    }

    async fn suggest(&self, query: &MetadataQuery) -> anyhow::Result<Vec<MetadataCandidate>> {
        let title = normalize_title(&query.title);
        let year_suffix = query
            .year
            .map(|year| format!(" ({year})"))
            .unwrap_or_default();

        Ok(vec![MetadataCandidate {
            provider: self.id().to_owned(),
            provider_id: format!("fixture:{}", title.to_lowercase().replace(' ', "-")),
            confidence_milli: if query.year.is_some() { 760 } else { 640 },
            patch: AddonMetadataPatch {
                title: Some(format!("{title}{year_suffix}")),
                original_title: Some(title.clone()),
                sort_title: Some(title.clone()),
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
            evidence: evidence(
                "Fixture provider echoes normalized title for smoke testing.",
                query.year.is_some(),
            ),
        }])
    }
}

fn normalize_title(title: &str) -> String {
    title.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn fixture_provider_returns_metadata_candidate() {
        let candidates = FixtureProvider
            .suggest(&MetadataQuery {
                title: "  The   Matrix  ".to_owned(),
                year: Some(1999),
                language: "en-US".to_owned(),
            })
            .await
            .unwrap();

        assert_eq!(
            candidates[0].patch.title.as_deref(),
            Some("The Matrix (1999)")
        );
        assert_eq!(candidates[0].confidence_milli, 760);
    }
}
