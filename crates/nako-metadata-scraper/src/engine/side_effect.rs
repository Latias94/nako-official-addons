use async_trait::async_trait;
use serde::de::DeserializeOwned;

use crate::nako_runtime::{
    NakoAccessCheckRequest, NakoPermission, NakoRuntimeClient, NakoRuntimeResult,
    NakoRuntimeTransport, NakoSideEffectResponse, NakoSideEffectSummary, NakoSideEffectTarget,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum SideEffectWritebackInput<Request> {
    Absent,
    Invalid { safe_error_code: &'static str },
    Requested(Request),
}

impl<Request> SideEffectWritebackInput<Request>
where
    Request: DeserializeOwned,
{
    #[must_use]
    pub(crate) fn from_payload(
        payload: &serde_json::Value,
        field_name: &'static str,
        invalid_request_code: &'static str,
    ) -> Self {
        let Some(writeback) = payload.get(field_name) else {
            return Self::Absent;
        };

        match serde_json::from_value::<Request>(writeback.clone()) {
            Ok(writeback_request) => Self::Requested(writeback_request),
            Err(_) => Self::Invalid {
                safe_error_code: invalid_request_code,
            },
        }
    }
}

pub(crate) trait SideEffectWritebackRequest {
    fn library_id(&self) -> &str;
    fn target(&self) -> &NakoSideEffectTarget;
    fn idempotency_key(&self) -> &str;
}

#[async_trait]
pub(crate) trait SideEffectWritebackAdapter<T>: Sync
where
    T: NakoRuntimeTransport,
{
    type Request: SideEffectWritebackRequest + Send + Sync;
    type Prepared: Send;
    type Result;

    fn operation_name(&self) -> &'static str;
    fn permission(&self) -> NakoPermission;
    fn validate_target(&self, target: &NakoSideEffectTarget) -> Result<(), &'static str>;
    fn prepare(&self, request: &Self::Request) -> Result<Self::Prepared, &'static str>;
    fn skipped(&self, safe_error_code: String) -> Self::Result;
    fn failed(&self, safe_error_code: String) -> Self::Result;
    fn submitted(&self, side_effect: NakoSideEffectSummary) -> Self::Result;

    async fn submit(
        &self,
        runtime: &NakoRuntimeClient<T>,
        request: &Self::Request,
        prepared: Self::Prepared,
    ) -> NakoRuntimeResult<NakoSideEffectResponse>;
}

pub(crate) async fn run_side_effect_writeback<T, A>(
    runtime: Option<&NakoRuntimeClient<T>>,
    request_id: &str,
    input: SideEffectWritebackInput<A::Request>,
    adapter: A,
) -> Option<A::Result>
where
    T: NakoRuntimeTransport,
    A: SideEffectWritebackAdapter<T>,
{
    let request = match input {
        SideEffectWritebackInput::Absent => return None,
        SideEffectWritebackInput::Invalid { safe_error_code } => {
            return Some(adapter.skipped(safe_error_code.to_owned()));
        }
        SideEffectWritebackInput::Requested(request) => request,
    };

    if let Err(safe_error_code) = adapter.validate_target(request.target()) {
        return Some(adapter.skipped(safe_error_code.to_owned()));
    }

    let prepared = match adapter.prepare(&request) {
        Ok(prepared) => prepared,
        Err(safe_error_code) => return Some(adapter.skipped(safe_error_code.to_owned())),
    };

    let Some(runtime) = runtime else {
        return Some(adapter.skipped("nako_runtime_disabled".to_owned()));
    };

    let access = runtime
        .access_check(NakoAccessCheckRequest {
            permission: adapter.permission(),
            library_id: Some(request.library_id().to_owned()),
        })
        .await;
    let Ok(access) = access else {
        tracing::warn!(
            request_id = %request_id,
            operation = adapter.operation_name(),
            "writeback access check failed"
        );
        return Some(adapter.skipped("access_check_failed".to_owned()));
    };
    if !access.allowed {
        return Some(adapter.skipped("access_denied".to_owned()));
    }

    match adapter.submit(runtime, &request, prepared).await {
        Ok(response) => Some(adapter.submitted(response.side_effect)),
        Err(error) => {
            tracing::warn!(
                request_id = %request_id,
                operation = adapter.operation_name(),
                safe_error_code = error.safe_code(),
                "writeback submission failed"
            );
            Some(adapter.failed(error.safe_code().to_owned()))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::VecDeque,
        sync::{Arc, Mutex},
    };

    use super::*;
    use crate::nako_runtime::{
        NakoRuntimeClientConfig, NakoRuntimeError, NakoRuntimeHttpRequest, NakoRuntimeHttpResponse,
        SubmitNakoSideEffectRequest,
    };

    #[tokio::test]
    async fn side_effect_state_machine_skips_invalid_target_before_access_check() {
        let transport = FakeTransport::default();
        let runtime = test_client(transport.clone());

        let result = run_side_effect_writeback(
            Some(&runtime),
            "request-1",
            SideEffectWritebackInput::Requested(test_request(media_item_target())),
            TestAdapter::metadata_write(),
        )
        .await
        .expect("expected writeback summary");

        assert_eq!(result.status, TestStatus::Skipped);
        assert_eq!(
            result.safe_error_code.as_deref(),
            Some("invalid_target_kind")
        );
        assert!(transport.requests().is_empty());
    }

    #[tokio::test]
    async fn side_effect_state_machine_skips_when_runtime_is_disabled() {
        let result = run_side_effect_writeback::<FakeTransport, _>(
            None,
            "request-1",
            SideEffectWritebackInput::Requested(test_request(media_source_target())),
            TestAdapter::metadata_write(),
        )
        .await
        .expect("expected writeback summary");

        assert_eq!(result.status, TestStatus::Skipped);
        assert_eq!(
            result.safe_error_code.as_deref(),
            Some("nako_runtime_disabled")
        );
    }

    #[tokio::test]
    async fn side_effect_state_machine_skips_access_check_failure() {
        let transport = FakeTransport::default();
        transport.push(Err(NakoRuntimeError::Http {
            message: "access failed".to_owned(),
        }));
        let runtime = test_client(transport.clone());

        let result = run_side_effect_writeback(
            Some(&runtime),
            "request-1",
            SideEffectWritebackInput::Requested(test_request(media_source_target())),
            TestAdapter::metadata_write(),
        )
        .await
        .expect("expected writeback summary");

        assert_eq!(result.status, TestStatus::Skipped);
        assert_eq!(
            result.safe_error_code.as_deref(),
            Some("access_check_failed")
        );
        assert_eq!(transport.requests().len(), 1);
    }

    #[tokio::test]
    async fn side_effect_state_machine_skips_access_denied() {
        let transport = FakeTransport::default();
        transport.push(access_response(false));
        let runtime = test_client(transport.clone());

        let result = run_side_effect_writeback(
            Some(&runtime),
            "request-1",
            SideEffectWritebackInput::Requested(test_request(media_source_target())),
            TestAdapter::metadata_write(),
        )
        .await
        .expect("expected writeback summary");

        assert_eq!(result.status, TestStatus::Skipped);
        assert_eq!(result.safe_error_code.as_deref(), Some("access_denied"));
        assert_eq!(transport.requests().len(), 1);
    }

    #[tokio::test]
    async fn side_effect_state_machine_reports_submit_failure() {
        let transport = FakeTransport::default();
        transport.push(access_response(true));
        transport.push(Err(NakoRuntimeError::Http {
            message: "submit failed".to_owned(),
        }));
        let runtime = test_client(transport.clone());

        let result = run_side_effect_writeback(
            Some(&runtime),
            "request-1",
            SideEffectWritebackInput::Requested(test_request(media_source_target())),
            TestAdapter::metadata_write(),
        )
        .await
        .expect("expected writeback summary");

        assert_eq!(result.status, TestStatus::Failed);
        assert_eq!(result.safe_error_code.as_deref(), Some("transport_error"));
        assert_eq!(transport.requests().len(), 2);
    }

    #[tokio::test]
    async fn side_effect_state_machine_reports_success() {
        let transport = FakeTransport::default();
        transport.push(access_response(true));
        transport.push(side_effect_response());
        let runtime = test_client(transport.clone());

        let result = run_side_effect_writeback(
            Some(&runtime),
            "request-1",
            SideEffectWritebackInput::Requested(test_request(media_source_target())),
            TestAdapter::metadata_write(),
        )
        .await
        .expect("expected writeback summary");

        assert_eq!(result.status, TestStatus::Submitted);
        assert_eq!(result.safe_error_code, None);
        assert_eq!(
            result.side_effect.as_ref().map(|effect| effect.id.as_str()),
            Some("effect-1")
        );
        assert_eq!(transport.requests().len(), 2);
    }

    #[derive(Clone, Debug, Eq, PartialEq)]
    struct TestRequest {
        library_id: String,
        target: NakoSideEffectTarget,
        idempotency_key: String,
    }

    impl SideEffectWritebackRequest for TestRequest {
        fn library_id(&self) -> &str {
            &self.library_id
        }

        fn target(&self) -> &NakoSideEffectTarget {
            &self.target
        }

        fn idempotency_key(&self) -> &str {
            &self.idempotency_key
        }
    }

    struct TestAdapter {
        permission: NakoPermission,
        required_target: crate::nako_runtime::NakoSideEffectTargetKind,
    }

    impl TestAdapter {
        fn metadata_write() -> Self {
            Self {
                permission: NakoPermission::MetadataWrite,
                required_target: crate::nako_runtime::NakoSideEffectTargetKind::MediaSource,
            }
        }
    }

    #[async_trait]
    impl SideEffectWritebackAdapter<FakeTransport> for TestAdapter {
        type Request = TestRequest;
        type Prepared = serde_json::Value;
        type Result = TestResult;

        fn operation_name(&self) -> &'static str {
            "test writeback"
        }

        fn permission(&self) -> NakoPermission {
            self.permission
        }

        fn validate_target(&self, target: &NakoSideEffectTarget) -> Result<(), &'static str> {
            (target.kind == self.required_target)
                .then_some(())
                .ok_or("invalid_target_kind")
        }

        fn prepare(&self, _request: &Self::Request) -> Result<Self::Prepared, &'static str> {
            Ok(serde_json::json!({"title": "The Matrix"}))
        }

        fn skipped(&self, safe_error_code: String) -> Self::Result {
            TestResult {
                status: TestStatus::Skipped,
                safe_error_code: Some(safe_error_code),
                side_effect: None,
            }
        }

        fn failed(&self, safe_error_code: String) -> Self::Result {
            TestResult {
                status: TestStatus::Failed,
                safe_error_code: Some(safe_error_code),
                side_effect: None,
            }
        }

        fn submitted(&self, side_effect: NakoSideEffectSummary) -> Self::Result {
            TestResult {
                status: TestStatus::Submitted,
                safe_error_code: None,
                side_effect: Some(side_effect),
            }
        }

        async fn submit(
            &self,
            runtime: &NakoRuntimeClient<FakeTransport>,
            request: &Self::Request,
            prepared: Self::Prepared,
        ) -> NakoRuntimeResult<NakoSideEffectResponse> {
            runtime
                .submit_side_effect(SubmitNakoSideEffectRequest {
                    permission: self.permission,
                    library_id: request.library_id().to_owned(),
                    target: request.target().clone(),
                    idempotency_key: request.idempotency_key().to_owned(),
                    provenance: serde_json::json!({"origin": "test"}),
                    payload: prepared,
                })
                .await
        }
    }

    #[derive(Clone, Debug, Eq, PartialEq)]
    struct TestResult {
        status: TestStatus,
        safe_error_code: Option<String>,
        side_effect: Option<NakoSideEffectSummary>,
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum TestStatus {
        Submitted,
        Skipped,
        Failed,
    }

    fn test_request(target: NakoSideEffectTarget) -> TestRequest {
        TestRequest {
            library_id: "library-1".to_owned(),
            target,
            idempotency_key: "writeback-1".to_owned(),
        }
    }

    fn media_source_target() -> NakoSideEffectTarget {
        NakoSideEffectTarget {
            kind: crate::nako_runtime::NakoSideEffectTargetKind::MediaSource,
            id: "source-1".to_owned(),
        }
    }

    fn media_item_target() -> NakoSideEffectTarget {
        NakoSideEffectTarget {
            kind: crate::nako_runtime::NakoSideEffectTargetKind::MediaItem,
            id: "item-1".to_owned(),
        }
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

    fn access_response(allowed: bool) -> NakoRuntimeResult<NakoRuntimeHttpResponse> {
        Ok(NakoRuntimeHttpResponse {
            status: 200,
            body: serde_json::json!({
                "addon_id": "addon-1",
                "token_id": "token-1",
                "permission": "metadata_write",
                "library_id": "library-1",
                "allowed": allowed
            })
            .to_string(),
        })
    }

    fn side_effect_response() -> NakoRuntimeResult<NakoRuntimeHttpResponse> {
        Ok(NakoRuntimeHttpResponse {
            status: 200,
            body: serde_json::json!({
                "side_effect": {
                    "id": "effect-1",
                    "addon_id": "addon-1",
                    "token_id": "token-1",
                    "permission": "metadata_write",
                    "library_id": "library-1",
                    "target": {"kind": "media_source", "id": "source-1"},
                    "idempotency_key": "writeback-1",
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
        })
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
                        message: "fake response queue was empty".to_owned(),
                    })
                })
        }
    }
}
