use std::{fmt, sync::Arc, time::Duration};

use async_trait::async_trait;
use serde::Deserialize;
use tokio::sync::Mutex;

use crate::config::TransmissionConfig;

const SESSION_ID_HEADER: &str = "x-transmission-session-id";

pub type SharedTransmissionClient = Arc<dyn TransmissionRunnerClient>;

#[async_trait]
pub trait TransmissionRunnerClient: fmt::Debug + Send + Sync {
    async fn add_torrent(
        &self,
        filename: String,
    ) -> Result<TransmissionAddOutcome, TransmissionError>;

    async fn get_torrent(
        &self,
        hash_string: &str,
    ) -> Result<Option<TransmissionTorrentFacts>, TransmissionError>;

    async fn start_torrent(&self, hash_string: &str) -> Result<(), TransmissionError>;

    async fn stop_torrent(&self, hash_string: &str) -> Result<(), TransmissionError>;
}

#[derive(Clone)]
pub struct TransmissionClient<T = ReqwestTransmissionTransport>
where
    T: TransmissionTransport,
{
    transport: T,
    session_id: Arc<Mutex<Option<String>>>,
}

#[async_trait]
impl<T> TransmissionRunnerClient for TransmissionClient<T>
where
    T: TransmissionTransport,
{
    async fn add_torrent(
        &self,
        filename: String,
    ) -> Result<TransmissionAddOutcome, TransmissionError> {
        Self::add_torrent(self, filename).await
    }

    async fn get_torrent(
        &self,
        hash_string: &str,
    ) -> Result<Option<TransmissionTorrentFacts>, TransmissionError> {
        Self::get_torrent(self, hash_string).await
    }

    async fn start_torrent(&self, hash_string: &str) -> Result<(), TransmissionError> {
        Self::start_torrent(self, hash_string).await
    }

    async fn stop_torrent(&self, hash_string: &str) -> Result<(), TransmissionError> {
        Self::stop_torrent(self, hash_string).await
    }
}

impl TransmissionClient<ReqwestTransmissionTransport> {
    pub fn new(config: TransmissionRpcConfig) -> Result<Self, TransmissionError> {
        Ok(Self::with_transport(ReqwestTransmissionTransport::new(
            config,
        )?))
    }
}

impl<T> TransmissionClient<T>
where
    T: TransmissionTransport,
{
    #[must_use]
    pub fn with_transport(transport: T) -> Self {
        Self {
            transport,
            session_id: Arc::default(),
        }
    }

    pub async fn add_torrent(
        &self,
        filename: impl Into<String>,
    ) -> Result<TransmissionAddOutcome, TransmissionError> {
        let arguments = serde_json::json!({
            "filename": filename.into(),
            "paused": false
        });
        let response = self.call("torrent-add", arguments).await?;
        let response: TorrentAddResponse = serde_json::from_value(response.arguments)
            .map_err(|_| TransmissionError::BadRpcShape)?;

        if let Some(torrent) = response.torrent_added {
            return Ok(TransmissionAddOutcome {
                kind: TransmissionAddOutcomeKind::Added,
                hash_string: non_empty_hash(torrent.hash_string)?,
            });
        }
        if let Some(torrent) = response.torrent_duplicate {
            return Ok(TransmissionAddOutcome {
                kind: TransmissionAddOutcomeKind::Duplicate,
                hash_string: non_empty_hash(torrent.hash_string)?,
            });
        }

        Err(TransmissionError::BadRpcShape)
    }

    pub async fn get_torrent(
        &self,
        hash_string: &str,
    ) -> Result<Option<TransmissionTorrentFacts>, TransmissionError> {
        let response = self
            .call(
                "torrent-get",
                serde_json::json!({
                    "ids": [hash_string],
                    "fields": [
                        "hashString",
                        "status",
                        "percentDone",
                        "downloadedEver",
                        "totalSize"
                    ]
                }),
            )
            .await?;
        let response: TorrentGetResponse = serde_json::from_value(response.arguments)
            .map_err(|_| TransmissionError::BadRpcShape)?;
        Ok(response
            .torrents
            .into_iter()
            .next()
            .map(TransmissionTorrentFacts::from))
    }

    pub async fn start_torrent(&self, hash_string: &str) -> Result<(), TransmissionError> {
        self.call("torrent-start", serde_json::json!({ "ids": [hash_string] }))
            .await?;
        Ok(())
    }

    pub async fn stop_torrent(&self, hash_string: &str) -> Result<(), TransmissionError> {
        self.call("torrent-stop", serde_json::json!({ "ids": [hash_string] }))
            .await?;
        Ok(())
    }

    async fn call(
        &self,
        method: &'static str,
        arguments: serde_json::Value,
    ) -> Result<TransmissionRpcSuccess, TransmissionError> {
        let body = serde_json::json!({
            "method": method,
            "arguments": arguments
        });
        let mut request = TransmissionHttpRequest {
            session_id: self.session_id.lock().await.clone(),
            body: body.clone(),
        };
        let mut response = self
            .transport
            .post_rpc(request.clone())
            .await
            .map_err(TransmissionError::Transport)?;

        if response.status == 409 {
            let Some(next_session_id) = response.session_id.take() else {
                return Err(TransmissionError::SessionIdMissing);
            };
            *self.session_id.lock().await = Some(next_session_id.clone());
            request.session_id = Some(next_session_id);
            response = self
                .transport
                .post_rpc(request)
                .await
                .map_err(TransmissionError::Transport)?;
        }

        if !(200..300).contains(&response.status) {
            return Err(TransmissionError::HttpStatus(response.status));
        }

        let envelope: TransmissionRpcEnvelope =
            serde_json::from_slice(&response.body).map_err(|_| TransmissionError::BadRpcShape)?;
        if envelope.result != "success" {
            return Err(TransmissionError::RpcRejected);
        }

        Ok(TransmissionRpcSuccess {
            arguments: envelope.arguments,
        })
    }
}

