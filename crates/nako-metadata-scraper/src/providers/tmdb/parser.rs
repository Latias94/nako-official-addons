use serde::Deserialize;

use super::TMDB_PROVIDER_ID;
use super::mapper::{TmdbMovieSearchResult, TmdbTvSearchResult};

#[derive(Clone, Debug, Deserialize)]
pub(super) struct TmdbSearchResponse {
    #[serde(default)]
    pub(super) results: Vec<TmdbMovieSearchResult>,
}

impl TmdbSearchResponse {
    pub(super) fn from_value(value: serde_json::Value) -> anyhow::Result<Self> {
        let items = value
            .get("results")
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "failed to parse TMDB search movie response: missing field `results`"
                )
            })?
            .as_array()
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "failed to parse TMDB search movie response: `results` must be an array"
                )
            })?;
        let mut skipped_count = 0usize;
        let results = items
            .iter()
            .filter_map(|item| match serde_json::from_value::<TmdbMovieSearchResult>(item.clone()) {
                Ok(result) if result.id > 0 => Some(result),
                Ok(_) => {
                    skipped_count += 1;
                    tracing::warn!(
                        provider = TMDB_PROVIDER_ID,
                        "skipping TMDB search result item with zero id"
                    );
                    None
                }
                Err(error) => {
                    skipped_count += 1;
                    tracing::warn!(provider = TMDB_PROVIDER_ID, %error, "skipping malformed TMDB search result item");
                    None
                }
            })
            .collect();
        if !items.is_empty() && skipped_count == items.len() {
            anyhow::bail!("all TMDB search result items were malformed");
        }
        Ok(Self { results })
    }
}

#[derive(Clone, Debug, Deserialize)]
pub(super) struct TmdbTvSearchResponse {
    #[serde(default)]
    pub(super) results: Vec<TmdbTvSearchResult>,
}

impl TmdbTvSearchResponse {
    pub(super) fn from_value(value: serde_json::Value) -> anyhow::Result<Self> {
        let items = value
            .get("results")
            .ok_or_else(|| {
                anyhow::anyhow!("failed to parse TMDB search TV response: missing field `results`")
            })?
            .as_array()
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "failed to parse TMDB search TV response: `results` must be an array"
                )
            })?;
        let mut skipped_count = 0usize;
        let results = items
            .iter()
            .filter_map(|item| match serde_json::from_value::<TmdbTvSearchResult>(item.clone()) {
                Ok(result) if result.id > 0 => Some(result),
                Ok(_) => {
                    skipped_count += 1;
                    tracing::warn!(
                        provider = TMDB_PROVIDER_ID,
                        "skipping TMDB search TV result item with zero id"
                    );
                    None
                }
                Err(error) => {
                    skipped_count += 1;
                    tracing::warn!(provider = TMDB_PROVIDER_ID, %error, "skipping malformed TMDB search TV result item");
                    None
                }
            })
            .collect();
        if !items.is_empty() && skipped_count == items.len() {
            anyhow::bail!("all TMDB search TV result items were malformed");
        }
        Ok(Self { results })
    }
}

#[derive(Clone, Debug, Deserialize)]
pub(super) struct TmdbFindResponse {
    #[serde(default)]
    pub(super) movie_results: Vec<TmdbFindMovieResult>,
    #[serde(default)]
    pub(super) tv_results: Vec<TmdbFindTvResult>,
}

impl TmdbFindResponse {
    pub(super) fn from_value(value: serde_json::Value) -> anyhow::Result<Self> {
        serde_json::from_value(value)
            .map_err(|error| anyhow::anyhow!("failed to parse TMDB find response: {error}"))
    }

    pub(super) fn first_movie_id(&self) -> Option<u64> {
        self.movie_results
            .iter()
            .map(|movie| movie.id)
            .find(|movie_id| *movie_id > 0)
    }

