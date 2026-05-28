use std::collections::BTreeMap;

use axum::{Json, http::StatusCode};
use nako_addon_protocol::{
    ADDON_PROTOCOL_VERSION, ADDON_RESOURCE_SEARCH_REQUEST_SCHEMA,
    ADDON_RESOURCE_SEARCH_RESPONSE_SCHEMA, AddonMergedResourceLink, AddonResource,
    AddonResourceLink, AddonResourceLinkType, AddonResourceRequest, AddonResourceResponse,
    AddonResourceSearchIntent, AddonResourceSearchProviderExecution,
    AddonResourceSearchProviderFinality, AddonResourceSearchProviderStatus,
    AddonResourceSearchRequest, AddonResourceSearchResponse, AddonResourceSearchResult,
};

use crate::{
    domain::{
        MergedResourceLink, ProviderExecutionFinality, ProviderExecutionStatus, ResourceLink,
        ResourceLinkType, ResourceSearchProviderExecution, ResourceSearchRequest,
        ResourceSearchResponse, ResourceSearchResult,
    },
    engine::ResourceSearchError,
    manifest::ADDON_ID,
};

pub(super) type RouteError = (StatusCode, Json<serde_json::Value>);

#[derive(Debug)]
pub(super) struct DecodedSearchRequest {
    pub request: ResourceSearchRequest,
    pub intent: AddonResourceSearchIntent,
}

pub(super) fn decode_search_request(
    request: &AddonResourceRequest,
) -> Result<DecodedSearchRequest, RouteError> {
    validate_resource_envelope(request)?;
    let payload = serde_json::from_value::<AddonResourceSearchRequest>(request.payload.clone())
        .map_err(|_| safe_bad_request("invalid_resource_search_payload"))?;

    if payload.schema != ADDON_RESOURCE_SEARCH_REQUEST_SCHEMA {
        return Err(safe_bad_request("invalid_resource_search_schema"));
    }

    Ok(DecodedSearchRequest {
        request: domain_search_request(&payload),
        intent: payload.intent,
    })
}

pub(super) fn encode_search_response(
    request: AddonResourceRequest,
    intent: AddonResourceSearchIntent,
    response: ResourceSearchResponse,
) -> Result<AddonResourceResponse, RouteError> {
    let payload = serde_json::to_value(addon_search_response(intent, response))
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
    if request.resource != AddonResource::ResourceSearch {
        return Err(safe_bad_request("invalid_resource"));
    }

    Ok(())
}

fn domain_search_request(payload: &AddonResourceSearchRequest) -> ResourceSearchRequest {
    ResourceSearchRequest {
        query: effective_query(payload),
        limit: payload.limit,
        sources: payload.sources.clone(),
        link_types: payload
            .link_types
            .iter()
            .copied()
            .map(domain_link_type)
            .collect(),
        refresh: payload.refresh,
        ext: context_with_intent(payload),
    }
}

fn effective_query(payload: &AddonResourceSearchRequest) -> String {
    if let AddonResourceSearchIntent::ExactLink { url } = &payload.intent {
        let url = url.trim();
        if !url.is_empty() {
            return url.to_owned();
        }
    }

    let query = payload.query.trim();
    if !query.is_empty() {
        return query.to_owned();
    }

    match &payload.intent {
        AddonResourceSearchIntent::FreeText { text } => text.trim().to_owned(),
        AddonResourceSearchIntent::MediaTitle { title, .. } => title.trim().to_owned(),
        AddonResourceSearchIntent::ExternalId { value, .. } => value.trim().to_owned(),
        AddonResourceSearchIntent::ExactLink { url } => url.trim().to_owned(),
    }
}

fn context_with_intent(payload: &AddonResourceSearchRequest) -> serde_json::Value {
    let mut context = match &payload.context {
        serde_json::Value::Object(object) => object.clone(),
        serde_json::Value::Null => serde_json::Map::new(),
        value => {
            let mut object = serde_json::Map::new();
            object.insert("context".to_owned(), value.clone());
            object
        }
    };

    match &payload.intent {
        AddonResourceSearchIntent::FreeText { text } => {
            insert_non_empty(&mut context, "free_text", text);
        }
        AddonResourceSearchIntent::MediaTitle {
            title,
            year,
            media_kind,
        } => {
            insert_non_empty(&mut context, "media_title", title);
            if let Some(year) = year {
                context.insert("year".to_owned(), serde_json::json!(year));
            }
            if let Some(media_kind) = media_kind {
                insert_non_empty(&mut context, "media_kind", media_kind);
            }
        }
        AddonResourceSearchIntent::ExternalId { id_kind, value } => {
            context.insert(
                "external_id".to_owned(),
                serde_json::json!({
                    "kind": id_kind,
                    "value": value
                }),
            );
        }
        AddonResourceSearchIntent::ExactLink { url } => {
            insert_non_empty(&mut context, "exact_link", url);
        }
    }

    serde_json::Value::Object(context)
}

