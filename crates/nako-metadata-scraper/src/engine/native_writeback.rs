use nako_addon_protocol::{
    AddonMetadataCollection, AddonMetadataCredit, AddonMetadataExternalId, AddonMetadataImage,
    AddonMetadataPatch, AddonMetadataStudio,
};

use super::{AvMetadataFacts, ProviderArtworkCandidate, ProviderExternalId};

pub(crate) fn materialize_native_metadata_patch(
    patch: &mut AddonMetadataPatch,
    provider: &str,
    av: Option<&AvMetadataFacts>,
    external_ids: &[ProviderExternalId],
    artwork_candidates: &[ProviderArtworkCandidate],
) {
    merge_external_ids(patch, external_ids);
    merge_images(patch, provider, av, artwork_candidates);

    let Some(av) = av else {
        return;
    };

    merge_credits(patch, av);
    merge_studios(patch, av);
    merge_collections(patch, av);
}

fn merge_external_ids(patch: &mut AddonMetadataPatch, external_ids: &[ProviderExternalId]) {
    let mut merged = patch.external_ids.take().unwrap_or_default();
    for external_id in external_ids {
        push_external_id(&mut merged, &external_id.provider, &external_id.value);
    }
    patch.external_ids = (!merged.is_empty()).then_some(merged);
}

fn merge_images(
    patch: &mut AddonMetadataPatch,
    provider: &str,
    av: Option<&AvMetadataFacts>,
    artwork_candidates: &[ProviderArtworkCandidate],
) {
    let mut merged = patch.images.take().unwrap_or_default();
    for candidate in artwork_candidates {
        push_image(
            &mut merged,
            candidate.facts.kind.as_str(),
            &candidate.facts.source_url,
            &candidate.provider,
            candidate.facts.width,
            candidate.facts.height,
            candidate.facts.language.clone(),
        );
    }

    if let Some(av) = av {
        if let Some(url) = av.thumb_url.as_deref() {
            push_image(&mut merged, "thumbnail", url, provider, None, None, None);
        }
        for url in &av.extrafanart_urls {
            push_image(&mut merged, "backdrop", url, provider, None, None, None);
        }
        if let Some(url) = av.trailer_url.as_deref() {
            push_external_id(
                patch.external_ids.get_or_insert_with(Vec::new),
                "trailer_url",
                url,
            );
        }
    }

    patch.images = (!merged.is_empty()).then_some(merged);
}

fn merge_credits(patch: &mut AddonMetadataPatch, av: &AvMetadataFacts) {
    let mut merged = patch.credits.take().unwrap_or_default();
    let actor_names = if av.all_actors.is_empty() {
        &av.actors
    } else {
        &av.all_actors
    };

    for (index, actor) in actor_names.iter().enumerate() {
        push_credit(&mut merged, actor, "actor", Some((index + 1) as u32));
    }
    for director in &av.directors {
        push_credit(&mut merged, director, "director", None);
    }

    patch.credits = (!merged.is_empty()).then_some(merged);
}

fn merge_studios(patch: &mut AddonMetadataPatch, av: &AvMetadataFacts) {
    let mut merged = patch.studios.take().unwrap_or_default();
    for name in [
        av.studio.as_deref(),
        av.maker.as_deref(),
        av.publisher.as_deref(),
        av.label.as_deref(),
    ]
    .into_iter()
    .flatten()
    {
        push_studio(&mut merged, name);
    }
    patch.studios = (!merged.is_empty()).then_some(merged);
}

fn merge_collections(patch: &mut AddonMetadataPatch, av: &AvMetadataFacts) {
    let Some(series) = av.series.as_deref() else {
        return;
    };

    let mut merged = patch.collections.take().unwrap_or_default();
    if normalized(series).is_some()
        && !merged
            .iter()
            .any(|collection| same_label(&collection.name, series))
    {
        merged.push(AddonMetadataCollection {
            name: series.trim().to_owned(),
            overview: None,
            sort_order: None,
            external_ids: Vec::new(),
        });
    }
    patch.collections = (!merged.is_empty()).then_some(merged);
}

fn push_external_id(values: &mut Vec<AddonMetadataExternalId>, provider: &str, value: &str) {
    let Some(provider) = normalized(provider) else {
        return;
    };
    let Some(value) = normalized(value) else {
        return;
    };
    if values.iter().any(|existing| {
        same_label(&existing.provider, &provider) && same_label(&existing.value, &value)
    }) {
        return;
    }
    values.push(AddonMetadataExternalId { provider, value });
}

