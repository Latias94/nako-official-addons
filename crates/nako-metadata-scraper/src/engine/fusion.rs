use nako_addon_protocol::{AddonArtworkKind, AddonMetadataPatch};

use super::{
    AvMetadataFacts, ProviderArtworkCandidate, ProviderFieldPolicy, ProviderMetadataCandidate,
    ranking::CandidateFieldSource, resolver::ResolvedProviderFact,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FusedProviderCandidate {
    pub(crate) candidate: ProviderMetadataCandidate,
    pub(crate) field_sources: Vec<CandidateFieldSource>,
}

pub(crate) fn fuse_provider_facts(
    facts: &[ResolvedProviderFact],
    base_index: usize,
    provider_field_policy: &ProviderFieldPolicy,
) -> FusedProviderCandidate {
    let mut candidate = facts[base_index].candidate.clone();
    let mut field_sources = apply_provider_field_policy(
        &mut candidate.patch,
        facts,
        base_index,
        provider_field_policy,
    );
    field_sources.extend(apply_provider_av_field_policy(
        &mut candidate.facts.av,
        facts,
        base_index,
        provider_field_policy,
    ));
    apply_provider_artwork_policy(
        &mut candidate.artwork_candidates,
        facts,
        base_index,
        provider_field_policy,
    );

    FusedProviderCandidate {
        candidate,
        field_sources,
    }
}

fn apply_provider_field_policy(
    patch: &mut AddonMetadataPatch,
    facts: &[ResolvedProviderFact],
    base_index: usize,
    provider_field_policy: &ProviderFieldPolicy,
) -> Vec<CandidateFieldSource> {
    let mut field_sources = Vec::new();

    let (title, source) =
        select_string_field(facts, base_index, provider_field_policy, "title", |patch| {
            patch.title.as_deref()
        });
    patch.title = title;
    push_selected_field_source(&mut field_sources, "title", facts, source);

    let (original_title, source) = select_string_field(
        facts,
        base_index,
        provider_field_policy,
        "original_title",
        |patch| patch.original_title.as_deref(),
    );
    patch.original_title = original_title;
    push_selected_field_source(&mut field_sources, "original_title", facts, source);

    let (sort_title, source) = select_string_field(
        facts,
        base_index,
        provider_field_policy,
        "sort_title",
        |patch| patch.sort_title.as_deref(),
    );
    patch.sort_title = sort_title;
    push_selected_field_source(&mut field_sources, "sort_title", facts, source);

    let (overview, source) = select_string_field(
        facts,
        base_index,
        provider_field_policy,
        "overview",
        |patch| patch.overview.as_deref(),
    );
    patch.overview = overview;
    push_selected_field_source(&mut field_sources, "overview", facts, source);

    let (release_date, source) = select_string_field(
        facts,
        base_index,
        provider_field_policy,
        "release_date",
        |patch| patch.release_date.as_deref(),
    );
    patch.release_date = release_date;
    push_selected_field_source(&mut field_sources, "release_date", facts, source);

    let (runtime_minutes, source) = select_copy_field(
        facts,
        base_index,
        provider_field_policy,
        "runtime_minutes",
        |patch| patch.runtime_minutes,
    );
    patch.runtime_minutes = runtime_minutes;
    push_selected_field_source(&mut field_sources, "runtime_minutes", facts, source);

    let (tagline, source) = select_string_field(
        facts,
        base_index,
        provider_field_policy,
        "tagline",
        |patch| patch.tagline.as_deref(),
    );
    patch.tagline = tagline;
    push_selected_field_source(&mut field_sources, "tagline", facts, source);

    let (genres, source) = select_vec_field(
        facts,
        base_index,
        provider_field_policy,
        "genres",
        |patch| patch.genres.as_ref(),
    );
    patch.genres = genres;
    push_selected_field_source(&mut field_sources, "genres", facts, source);

    let (tags, source) =
        select_vec_field(facts, base_index, provider_field_policy, "tags", |patch| {
            patch.tags.as_ref()
        });
    patch.tags = tags;
    push_selected_field_source(&mut field_sources, "tags", facts, source);

    field_sources
}

fn apply_provider_av_field_policy(
    av: &mut Option<AvMetadataFacts>,
    facts: &[ResolvedProviderFact],
    base_index: usize,
    provider_field_policy: &ProviderFieldPolicy,
) -> Vec<CandidateFieldSource> {
    let mut selected = AvMetadataFacts::default();
    let mut field_sources = Vec::new();

    let (actors, source) =
        select_av_vec_field(facts, base_index, provider_field_policy, "actors", |av| {
            &av.actors
        });
    selected.actors = actors;
    push_selected_field_source(&mut field_sources, "actors", facts, source);

    let (all_actors, source) = select_av_vec_field(
        facts,
        base_index,
        provider_field_policy,
        "all_actors",
        |av| &av.all_actors,
    );
    selected.all_actors = all_actors;
    push_selected_field_source(&mut field_sources, "all_actors", facts, source);

    let (directors, source) = select_av_vec_field(
        facts,
        base_index,
        provider_field_policy,
        "directors",
        |av| &av.directors,
    );
    selected.directors = directors;
    push_selected_field_source(&mut field_sources, "directors", facts, source);

    let (series, source) =
        select_av_string_field(facts, base_index, provider_field_policy, "series", |av| {
            av.series.as_deref()
        });
    selected.series = series;
    push_selected_field_source(&mut field_sources, "series", facts, source);

    let (studio, source) =
        select_av_string_field(facts, base_index, provider_field_policy, "studio", |av| {
            av.studio.as_deref()
        });
    selected.studio = studio;
    push_selected_field_source(&mut field_sources, "studio", facts, source);

    let (publisher, source) = select_av_string_field(
        facts,
        base_index,
        provider_field_policy,
        "publisher",
        |av| av.publisher.as_deref(),
    );
    selected.publisher = publisher;
    push_selected_field_source(&mut field_sources, "publisher", facts, source);

    let (maker, source) =
        select_av_string_field(facts, base_index, provider_field_policy, "maker", |av| {
            av.maker.as_deref()
        });
    selected.maker = maker;
    push_selected_field_source(&mut field_sources, "maker", facts, source);

    let (label, source) =
        select_av_string_field(facts, base_index, provider_field_policy, "label", |av| {
            av.label.as_deref()
        });
    selected.label = label;
    push_selected_field_source(&mut field_sources, "label", facts, source);

    let (wanted_count, source) = select_av_copy_field(
        facts,
        base_index,
        provider_field_policy,
        "wanted_count",
        |av| av.wanted_count,
    );
    selected.wanted_count = wanted_count;
    push_selected_field_source(&mut field_sources, "wanted_count", facts, source);

    let (thumb_url, source) = select_av_string_field(
        facts,
        base_index,
        provider_field_policy,
        "thumb_url",
        |av| av.thumb_url.as_deref(),
    );
    selected.thumb_url = thumb_url;
    push_selected_field_source(&mut field_sources, "thumb_url", facts, source);

    let (trailer_url, source) = select_av_string_field(
        facts,
        base_index,
        provider_field_policy,
        "trailer_url",
        |av| av.trailer_url.as_deref(),
    );
    selected.trailer_url = trailer_url;
    push_selected_field_source(&mut field_sources, "trailer_url", facts, source);

    let (extrafanart_urls, source) = select_av_vec_field(
        facts,
        base_index,
        provider_field_policy,
        "extrafanart_urls",
        |av| &av.extrafanart_urls,
    );
    selected.extrafanart_urls = extrafanart_urls;
    push_selected_field_source(&mut field_sources, "extrafanart_urls", facts, source);

    *av = selected.non_empty();
    field_sources
}

fn select_string_field(
    facts: &[ResolvedProviderFact],
    base_index: usize,
    provider_field_policy: &ProviderFieldPolicy,
    field: &'static str,
    get_value: impl Fn(&AddonMetadataPatch) -> Option<&str>,
) -> (Option<String>, Option<usize>) {
    let Some(index) = select_fact_index(facts, base_index, provider_field_policy, field, |patch| {
        get_value(patch).is_some_and(|value| !value.trim().is_empty())
    }) else {
        return (None, None);
    };

    (
        get_value(&facts[index].candidate.patch).map(str::to_owned),
        Some(index),
    )
}

fn select_vec_field(
    facts: &[ResolvedProviderFact],
    base_index: usize,
    provider_field_policy: &ProviderFieldPolicy,
    field: &'static str,
    get_value: impl Fn(&AddonMetadataPatch) -> Option<&Vec<String>>,
) -> (Option<Vec<String>>, Option<usize>) {
    let Some(index) = select_fact_index(facts, base_index, provider_field_policy, field, |patch| {
        get_value(patch).is_some_and(|values| !values.is_empty())
    }) else {
        return (None, None);
    };

    (
        get_value(&facts[index].candidate.patch).cloned(),
        Some(index),
    )
}

fn select_copy_field<T: Copy>(
    facts: &[ResolvedProviderFact],
    base_index: usize,
    provider_field_policy: &ProviderFieldPolicy,
    field: &'static str,
    get_value: impl Fn(&AddonMetadataPatch) -> Option<T>,
) -> (Option<T>, Option<usize>) {
    let Some(index) = select_fact_index(facts, base_index, provider_field_policy, field, |patch| {
        get_value(patch).is_some()
    }) else {
        return (None, None);
    };

    (get_value(&facts[index].candidate.patch), Some(index))
}

fn select_av_string_field(
    facts: &[ResolvedProviderFact],
    base_index: usize,
    provider_field_policy: &ProviderFieldPolicy,
    field: &'static str,
    get_value: impl Fn(&AvMetadataFacts) -> Option<&str>,
) -> (Option<String>, Option<usize>) {
    let Some(index) = select_av_fact_index(facts, base_index, provider_field_policy, field, |av| {
        get_value(av).is_some_and(|value| !value.trim().is_empty())
    }) else {
        return (None, None);
    };

    (
        facts[index]
            .candidate
            .facts
            .av
            .as_ref()
            .and_then(&get_value)
            .map(str::to_owned),
        Some(index),
    )
}

fn select_av_vec_field(
    facts: &[ResolvedProviderFact],
    base_index: usize,
    provider_field_policy: &ProviderFieldPolicy,
    field: &'static str,
    get_value: impl Fn(&AvMetadataFacts) -> &Vec<String>,
) -> (Vec<String>, Option<usize>) {
    let Some(index) = select_av_fact_index(facts, base_index, provider_field_policy, field, |av| {
        !get_value(av).is_empty()
    }) else {
        return (Vec::new(), None);
    };

    (
        facts[index]
            .candidate
            .facts
            .av
            .as_ref()
            .map(&get_value)
            .cloned()
            .unwrap_or_default(),
        Some(index),
    )
}

fn select_av_copy_field<T: Copy>(
    facts: &[ResolvedProviderFact],
    base_index: usize,
    provider_field_policy: &ProviderFieldPolicy,
    field: &'static str,
    get_value: impl Fn(&AvMetadataFacts) -> Option<T>,
) -> (Option<T>, Option<usize>) {
    let Some(index) = select_av_fact_index(facts, base_index, provider_field_policy, field, |av| {
        get_value(av).is_some()
    }) else {
        return (None, None);
    };

    (
        facts[index].candidate.facts.av.as_ref().and_then(get_value),
        Some(index),
    )
}

fn select_fact_index(
    facts: &[ResolvedProviderFact],
    base_index: usize,
    provider_field_policy: &ProviderFieldPolicy,
    field: &str,
    has_value: impl Fn(&AddonMetadataPatch) -> bool,
) -> Option<usize> {
    let preferences = provider_field_policy.providers_for(field);
    for provider in preferences {
        if let Some(index) = facts
            .iter()
            .position(|fact| fact.source.provider == *provider && has_value(&fact.candidate.patch))
        {
            return Some(index);
        }
    }

    if has_value(&facts[base_index].candidate.patch) {
        return Some(base_index);
    }

    if preferences.is_empty() {
        return None;
    }

    facts
        .iter()
        .position(|fact| has_value(&fact.candidate.patch))
}

fn select_av_fact_index(
    facts: &[ResolvedProviderFact],
    base_index: usize,
    provider_field_policy: &ProviderFieldPolicy,
    field: &str,
    has_value: impl Fn(&AvMetadataFacts) -> bool,
) -> Option<usize> {
    let preferences = provider_field_policy.providers_for(field);
    for provider in preferences {
        if let Some(index) = facts.iter().position(|fact| {
            fact.source.provider == *provider
                && fact.candidate.facts.av.as_ref().is_some_and(&has_value)
        }) {
            return Some(index);
        }
    }

    if facts[base_index]
        .candidate
        .facts
        .av
        .as_ref()
        .is_some_and(&has_value)
    {
        return Some(base_index);
    }

    if preferences.is_empty() {
        return None;
    }

    facts
        .iter()
        .position(|fact| fact.candidate.facts.av.as_ref().is_some_and(&has_value))
}

fn apply_provider_artwork_policy(
    artwork_candidates: &mut Vec<ProviderArtworkCandidate>,
    facts: &[ResolvedProviderFact],
    base_index: usize,
    provider_field_policy: &ProviderFieldPolicy,
) {
    let mut selected = Vec::new();
    push_selected_artwork_kind(
        &mut selected,
        facts,
        base_index,
        provider_field_policy,
        "poster",
        AddonArtworkKind::Poster,
    );
    push_selected_artwork_kind(
        &mut selected,
        facts,
        base_index,
        provider_field_policy,
        "backdrop",
        AddonArtworkKind::Backdrop,
    );

    if selected.is_empty() {
        return;
    }

    selected.extend(
        artwork_candidates
            .iter()
            .filter(|candidate| {
                candidate.facts.kind != AddonArtworkKind::Poster
                    && candidate.facts.kind != AddonArtworkKind::Backdrop
            })
            .cloned(),
    );
    *artwork_candidates = selected;
}

fn push_selected_artwork_kind(
    selected: &mut Vec<ProviderArtworkCandidate>,
    facts: &[ResolvedProviderFact],
    base_index: usize,
    provider_field_policy: &ProviderFieldPolicy,
    field: &str,
    kind: AddonArtworkKind,
) {
    let Some(index) =
        select_artwork_fact_index(facts, base_index, provider_field_policy, field, kind)
    else {
        return;
    };
    selected.extend(
        facts[index]
            .candidate
            .artwork_candidates
            .iter()
            .filter(|candidate| candidate.facts.kind == kind)
            .cloned(),
    );
}

fn select_artwork_fact_index(
    facts: &[ResolvedProviderFact],
    base_index: usize,
    provider_field_policy: &ProviderFieldPolicy,
    field: &str,
    kind: AddonArtworkKind,
) -> Option<usize> {
    let mut preferences = provider_field_policy
        .providers_for(field)
        .iter()
        .collect::<Vec<_>>();
    for provider in provider_field_policy.providers_for("artwork") {
        if !preferences.contains(&provider) {
            preferences.push(provider);
        }
    }

    for provider in &preferences {
        if let Some(index) = facts.iter().position(|fact| {
            fact.source.provider.eq_ignore_ascii_case(provider)
                && fact
                    .candidate
                    .artwork_candidates
                    .iter()
                    .any(|candidate| candidate.facts.kind == kind)
        }) {
            return Some(index);
        }
    }

    if facts[base_index]
        .candidate
        .artwork_candidates
        .iter()
        .any(|candidate| candidate.facts.kind == kind)
    {
        return Some(base_index);
    }

    if preferences.is_empty() {
        return None;
    }

    facts.iter().position(|fact| {
        fact.candidate
            .artwork_candidates
            .iter()
            .any(|candidate| candidate.facts.kind == kind)
    })
}

fn push_selected_field_source(
    field_sources: &mut Vec<CandidateFieldSource>,
    field: &'static str,
    facts: &[ResolvedProviderFact],
    source_index: Option<usize>,
) {
    let Some(source_index) = source_index else {
        return;
    };
    let source = &facts[source_index].source;
    field_sources.push(CandidateFieldSource {
        field,
        provider: source.provider.clone(),
        provider_id: source.provider_id.clone(),
    });
}
