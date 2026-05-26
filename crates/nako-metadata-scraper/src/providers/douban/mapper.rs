use nako_addon_protocol::{AddonArtworkKind, AddonMetadataPatch};

use crate::engine::{
    MetadataQuery, ProviderArtworkCandidate, ProviderArtworkCandidateFacts, ProviderCandidateFacts,
    ProviderExternalId, ProviderMetadataCandidate, ProviderOutcome,
};

use super::{DOUBAN_PROVIDER_ID, parser::DoubanDetailFacts};

impl DoubanDetailFacts {
    pub(super) fn into_candidate(self, query: &MetadataQuery) -> ProviderMetadataCandidate {
        let mut tags = vec![DOUBAN_PROVIDER_ID.to_owned()];
        if let Some(rating) = self.rating_milli {
            tags.push(format!("douban_rating:{:.1}", f64::from(rating) / 100.0));
        }
        if let Some(votes) = self.vote_count {
            tags.push(format!("douban_votes:{votes}"));
        }
        let mut artwork_candidates = Vec::new();
        if let Some(poster_url) = self.poster_url.clone() {
            artwork_candidates.push(ProviderArtworkCandidate {
                provider: DOUBAN_PROVIDER_ID.to_owned(),
                provider_id: format!("douban:subject:{}:poster", self.subject_id),
                facts: ProviderArtworkCandidateFacts {
                    kind: AddonArtworkKind::Poster,
                    source_url: poster_url,
                    language: None,
                    width: None,
                    height: None,
                },
            });
        }

        ProviderMetadataCandidate {
            provider: DOUBAN_PROVIDER_ID.to_owned(),
            provider_id: format!("douban:subject:{}", self.subject_id),
            patch: AddonMetadataPatch {
                title: Some(self.title.clone()),
                original_title: self
                    .original_title
                    .clone()
                    .filter(|original_title| original_title != &self.title),
                sort_title: Some(self.title.clone()),
                overview: self.summary.clone(),
                release_date: self.release_date.clone(),
                runtime_minutes: self.runtime_minutes,
                tagline: Some("Douban movie subject".to_owned()),
                genres: Some(self.genres.clone()).filter(|genres| !genres.is_empty()),
                tags: Some(tags).filter(|tags| !tags.is_empty()),
            },
            facts: ProviderCandidateFacts {
                title: Some(self.title),
                alternate_titles: Vec::new(),
                release_year: self.release_year,
                language: Some(query.language.clone()),
                av: None,
                community_score_milli: self.rating_milli,
                community_vote_count: self.vote_count,
                external_ids: vec![
                    ProviderExternalId {
                        provider: DOUBAN_PROVIDER_ID.to_owned(),
                        value: self.subject_id,
                    },
                    ProviderExternalId {
                        provider: "douban_url".to_owned(),
                        value: self.url,
                    },
                ],
                provider_outcomes: vec![ProviderOutcome::DoubanRenderedHtmlParsed],
                provider_note: None,
            },
            artwork_candidates,
        }
    }
}
