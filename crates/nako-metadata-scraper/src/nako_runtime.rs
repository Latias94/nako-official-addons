use nako_addon_client::AddonClientError;

use crate::config::NakoRuntimeConfig;

pub use nako_addon_client::{
    AddonClientResult as NakoRuntimeResult, AddonHttpRequest as NakoRuntimeHttpRequest,
    AddonHttpResponse as NakoRuntimeHttpResponse, AddonTransport as NakoRuntimeTransport,
    ReqwestAddonTransport as ReqwestNakoRuntimeTransport,
};
pub use nako_addon_protocol::{
    AddonAccessCheckRequest as NakoAccessCheckRequest,
    AddonAccessCheckResponse as NakoAccessCheckResponse, AddonPermission as NakoPermission,
    AddonSideEffectResponse as NakoSideEffectResponse,
    AddonSideEffectSummary as NakoSideEffectSummary, AddonSideEffectTarget as NakoSideEffectTarget,
    AddonSideEffectTargetKind as NakoSideEffectTargetKind,
    SubmitAddonArtworkWriteRequest as SubmitNakoArtworkWriteRequest,
    SubmitAddonMetadataWriteRequest as SubmitNakoMetadataWriteRequest,
    SubmitAddonSideEffectRequest as SubmitNakoSideEffectRequest,
};

pub type NakoRuntimeError = AddonClientError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NakoRuntimeClientConfig {
    pub base_url: String,
    pub addon_token: String,
    pub timeout_ms: u64,
}

impl NakoRuntimeClientConfig {
    #[must_use]
    pub fn from_runtime_config(config: &NakoRuntimeConfig) -> Option<Self> {
        config.can_submit_side_effects().then(|| Self {
            base_url: config
                .base_url
                .clone()
                .expect("checked by can_submit_side_effects"),
            addon_token: config
                .addon_token
                .clone()
                .expect("checked by can_submit_side_effects"),
            timeout_ms: config.timeout_ms,
        })
    }
}

#[derive(Clone, Debug)]
pub struct NakoRuntimeClient<T = ReqwestNakoRuntimeTransport>
where
    T: NakoRuntimeTransport,
{
    inner: nako_addon_client::NakoRuntimeClient<T>,
}

impl NakoRuntimeClient<ReqwestNakoRuntimeTransport> {
    #[must_use]
    pub fn new(config: NakoRuntimeClientConfig) -> Self {
        Self {
            inner: nako_addon_client::NakoRuntimeClient::new(config.into()),
        }
    }
}

impl<T> NakoRuntimeClient<T>
where
    T: NakoRuntimeTransport,
{
    #[must_use]
    pub fn with_transport(config: NakoRuntimeClientConfig, transport: T) -> Self {
        Self {
            inner: nako_addon_client::NakoRuntimeClient::with_transport(config.into(), transport),
        }
    }

    pub async fn access_check(
        &self,
        request: NakoAccessCheckRequest,
    ) -> NakoRuntimeResult<NakoAccessCheckResponse> {
        self.inner.access_check(request).await
    }

    pub async fn submit_side_effect(
        &self,
        request: SubmitNakoSideEffectRequest,
    ) -> NakoRuntimeResult<NakoSideEffectResponse> {
        self.inner.submit_side_effect(request).await
    }

    pub async fn submit_metadata_write(
        &self,
        request: SubmitNakoMetadataWriteRequest,
    ) -> NakoRuntimeResult<NakoSideEffectResponse> {
        self.inner.submit_metadata_write(request).await
    }

    pub async fn submit_artwork_write(
        &self,
        request: SubmitNakoArtworkWriteRequest,
    ) -> NakoRuntimeResult<NakoSideEffectResponse> {
        self.inner.submit_artwork_write(request).await
    }
}

