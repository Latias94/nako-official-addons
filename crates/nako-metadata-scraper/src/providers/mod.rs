use async_trait::async_trait;

use crate::engine::{CandidateEvidence, MetadataCandidate, MetadataQuery};

pub mod fixture;

#[async_trait]
pub trait MetadataProvider: Send + Sync {
    fn id(&self) -> &'static str;

    async fn suggest(&self, query: &MetadataQuery) -> anyhow::Result<Vec<MetadataCandidate>>;
}

#[must_use]
pub fn default_providers() -> Vec<Box<dyn MetadataProvider>> {
    vec![Box::new(fixture::FixtureProvider)]
}

#[must_use]
pub fn evidence(note: impl Into<String>, matched_year: bool) -> CandidateEvidence {
    CandidateEvidence {
        matched_title: true,
        matched_year,
        note: note.into(),
    }
}
