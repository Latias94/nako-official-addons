use std::collections::HashSet;

use crate::engine::MetadataQuery;

pub(super) fn tmdb_query_movie_ids(query: &MetadataQuery) -> impl Iterator<Item = u64> + '_ {
    let mut seen = HashSet::new();
    query
        .external_ids
        .iter()
        .filter(|external_id| external_id.provider.eq_ignore_ascii_case("tmdb"))
        .filter_map(|external_id| external_id.value.trim().parse().ok())
        .filter(|movie_id| *movie_id > 0)
        .filter(move |movie_id| seen.insert(*movie_id))
}

pub(super) fn tmdb_query_imdb_ids(query: &MetadataQuery) -> impl Iterator<Item = String> + '_ {
    let mut seen = HashSet::new();
    query
        .external_ids
        .iter()
        .filter(|external_id| external_id.provider.eq_ignore_ascii_case("imdb"))
        .filter_map(|external_id| normalized_imdb_id(&external_id.value))
        .filter(move |imdb_id| seen.insert(imdb_id.clone()))
}

fn normalized_imdb_id(value: &str) -> Option<String> {
    let value = value.trim();
    if value.len() > 2
        && value[..2].eq_ignore_ascii_case("tt")
        && value[2..]
            .chars()
            .all(|character| character.is_ascii_digit())
    {
        Some(format!("tt{}", &value[2..]))
    } else {
        None
    }
}
