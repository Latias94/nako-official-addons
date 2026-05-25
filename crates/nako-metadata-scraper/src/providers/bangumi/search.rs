use std::collections::HashSet;

use crate::engine::MetadataQuery;

use super::BANGUMI_PROVIDER_ID;

pub(super) const BANGUMI_DETAIL_ENRICHMENT_LIMIT: usize = 3;

pub(super) fn bangumi_query_subject_ids(query: &MetadataQuery) -> impl Iterator<Item = u64> + '_ {
    let mut seen = HashSet::new();
    query
        .external_ids
        .iter()
        .filter(|external_id| {
            external_id
                .provider
                .eq_ignore_ascii_case(BANGUMI_PROVIDER_ID)
        })
        .filter_map(|external_id| external_id.value.trim().parse().ok())
        .filter(|subject_id| *subject_id > 0)
        .filter(move |subject_id| seen.insert(*subject_id))
}

pub(super) fn bangumi_air_date_filter(year: Option<i32>) -> Option<[String; 2]> {
    let year = year?;
    if !(1..=9999).contains(&year) {
        return None;
    }
    Some([
        format!(">={year:04}-01-01"),
        format!("<{:04}-01-01", year.saturating_add(1)),
    ])
}