fn insert_non_empty(
    context: &mut serde_json::Map<String, serde_json::Value>,
    key: &str,
    value: &str,
) {
    let value = value.trim();
    if !value.is_empty() {
        context.insert(key.to_owned(), serde_json::json!(value));
    }
}

fn addon_search_response(
    intent: AddonResourceSearchIntent,
    response: ResourceSearchResponse,
) -> AddonResourceSearchResponse {
    AddonResourceSearchResponse {
        schema: ADDON_RESOURCE_SEARCH_RESPONSE_SCHEMA.to_owned(),
        query: response.query,
        intent,
        total: response.total,
        results: response
            .results
            .into_iter()
            .map(addon_search_result)
            .collect(),
        merged_by_type: response
            .merged_by_type
            .into_iter()
            .map(|(link_type, links)| {
                (
                    addon_link_type(link_type),
                    links.into_iter().map(addon_merged_link).collect(),
                )
            })
            .collect::<BTreeMap<_, _>>(),
        provider_executions: response
            .provider_executions
            .into_iter()
            .map(addon_provider_execution)
            .collect(),
    }
}

fn addon_search_result(result: ResourceSearchResult) -> AddonResourceSearchResult {
    AddonResourceSearchResult {
        id: result.id,
        title: result.title,
        source: result.source,
        content: result.content,
        links: result.links.into_iter().map(addon_link).collect(),
        tags: result.tags,
        images: result.images,
        score: result.score,
    }
}

fn addon_link(link: ResourceLink) -> AddonResourceLink {
    AddonResourceLink {
        url: link.url,
        normalized_url: link.normalized_url,
        link_type: addon_link_type(link.link_type),
        source: link.source,
        password: link.password,
        note: link.note,
    }
}

fn addon_merged_link(link: MergedResourceLink) -> AddonMergedResourceLink {
    AddonMergedResourceLink {
        url: link.url,
        normalized_url: link.normalized_url,
        link_type: addon_link_type(link.link_type),
        password: link.password,
        note: link.note,
        sources: link.sources,
    }
}

fn addon_provider_execution(
    execution: ResourceSearchProviderExecution,
) -> AddonResourceSearchProviderExecution {
    AddonResourceSearchProviderExecution {
        provider_id: execution.provider_id,
        status: addon_provider_status(execution.status),
        result_count: execution.result_count,
        finality: addon_provider_finality(execution.finality),
        safe_message: execution.safe_message,
    }
}

const fn addon_provider_status(
    status: ProviderExecutionStatus,
) -> AddonResourceSearchProviderStatus {
    match status {
        ProviderExecutionStatus::Ok => AddonResourceSearchProviderStatus::Ok,
        ProviderExecutionStatus::Error => AddonResourceSearchProviderStatus::Error,
        ProviderExecutionStatus::Skipped => AddonResourceSearchProviderStatus::Skipped,
    }
}

const fn addon_provider_finality(
    finality: ProviderExecutionFinality,
) -> AddonResourceSearchProviderFinality {
    match finality {
        ProviderExecutionFinality::Complete => AddonResourceSearchProviderFinality::Complete,
        ProviderExecutionFinality::Partial => AddonResourceSearchProviderFinality::Partial,
        ProviderExecutionFinality::Unknown => AddonResourceSearchProviderFinality::Unknown,
    }
}

const fn domain_link_type(link_type: AddonResourceLinkType) -> ResourceLinkType {
    match link_type {
        AddonResourceLinkType::Aliyun => ResourceLinkType::Aliyun,
        AddonResourceLinkType::Baidu => ResourceLinkType::Baidu,
        AddonResourceLinkType::Quark => ResourceLinkType::Quark,
        AddonResourceLinkType::Tianyi => ResourceLinkType::Tianyi,
        AddonResourceLinkType::Uc => ResourceLinkType::Uc,
        AddonResourceLinkType::Mobile => ResourceLinkType::Mobile,
        AddonResourceLinkType::OneOneFive => ResourceLinkType::OneOneFive,
        AddonResourceLinkType::Pikpak => ResourceLinkType::Pikpak,
        AddonResourceLinkType::Xunlei => ResourceLinkType::Xunlei,
        AddonResourceLinkType::OneTwoThree => ResourceLinkType::OneTwoThree,
        AddonResourceLinkType::Magnet => ResourceLinkType::Magnet,
        AddonResourceLinkType::Ed2k => ResourceLinkType::Ed2k,
        AddonResourceLinkType::Web => ResourceLinkType::Web,
        AddonResourceLinkType::Other => ResourceLinkType::Other,
    }
}