impl<T> fmt::Debug for TransmissionClient<T>
where
    T: TransmissionTransport,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TransmissionClient")
            .field("transport", &self.transport)
            .field("session_id", &"<redacted>")
            .finish()
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct TransmissionRpcConfig {
    pub rpc_url: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub timeout_ms: u64,
    pub allow_invalid_tls_certificates: bool,
}

impl TransmissionRpcConfig {
    #[must_use]
    pub fn from_config(config: &TransmissionConfig) -> Self {
        Self {
            rpc_url: config.rpc_url.clone(),
            username: config.username.clone(),
            password: config.password.clone(),
            timeout_ms: config.timeout_ms,
            allow_invalid_tls_certificates: config.allow_invalid_tls_certificates,
        }
    }
}

#[must_use]
pub fn transmission_client_from_config(
    config: &TransmissionConfig,
) -> Option<SharedTransmissionClient> {
    if !config.enabled {
        return None;
    }
    TransmissionClient::new(TransmissionRpcConfig::from_config(config))
        .ok()
        .map(|client| Arc::new(client) as SharedTransmissionClient)
}

impl fmt::Debug for TransmissionRpcConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TransmissionRpcConfig")
            .field("rpc_url", &"<configured>")
            .field("username", &self.username.as_ref().map(|_| "<redacted>"))
            .field("password", &self.password.as_ref().map(|_| "<redacted>"))
            .field("timeout_ms", &self.timeout_ms)
            .field(
                "allow_invalid_tls_certificates",
                &self.allow_invalid_tls_certificates,
            )
            .finish()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransmissionHttpRequest {
    pub session_id: Option<String>,
    pub body: serde_json::Value,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransmissionHttpResponse {
    pub status: u16,
    pub session_id: Option<String>,
    pub body: Vec<u8>,
}

impl TransmissionHttpResponse {
    #[must_use]
    pub fn json(status: u16, body: serde_json::Value) -> Self {
        Self {
            status,
            session_id: None,
            body: serde_json::to_vec(&body).expect("fake rpc response serializes"),
        }
    }

    #[must_use]
    pub fn session_required(session_id: impl Into<String>) -> Self {
        Self {
            status: 409,
            session_id: Some(session_id.into()),
            body: Vec::new(),
        }
    }
}

#[async_trait]
pub trait TransmissionTransport: Clone + fmt::Debug + Send + Sync + 'static {
    async fn post_rpc(
        &self,
        request: TransmissionHttpRequest,
    ) -> Result<TransmissionHttpResponse, TransmissionTransportError>;
}