fn push_image(
    values: &mut Vec<AddonMetadataImage>,
    kind: &str,
    uri: &str,
    provider: &str,
    width: Option<u32>,
    height: Option<u32>,
    language: Option<String>,
) {
    let Some(kind) = normalized(kind) else {
        return;
    };
    let Some(uri) = normalized(uri) else {
        return;
    };
    let Some(provider) = normalized(provider) else {
        return;
    };
    if values
        .iter()
        .any(|existing| same_label(&existing.kind, &kind) && same_label(&existing.uri, &uri))
    {
        return;
    }
    values.push(AddonMetadataImage {
        kind,
        uri,
        provider,
        width,
        height,
        language: language.and_then(|value| normalized(&value)),
    });
}

fn push_credit(values: &mut Vec<AddonMetadataCredit>, name: &str, role: &str, order: Option<u32>) {
    let Some(name) = normalized(name) else {
        return;
    };
    if values
        .iter()
        .any(|existing| same_label(&existing.name, &name) && same_label(&existing.role, role))
    {
        return;
    }
    values.push(AddonMetadataCredit {
        name,
        role: role.to_owned(),
        character: None,
        order,
        external_ids: Vec::new(),
    });
}

fn push_studio(values: &mut Vec<AddonMetadataStudio>, name: &str) {
    let Some(name) = normalized(name) else {
        return;
    };
    if values
        .iter()
        .any(|existing| same_label(&existing.name, &name))
    {
        return;
    }
    values.push(AddonMetadataStudio {
        name,
        external_ids: Vec::new(),
    });
}

fn normalized(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_owned())
}

fn same_label(left: &str, right: &str) -> bool {
    left.trim().eq_ignore_ascii_case(right.trim())
}

#[cfg(test)]
mod tests {
    use nako_addon_protocol::{AddonArtworkKind, AddonMetadataPatch};

    use crate::engine::{
        AvMetadataFacts, ProviderArtworkCandidate, ProviderArtworkCandidateFacts,
        ProviderExternalId,
    };

    use super::materialize_native_metadata_patch;

    #[test]
    fn av_facts_materialize_into_native_metadata_patch() {
        let av = AvMetadataFacts {
            actors: vec!["Displayed Actor".to_owned()],
            all_actors: vec!["Alice Actor".to_owned(), "Bob Actor".to_owned()],
            directors: vec!["Diana Director".to_owned()],
            series: Some("Series One".to_owned()),
            studio: Some("Studio One".to_owned()),
            maker: Some("Studio One".to_owned()),
            publisher: Some("Publisher One".to_owned()),
            label: Some("Label One".to_owned()),
            thumb_url: Some("https://img.example/thumb.jpg".to_owned()),
            trailer_url: Some("https://video.example/trailer.mp4".to_owned()),
            extrafanart_urls: vec!["https://img.example/backdrop.jpg".to_owned()],
            ..AvMetadataFacts::default()
        };
        let artwork = vec![ProviderArtworkCandidate {
            provider: "javdb".to_owned(),
            provider_id: "javdb:movie:abc-001:poster".to_owned(),
            facts: ProviderArtworkCandidateFacts {
                kind: AddonArtworkKind::Poster,
                source_url: "https://img.example/poster.jpg".to_owned(),
                language: Some("ja".to_owned()),
                width: Some(800),
                height: Some(1200),
            },
        }];
        let external_ids = vec![ProviderExternalId {
            provider: "javdb".to_owned(),
            value: "abc-001".to_owned(),
        }];
        let mut patch = AddonMetadataPatch::default();

        materialize_native_metadata_patch(&mut patch, "javdb", Some(&av), &external_ids, &artwork);

        let credits = patch.credits.as_ref().unwrap();
        assert_eq!(credits.len(), 3);
        assert_eq!(credits[0].name, "Alice Actor");
        assert_eq!(credits[0].role, "actor");
        assert_eq!(credits[2].name, "Diana Director");
        assert_eq!(credits[2].role, "director");

        let studios = patch.studios.as_ref().unwrap();
        assert_eq!(studios.len(), 3);
        assert!(studios.iter().any(|studio| studio.name == "Studio One"));
        assert!(studios.iter().any(|studio| studio.name == "Publisher One"));
        assert!(studios.iter().any(|studio| studio.name == "Label One"));
        assert_eq!(patch.collections.as_ref().unwrap()[0].name, "Series One");

        let images = patch.images.as_ref().unwrap();
        assert_eq!(images.len(), 3);
        assert!(images.iter().any(|image| image.kind == "poster"));
        assert!(images.iter().any(|image| image.kind == "thumbnail"));
        assert!(images.iter().any(|image| image.kind == "backdrop"));

        let ids = patch.external_ids.as_ref().unwrap();
        assert!(
            ids.iter()
                .any(|id| id.provider == "javdb" && id.value == "abc-001")
        );
        assert!(ids.iter().any(|id| id.provider == "trailer_url"));
    }
}