const fn addon_link_type(link_type: ResourceLinkType) -> AddonResourceLinkType {
    match link_type {
        ResourceLinkType::Aliyun => AddonResourceLinkType::Aliyun,
        ResourceLinkType::Baidu => AddonResourceLinkType::Baidu,
        ResourceLinkType::Quark => AddonResourceLinkType::Quark,
        ResourceLinkType::Tianyi => AddonResourceLinkType::Tianyi,
        ResourceLinkType::Uc => AddonResourceLinkType::Uc,
        ResourceLinkType::Mobile => AddonResourceLinkType::Mobile,
        ResourceLinkType::OneOneFive => AddonResourceLinkType::OneOneFive,
        ResourceLinkType::Pikpak => AddonResourceLinkType::Pikpak,
        ResourceLinkType::Xunlei => AddonResourceLinkType::Xunlei,
        ResourceLinkType::OneTwoThree => AddonResourceLinkType::OneTwoThree,
        ResourceLinkType::Magnet => AddonResourceLinkType::Magnet,
        ResourceLinkType::Ed2k => AddonResourceLinkType::Ed2k,
        ResourceLinkType::Web => AddonResourceLinkType::Web,
        ResourceLinkType::Other => AddonResourceLinkType::Other,
    }
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
            resource: AddonResource::ResourceSearch,
            request_id: "request-1".to_owned(),
            payload: resource_search_payload(),
        };

        let error = decode_search_request(&request).unwrap_err();

        assert_safe_error(error, StatusCode::BAD_REQUEST, "invalid_protocol_version");
    }

    #[test]
    fn decode_search_request_rejects_invalid_payload_shape() {
        let request = AddonResourceRequest {
            protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
            addon_id: ADDON_ID.to_owned(),
            resource: AddonResource::ResourceSearch,
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

    #[test]
    fn decode_search_request_rejects_invalid_payload_schema() {
        let request = AddonResourceRequest {
            protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
            addon_id: ADDON_ID.to_owned(),
            resource: AddonResource::ResourceSearch,
            request_id: "request-1".to_owned(),
            payload: serde_json::json!({
                "schema": "nako.addon.resource_search.request.v0",
                "intent": { "kind": "free_text", "text": "Demo Movie" },
                "query": "Demo Movie"
            }),
        };

        let error = decode_search_request(&request).unwrap_err();

        assert_safe_error(
            error,
            StatusCode::BAD_REQUEST,
            "invalid_resource_search_schema",
        );
    }

    #[test]
    fn decode_search_request_maps_first_class_intent_to_domain_context() {
        let request = AddonResourceRequest {
            protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
            addon_id: ADDON_ID.to_owned(),
            resource: AddonResource::ResourceSearch,
            request_id: "request-1".to_owned(),
            payload: serde_json::json!({
                "schema": ADDON_RESOURCE_SEARCH_REQUEST_SCHEMA,
                "intent": {
                    "kind": "media_title",
                    "title": "Demo Movie",
                    "year": 2026,
                    "media_kind": "movie"
                },
                "query": "",
                "limit": 5,
                "link_types": ["quark"]
            }),
        };

        let decoded = decode_search_request(&request).unwrap();

        assert_eq!(decoded.request.query, "Demo Movie");
        assert_eq!(decoded.request.limit, Some(5));
        assert_eq!(decoded.request.link_types, vec![ResourceLinkType::Quark]);
        assert_eq!(decoded.request.ext["media_title"], "Demo Movie");
        assert_eq!(decoded.request.ext["year"], 2026);
        assert_eq!(decoded.request.ext["media_kind"], "movie");
    }

    #[test]
    fn decode_search_request_uses_exact_link_intent_url_as_query() {
        let request = AddonResourceRequest {
            protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
            addon_id: ADDON_ID.to_owned(),
            resource: AddonResource::ResourceSearch,
            request_id: "request-1".to_owned(),
            payload: serde_json::json!({
                "schema": ADDON_RESOURCE_SEARCH_REQUEST_SCHEMA,
                "intent": {
                    "kind": "exact_link",
                    "url": "magnet:?xt=urn:btih:ABCDEF"
                },
                "query": "Demo Movie"
            }),
        };

        let decoded = decode_search_request(&request).unwrap();

        assert_eq!(decoded.request.query, "magnet:?xt=urn:btih:ABCDEF");
    }

    fn resource_search_payload() -> serde_json::Value {
        serde_json::json!({
            "schema": ADDON_RESOURCE_SEARCH_REQUEST_SCHEMA,
            "intent": { "kind": "free_text", "text": "Demo Movie" },
            "query": "Demo Movie"
        })
    }

    fn assert_safe_error(error: RouteError, expected_status: StatusCode, expected_code: &str) {
        let (status, Json(payload)) = error;

        assert_eq!(status, expected_status);
        assert_eq!(payload["safe_error_code"], expected_code);
        assert_eq!(payload["retryable"], false);
    }
}