#[derive(Clone)]
pub struct ReqwestTransmissionTransport {
    config: TransmissionRpcConfig,
    client: reqwest::Client,
}

impl ReqwestTransmissionTransport {
    pub fn new(config: TransmissionRpcConfig) -> Result<Self, TransmissionError> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_millis(config.timeout_ms))
            .danger_accept_invalid_certs(config.allow_invalid_tls_certificates)
            .build()
            .map_err(TransmissionError::BuildHttpClient)?;

        Ok(Self { config, client })
    }
}

impl fmt::Debug for ReqwestTransmissionTransport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ReqwestTransmissionTransport")
            .field("config", &self.config)
            .field("client", &"<redacted>")
            .finish()
    }
}

#[async_trait]
impl TransmissionTransport for ReqwestTransmissionTransport {
    async fn post_rpc(
        &self,
        request: TransmissionHttpRequest,
    ) -> Result<TransmissionHttpResponse, TransmissionTransportError> {
        let mut builder = self
            .client
            .post(&self.config.rpc_url)
            .header("content-type", "application/json")
            .body(
                serde_json::to_vec(&request.body)
                    .map_err(|_| TransmissionTransportError::new("request_serialize_failed"))?,
            );
        if let Some(session_id) = request.session_id {
            builder = builder.header(SESSION_ID_HEADER, session_id);
        }
        if let Some(username) = self.config.username.as_deref() {
            builder = builder.basic_auth(username, self.config.password.as_deref());
        }

        let response = builder
            .send()
            .await
            .map_err(|_| TransmissionTransportError::new("http_send_failed"))?;
        let status = response.status().as_u16();
        let session_id = response
            .headers()
            .get(SESSION_ID_HEADER)
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned);
        let body = response
            .bytes()
            .await
            .map_err(|_| TransmissionTransportError::new("http_body_failed"))?
            .to_vec();

