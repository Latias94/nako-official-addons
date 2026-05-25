use nako_addon_protocol::{
    ADDON_PROTOCOL_VERSION, AddonArtifact, AddonResourceRequest, AddonResourceResponse,
};

use super::{
    MetadataCandidate, MetadataQuery, MetadataWritebackResult, artwork::ArtworkWritebackResult, av,
};

pub(crate) fn metadata_response(
    request: AddonResourceRequest,
    query: &MetadataQuery,
    candidates: Vec<MetadataCandidate>,
    writeback_result: Option<MetadataWritebackResult>,
    artwork_writeback_result: Option<ArtworkWritebackResult>,
) -> AddonResourceResponse {
    let av_facts = av::facts_from_payload(&request.payload).or_else(|| av::facts_from_query(query));
    let mut payload = serde_json::json!({
        "query": {
            "title": &query.title,
            "year": query.year,
            "language": &query.language
        },
        "candidates": candidates
    });
    if let Some(writeback_result) = writeback_result {
        payload["writeback"] = serde_json::to_value(writeback_result)
            .expect("writeback result is always serializable");
    }
    if let Some(av_facts) = av_facts {
        payload["query"]["av"] =
            serde_json::to_value(av_facts).expect("AV query facts are always serializable");
    }
    if let Some(artwork_writeback_result) = artwork_writeback_result {
        payload["artwork_writeback"] = serde_json::to_value(artwork_writeback_result)
            .expect("artwork writeback result is always serializable");
    }

    AddonResourceResponse {
        protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
        addon_id: request.addon_id,
        resource: request.resource,
        request_id: request.request_id,
        payload: payload.clone(),
        artifacts: vec![AddonArtifact {
            kind: "metadata_suggestion".to_owned(),
            payload,
        }],
    }
}
