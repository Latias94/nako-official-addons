use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use nako_addon_protocol::{AddonArtworkWritePayload, AddonMetadataPatch};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::config::NakoRuntimeConfig;

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
pub struct NakoRuntimeClient<T = ReqwestNakoRuntimeTransport> {
    config: NakoRuntimeClientConfig,
    transport: Arc<T>,
}

impl NakoRuntimeClient<ReqwestNakoRuntimeTransport> {
    #[must_use]
    pub fn new(config: NakoRuntimeClientConfig) -> Self {
        Self::with_transport(config, ReqwestNakoRuntimeTransport::default())
    }
}

impl<T> NakoRuntimeClient<T>
where
    T: NakoRuntimeTransport,
{
    #[must_use]
    pub fn with_transport(config: NakoRuntimeClientConfig, transport: T) -> Self {
        Self {
            config,
            transport: Arc::new(transport),
        }
    }

    pub async fn access_check(
        &self,
        request: NakoAccessCheckRequest,
    ) -> NakoRuntimeResult<NakoAccessCheckResponse> {
        self.post_json("/addon/v1/access-check", &request).await
    }

    pub async fn submit_side_effect(
        &self,
        request: SubmitNakoSideEffectRequest,
    ) -> NakoRuntimeResult<NakoSideEffectResponse> {
        self.post_json("/addon/v1/side-effects", &request).await
    }

    pub async fn submit_metadata_write(
        &self,
        request: SubmitNakoMetadataWriteRequest,
    ) -> NakoRuntimeResult<NakoSideEffectResponse> {
        let payload = serde_json::to_value(&request.patch).map_err(|source| {
            NakoRuntimeError::InvalidRequest {
                message: format!("failed to serialize Nako metadata_write payload: {source}"),
            }
        })?;
        self.submit_side_effect(SubmitNakoSideEffectRequest {
            permission: NakoPermission::MetadataWrite,
            library_id: request.library_id,
            target: request.target,
            idempotency_key: request.idempotency_key,
            provenance: request.provenance,
            payload,
        })
        .await
    }

    pub async fn submit_artwork_write(
        &self,
        request: SubmitNakoArtworkWriteRequest,
    ) -> NakoRuntimeResult<NakoSideEffectResponse> {
        if request.target.kind != NakoSideEffectTargetKind::MediaItem {
            return Err(NakoRuntimeError::InvalidRequest {
                message: "artwork_write target must be media_item".to_owned(),
            });
        }
        let payload = serde_json::to_value(&request.artwork).map_err(|source| {
            NakoRuntimeError::InvalidRequest {
                message: format!("failed to serialize Nako artwork_write payload: {source}"),
            }
        })?;
        self.submit_side_effect(SubmitNakoSideEffectRequest {
            permission: NakoPermission::ArtworkWrite,
            library_id: request.library_id,
            target: request.target,
            idempotency_key: request.idempotency_key,
            provenance: request.provenance,
            payload,
        })
        .await
    }

    async fn post_json<B, R>(&self, path: &str, body: &B) -> NakoRuntimeResult<R>
    where
        B: Serialize,
        R: for<'de> Deserialize<'de>,
    {
        let body =
            serde_json::to_string(body).map_err(|source| NakoRuntimeError::InvalidRequest {
                message: format!("failed to serialize Nako runtime request: {source}"),
            })?;
        reject_body_containing_token(&body, &self.config.addon_token)?;

        let response = self
            .transport
            .post(NakoRuntimeHttpRequest {
                url: join_url(&self.config.base_url, path),
                headers: vec![
                    ("accept".to_owned(), "application/json".to_owned()),
                    ("content-type".to_owned(), "application/json".to_owned()),
                    (
                        "authorization".to_owned(),
                        format!("Bearer {}", self.config.addon_token),
                    ),
                ],
                body,
                timeout_ms: self.config.timeout_ms,
            })
            .await?;

        if !(200..300).contains(&response.status) {
            return Err(NakoRuntimeError::HttpStatus {
                status: response.status,
                body_excerpt: safe_text(&response.body),
            });
        }

        serde_json::from_str(&response.body).map_err(|source| NakoRuntimeError::InvalidResponse {
            message: format!("failed to parse Nako runtime response: {source}"),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NakoRuntimeHttpRequest {
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: String,
    pub timeout_ms: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NakoRuntimeHttpResponse {
    pub status: u16,
    pub body: String,
}

#[async_trait]
pub trait NakoRuntimeTransport: Send + Sync + 'static {
    async fn post(
        &self,
        request: NakoRuntimeHttpRequest,
    ) -> NakoRuntimeResult<NakoRuntimeHttpResponse>;
}

#[derive(Clone, Debug, Default)]
pub struct ReqwestNakoRuntimeTransport {
    client: reqwest::Client,
}

#[async_trait]
impl NakoRuntimeTransport for ReqwestNakoRuntimeTransport {
    async fn post(
        &self,
        request: NakoRuntimeHttpRequest,
    ) -> NakoRuntimeResult<NakoRuntimeHttpResponse> {
        let mut builder = self
            .client
            .post(&request.url)
            .timeout(Duration::from_millis(request.timeout_ms))
            .body(request.body);
        for (name, value) in request.headers {
            builder = builder.header(name, value);
        }

        let response = builder
            .send()
            .await
            .map_err(|source| NakoRuntimeError::Transport {
                message: safe_text(&source.without_url().to_string()),
            })?;
        let status = response.status().as_u16();
        let body = response
            .text()
            .await
            .map_err(|source| NakoRuntimeError::Transport {
                message: safe_text(&source.to_string()),
            })?;

        Ok(NakoRuntimeHttpResponse { status, body })
    }
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum NakoRuntimeError {
    #[error("Nako runtime invalid request: {message}")]
    InvalidRequest { message: String },
    #[error("Nako runtime transport error: {message}")]
    Transport { message: String },
    #[error("Nako runtime returned HTTP {status}: {body_excerpt}")]
    HttpStatus { status: u16, body_excerpt: String },
    #[error("Nako runtime invalid response: {message}")]
    InvalidResponse { message: String },
    #[error("Nako runtime request body contained Addon Token material")]
    UnsafeRequestBody,
}

impl NakoRuntimeError {
    #[must_use]
    pub const fn safe_code(&self) -> &'static str {
        match self {
            Self::InvalidRequest { .. } => "invalid_request",
            Self::Transport { .. } => "transport_error",
            Self::HttpStatus { status, .. } => match *status {
                400..=499 => "http_client_error",
                500..=599 => "http_server_error",
                _ => "http_status_error",
            },
            Self::InvalidResponse { .. } => "invalid_response",
            Self::UnsafeRequestBody => "unsafe_request_body",
        }
    }
}

pub type NakoRuntimeResult<T> = Result<T, NakoRuntimeError>;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NakoPermission {
    MetadataWrite,
    ArtworkWrite,
    SubtitleWrite,
    LibraryFileWrite,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NakoSideEffectTargetKind {
    MediaItem,
    MediaSource,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NakoSideEffectTarget {
    pub kind: NakoSideEffectTargetKind,
    pub id: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NakoAccessCheckRequest {
    pub permission: NakoPermission,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub library_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NakoAccessCheckResponse {
    pub addon_id: String,
    pub token_id: String,
    pub permission: NakoPermission,
    #[serde(default)]
    pub library_id: Option<String>,
    pub allowed: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SubmitNakoSideEffectRequest {
    pub permission: NakoPermission,
    pub library_id: String,
    pub target: NakoSideEffectTarget,
    pub idempotency_key: String,
    pub provenance: serde_json::Value,
    pub payload: serde_json::Value,
}

#[derive(Clone, Debug)]
pub struct SubmitNakoMetadataWriteRequest {
    pub library_id: String,
    pub target: NakoSideEffectTarget,
    pub idempotency_key: String,
    pub provenance: serde_json::Value,
    pub patch: AddonMetadataPatch,
}

#[derive(Clone, Debug)]
pub struct SubmitNakoArtworkWriteRequest {
    pub library_id: String,
    pub target: NakoSideEffectTarget,
    pub idempotency_key: String,
    pub provenance: serde_json::Value,
    pub artwork: AddonArtworkWritePayload,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NakoSideEffectResponse {
    pub side_effect: NakoSideEffectSummary,
    pub idempotent_replay: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NakoSideEffectSummary {
    pub id: String,
    pub permission: NakoPermission,
    pub library_id: String,
    pub target: NakoSideEffectTarget,
    pub idempotency_key: String,
    pub validation_status: String,
    #[serde(default)]
    pub safe_error_code: Option<String>,
    pub apply_status: String,
    #[serde(default)]
    pub apply_error_code: Option<String>,
    #[serde(default)]
    pub applied_item_id: Option<String>,
    #[serde(default)]
    pub applied_source: Option<String>,
    #[serde(default)]
    pub apply_report: Option<serde_json::Value>,
}

fn reject_body_containing_token(body: &str, token: &str) -> NakoRuntimeResult<()> {
    if !token.trim().is_empty() && body.contains(token) {
        return Err(NakoRuntimeError::UnsafeRequestBody);
    }
    Ok(())
}

fn join_url(base_url: &str, path: &str) -> String {
    format!("{}{}", base_url.trim_end_matches('/'), path)
}

fn safe_text(value: &str) -> String {
    value.replace(['\r', '\n'], " ").chars().take(240).collect()
}

#[cfg(test)]
mod tests {
    use std::{
        collections::VecDeque,
        sync::{Arc, Mutex},
    };

    use super::*;

    #[tokio::test]
    async fn access_check_sends_bearer_token_only_in_header() {
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
        assert_eq!(response.permission, NakoPermission::MetadataWrite);
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
        assert!(
            requests[0]
                .body
                .contains("\"permission\":\"metadata_write\"")
        );
    }

    #[tokio::test]
    async fn side_effect_request_posts_redaction_safe_payload_and_parses_outcome() {
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
        let client = test_client(transport.clone());

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

        assert_eq!(response.side_effect.apply_status, "applied");
        assert!(!response.idempotent_replay);
        let requests = transport.requests();
        assert_eq!(
            requests[0].url,
            "https://nako.example/addon/v1/side-effects"
        );
        assert_eq!(
            header_value(&requests[0], "authorization"),
            Some("Bearer addon-token-secret")
        );
        assert!(!requests[0].body.contains("addon-token-secret"));
        assert!(
            requests[0]
                .body
                .contains("\"idempotency_key\":\"metadata-demo-1\"")
        );
    }

    #[tokio::test]
    async fn metadata_side_effect_request_serializes_patch_and_sets_permission() {
        let transport = FakeTransport::default();
        transport.push(Ok(NakoRuntimeHttpResponse {
            status: 200,
            body: serde_json::json!({
                "side_effect": {
                    "id": "effect-1",
                    "permission": "metadata_write",
                    "library_id": "library-1",
                    "target": {"kind": "media_source", "id": "source-1"},
                    "idempotency_key": "metadata-demo-2",
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
        let client = test_client(transport.clone());

        let response = client
            .submit_metadata_write(SubmitNakoMetadataWriteRequest {
                library_id: "library-1".to_owned(),
                target: NakoSideEffectTarget {
                    kind: NakoSideEffectTargetKind::MediaSource,
                    id: "source-1".to_owned(),
                },
                idempotency_key: "metadata-demo-2".to_owned(),
                provenance: serde_json::json!({"origin": "nako-metadata-scraper"}),
                patch: AddonMetadataPatch {
                    title: Some("The Matrix".to_owned()),
                    overview: Some("A demo patch".to_owned()),
                    ..AddonMetadataPatch::default()
                },
            })
            .await
            .unwrap();

        assert_eq!(
            response.side_effect.permission,
            NakoPermission::MetadataWrite
        );
        let requests = transport.requests();
        assert_eq!(requests.len(), 1);
        let body: serde_json::Value = serde_json::from_str(&requests[0].body).unwrap();
        assert_eq!(
            body,
            serde_json::json!({
                "permission": "metadata_write",
                "library_id": "library-1",
                "target": {
                    "kind": "media_source",
                    "id": "source-1"
                },
                "idempotency_key": "metadata-demo-2",
                "provenance": {
                    "origin": "nako-metadata-scraper"
                },
                "payload": {
                    "title": "The Matrix",
                    "original_title": null,
                    "sort_title": null,
                    "overview": "A demo patch",
                    "release_date": null,
                    "runtime_minutes": null,
                    "tagline": null,
                    "genres": null,
                    "tags": null
                }
            })
        );
    }

    #[tokio::test]
    async fn artwork_side_effect_request_serializes_payload_and_sets_permission() {
        let transport = FakeTransport::default();
        transport.push(Ok(NakoRuntimeHttpResponse {
            status: 200,
            body: serde_json::json!({
                "side_effect": {
                    "id": "effect-2",
                    "permission": "artwork_write",
                    "library_id": "library-1",
                    "target": {"kind": "media_item", "id": "item-1"},
                    "idempotency_key": "artwork-demo-2",
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
        let client = test_client(transport.clone());

        let response = client
            .submit_artwork_write(SubmitNakoArtworkWriteRequest {
                library_id: "library-1".to_owned(),
                target: NakoSideEffectTarget {
                    kind: NakoSideEffectTargetKind::MediaItem,
                    id: "item-1".to_owned(),
                },
                idempotency_key: "artwork-demo-2".to_owned(),
                provenance: serde_json::json!({"origin": "nako-metadata-scraper"}),
                artwork: AddonArtworkWritePayload {
                    intent: nako_addon_protocol::AddonArtworkIntent::ProposeArtwork,
                    kind: nako_addon_protocol::AddonArtworkKind::Poster,
                    source: nako_addon_protocol::AddonArtworkSourcePayload {
                        kind: nako_addon_protocol::AddonArtworkSourceKind::RemoteUrl,
                        url: "https://example.test/poster.jpg".to_owned(),
                    },
                    language: Some("en".to_owned()),
                    width: Some(1000),
                    height: Some(1500),
                },
            })
            .await
            .unwrap();

        assert_eq!(
            response.side_effect.permission,
            NakoPermission::ArtworkWrite
        );
        let requests = transport.requests();
        assert_eq!(requests.len(), 1);
        let body: serde_json::Value = serde_json::from_str(&requests[0].body).unwrap();
        assert_eq!(body["permission"], "artwork_write");
        assert_eq!(body["library_id"], "library-1");
        assert_eq!(body["target"]["kind"], "media_item");
        assert_eq!(body["target"]["id"], "item-1");
        assert_eq!(body["idempotency_key"], "artwork-demo-2");
        assert_eq!(body["payload"]["intent"], "propose_artwork");
        assert_eq!(body["payload"]["kind"], "poster");
        assert_eq!(body["payload"]["source"]["kind"], "remote_url");
        assert_eq!(
            body["payload"]["source"]["url"],
            "https://example.test/poster.jpg"
        );
        assert_eq!(body["payload"]["language"], "en");
        assert_eq!(body["payload"]["width"], 1000);
        assert_eq!(body["payload"]["height"], 1500);
    }

    #[tokio::test]
    async fn artwork_side_effect_request_rejects_non_media_item_targets() {
        let transport = FakeTransport::default();
        let client = test_client(transport.clone());

        let error = client
            .submit_artwork_write(SubmitNakoArtworkWriteRequest {
                library_id: "library-1".to_owned(),
                target: NakoSideEffectTarget {
                    kind: NakoSideEffectTargetKind::MediaSource,
                    id: "source-1".to_owned(),
                },
                idempotency_key: "artwork-demo-3".to_owned(),
                provenance: serde_json::json!({"origin": "nako-metadata-scraper"}),
                artwork: AddonArtworkWritePayload {
                    intent: nako_addon_protocol::AddonArtworkIntent::ProposeArtwork,
                    kind: nako_addon_protocol::AddonArtworkKind::Poster,
                    source: nako_addon_protocol::AddonArtworkSourcePayload {
                        kind: nako_addon_protocol::AddonArtworkSourceKind::RemoteUrl,
                        url: "https://example.test/poster.jpg".to_owned(),
                    },
                    language: None,
                    width: None,
                    height: None,
                },
            })
            .await
            .unwrap_err();

        assert_eq!(
            error,
            NakoRuntimeError::InvalidRequest {
                message: "artwork_write target must be media_item".to_owned(),
            }
        );
        assert!(transport.requests().is_empty());
    }

    #[tokio::test]
    async fn runtime_rejects_body_that_would_leak_addon_token() {
        let transport = FakeTransport::default();
        let client = test_client(transport.clone());

        let error = client
            .submit_side_effect(SubmitNakoSideEffectRequest {
                permission: NakoPermission::MetadataWrite,
                library_id: "library-1".to_owned(),
                target: NakoSideEffectTarget {
                    kind: NakoSideEffectTargetKind::MediaItem,
                    id: "item-1".to_owned(),
                },
                idempotency_key: "unsafe".to_owned(),
                provenance: serde_json::json!({"token": "addon-token-secret"}),
                payload: serde_json::json!({"title": "Demo"}),
            })
            .await
            .unwrap_err();

        assert_eq!(error, NakoRuntimeError::UnsafeRequestBody);
        assert!(transport.requests().is_empty());
    }

    fn test_client(transport: FakeTransport) -> NakoRuntimeClient<FakeTransport> {
        NakoRuntimeClient::with_transport(
            NakoRuntimeClientConfig {
                base_url: "https://nako.example/".to_owned(),
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
                    Err(NakoRuntimeError::Transport {
                        message: "fake response queue was empty".to_owned(),
                    })
                })
        }
    }
}