    pub(super) fn first_tv_id(&self) -> Option<u64> {
        self.tv_results
            .iter()
            .map(|tv| tv.id)
            .find(|tv_id| *tv_id > 0)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub(super) struct TmdbFindMovieResult {
    pub(super) id: u64,
}

#[derive(Clone, Debug, Deserialize)]
pub(super) struct TmdbFindTvResult {
    pub(super) id: u64,
}

#[derive(Clone, Debug, Deserialize)]
pub(super) struct TmdbMovieDetail {
    pub(super) id: u64,
    pub(super) title: Option<String>,
    pub(super) original_title: Option<String>,
    pub(super) overview: Option<String>,
    pub(super) release_date: Option<String>,
    pub(super) runtime: Option<u32>,
    pub(super) tagline: Option<String>,
    pub(super) original_language: Option<String>,
    pub(super) poster_path: Option<String>,
    pub(super) backdrop_path: Option<String>,
    #[serde(default)]
    pub(super) genres: Vec<TmdbGenre>,
    pub(super) vote_average: Option<f64>,
    pub(super) vote_count: Option<u32>,
}

impl TmdbMovieDetail {
    pub(super) fn from_value(value: serde_json::Value) -> anyhow::Result<Self> {
        serde_json::from_value(value)
            .map_err(|error| anyhow::anyhow!("failed to parse TMDB movie detail response: {error}"))
    }
}

#[derive(Clone, Debug, Deserialize)]
pub(super) struct TmdbTvDetail {
    pub(super) id: u64,
    pub(super) name: Option<String>,
    pub(super) original_name: Option<String>,
    pub(super) overview: Option<String>,
    pub(super) first_air_date: Option<String>,
    #[serde(default)]
    pub(super) episode_run_time: Vec<u32>,
    pub(super) tagline: Option<String>,
    pub(super) original_language: Option<String>,
    pub(super) poster_path: Option<String>,
    pub(super) backdrop_path: Option<String>,
    #[serde(default)]
    pub(super) genres: Vec<TmdbGenre>,
    #[serde(default)]
    pub(super) networks: Vec<TmdbNamedValue>,
    #[serde(default)]
    pub(super) origin_country: Vec<String>,
    pub(super) number_of_episodes: Option<u32>,
    pub(super) number_of_seasons: Option<u32>,
    pub(super) status: Option<String>,
    pub(super) vote_average: Option<f64>,
    pub(super) vote_count: Option<u32>,
}

impl TmdbTvDetail {
    pub(super) fn from_value(value: serde_json::Value) -> anyhow::Result<Self> {
        serde_json::from_value(value)
            .map_err(|error| anyhow::anyhow!("failed to parse TMDB TV detail response: {error}"))
    }
}

#[derive(Clone, Debug, Deserialize)]
pub(super) struct TmdbGenre {
    pub(super) id: u64,
    pub(super) name: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub(super) struct TmdbNamedValue {
    pub(super) id: u64,
    pub(super) name: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub(super) struct TmdbMovieExternalIds {
    pub(super) imdb_id: Option<String>,
    pub(super) wikidata_id: Option<String>,
    pub(super) facebook_id: Option<String>,
    pub(super) instagram_id: Option<String>,
    pub(super) twitter_id: Option<String>,
}

impl TmdbMovieExternalIds {
    pub(super) fn from_value(value: serde_json::Value) -> anyhow::Result<Self> {
        serde_json::from_value(value).map_err(|error| {
            anyhow::anyhow!("failed to parse TMDB movie external IDs response: {error}")
        })
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
pub(super) struct TmdbTvExternalIds {
    pub(super) imdb_id: Option<String>,
    pub(super) tvdb_id: Option<serde_json::Value>,
    pub(super) wikidata_id: Option<String>,
    pub(super) facebook_id: Option<String>,
    pub(super) instagram_id: Option<String>,
    pub(super) twitter_id: Option<String>,
}

impl TmdbTvExternalIds {
    pub(super) fn from_value(value: serde_json::Value) -> anyhow::Result<Self> {
        serde_json::from_value(value).map_err(|error| {
            anyhow::anyhow!("failed to parse TMDB TV external IDs response: {error}")
        })
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
pub(super) struct TmdbMovieAlternativeTitles {
    #[serde(default)]
    pub(super) titles: Vec<TmdbAlternativeTitle>,
}

impl TmdbMovieAlternativeTitles {
    pub(super) fn from_value(value: serde_json::Value) -> anyhow::Result<Self> {
        serde_json::from_value(value).map_err(|error| {
            anyhow::anyhow!("failed to parse TMDB movie alternative titles response: {error}")
        })
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
pub(super) struct TmdbTvAlternativeTitles {
    #[serde(default)]
    pub(super) results: Vec<TmdbAlternativeTitle>,
}

impl TmdbTvAlternativeTitles {
    pub(super) fn from_value(value: serde_json::Value) -> anyhow::Result<Self> {
        serde_json::from_value(value).map_err(|error| {
            anyhow::anyhow!("failed to parse TMDB TV alternative titles response: {error}")
        })
    }
}

#[derive(Clone, Debug, Deserialize)]
pub(super) struct TmdbAlternativeTitle {
    pub(super) title: Option<String>,
}