impl From<NakoRuntimeClientConfig> for nako_addon_client::NakoRuntimeClientConfig {
    fn from(value: NakoRuntimeClientConfig) -> Self {
        Self {
            base_url: value.base_url,
            addon_token: value.addon_token,
            timeout_ms: value.timeout_ms,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::VecDeque,
        sync::{Arc, Mutex},
    };

    use async_trait::async_trait;

    use super::*;

    #[tokio::test]
    async fn runtime_facade_sends_bearer_token_only_in_header() {
        let transport = FakeTransport::default();
        transport.push(Ok(NakoRuntimeHttpResponse {
            status: 200,
            body: serde_json::json!({
                "addon_id": "addon-1",
                "token_id": "token-1",
                "permission": "metadata_write",
                "library_id": "library-1",
                "allowed": true
            })
            .to_string(),
        }));
        let client = test_client(transport.clone());

        let response = client
            .access_check(NakoAccessCheckRequest {
                permission: NakoPermission::MetadataWrite,
                library_id: Some("library-1".to_owned()),
            })
            .await
            .unwrap();

        assert!(response.allowed);
        let requests = transport.requests();
        assert_eq!(requests.len(), 1);
        assert_eq!(
            requests[0].url,
            "https://nako.example/addon/v1/access-check"
        );
        assert_eq!(
            header_value(&requests[0], "authorization"),
            Some("Bearer addon-token-secret")
        );
        assert!(!requests[0].body.contains("addon-token-secret"));
    }

    #[tokio::test]
    async fn runtime_facade_rejects_body_token_material_before_http() {
        let transport = FakeTransport::default();
        let client = test_client(transport.clone());

        let error = client
            .submit_side_effect(SubmitNakoSideEffectRequest {
                permission: NakoPermission::MetadataWrite,
                library_id: "library-1".to_owned(),
                target: NakoSideEffectTarget {
                    kind: NakoSideEffectTargetKind::MediaSource,
                    id: "source-1".to_owned(),
                },
                idempotency_key: "metadata-demo-1".to_owned(),
                provenance: serde_json::json!({"origin": "nako-metadata-scraper"}),
                payload: serde_json::json!({"leak": "addon-token-secret"}),
            })
            .await
            .unwrap_err();

        assert_eq!(error.safe_code(), "unsafe_request_body");
        assert!(transport.requests().is_empty());
    }

    #[tokio::test]
    async fn runtime_facade_parses_side_effect_summary_with_optional_host_fields() {
        let transport = FakeTransport::default();
        transport.push(Ok(NakoRuntimeHttpResponse {
            status: 200,
            body: serde_json::json!({
                "side_effect": {
                    "id": "effect-1",
                    "permission": "metadata_write",
                    "library_id": "library-1",
                    "target": {"kind": "media_source", "id": "source-1"},
                    "idempotency_key": "metadata-demo-1",
                    "validation_status": "accepted",
                    "safe_error_code": null,
                    "apply_status": "applied",
                    "apply_error_code": null,
                    "applied_item_id": "item-1",
                    "applied_source": "addon:addon-1",
                    "apply_report": null
                },
                "idempotent_replay": false
            })
            .to_string(),
        }));
        let client = test_client(transport);

        let response = client
            .submit_side_effect(SubmitNakoSideEffectRequest {
                permission: NakoPermission::MetadataWrite,
                library_id: "library-1".to_owned(),
                target: NakoSideEffectTarget {
                    kind: NakoSideEffectTargetKind::MediaSource,
                    id: "source-1".to_owned(),
                },
                idempotency_key: "metadata-demo-1".to_owned(),
                provenance: serde_json::json!({"origin": "nako-metadata-scraper"}),
                payload: serde_json::json!({"title": "Demo"}),
            })
            .await
            .unwrap();

        assert_eq!(
            response.side_effect.permission,
            NakoPermission::MetadataWrite
        );
        assert_eq!(
            response.side_effect.applied_item_id.as_deref(),
            Some("item-1")
        );
        assert_eq!(response.side_effect.addon_id, None);
    }

    fn test_client(transport: FakeTransport) -> NakoRuntimeClient<FakeTransport> {
        NakoRuntimeClient::with_transport(
            NakoRuntimeClientConfig {
                base_url: "https://nako.example".to_owned(),
                addon_token: "addon-token-secret".to_owned(),
                timeout_ms: 1500,
            },
            transport,
        )
    }

    fn header_value<'a>(request: &'a NakoRuntimeHttpRequest, name: &str) -> Option<&'a str> {
        request
            .headers
            .iter()
            .find(|(candidate, _)| candidate == name)
            .map(|(_, value)| value.as_str())
    }

    #[derive(Clone, Default)]
    struct FakeTransport {
        responses: Arc<Mutex<VecDeque<NakoRuntimeResult<NakoRuntimeHttpResponse>>>>,
        requests: Arc<Mutex<Vec<NakoRuntimeHttpRequest>>>,
    }

    impl FakeTransport {
        fn push(&self, response: NakoRuntimeResult<NakoRuntimeHttpResponse>) {
            self.responses.lock().unwrap().push_back(response);
        }

        fn requests(&self) -> Vec<NakoRuntimeHttpRequest> {
            self.requests.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl NakoRuntimeTransport for FakeTransport {
        async fn post(
            &self,
            request: NakoRuntimeHttpRequest,
        ) -> NakoRuntimeResult<NakoRuntimeHttpResponse> {
            self.requests.lock().unwrap().push(request);
            self.responses
                .lock()
                .unwrap()
                .pop_front()
                .unwrap_or_else(|| {
                    Err(NakoRuntimeError::Http {
                        message: "fake transport response queue was empty".to_owned(),
                    })
                })
        }
    }
}