        Ok(TransmissionHttpResponse {
            status,
            session_id,
            body,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransmissionAddOutcome {
    pub kind: TransmissionAddOutcomeKind,
    pub hash_string: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransmissionAddOutcomeKind {
    Added,
    Duplicate,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TransmissionTorrentFacts {
    pub hash_string: String,
    pub state: TransmissionTorrentState,
    pub percent_milli: Option<u64>,
    pub downloaded_bytes: Option<u64>,
    pub total_bytes: Option<u64>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransmissionTorrentState {
    Stopped,
    Checking,
    Queued,
    Downloading,
    Seeding,
    Unknown(i64),
}

impl From<TorrentFactsWire> for TransmissionTorrentFacts {
    fn from(value: TorrentFactsWire) -> Self {
        Self {
            hash_string: value.hash_string,
            state: TransmissionTorrentState::from_status_code(value.status),
            percent_milli: value.percent_done.and_then(percent_milli),
            downloaded_bytes: value.downloaded_ever,
            total_bytes: value.total_size,
        }
    }
}

impl TransmissionTorrentState {
    #[must_use]
    pub fn from_status_code(status: i64) -> Self {
        match status {
            0 => Self::Stopped,
            1 | 2 => Self::Checking,
            3 | 5 => Self::Queued,
            4 => Self::Downloading,
            6 => Self::Seeding,
            other => Self::Unknown(other),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TransmissionError {
    #[error("transmission_transport_failed")]
    Transport(TransmissionTransportError),
    #[error("transmission_session_id_missing")]
    SessionIdMissing,
    #[error("transmission_http_status")]
    HttpStatus(u16),
    #[error("transmission_rpc_rejected")]
    RpcRejected,
    #[error("transmission_bad_rpc_shape")]
    BadRpcShape,
    #[error("transmission_hash_missing")]
    HashMissing,
    #[error("transmission_http_client_build_failed")]
    BuildHttpClient(#[from] reqwest::Error),
}

impl TransmissionError {
    #[must_use]
    pub fn safe_error_code(&self) -> &'static str {
        match self {
            Self::Transport(_) => "transmission_transport_failed",
            Self::SessionIdMissing => "transmission_session_id_missing",
            Self::HttpStatus(_) => "transmission_http_status",
            Self::RpcRejected => "transmission_rpc_rejected",
            Self::BadRpcShape => "transmission_bad_rpc_shape",
            Self::HashMissing => "transmission_hash_missing",
            Self::BuildHttpClient(_) => "transmission_http_client_build_failed",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
#[error("{safe_code}")]
pub struct TransmissionTransportError {
    safe_code: String,
}

impl TransmissionTransportError {
    #[must_use]
    pub fn new(safe_code: impl Into<String>) -> Self {
        Self {
            safe_code: safe_code.into(),
        }
    }

    #[must_use]
    pub fn safe_code(&self) -> &str {
        &self.safe_code
    }
}

#[derive(Debug, Deserialize)]
struct TransmissionRpcEnvelope {
    result: String,
    #[serde(default)]
    arguments: serde_json::Value,
}

#[derive(Debug)]
struct TransmissionRpcSuccess {
    arguments: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct TorrentAddResponse {
    #[serde(rename = "torrent-added")]
    torrent_added: Option<TorrentIdentityWire>,
    #[serde(rename = "torrent-duplicate")]
    torrent_duplicate: Option<TorrentIdentityWire>,
}

#[derive(Debug, Deserialize)]
struct TorrentIdentityWire {
    #[serde(rename = "hashString")]
    hash_string: String,
}

#[derive(Debug, Deserialize)]
struct TorrentGetResponse {
    #[serde(default)]
    torrents: Vec<TorrentFactsWire>,
}

#[derive(Debug, Deserialize)]
struct TorrentFactsWire {
    #[serde(rename = "hashString")]
    hash_string: String,
    status: i64,
    #[serde(default, rename = "percentDone")]
    percent_done: Option<f64>,
    #[serde(default, rename = "downloadedEver")]
    downloaded_ever: Option<u64>,
    #[serde(default, rename = "totalSize")]
    total_size: Option<u64>,
}

fn non_empty_hash(hash_string: String) -> Result<String, TransmissionError> {
    if hash_string.trim().is_empty() {
        Err(TransmissionError::HashMissing)
    } else {
        Ok(hash_string)
    }
}

fn percent_milli(percent_done: f64) -> Option<u64> {
    if !percent_done.is_finite() || percent_done < 0.0 {
        return None;
    }
    Some(((percent_done.min(1.0) * 100_000.0).round()) as u64)
}

#[cfg(test)]
mod tests {
    use std::{collections::VecDeque, sync::Mutex as StdMutex};

    use super::*;

    #[tokio::test]
    async fn transmission_client_retries_with_session_id_after_409() {
        let transport = RecordingTransport::with_responses([
            Ok(TransmissionHttpResponse::session_required("session-secret")),
            Ok(success_response(serde_json::json!({
                "torrent-added": { "hashString": "ABCDEF" }
            }))),
        ]);
        let client = TransmissionClient::with_transport(transport.clone());

        let outcome = client
            .add_torrent("magnet:?xt=urn:btih:secret")
            .await
            .unwrap();

        assert_eq!(outcome.kind, TransmissionAddOutcomeKind::Added);
        assert_eq!(outcome.hash_string, "ABCDEF");
        let requests = transport.requests();
        assert_eq!(requests.len(), 2);
        assert_eq!(requests[0].session_id, None);
        assert_eq!(requests[1].session_id.as_deref(), Some("session-secret"));
        assert_eq!(requests[1].body["method"], "torrent-add");
    }

    #[tokio::test]
    async fn transmission_client_maps_duplicate_add() {
        let transport =
            RecordingTransport::with_responses([Ok(success_response(serde_json::json!({
                "torrent-duplicate": { "hashString": "DUPLICATE" }
            })))]);
        let client = TransmissionClient::with_transport(transport);

        let outcome = client
            .add_torrent("magnet:?xt=urn:btih:duplicate")
            .await
            .unwrap();

        assert_eq!(outcome.kind, TransmissionAddOutcomeKind::Duplicate);
        assert_eq!(outcome.hash_string, "DUPLICATE");
    }

    #[tokio::test]
    async fn transmission_client_gets_torrent_status() {
        let transport =
            RecordingTransport::with_responses([Ok(success_response(serde_json::json!({
                "torrents": [{
                    "hashString": "HASH",
                    "status": 4,
                    "percentDone": 0.625,
                    "downloadedEver": 1250,
                    "totalSize": 2000
                }]
            })))]);
        let client = TransmissionClient::with_transport(transport.clone());

        let facts = client.get_torrent("HASH").await.unwrap().unwrap();

        assert_eq!(facts.hash_string, "HASH");
        assert_eq!(facts.state, TransmissionTorrentState::Downloading);
        assert_eq!(facts.percent_milli, Some(62_500));
        assert_eq!(facts.downloaded_bytes, Some(1250));
        assert_eq!(facts.total_bytes, Some(2000));
        let requests = transport.requests();
        assert_eq!(requests[0].body["method"], "torrent-get");
        assert_eq!(requests[0].body["arguments"]["ids"][0], "HASH");
    }

    #[tokio::test]
    async fn transmission_client_calls_start_and_stop() {
        let transport = RecordingTransport::with_responses([
            Ok(success_response(serde_json::json!({}))),
            Ok(success_response(serde_json::json!({}))),
        ]);
        let client = TransmissionClient::with_transport(transport.clone());

        client.start_torrent("HASH").await.unwrap();
        client.stop_torrent("HASH").await.unwrap();

        let requests = transport.requests();
        assert_eq!(requests[0].body["method"], "torrent-start");
        assert_eq!(requests[1].body["method"], "torrent-stop");
    }

    #[tokio::test]
    async fn transmission_client_errors_are_redaction_safe() {
        let transport = RecordingTransport::with_responses([Err(TransmissionTransportError::new(
            "http_send_failed",
        ))]);
        let client = TransmissionClient::with_transport(transport);

        let error = client
            .add_torrent("magnet:?xt=urn:btih:secret")
            .await
            .unwrap_err();

        assert_eq!(error.safe_error_code(), "transmission_transport_failed");
        let debug = format!("{error:?}");
        assert!(!debug.contains("magnet:"));
        assert!(!debug.contains("secret"));
    }

    #[test]
    fn transmission_config_debug_redacts_credentials_and_endpoint() {
        let config = TransmissionRpcConfig {
            rpc_url: "http://runner:secret@transmission.local/transmission/rpc".to_owned(),
            username: Some("runner".to_owned()),
            password: Some("transmission-password-secret".to_owned()),
            timeout_ms: 1000,
            allow_invalid_tls_certificates: true,
        };

        let debug = format!("{config:?}");

        assert!(debug.contains("<configured>"));
        assert!(debug.contains("<redacted>"));
        for forbidden in [
            "runner:secret",
            "transmission.local",
            "runner",
            "transmission-password-secret",
        ] {
            assert!(
                !debug.contains(forbidden),
                "debug leaked forbidden config term: {forbidden}"
            );
        }
    }

    #[derive(Clone, Debug)]
    struct RecordingTransport {
        requests: Arc<StdMutex<Vec<TransmissionHttpRequest>>>,
        responses:
            Arc<StdMutex<VecDeque<Result<TransmissionHttpResponse, TransmissionTransportError>>>>,
    }

    impl RecordingTransport {
        fn with_responses(
            responses: impl IntoIterator<
                Item = Result<TransmissionHttpResponse, TransmissionTransportError>,
            >,
        ) -> Self {
            Self {
                requests: Arc::default(),
                responses: Arc::new(StdMutex::new(responses.into_iter().collect())),
            }
        }

        fn requests(&self) -> Vec<TransmissionHttpRequest> {
            self.requests.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl TransmissionTransport for RecordingTransport {
        async fn post_rpc(
            &self,
            request: TransmissionHttpRequest,
        ) -> Result<TransmissionHttpResponse, TransmissionTransportError> {
            self.requests.lock().unwrap().push(request);
            self.responses
                .lock()
                .unwrap()
                .pop_front()
                .unwrap_or_else(|| Err(TransmissionTransportError::new("unexpected_request")))
        }
    }

    fn success_response(arguments: serde_json::Value) -> TransmissionHttpResponse {
        TransmissionHttpResponse::json(
            200,
            serde_json::json!({
                "result": "success",
                "arguments": arguments
            }),
        )
    }
}
