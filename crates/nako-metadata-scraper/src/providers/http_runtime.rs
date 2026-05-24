use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use serde::Serialize;
use thiserror::Error;
use tokio::time::{sleep, timeout};

const DEFAULT_TIMEOUT_MS: u64 = 10_000;
const DEFAULT_MAX_ATTEMPTS: u32 = 2;
const DEFAULT_RESPONSE_SIZE_LIMIT_BYTES: usize = 512 * 1024;
const DEFAULT_RETRY_BACKOFF_MS: u64 = 100;
const HTTP_STATUS_BODY_READ_LIMIT_BYTES: usize = 4 * 1024;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderHttpRuntimeConfig {
    pub timeout_ms: u64,
    pub max_attempts: u32,
    pub user_agent: String,
    pub proxy_url: Option<String>,
    pub response_size_limit_bytes: usize,
    pub retry_backoff_ms: u64,
}

impl Default for ProviderHttpRuntimeConfig {
    fn default() -> Self {
        Self {
            timeout_ms: DEFAULT_TIMEOUT_MS,
            max_attempts: DEFAULT_MAX_ATTEMPTS,
            user_agent: default_user_agent(),
            proxy_url: None,
            response_size_limit_bytes: DEFAULT_RESPONSE_SIZE_LIMIT_BYTES,
            retry_backoff_ms: DEFAULT_RETRY_BACKOFF_MS,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ProviderHttpRuntime<T = ReqwestProviderHttpTransport> {
    config: ProviderHttpRuntimeConfig,
    transport: Arc<T>,
}

impl ProviderHttpRuntime<ReqwestProviderHttpTransport> {
    pub fn new(config: ProviderHttpRuntimeConfig) -> ProviderHttpResult<Self> {
        let transport = ReqwestProviderHttpTransport::new(&config)?;
        Ok(Self::with_transport(config, transport))
    }
}

impl<T> ProviderHttpRuntime<T>
where
    T: ProviderHttpTransport,
{
    #[must_use]
    pub fn with_transport(config: ProviderHttpRuntimeConfig, transport: T) -> Self {
        Self {
            config,
            transport: Arc::new(transport),
        }
    }

    #[must_use]
    pub const fn config(&self) -> &ProviderHttpRuntimeConfig {
        &self.config
    }

    pub async fn get_json(
        &self,
        provider_id: &'static str,
        operation: &'static str,
        url: impl Into<String>,
        query: Vec<(String, String)>,
        headers: Vec<(String, String)>,
    ) -> ProviderHttpResult<ProviderHttpJsonResponse> {
        self.execute(ProviderHttpRequest {
            method: ProviderHttpMethod::Get,
            provider_id,
            operation,
            url: url.into(),
            query,
            headers,
            json_body: None,
        })
        .await
    }

    pub async fn post_json<B>(
        &self,
        provider_id: &'static str,
        operation: &'static str,
        url: impl Into<String>,
        query: Vec<(String, String)>,
        headers: Vec<(String, String)>,
        body: &B,
    ) -> ProviderHttpResult<ProviderHttpJsonResponse>
    where
        B: Serialize,
    {
        let json_body =
            serde_json::to_vec(body).map_err(|source| ProviderHttpError::InvalidRequest {
                provider_id,
                operation,
                message: format!("failed to serialize provider request body: {source}"),
            })?;
        self.execute(ProviderHttpRequest {
            method: ProviderHttpMethod::Post,
            provider_id,
            operation,
            url: url.into(),
            query,
            headers,
            json_body: Some(json_body),
        })
        .await
    }

    async fn execute(
        &self,
        request: ProviderHttpRequest,
    ) -> ProviderHttpResult<ProviderHttpJsonResponse> {
        let max_attempts = self.config.max_attempts.max(1);
        let mut last_retryable_error = None;

        for attempt in 1..=max_attempts {
            let response = timeout(
                Duration::from_millis(self.config.timeout_ms),
                self.transport.send(request.clone(), self.config.clone()),
            )
            .await
            .map_err(|_| ProviderHttpError::Timeout {
                provider_id: request.provider_id,
                operation: request.operation,
                timeout_ms: self.config.timeout_ms,
                attempts: attempt,
            });

            let response = match response {
                Ok(Ok(response)) => response,
                Ok(Err(error)) | Err(error) => {
                    if attempt < max_attempts && error.is_retryable() {
                        last_retryable_error = Some(error.with_attempts(attempt));
                        self.sleep_before_retry(attempt).await;
                        continue;
                    }
                    return Err(error.with_attempts(attempt));
                }
            };

            let status = response.status;
            if !(200..300).contains(&status) {
                let error = ProviderHttpError::HttpStatus {
                    provider_id: request.provider_id,
                    operation: request.operation,
                    status,
                    retryable: is_retryable_status(status),
                    body_excerpt: safe_excerpt(&response.body),
                    attempts: attempt,
                };
                if attempt < max_attempts && error.is_retryable() {
                    last_retryable_error = Some(error);
                    self.sleep_before_retry(attempt).await;
                    continue;
                }
                return Err(error);
            }

            if response.body.len() > self.config.response_size_limit_bytes {
                return Err(ProviderHttpError::ResponseTooLarge {
                    provider_id: request.provider_id,
                    operation: request.operation,
                    limit_bytes: self.config.response_size_limit_bytes,
                    actual_bytes: response.body.len(),
                    attempts: attempt,
                });
            }

            let body = serde_json::from_slice(&response.body).map_err(|source| {
                ProviderHttpError::InvalidJson {
                    provider_id: request.provider_id,
                    operation: request.operation,
                    message: safe_excerpt(source.to_string().as_bytes()),
                    attempts: attempt,
                }
            })?;

            return Ok(ProviderHttpJsonResponse {
                status,
                body,
                attempts: attempt,
            });
        }

        Err(
            last_retryable_error.unwrap_or(ProviderHttpError::InvalidRequest {
                provider_id: request.provider_id,
                operation: request.operation,
                message: "provider HTTP runtime exhausted attempts without response".to_owned(),
            }),
        )
    }

    async fn sleep_before_retry(&self, attempt: u32) {
        let backoff_ms = self
            .config
            .retry_backoff_ms
            .saturating_mul(u64::from(attempt));
        if backoff_ms > 0 {
            sleep(Duration::from_millis(backoff_ms)).await;
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderHttpRequest {
    pub method: ProviderHttpMethod,
    pub provider_id: &'static str,
    pub operation: &'static str,
    pub url: String,
    pub query: Vec<(String, String)>,
    pub headers: Vec<(String, String)>,
    pub json_body: Option<Vec<u8>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProviderHttpMethod {
    Get,
    Post,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderHttpResponse {
    pub status: u16,
    pub body: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProviderHttpJsonResponse {
    pub status: u16,
    pub body: serde_json::Value,
    pub attempts: u32,
}

#[async_trait]
pub trait ProviderHttpTransport: Send + Sync + 'static {
    async fn send(
        &self,
        request: ProviderHttpRequest,
        config: ProviderHttpRuntimeConfig,
    ) -> ProviderHttpResult<ProviderHttpResponse>;
}

#[derive(Clone, Debug)]
pub struct ReqwestProviderHttpTransport {
    client: reqwest::Client,
}

impl ReqwestProviderHttpTransport {
    pub fn new(config: &ProviderHttpRuntimeConfig) -> ProviderHttpResult<Self> {
        let mut builder = reqwest::Client::builder()
            .user_agent(config.user_agent.clone())
            .timeout(Duration::from_millis(config.timeout_ms));

        if let Some(proxy_url) = config
            .proxy_url
            .as_ref()
            .filter(|value| !value.trim().is_empty())
        {
            builder = builder.proxy(reqwest::Proxy::all(proxy_url).map_err(|source| {
                ProviderHttpError::InvalidRequest {
                    provider_id: "runtime",
                    operation: "build_client",
                    message: format!("invalid proxy configuration: {source}"),
                }
            })?);
        }

        let client = builder
            .build()
            .map_err(|source| ProviderHttpError::InvalidRequest {
                provider_id: "runtime",
                operation: "build_client",
                message: format!("failed to build provider HTTP client: {source}"),
            })?;
        Ok(Self { client })
    }
}

#[async_trait]
impl ProviderHttpTransport for ReqwestProviderHttpTransport {
    async fn send(
        &self,
        request: ProviderHttpRequest,
        _config: ProviderHttpRuntimeConfig,
    ) -> ProviderHttpResult<ProviderHttpResponse> {
        let mut builder = match request.method {
            ProviderHttpMethod::Get => self.client.get(&request.url),
            ProviderHttpMethod::Post => self.client.post(&request.url),
        }
        .query(&request.query);

        for (name, value) in request.headers {
            builder = builder.header(name, value);
        }
        if let Some(json_body) = request.json_body {
            builder = builder
                .header("content-type", "application/json")
                .body(json_body);
        }

        let response = builder
            .send()
            .await
            .map_err(|source| ProviderHttpError::Transport {
                provider_id: request.provider_id,
                operation: request.operation,
                message: safe_error_message(source),
                attempts: 0,
            })?;
        let status = response.status().as_u16();
        let body = if (200..300).contains(&status) {
            read_bounded_body(
                response,
                request.provider_id,
                request.operation,
                _config.response_size_limit_bytes,
            )
            .await?
        } else {
            read_truncated_body(
                response,
                request.provider_id,
                request.operation,
                HTTP_STATUS_BODY_READ_LIMIT_BYTES,
            )
            .await?
        };

        Ok(ProviderHttpResponse { status, body })
    }
}

async fn read_bounded_body(
    mut response: reqwest::Response,
    provider_id: &'static str,
    operation: &'static str,
    limit_bytes: usize,
) -> ProviderHttpResult<Vec<u8>> {
    let mut body = Vec::new();
    while let Some(chunk) =
        response
            .chunk()
            .await
            .map_err(|source| ProviderHttpError::Transport {
                provider_id,
                operation,
                message: safe_error_message(source),
                attempts: 0,
            })?
    {
        let actual_bytes = body.len().saturating_add(chunk.len());
        if actual_bytes > limit_bytes {
            return Err(ProviderHttpError::ResponseTooLarge {
                provider_id,
                operation,
                limit_bytes,
                actual_bytes,
                attempts: 0,
            });
        }
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

async fn read_truncated_body(
    mut response: reqwest::Response,
    provider_id: &'static str,
    operation: &'static str,
    read_limit_bytes: usize,
) -> ProviderHttpResult<Vec<u8>> {
    let mut body = Vec::new();
    while let Some(chunk) =
        response
            .chunk()
            .await
            .map_err(|source| ProviderHttpError::Transport {
                provider_id,
                operation,
                message: safe_error_message(source),
                attempts: 0,
            })?
    {
        let remaining = read_limit_bytes.saturating_sub(body.len());
        if remaining == 0 {
            break;
        }
        let take_bytes = remaining.min(chunk.len());
        body.extend_from_slice(&chunk[..take_bytes]);
        if take_bytes < chunk.len() {
            break;
        }
    }
    Ok(body)
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum ProviderHttpError {
    #[error("{provider_id} {operation} invalid request: {message}")]
    InvalidRequest {
        provider_id: &'static str,
        operation: &'static str,
        message: String,
    },
    #[error("{provider_id} {operation} timed out after {timeout_ms}ms")]
    Timeout {
        provider_id: &'static str,
        operation: &'static str,
        timeout_ms: u64,
        attempts: u32,
    },
    #[error("{provider_id} {operation} transport error: {message}")]
    Transport {
        provider_id: &'static str,
        operation: &'static str,
        message: String,
        attempts: u32,
    },
    #[error("{provider_id} {operation} returned HTTP {status}: {body_excerpt}")]
    HttpStatus {
        provider_id: &'static str,
        operation: &'static str,
        status: u16,
        retryable: bool,
        body_excerpt: String,
        attempts: u32,
    },
    #[error("{provider_id} {operation} response exceeded {limit_bytes} bytes")]
    ResponseTooLarge {
        provider_id: &'static str,
        operation: &'static str,
        limit_bytes: usize,
        actual_bytes: usize,
        attempts: u32,
    },
    #[error("{provider_id} {operation} returned invalid JSON: {message}")]
    InvalidJson {
        provider_id: &'static str,
        operation: &'static str,
        message: String,
        attempts: u32,
    },
}

impl ProviderHttpError {
    #[must_use]
    pub const fn is_retryable(&self) -> bool {
        match self {
            Self::Timeout { .. } | Self::Transport { .. } => true,
            Self::HttpStatus { retryable, .. } => *retryable,
            Self::InvalidRequest { .. }
            | Self::ResponseTooLarge { .. }
            | Self::InvalidJson { .. } => false,
        }
    }

    #[must_use]
    pub const fn attempts(&self) -> u32 {
        match self {
            Self::InvalidRequest { .. } => 0,
            Self::Timeout { attempts, .. }
            | Self::Transport { attempts, .. }
            | Self::HttpStatus { attempts, .. }
            | Self::ResponseTooLarge { attempts, .. }
            | Self::InvalidJson { attempts, .. } => *attempts,
        }
    }

    fn with_attempts(self, attempts: u32) -> Self {
        match self {
            Self::Timeout {
                provider_id,
                operation,
                timeout_ms,
                ..
            } => Self::Timeout {
                provider_id,
                operation,
                timeout_ms,
                attempts,
            },
            Self::Transport {
                provider_id,
                operation,
                message,
                ..
            } => Self::Transport {
                provider_id,
                operation,
                message,
                attempts,
            },
            Self::HttpStatus {
                provider_id,
                operation,
                status,
                retryable,
                body_excerpt,
                ..
            } => Self::HttpStatus {
                provider_id,
                operation,
                status,
                retryable,
                body_excerpt,
                attempts,
            },
            Self::ResponseTooLarge {
                provider_id,
                operation,
                limit_bytes,
                actual_bytes,
                ..
            } => Self::ResponseTooLarge {
                provider_id,
                operation,
                limit_bytes,
                actual_bytes,
                attempts,
            },
            Self::InvalidJson {
                provider_id,
                operation,
                message,
                ..
            } => Self::InvalidJson {
                provider_id,
                operation,
                message,
                attempts,
            },
            Self::InvalidRequest {
                provider_id,
                operation,
                message,
            } => Self::InvalidRequest {
                provider_id,
                operation,
                message,
            },
        }
    }
}

pub type ProviderHttpResult<T> = Result<T, ProviderHttpError>;

fn default_user_agent() -> String {
    format!("nako-metadata-scraper/{}", env!("CARGO_PKG_VERSION"))
}

fn is_retryable_status(status: u16) -> bool {
    status == 408 || status == 429 || (500..600).contains(&status)
}

fn safe_error_message(source: reqwest::Error) -> String {
    safe_text(&source.without_url().to_string())
}

fn safe_excerpt(body: &[u8]) -> String {
    safe_text(&String::from_utf8_lossy(body))
}

fn safe_text(value: &str) -> String {
    value.replace(['\r', '\n'], " ").chars().take(240).collect()
}

#[cfg(test)]
mod tests {
    use std::{
        collections::VecDeque,
        sync::{
            Arc, Mutex,
            atomic::{AtomicUsize, Ordering},
        },
    };

    use axum::{Router, http::StatusCode, response::IntoResponse, routing::get};

    use super::*;

    #[tokio::test]
    async fn http_runtime_retries_retryable_status_and_sends_runtime_policy() {
        let transport = FakeTransport::default();
        transport.push(Ok(ProviderHttpResponse {
            status: 500,
            body: b"temporary".to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{"ok":true}"#.to_vec(),
        }));
        let runtime = ProviderHttpRuntime::with_transport(
            ProviderHttpRuntimeConfig {
                timeout_ms: 2_000,
                max_attempts: 2,
                user_agent: "nako-test-agent".to_owned(),
                retry_backoff_ms: 0,
                ..ProviderHttpRuntimeConfig::default()
            },
            transport.clone(),
        );

        let response = runtime
            .get_json(
                "fixture",
                "search",
                "https://provider.example/search",
                vec![("query".to_owned(), "matrix".to_owned())],
                vec![("x-provider".to_owned(), "fixture".to_owned())],
            )
            .await
            .unwrap();

        assert_eq!(response.attempts, 2);
        assert_eq!(response.body["ok"], true);
        let requests = transport.requests();
        assert_eq!(requests.len(), 2);
        assert_eq!(requests[0].method, ProviderHttpMethod::Get);
        assert_eq!(
            requests[0].query[0],
            ("query".to_owned(), "matrix".to_owned())
        );
        assert_eq!(
            requests[0].headers[0],
            ("x-provider".to_owned(), "fixture".to_owned())
        );
        let configs = transport.configs();
        assert_eq!(configs[0].timeout_ms, 2_000);
        assert_eq!(configs[0].user_agent, "nako-test-agent");
    }

    #[tokio::test]
    async fn http_runtime_does_not_retry_non_retryable_status() {
        let transport = FakeTransport::default();
        transport.push(Ok(ProviderHttpResponse {
            status: 404,
            body: b"missing".to_vec(),
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{"ok":true}"#.to_vec(),
        }));
        let runtime = ProviderHttpRuntime::with_transport(
            ProviderHttpRuntimeConfig {
                max_attempts: 2,
                retry_backoff_ms: 0,
                ..ProviderHttpRuntimeConfig::default()
            },
            transport.clone(),
        );

        let error = runtime
            .get_json(
                "fixture",
                "search",
                "https://provider.example/search",
                Vec::new(),
                Vec::new(),
            )
            .await
            .unwrap_err();

        assert_eq!(transport.requests().len(), 1);
        assert_eq!(
            error,
            ProviderHttpError::HttpStatus {
                provider_id: "fixture",
                operation: "search",
                status: 404,
                retryable: false,
                body_excerpt: "missing".to_owned(),
                attempts: 1,
            }
        );
    }

    #[tokio::test]
    async fn http_runtime_retries_transport_error_with_redacted_message() {
        let transport = FakeTransport::default();
        transport.push(Err(ProviderHttpError::Transport {
            provider_id: "fixture",
            operation: "search",
            message: "network failed for https://secret.example/path?token=abc".to_owned(),
            attempts: 0,
        }));
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{"ok":true}"#.to_vec(),
        }));
        let runtime = ProviderHttpRuntime::with_transport(
            ProviderHttpRuntimeConfig {
                max_attempts: 2,
                retry_backoff_ms: 0,
                ..ProviderHttpRuntimeConfig::default()
            },
            transport.clone(),
        );

        let response = runtime
            .get_json(
                "fixture",
                "search",
                "https://provider.example/search",
                Vec::new(),
                Vec::new(),
            )
            .await
            .unwrap();

        assert_eq!(response.attempts, 2);
        assert_eq!(transport.requests().len(), 2);
    }

    #[tokio::test]
    async fn http_runtime_bounds_response_size_before_json_parse() {
        let transport = FakeTransport::default();
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: b"{\"value\":\"too-large\"}".to_vec(),
        }));
        let runtime = ProviderHttpRuntime::with_transport(
            ProviderHttpRuntimeConfig {
                response_size_limit_bytes: 4,
                retry_backoff_ms: 0,
                ..ProviderHttpRuntimeConfig::default()
            },
            transport,
        );

        let error = runtime
            .get_json(
                "fixture",
                "search",
                "https://provider.example/search",
                Vec::new(),
                Vec::new(),
            )
            .await
            .unwrap_err();

        assert_eq!(
            error,
            ProviderHttpError::ResponseTooLarge {
                provider_id: "fixture",
                operation: "search",
                limit_bytes: 4,
                actual_bytes: 21,
                attempts: 1,
            }
        );
    }

    #[tokio::test]
    async fn reqwest_transport_bounds_response_size_while_reading_body() {
        let app = Router::new().route(
            "/large",
            get(|| async { "{\"value\":\"response-body-is-too-large\"}" }),
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        let transport =
            ReqwestProviderHttpTransport::new(&ProviderHttpRuntimeConfig::default()).unwrap();
        let error = transport
            .send(
                ProviderHttpRequest {
                    method: ProviderHttpMethod::Get,
                    provider_id: "fixture",
                    operation: "search",
                    url: format!("http://{addr}/large"),
                    query: Vec::new(),
                    headers: Vec::new(),
                    json_body: None,
                },
                ProviderHttpRuntimeConfig {
                    response_size_limit_bytes: 8,
                    ..ProviderHttpRuntimeConfig::default()
                },
            )
            .await
            .unwrap_err();

        server.abort();
        match error {
            ProviderHttpError::ResponseTooLarge {
                provider_id,
                operation,
                limit_bytes,
                actual_bytes,
                attempts,
            } => {
                assert_eq!(provider_id, "fixture");
                assert_eq!(operation, "search");
                assert_eq!(limit_bytes, 8);
                assert!(actual_bytes > limit_bytes);
                assert_eq!(attempts, 0);
            }
            other => panic!("expected response size error, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn reqwest_transport_preserves_large_retryable_status_for_runtime_retry() {
        let attempts = Arc::new(AtomicUsize::new(0));
        let route_attempts = Arc::clone(&attempts);
        let app = Router::new().route(
            "/flaky",
            get(move || {
                let attempts = Arc::clone(&route_attempts);
                async move {
                    if attempts.fetch_add(1, Ordering::SeqCst) == 0 {
                        (StatusCode::SERVICE_UNAVAILABLE, "x".repeat(8 * 1024)).into_response()
                    } else {
                        (StatusCode::OK, "{\"ok\":true}").into_response()
                    }
                }
            }),
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        let runtime = ProviderHttpRuntime::new(ProviderHttpRuntimeConfig {
            max_attempts: 2,
            retry_backoff_ms: 0,
            response_size_limit_bytes: 16,
            ..ProviderHttpRuntimeConfig::default()
        })
        .unwrap();

        let response = runtime
            .get_json(
                "fixture",
                "search",
                format!("http://{addr}/flaky"),
                Vec::new(),
                Vec::new(),
            )
            .await
            .unwrap();

        server.abort();
        assert_eq!(response.attempts, 2);
        assert_eq!(response.body["ok"], true);
        assert_eq!(attempts.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn http_runtime_reports_invalid_json_as_non_retryable() {
        let transport = FakeTransport::default();
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: b"not-json".to_vec(),
        }));
        let runtime = ProviderHttpRuntime::with_transport(
            ProviderHttpRuntimeConfig {
                max_attempts: 2,
                retry_backoff_ms: 0,
                ..ProviderHttpRuntimeConfig::default()
            },
            transport.clone(),
        );

        let error = runtime
            .get_json(
                "fixture",
                "search",
                "https://provider.example/search",
                Vec::new(),
                Vec::new(),
            )
            .await
            .unwrap_err();

        assert!(!error.is_retryable());
        assert_eq!(error.attempts(), 1);
        assert_eq!(transport.requests().len(), 1);
    }

    #[tokio::test]
    async fn http_runtime_serializes_post_body_once_per_attempt() {
        let transport = FakeTransport::default();
        transport.push(Ok(ProviderHttpResponse {
            status: 200,
            body: br#"{"created":true}"#.to_vec(),
        }));
        let runtime = ProviderHttpRuntime::with_transport(
            ProviderHttpRuntimeConfig {
                retry_backoff_ms: 0,
                ..ProviderHttpRuntimeConfig::default()
            },
            transport.clone(),
        );

        let response = runtime
            .post_json(
                "fixture",
                "lookup",
                "https://provider.example/lookup",
                Vec::new(),
                Vec::new(),
                &serde_json::json!({"id": "fixture:1"}),
            )
            .await
            .unwrap();

        assert_eq!(response.body["created"], true);
        let requests = transport.requests();
        assert_eq!(requests[0].method, ProviderHttpMethod::Post);
        assert_eq!(
            requests[0].json_body.as_deref(),
            Some(br#"{"id":"fixture:1"}"#.as_slice())
        );
    }

    #[derive(Clone, Default)]
    struct FakeTransport {
        responses: Arc<Mutex<VecDeque<ProviderHttpResult<ProviderHttpResponse>>>>,
        requests: Arc<Mutex<Vec<ProviderHttpRequest>>>,
        configs: Arc<Mutex<Vec<ProviderHttpRuntimeConfig>>>,
    }

    impl FakeTransport {
        fn push(&self, response: ProviderHttpResult<ProviderHttpResponse>) {
            self.responses.lock().unwrap().push_back(response);
        }

        fn requests(&self) -> Vec<ProviderHttpRequest> {
            self.requests.lock().unwrap().clone()
        }

        fn configs(&self) -> Vec<ProviderHttpRuntimeConfig> {
            self.configs.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl ProviderHttpTransport for FakeTransport {
        async fn send(
            &self,
            request: ProviderHttpRequest,
            config: ProviderHttpRuntimeConfig,
        ) -> ProviderHttpResult<ProviderHttpResponse> {
            self.requests.lock().unwrap().push(request);
            self.configs.lock().unwrap().push(config);
            self.responses
                .lock()
                .unwrap()
                .pop_front()
                .unwrap_or_else(|| {
                    Err(ProviderHttpError::Transport {
                        provider_id: "fake",
                        operation: "send",
                        message: "fake transport response queue was empty".to_owned(),
                        attempts: 0,
                    })
                })
        }
    }
}
