use nako_addon_protocol::{AddonArtworkKind, AddonMetadataPatch};
use serde::Deserialize;

use crate::engine::{
    MetadataQuery, ProviderArtworkCandidate, ProviderArtworkCandidateFacts, ProviderCandidateFacts,
    ProviderExternalId, ProviderMetadataCandidate, ProviderOutcome,
};

use super::{
    TMDB_PROVIDER_ID, TMDB_TV_EXTERNAL_ID_PROVIDER_ID,
    parser::{
        TmdbGenre, TmdbMovieAlternativeTitles, TmdbMovieDetail, TmdbMovieExternalIds,
        TmdbTvAlternativeTitles, TmdbTvDetail, TmdbTvExternalIds,
    },
};

const TMDB_IMAGE_BASE_URL: &str = "https://image.tmdb.org/t/p/original";

#[derive(Clone, Debug, Deserialize)]
pub(super) struct TmdbMovieSearchResult {
    pub(super) id: u64,
    pub(super) title: Option<String>,
    pub(super) original_title: Option<String>,
    pub(super) overview: Option<String>,
    pub(super) release_date: Option<String>,
    pub(super) poster_path: Option<String>,
    pub(super) backdrop_path: Option<String>,
    #[serde(default)]
    pub(super) genre_ids: Vec<u64>,
    pub(super) vote_average: Option<f64>,
    pub(super) vote_count: Option<u32>,
}

impl TmdbMovieSearchResult {
    pub(super) fn direct_lookup_seed(id: u64) -> Self {
        Self {
            id,
            title: None,
            original_title: None,
            overview: None,
            release_date: None,
            poster_path: None,
            backdrop_path: None,
            genre_ids: Vec::new(),
            vote_average: None,
            vote_count: None,
        }
    }

