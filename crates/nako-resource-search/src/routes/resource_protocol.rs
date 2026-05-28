use axum::{Json, http::StatusCode};
use nako_addon_protocol::{
    ADDON_PROTOCOL_VERSION, AddonResource, AddonResourceRequest, AddonResourceResponse,
};

use crate::{
    domain::{ResourceSearchRequest, ResourceSearchResponse},
    engine::ResourceSearchError,
    manifest::ADDON_ID,
};

pub(super) type RouteError = (StatusCode, Json<serde_json::Value>);

pub(super) fn decode_search_request(
    request: &AddonResourceRequest,
) -> Result<ResourceSearchRequest, RouteError> {
    validate_resource_envelope(request)?;
    serde_json::from_value::<ResourceSearchRequest>(request.payload.clone())
        .map_err(|_| safe_bad_request("invalid_resource_search_payload"))
}

pub(super) fn encode_search_response(
    request: AddonResourceRequest,
    response: ResourceSearchResponse,
) -> Result<AddonResourceResponse, RouteError> {
    let payload = serde_json::to_value(response)
        .map_err(|_| safe_internal_error("resource_search_response_serialize_failed"))?;

    Ok(AddonResourceResponse {
        protocol_version: request.protocol_version,
        addon_id: request.addon_id,
        resource: request.resource,
        request_id: request.request_id,
        payload,
        artifacts: Vec::new(),
    })
}

pub(super) fn search_error_response(error: ResourceSearchError) -> RouteError {
    match error {
        ResourceSearchError::EmptyQuery => safe_bad_request("empty_query"),
    }
}

fn validate_resource_envelope(request: &AddonResourceRequest) -> Result<(), RouteError> {
    if request.protocol_version != ADDON_PROTOCOL_VERSION {
        return Err(safe_bad_request("invalid_protocol_version"));
    }
    if request.addon_id != ADDON_ID {
        return Err(safe_bad_request("invalid_addon_id"));
    }
    if request.resource != AddonResource::Automation {
        return Err(safe_bad_request("invalid_resource"));
    }

    Ok(())
}

fn safe_bad_request(safe_error_code: &str) -> RouteError {
    safe_error(StatusCode::BAD_REQUEST, safe_error_code, false)
}

fn safe_internal_error(safe_error_code: &str) -> RouteError {
    safe_error(StatusCode::INTERNAL_SERVER_ERROR, safe_error_code, true)
}

fn safe_error(status: StatusCode, safe_error_code: &str, retryable: bool) -> RouteError {
    (
        status,
        Json(serde_json::json!({
            "safe_error_code": safe_error_code,
            "retryable": retryable
        })),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_search_request_rejects_invalid_protocol_envelope() {
        let request = AddonResourceRequest {
            protocol_version: "bad-version".to_owned(),
            addon_id: ADDON_ID.to_owned(),
            resource: AddonResource::Automation,
            request_id: "request-1".to_owned(),
            payload: serde_json::json!({ "query": "Demo Movie" }),
        };

        let error = decode_search_request(&request).unwrap_err();

        assert_safe_error(error, StatusCode::BAD_REQUEST, "invalid_protocol_version");
    }

    #[test]
    fn decode_search_request_rejects_invalid_payload_shape() {
        let request = AddonResourceRequest {
            protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
            addon_id: ADDON_ID.to_owned(),
            resource: AddonResource::Automation,
            request_id: "request-1".to_owned(),
            payload: serde_json::json!("not-an-object"),
        };

        let error = decode_search_request(&request).unwrap_err();

        assert_safe_error(
            error,
            StatusCode::BAD_REQUEST,
            "invalid_resource_search_payload",
        );
    }

    fn assert_safe_error(error: RouteError, expected_status: StatusCode, expected_code: &str) {
        let (status, Json(payload)) = error;

        assert_eq!(status, expected_status);
        assert_eq!(payload["safe_error_code"], expected_code);
        assert_eq!(payload["retryable"], false);
    }
}
