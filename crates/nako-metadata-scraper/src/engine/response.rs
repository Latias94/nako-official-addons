use nako_addon_protocol::{
    ADDON_PROTOCOL_VERSION, AddonArtifact, AddonResourceRequest, AddonResourceResponse,
};

use super::MetadataScrapeOutcome;

pub(crate) fn metadata_response(
    request: AddonResourceRequest,
    outcome: MetadataScrapeOutcome,
) -> AddonResourceResponse {
    let payload = metadata_payload(&outcome);

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

pub(crate) fn metadata_payload(outcome: &MetadataScrapeOutcome) -> serde_json::Value {
    let mut payload = serde_json::json!({
        "query": {
            "title": &outcome.query.title,
            "year": outcome.query.year,
            "language": &outcome.query.language
        },
        "provider_execution": &outcome.provider_execution,
        "candidates": &outcome.candidates
    });
    if let Some(writeback_result) = &outcome.writeback_result {
        payload["writeback"] = serde_json::to_value(writeback_result)
            .expect("writeback result is always serializable");
    }
    if let Some(av_facts) = &outcome.av {
        payload["query"]["av"] =
            serde_json::to_value(av_facts).expect("AV query facts are always serializable");
    }
    if let Some(artwork_writeback_result) = &outcome.artwork_writeback_result {
        payload["artwork_writeback"] = serde_json::to_value(artwork_writeback_result)
            .expect("artwork writeback result is always serializable");
    }

    payload
}
