use nako_addon_protocol::{AddonArtworkKind, AddonMetadataPatch};

use crate::engine::{
    AvMetadataFacts, MetadataQuery, ProviderArtworkCandidate, ProviderArtworkCandidateFacts,
    ProviderCandidateFacts, ProviderExternalId, ProviderMetadataCandidate, ProviderOutcome,
    av::AV_NUMBER_EXTERNAL_ID_PROVIDER,
};

use super::{FC2_PROVIDER_ID, parser::Fc2DetailFacts};

impl Fc2DetailFacts {
    pub(super) fn into_candidate(self, query: &MetadataQuery) -> ProviderMetadataCandidate {
        let mut tags = vec![
            FC2_PROVIDER_ID.to_owned(),
            format!("av_number:{}", self.av.number),
            "av_route:fc2".to_owned(),
        ];
        if let Some(seller) = &self.seller {
            tags.push(format!("seller:{seller}"));
        }

        let mut artwork_candidates = Vec::new();
        if let Some(poster_url) = self.poster_url.clone() {
            artwork_candidates.push(ProviderArtworkCandidate {
                provider: FC2_PROVIDER_ID.to_owned(),
                provider_id: format!("fc2:article:{}:poster", self.article_id),
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
            provider: FC2_PROVIDER_ID.to_owned(),
            provider_id: format!("fc2:article:{}", self.article_id),
            patch: AddonMetadataPatch {
                title: Some(self.title.clone()),
                original_title: None,
                sort_title: Some(self.title.clone()),
                overview: self.overview.clone(),
                release_date: self.release_date.clone(),
                runtime_minutes: self.runtime_minutes,
                tagline: Some("FC2 AV article".to_owned()),
                genres: Some(self.tags.clone()).filter(|genres| !genres.is_empty()),
                tags: Some(tags),
            },
            facts: ProviderCandidateFacts {
                title: Some(self.title),
                alternate_titles: vec![self.av.number.clone()],
                release_year: self.release_year,
                language: Some(query.language.clone()),
                av: AvMetadataFacts {
                    studio: self.seller.clone(),
                    publisher: self.seller.clone(),
                    thumb_url: self.poster_url.clone(),
                    ..AvMetadataFacts::default()
                }
                .non_empty(),
                community_score_milli: None,
                community_vote_count: None,
                external_ids: vec![
                    ProviderExternalId {
                        provider: FC2_PROVIDER_ID.to_owned(),
                        value: self.article_id,
                    },
                    ProviderExternalId {
                        provider: "fc2_url".to_owned(),
                        value: self.url,
                    },
                    ProviderExternalId {
                        provider: AV_NUMBER_EXTERNAL_ID_PROVIDER.to_owned(),
                        value: self.av.number,
                    },
                ],
                provider_outcomes: vec![ProviderOutcome::Fc2RenderedHtmlParsed],
                provider_note: None,
            },
            artwork_candidates,
        }
    }
}
