use nako_addon_protocol::{AddonArtworkKind, AddonMetadataPatch};

use crate::engine::{
    AvMetadataFacts, MetadataQuery, ProviderArtworkCandidate, ProviderArtworkCandidateFacts,
    ProviderCandidateFacts, ProviderExternalId, ProviderMetadataCandidate, ProviderOutcome,
    av::AV_NUMBER_EXTERNAL_ID_PROVIDER,
};

use super::{JAVDB_PROVIDER_ID, parser::JavdbDetailFacts};

impl JavdbDetailFacts {
    pub(super) fn into_candidate(self, query: &MetadataQuery) -> ProviderMetadataCandidate {
        let mut tags = vec![
            JAVDB_PROVIDER_ID.to_owned(),
            format!("av_number:{}", self.av.number),
            format!("av_route:{:?}", self.av.route).to_ascii_lowercase(),
        ];
        tags.extend(self.actors.iter().map(|actor| format!("actor:{actor}")));
        if let Some(studio) = &self.studio {
            tags.push(format!("studio:{studio}"));
        }
        if let Some(publisher) = &self.publisher {
            tags.push(format!("publisher:{publisher}"));
        }
        if let Some(series) = &self.series {
            tags.push(format!("series:{series}"));
        }
        if let Some(director) = &self.director {
            tags.push(format!("director:{director}"));
        }
        if let Some(rating) = self.rating_milli {
            tags.push(format!("javdb_rating:{:.1}", f64::from(rating) / 200.0));
        }
        if let Some(wanted) = self.wanted_count {
            tags.push(format!("javdb_wanted:{wanted}"));
        }

        let mut artwork_candidates = Vec::new();
        if let Some(poster_url) = self.poster_url.clone() {
            artwork_candidates.push(javdb_artwork_candidate(
                &self.movie_id,
                AddonArtworkKind::Poster,
                poster_url,
                0,
            ));
        }
        for (index, url) in self.backdrop_urls.iter().cloned().enumerate() {
            artwork_candidates.push(javdb_artwork_candidate(
                &self.movie_id,
                AddonArtworkKind::Backdrop,
                url,
                index + 1,
            ));
        }

        ProviderMetadataCandidate {
            provider: JAVDB_PROVIDER_ID.to_owned(),
            provider_id: format!("javdb:movie:{}", self.movie_id),
            patch: AddonMetadataPatch {
                title: Some(self.title.clone()),
                original_title: None,
                sort_title: Some(self.title.clone()),
                overview: None,
                release_date: self.release_date.clone(),
                runtime_minutes: self.runtime_minutes,
                tagline: Some("JavDB AV title".to_owned()),
                genres: Some(self.tags.clone()).filter(|genres| !genres.is_empty()),
                tags: Some(tags).filter(|tags| !tags.is_empty()),
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
                    studio: self.studio.clone(),
                    publisher: self.publisher.clone(),
                    maker: self.studio.clone(),
                    label: self.publisher.clone(),
                    wanted_count: self.wanted_count,
                    thumb_url: self.poster_url.clone(),
                    trailer_url: None,
                    extrafanart_urls: self.backdrop_urls.clone(),
                }
                .non_empty(),
                community_score_milli: self.rating_milli,
                community_vote_count: self.wanted_count,
                external_ids: vec![
                    ProviderExternalId {
                        provider: JAVDB_PROVIDER_ID.to_owned(),
                        value: self.movie_id,
                    },
                    ProviderExternalId {
                        provider: "javdb_url".to_owned(),
                        value: self.url,
                    },
                    ProviderExternalId {
                        provider: AV_NUMBER_EXTERNAL_ID_PROVIDER.to_owned(),
                        value: self.av.number,
                    },
                ],
                provider_outcomes: vec![ProviderOutcome::JavdbRenderedHtmlParsed],
                provider_note: None,
            },
            artwork_candidates,
        }
    }
}

fn javdb_artwork_candidate(
    movie_id: &str,
    kind: AddonArtworkKind,
    source_url: String,
    index: usize,
) -> ProviderArtworkCandidate {
    ProviderArtworkCandidate {
        provider: JAVDB_PROVIDER_ID.to_owned(),
        provider_id: format!("javdb:movie:{movie_id}:artwork:{index}"),
        facts: ProviderArtworkCandidateFacts {
            kind,
            source_url,
            language: None,
            width: None,
            height: None,
        },
    }
}