    pub(super) fn into_degraded_candidate(
        self,
        query: &MetadataQuery,
    ) -> ProviderMetadataCandidate {
        let TmdbMovieSearchResult {
            id,
            title: search_title,
            original_title,
            overview,
            release_date,
            poster_path,
            backdrop_path,
            genre_ids,
            vote_average,
            vote_count,
        } = self;
        let title = first_non_empty(&[search_title.as_deref(), original_title.as_deref()])
            .unwrap_or_else(|| query.title.clone());
        let original_title = first_non_empty(&[original_title.as_deref()]);
        let overview = non_empty(overview);
        let release_date = non_empty(release_date);
        let release_year = release_year(release_date.as_deref());
        let genres = Some(
            genre_ids
                .into_iter()
                .filter_map(tmdb_genre_name)
                .map(str::to_owned)
                .collect(),
        )
        .filter(|genres: &Vec<String>| !genres.is_empty());
        let alternate_titles = tmdb_alternate_titles(
            &title,
            [original_title.as_deref(), search_title.as_deref()],
            TmdbMovieAlternativeTitles::default(),
        );
        let mut tags = vec!["tmdb".to_owned(), "tmdb_degraded".to_owned()];
        if let Some(vote_average) = vote_average {
            tags.push(format!("tmdb_vote_average:{vote_average:.1}"));
        }
        if let Some(vote_count) = vote_count {
            tags.push(format!("tmdb_vote_count:{vote_count}"));
        }
        let provider_id = tmdb_movie_provider_id(id);
        let mut artwork_candidates = Vec::new();
        if let Some(poster_path) = non_empty(poster_path) {
            artwork_candidates.push(tmdb_artwork_candidate(
                TMDB_PROVIDER_ID,
                &provider_id,
                AddonArtworkKind::Poster,
                tmdb_image_url(&poster_path),
            ));
        }
        if let Some(backdrop_path) = non_empty(backdrop_path) {
            artwork_candidates.push(tmdb_artwork_candidate(
                TMDB_PROVIDER_ID,
                &provider_id,
                AddonArtworkKind::Backdrop,
                tmdb_image_url(&backdrop_path),
            ));
        }

        ProviderMetadataCandidate {
            provider: TMDB_PROVIDER_ID.to_owned(),
            provider_id,
            patch: AddonMetadataPatch {
                title: Some(title.clone()),
                original_title: original_title.clone().filter(|value| value != &title),
                sort_title: Some(title.clone()),
                overview,
                release_date,
                runtime_minutes: None,
                tagline: None,
                genres,
                tags: Some(tags),
                ..AddonMetadataPatch::default()
            },
            facts: ProviderCandidateFacts {
                title: Some(title),
                alternate_titles,
                release_year: release_year.map(i32::from),
                language: Some(query.language.clone()),
                av: None,
                community_score_milli: vote_average
                    .map(|value| (value * 100.0).round().clamp(0.0, 1000.0) as u16),
                community_vote_count: vote_count,
                external_ids: vec![ProviderExternalId {
                    provider: TMDB_PROVIDER_ID.to_owned(),
                    value: id.to_string(),
                }],
                provider_outcomes: vec![ProviderOutcome::TmdbMovieDegraded],
                provider_note: None,
            },
            artwork_candidates,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub(super) struct TmdbTvSearchResult {
    pub(super) id: u64,
    pub(super) name: Option<String>,
    pub(super) original_name: Option<String>,
    pub(super) overview: Option<String>,
    pub(super) first_air_date: Option<String>,
    pub(super) poster_path: Option<String>,
    pub(super) backdrop_path: Option<String>,
    #[serde(default)]
    pub(super) genre_ids: Vec<u64>,
    pub(super) vote_average: Option<f64>,
    pub(super) vote_count: Option<u32>,
}

impl TmdbTvSearchResult {
    pub(super) fn direct_lookup_seed(id: u64) -> Self {
        Self {
            id,
            name: None,
            original_name: None,
            overview: None,
            first_air_date: None,
            poster_path: None,
            backdrop_path: None,
            genre_ids: Vec::new(),
            vote_average: None,
            vote_count: None,
        }
    }

    pub(super) fn into_degraded_candidate(
        self,
        query: &MetadataQuery,
    ) -> ProviderMetadataCandidate {
        let TmdbTvSearchResult {
            id,
            name: search_title,
            original_name,
            overview,
            first_air_date,
            poster_path,
            backdrop_path,
            genre_ids,
            vote_average,
            vote_count,
        } = self;
        let title = first_non_empty(&[search_title.as_deref(), original_name.as_deref()])
            .unwrap_or_else(|| query.title.clone());
        let original_title = first_non_empty(&[original_name.as_deref()]);
        let release_date = non_empty(first_air_date);
        let release_year = release_year(release_date.as_deref());
        let genres = Some(
            genre_ids
                .into_iter()
                .filter_map(tmdb_genre_name)
                .map(str::to_owned)
                .collect(),
        )
        .filter(|genres: &Vec<String>| !genres.is_empty());
        let alternate_titles = tmdb_tv_alternate_titles(
            &title,
            [original_title.as_deref(), search_title.as_deref()],
            TmdbTvAlternativeTitles::default(),
        );
        let mut tags = vec![
            "tmdb".to_owned(),
            "tmdb_tv".to_owned(),
            "tmdb_degraded".to_owned(),
        ];
        if let Some(vote_average) = vote_average {
            tags.push(format!("tmdb_vote_average:{vote_average:.1}"));
        }
        if let Some(vote_count) = vote_count {
            tags.push(format!("tmdb_vote_count:{vote_count}"));
        }
        let provider_id = tmdb_tv_provider_id(id);
        let mut artwork_candidates = Vec::new();
        if let Some(poster_path) = non_empty(poster_path) {
            artwork_candidates.push(tmdb_artwork_candidate(
                TMDB_PROVIDER_ID,
                &provider_id,
                AddonArtworkKind::Poster,
                tmdb_image_url(&poster_path),
            ));
        }
        if let Some(backdrop_path) = non_empty(backdrop_path) {
            artwork_candidates.push(tmdb_artwork_candidate(
                TMDB_PROVIDER_ID,
                &provider_id,
                AddonArtworkKind::Backdrop,
                tmdb_image_url(&backdrop_path),
            ));
        }

        ProviderMetadataCandidate {
            provider: TMDB_PROVIDER_ID.to_owned(),
            provider_id,
            patch: AddonMetadataPatch {
                title: Some(title.clone()),
                original_title: original_title.clone().filter(|value| value != &title),
                sort_title: Some(title.clone()),
                overview: non_empty(overview),
                release_date,
                runtime_minutes: None,
                tagline: Some("TV".to_owned()),
                genres,
                tags: Some(tags),
                ..AddonMetadataPatch::default()
            },
            facts: ProviderCandidateFacts {
                title: Some(title),
                alternate_titles,
                release_year: release_year.map(i32::from),
                language: Some(query.language.clone()),
                av: None,
                community_score_milli: vote_average
                    .map(|value| (value * 100.0).round().clamp(0.0, 1000.0) as u16),
                community_vote_count: vote_count,
                external_ids: vec![ProviderExternalId {
                    provider: TMDB_TV_EXTERNAL_ID_PROVIDER_ID.to_owned(),
                    value: id.to_string(),
                }],
                provider_outcomes: vec![ProviderOutcome::TmdbTvDegraded],
                provider_note: None,
            },
            artwork_candidates,
        }
    }
}

#[derive(Clone, Debug)]
pub(super) struct TmdbMovieCandidate {
    pub(super) search: TmdbMovieSearchResult,
    pub(super) detail: TmdbMovieDetail,
    pub(super) external_ids: TmdbMovieExternalIds,
    pub(super) alternative_titles: TmdbMovieAlternativeTitles,
    pub(super) partial_enrichment: bool,
}

impl TmdbMovieCandidate {
    pub(super) fn into_candidate(self, query: &MetadataQuery) -> ProviderMetadataCandidate {
        let title = first_non_empty(&[
            self.detail.title.as_deref(),
            self.search.title.as_deref(),
            self.detail.original_title.as_deref(),
            self.search.original_title.as_deref(),
        ])
        .unwrap_or_else(|| query.title.clone());
        let original_title = first_non_empty(&[
            self.detail.original_title.as_deref(),
            self.search.original_title.as_deref(),
        ]);
        let overview = non_empty(self.detail.overview).or_else(|| non_empty(self.search.overview));
        let release_date =
            non_empty(self.detail.release_date).or_else(|| non_empty(self.search.release_date));
        let release_year = release_year(release_date.as_deref());
        let genres = detail_genre_names(&self.detail.genres).or_else(|| {
            Some(
                self.search
                    .genre_ids
                    .into_iter()
                    .filter_map(tmdb_genre_name)
                    .map(str::to_owned)
                    .collect(),
            )
            .filter(|genres: &Vec<String>| !genres.is_empty())
        });
        let vote_average = self.detail.vote_average.or(self.search.vote_average);
        let vote_count = self.detail.vote_count.or(self.search.vote_count);
        let external_ids = self.external_ids.into_external_ids(self.detail.id);
        let alternate_titles = tmdb_alternate_titles(
            &title,
            [
                original_title.as_deref(),
                self.detail.title.as_deref(),
                self.search.title.as_deref(),
                self.detail.original_title.as_deref(),
                self.search.original_title.as_deref(),
            ],
            self.alternative_titles,
        );
        let mut tags = vec!["tmdb".to_owned()];
        if let Some(vote_average) = vote_average {
            tags.push(format!("tmdb_vote_average:{vote_average:.1}"));
        }
        if let Some(vote_count) = vote_count {
            tags.push(format!("tmdb_vote_count:{vote_count}"));
        }
        let provider_id = tmdb_movie_provider_id(self.detail.id);
        let mut artwork_candidates = Vec::new();
        if let Some(poster_path) = non_empty(self.detail.poster_path) {
            artwork_candidates.push(tmdb_artwork_candidate(
                TMDB_PROVIDER_ID,
                &provider_id,
                AddonArtworkKind::Poster,
                tmdb_image_url(&poster_path),
            ));
        }
        if let Some(backdrop_path) = non_empty(self.detail.backdrop_path) {
            artwork_candidates.push(tmdb_artwork_candidate(
                TMDB_PROVIDER_ID,
                &provider_id,
                AddonArtworkKind::Backdrop,
                tmdb_image_url(&backdrop_path),
            ));
        }

        ProviderMetadataCandidate {
            provider: TMDB_PROVIDER_ID.to_owned(),
            provider_id,
            patch: AddonMetadataPatch {
                title: Some(title.clone()),
                original_title: original_title.clone().filter(|value| value != &title),
                sort_title: Some(title.clone()),
                overview,
                release_date,
                runtime_minutes: self.detail.runtime,
                tagline: non_empty(self.detail.tagline),
                genres,
                tags: Some(tags).filter(|tags| !tags.is_empty()),
                ..AddonMetadataPatch::default()
            },
            facts: ProviderCandidateFacts {
                title: Some(title),
                alternate_titles,
                release_year: release_year.map(i32::from),
                language: non_empty(self.detail.original_language)
                    .or_else(|| Some(query.language.clone())),
                av: None,
                community_score_milli: vote_average
                    .map(|value| (value * 100.0).round().clamp(0.0, 1000.0) as u16),
                community_vote_count: vote_count,
                external_ids,
                provider_outcomes: vec![if self.partial_enrichment {
                    ProviderOutcome::TmdbMoviePartiallyEnriched
                } else {
                    ProviderOutcome::TmdbMovieEnriched
                }],
                provider_note: None,
            },
            artwork_candidates,
        }
    }
}

#[derive(Clone, Debug)]
pub(super) struct TmdbTvCandidate {
    pub(super) search: TmdbTvSearchResult,
    pub(super) detail: TmdbTvDetail,
    pub(super) external_ids: TmdbTvExternalIds,
    pub(super) alternative_titles: TmdbTvAlternativeTitles,
    pub(super) partial_enrichment: bool,
}

impl TmdbTvCandidate {
    pub(super) fn into_candidate(self, query: &MetadataQuery) -> ProviderMetadataCandidate {
        let title = first_non_empty(&[
            self.detail.name.as_deref(),
            self.search.name.as_deref(),
            self.detail.original_name.as_deref(),
            self.search.original_name.as_deref(),
        ])
        .unwrap_or_else(|| query.title.clone());
        let original_title = first_non_empty(&[
            self.detail.original_name.as_deref(),
            self.search.original_name.as_deref(),
        ]);
        let overview = non_empty(self.detail.overview).or_else(|| non_empty(self.search.overview));
        let release_date =
            non_empty(self.detail.first_air_date).or_else(|| non_empty(self.search.first_air_date));
        let release_year = release_year(release_date.as_deref());
        let runtime_minutes = self
            .detail
            .episode_run_time
            .iter()
            .copied()
            .find(|runtime| *runtime > 0);
        let genres = detail_genre_names(&self.detail.genres).or_else(|| {
            Some(
                self.search
                    .genre_ids
                    .into_iter()
                    .filter_map(tmdb_genre_name)
                    .map(str::to_owned)
                    .collect(),
            )
            .filter(|genres: &Vec<String>| !genres.is_empty())
        });
        let vote_average = self.detail.vote_average.or(self.search.vote_average);
        let vote_count = self.detail.vote_count.or(self.search.vote_count);
        let external_ids = self.external_ids.into_external_ids(self.detail.id);
        let alternate_titles = tmdb_tv_alternate_titles(
            &title,
            [
                original_title.as_deref(),
                self.detail.name.as_deref(),
                self.search.name.as_deref(),
                self.detail.original_name.as_deref(),
                self.search.original_name.as_deref(),
            ],
            self.alternative_titles,
        );
        let mut tags = vec!["tmdb".to_owned(), "tmdb_tv".to_owned()];
        for country in self
            .detail
            .origin_country
            .iter()
            .filter_map(|country| normalize_non_empty(country))
        {
            tags.push(format!("tmdb_origin_country:{country}"));
        }
        for network in named_values(&self.detail.networks) {
            tags.push(format!("tmdb_network:{network}"));
        }
        if let Some(number_of_seasons) = self.detail.number_of_seasons.filter(|value| *value > 0) {
            tags.push(format!("tmdb_number_of_seasons:{number_of_seasons}"));
        }
        if let Some(number_of_episodes) = self.detail.number_of_episodes.filter(|value| *value > 0)
        {
            tags.push(format!("tmdb_number_of_episodes:{number_of_episodes}"));
        }
        if let Some(status) = self
            .detail
            .status
            .as_ref()
            .and_then(|status| normalize_non_empty(status))
        {
            tags.push(format!("tmdb_status:{status}"));
        }
        if let Some(vote_average) = vote_average {
            tags.push(format!("tmdb_vote_average:{vote_average:.1}"));
        }
        if let Some(vote_count) = vote_count {
            tags.push(format!("tmdb_vote_count:{vote_count}"));
        }
        let provider_id = tmdb_tv_provider_id(self.detail.id);
        let mut artwork_candidates = Vec::new();
        if let Some(poster_path) = non_empty(self.detail.poster_path) {
            artwork_candidates.push(tmdb_artwork_candidate(
                TMDB_PROVIDER_ID,
                &provider_id,
                AddonArtworkKind::Poster,
                tmdb_image_url(&poster_path),
            ));
        }
        if let Some(backdrop_path) = non_empty(self.detail.backdrop_path) {
            artwork_candidates.push(tmdb_artwork_candidate(
                TMDB_PROVIDER_ID,
                &provider_id,
                AddonArtworkKind::Backdrop,
                tmdb_image_url(&backdrop_path),
            ));
        }

        ProviderMetadataCandidate {
            provider: TMDB_PROVIDER_ID.to_owned(),
            provider_id,
            patch: AddonMetadataPatch {
                title: Some(title.clone()),
                original_title: original_title.clone().filter(|value| value != &title),
                sort_title: Some(title.clone()),
                overview,
                release_date,
                runtime_minutes,
                tagline: non_empty(self.detail.tagline).or_else(|| Some("TV".to_owned())),
                genres,
                tags: Some(tags).filter(|tags| !tags.is_empty()),
                ..AddonMetadataPatch::default()
            },
            facts: ProviderCandidateFacts {
                title: Some(title),
                alternate_titles,
                release_year: release_year.map(i32::from),
                language: non_empty(self.detail.original_language)
                    .or_else(|| Some(query.language.clone())),
                av: None,
                community_score_milli: vote_average
                    .map(|value| (value * 100.0).round().clamp(0.0, 1000.0) as u16),
                community_vote_count: vote_count,
                external_ids,
                provider_outcomes: vec![if self.partial_enrichment {
                    ProviderOutcome::TmdbTvPartiallyEnriched
                } else {
                    ProviderOutcome::TmdbTvEnriched
                }],
                provider_note: None,
            },
            artwork_candidates,
        }
    }
}

pub(super) fn release_year(value: Option<&str>) -> Option<u16> {
    let value = value?.trim();
    if value
        .as_bytes()
        .get(4)
        .is_some_and(|value| value.is_ascii_digit())
    {
        return None;
    }
    let year = value.get(0..4)?;
    year.parse::<u16>().ok().filter(|year| *year > 0)
}

fn first_non_empty(values: &[Option<&str>]) -> Option<String> {
    values
        .iter()
        .flatten()
        .find_map(|value| normalize_non_empty(value))
}

fn non_empty(value: Option<String>) -> Option<String> {
    value.and_then(|value| normalize_non_empty(&value))
}

fn tmdb_alternate_titles<const N: usize>(
    selected_title: &str,
    known_titles: [Option<&str>; N],
    alternative_titles: TmdbMovieAlternativeTitles,
) -> Vec<String> {
    let mut titles = Vec::new();
    for title in known_titles.into_iter().flatten() {
        push_unique_title(&mut titles, selected_title, title);
    }
    for title in alternative_titles
        .titles
        .into_iter()
        .filter_map(|title| title.title)
    {
        push_unique_title(&mut titles, selected_title, &title);
    }
    titles
}

fn tmdb_tv_alternate_titles<const N: usize>(
    selected_title: &str,
    known_titles: [Option<&str>; N],
    alternative_titles: TmdbTvAlternativeTitles,
) -> Vec<String> {
    let mut titles = Vec::new();
    for title in known_titles.into_iter().flatten() {
        push_unique_title(&mut titles, selected_title, title);
    }
    for title in alternative_titles
        .results
        .into_iter()
        .filter_map(|title| title.title)
    {
        push_unique_title(&mut titles, selected_title, &title);
    }
    titles
}

fn push_unique_title(values: &mut Vec<String>, selected_title: &str, title: &str) {
    let title = title.trim();
    if title.is_empty() || title == selected_title || values.iter().any(|value| value == title) {
        return;
    }
    values.push(title.to_owned());
}

fn tmdb_artwork_candidate(
    provider: &str,
    provider_id: &str,
    kind: AddonArtworkKind,
    source_url: String,
) -> ProviderArtworkCandidate {
    ProviderArtworkCandidate {
        provider: provider.to_owned(),
        provider_id: provider_id.to_owned(),
        facts: ProviderArtworkCandidateFacts {
            kind,
            source_url,
            language: None,
            width: None,
            height: None,
        },
    }
}

fn tmdb_movie_provider_id(movie_id: u64) -> String {
    format!("tmdb:movie:{movie_id}")
}

fn tmdb_tv_provider_id(tv_id: u64) -> String {
    format!("tmdb:tv:{tv_id}")
}

fn tmdb_image_url(path: &str) -> String {
    format!(
        "{}/{}",
        TMDB_IMAGE_BASE_URL.trim_end_matches('/'),
        path.trim_start_matches('/')
    )
}

fn push_external_id(ids: &mut Vec<ProviderExternalId>, provider: &str, value: Option<String>) {
    if let Some(value) = non_empty(value) {
        ids.push(ProviderExternalId {
            provider: provider.to_owned(),
            value,
        });
    }
}

fn normalize_non_empty(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_owned())
}

fn normalized_json_id(value: Option<serde_json::Value>) -> Option<String> {
    match value? {
        serde_json::Value::Number(number) => Some(number.to_string()),
        serde_json::Value::String(value) => normalize_non_empty(&value),
        _ => None,
    }
}

fn detail_genre_names(genres: &[TmdbGenre]) -> Option<Vec<String>> {
    Some(
        genres
            .iter()
            .filter(|genre| genre.id != 0)
            .filter_map(|genre| genre.name.as_ref())
            .filter_map(|name| normalize_non_empty(name))
            .collect(),
    )
    .filter(|genres: &Vec<String>| !genres.is_empty())
}

fn named_values(values: &[super::parser::TmdbNamedValue]) -> Vec<String> {
    values
        .iter()
        .filter(|value| value.id != 0)
        .filter_map(|value| value.name.as_ref())
        .filter_map(|name| normalize_non_empty(name))
        .collect()
}

fn tmdb_genre_name(id: u64) -> Option<&'static str> {
    match id {
        12 => Some("Adventure"),
        16 => Some("Animation"),
        18 => Some("Drama"),
        28 => Some("Action"),
        35 => Some("Comedy"),
        878 => Some("Science Fiction"),
        _ => None,
    }
}

impl TmdbMovieExternalIds {
    pub(super) fn into_external_ids(self, tmdb_id: u64) -> Vec<ProviderExternalId> {
        let mut ids = vec![ProviderExternalId {
            provider: TMDB_PROVIDER_ID.to_owned(),
            value: tmdb_id.to_string(),
        }];
        push_external_id(&mut ids, "imdb", self.imdb_id);
        push_external_id(&mut ids, "wikidata", self.wikidata_id);
        push_external_id(&mut ids, "facebook", self.facebook_id);
        push_external_id(&mut ids, "instagram", self.instagram_id);
        push_external_id(&mut ids, "twitter", self.twitter_id);
        ids
    }
}

impl TmdbTvExternalIds {
    pub(super) fn into_external_ids(self, tv_id: u64) -> Vec<ProviderExternalId> {
        let mut ids = vec![ProviderExternalId {
            provider: TMDB_TV_EXTERNAL_ID_PROVIDER_ID.to_owned(),
            value: tv_id.to_string(),
        }];
        push_external_id(&mut ids, "imdb", self.imdb_id);
        push_external_id(&mut ids, "tvdb", normalized_json_id(self.tvdb_id));
        push_external_id(&mut ids, "wikidata", self.wikidata_id);
        push_external_id(&mut ids, "facebook", self.facebook_id);
        push_external_id(&mut ids, "instagram", self.instagram_id);
        push_external_id(&mut ids, "twitter", self.twitter_id);
        ids
    }
}
