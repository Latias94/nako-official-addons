use nako_addon_protocol::{AddonArtworkKind, AddonMetadataPatch};

use crate::engine::{
    AvMetadataFacts, MetadataQuery, ProviderArtworkCandidate, ProviderArtworkCandidateFacts,
    ProviderCandidateFacts, ProviderExternalId, ProviderMetadataCandidate, ProviderOutcome,
    av::AV_NUMBER_EXTERNAL_ID_PROVIDER,
};

use super::{DMM_PROVIDER_ID, DMM_URL_EXTERNAL_ID_PROVIDER, parser::DmmDetailFacts};

impl DmmDetailFacts {
    pub(super) fn into_candidate(self, query: &MetadataQuery) -> ProviderMetadataCandidate {
        let mut tags = vec![
            DMM_PROVIDER_ID.to_owned(),
            format!("av_number:{}", self.av.number),
            format!("av_route:{:?}", self.av.route).to_ascii_lowercase(),
        ];
        tags.extend(self.actors.iter().map(|actor| format!("actor:{actor}")));
        if let Some(maker) = &self.maker {
            tags.push(format!("maker:{maker}"));
        }
        if let Some(label) = &self.label {
            tags.push(format!("label:{label}"));
        }
        if let Some(series) = &self.series {
            tags.push(format!("series:{series}"));
        }
        if let Some(director) = &self.director {
            tags.push(format!("director:{director}"));
        }
        if let Some(rating) = self.rating_milli {
            tags.push(format!("dmm_rating:{:.1}", f64::from(rating) / 200.0));
        }

        let mut artwork_candidates = Vec::new();
        if let Some(poster_url) = self.poster_url.clone() {
            artwork_candidates.push(dmm_artwork_candidate(
                &self.cid,
                AddonArtworkKind::Poster,
                poster_url,
                0,
            ));
        }
        for (index, url) in self.backdrop_urls.iter().cloned().enumerate() {
            artwork_candidates.push(dmm_artwork_candidate(
                &self.cid,
                AddonArtworkKind::Backdrop,
                url,
                index + 1,
            ));
        }

        ProviderMetadataCandidate {
            provider: DMM_PROVIDER_ID.to_owned(),
            provider_id: format!("dmm:cid:{}", self.cid),
            patch: AddonMetadataPatch {
                title: Some(self.title.clone()),
                original_title: None,
                sort_title: Some(self.title.clone()),
                overview: self.overview.clone(),
                release_date: self.release_date.clone(),
                runtime_minutes: self.runtime_minutes,
                tagline: Some("DMM AV title".to_owned()),
                genres: Some(self.tags.clone()).filter(|genres| !genres.is_empty()),
                tags: Some(tags).filter(|tags| !tags.is_empty()),
                ..AddonMetadataPatch::default()
            },
            facts: ProviderCandidateFacts {
                title: Some(self.title),
                alternate_titles: vec![self.av.number.clone()],
                release_year: self.release_year,
                language: Some(query.language.clone()),
                av: AvMetadataFacts {
                    actors: self.actors.clone(),
                    all_actors: self.actors.clone(),
                    directors: self.director.clone().into_iter().collect(),
                    series: self.series.clone(),
                    studio: self.maker.clone(),
                    publisher: self.label.clone(),
                    maker: self.maker.clone(),
                    label: self.label.clone(),
                    wanted_count: None,
                    thumb_url: self.poster_url.clone(),
                    trailer_url: None,
                    extrafanart_urls: self.backdrop_urls.clone(),
                }
                .non_empty(),
                community_score_milli: self.rating_milli,
                community_vote_count: None,
                external_ids: vec![
                    ProviderExternalId {
                        provider: DMM_PROVIDER_ID.to_owned(),
                        value: self.cid,
                    },
                    ProviderExternalId {
                        provider: DMM_URL_EXTERNAL_ID_PROVIDER.to_owned(),
                        value: self.url,
                    },
                    ProviderExternalId {
                        provider: AV_NUMBER_EXTERNAL_ID_PROVIDER.to_owned(),
                        value: self.av.number,
                    },
                ],
                provider_outcomes: vec![ProviderOutcome::DmmRenderedHtmlParsed],
                provider_note: None,
            },
            artwork_candidates,
        }
    }
}

fn dmm_artwork_candidate(
    cid: &str,
    kind: AddonArtworkKind,
    source_url: String,
    index: usize,
) -> ProviderArtworkCandidate {
    ProviderArtworkCandidate {
        provider: DMM_PROVIDER_ID.to_owned(),
        provider_id: format!("dmm:cid:{cid}:artwork:{index}"),
        facts: ProviderArtworkCandidateFacts {
            kind,
            source_url,
            language: None,
            width: None,
            height: None,
        },
    }
}
