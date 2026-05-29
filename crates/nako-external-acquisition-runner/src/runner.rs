use std::{collections::BTreeMap, sync::Arc};

use nako_addon_protocol::{
    AddonExternalAcquisitionActionRequest, AddonExternalAcquisitionActionResponse,
    AddonExternalAcquisitionActionStatus, AddonExternalAcquisitionOperation,
    AddonExternalAcquisitionProgress, AddonExternalAcquisitionRunnerState,
    AddonExternalAcquisitionTargetRef,
};
use tokio::sync::Mutex;

use crate::{
    Config,
    materialization::{
        ExternalAcquisitionMaterializer, FixtureActionContext, MaterializationError,
        SharedMaterializer, materialization_request, materialized_target_kind,
        materializer_from_config,
    },
};

#[derive(Clone, Debug)]
pub struct FixtureRunner {
    config: Config,
    state: Arc<Mutex<FixtureRunnerState>>,
    materializer: SharedMaterializer,
}

#[derive(Clone, Debug, Default)]
struct FixtureRunnerState {
    next_job_index: u64,
    idempotency_index: BTreeMap<String, String>,
    jobs: BTreeMap<String, FixtureJob>,
}

#[derive(Clone, Debug)]
struct FixtureJob {
    runner_job_ref: String,
    runner_profile_id: String,
    target_kind: &'static str,
    materialization: FixtureMaterializationSummary,
    state: AddonExternalAcquisitionRunnerState,
    progress: AddonExternalAcquisitionProgress,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct FixtureMaterializationSummary {
    client_kind: &'static str,
    link_type: String,
    password_present: bool,
    materialization_ref_present: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FixtureActionError {
    ProfileUnavailable,
    InvalidStatusTarget,
    JobNotFound,
    Materialization(MaterializationError),
}

impl FixtureActionError {
    #[must_use]
    pub fn safe_error_code(&self) -> String {
        match self {
            Self::ProfileUnavailable => "runner_profile_unavailable".to_owned(),
            Self::InvalidStatusTarget => "runner_job_ref_required".to_owned(),
            Self::JobNotFound => "runner_job_not_found".to_owned(),
            Self::Materialization(error) => error.safe_error_code(),
        }
    }

    #[must_use]
    pub fn to_response(&self) -> AddonExternalAcquisitionActionResponse {
        let status = match self {
            Self::JobNotFound => AddonExternalAcquisitionActionStatus::NotFound,
            Self::ProfileUnavailable | Self::InvalidStatusTarget | Self::Materialization(_) => {
                AddonExternalAcquisitionActionStatus::Rejected
            }
        };

        AddonExternalAcquisitionActionResponse {
            schema: crate::manifest::ACTION_RESPONSE_SCHEMA.to_owned(),
            status,
            state: AddonExternalAcquisitionRunnerState::Unknown,
            runner_job_ref: None,
            progress: None,
            retryable: false,
            retry_after_ms: None,
            safe_message: Some(self.safe_error_code()),
            safe_facts: BTreeMap::new(),
        }
    }
}

impl FixtureRunner {
    #[must_use]
    pub fn new(config: Config) -> Self {
        let materializer = materializer_from_config(&config);
        Self::with_materializer(config, materializer)
    }

    #[must_use]
    pub fn with_materializer(
        config: Config,
        materializer: Arc<dyn ExternalAcquisitionMaterializer>,
    ) -> Self {
        Self {
            config,
            state: Arc::new(Mutex::new(FixtureRunnerState::default())),
            materializer,
        }
    }

    #[must_use]
    pub fn diagnostics(&self) -> serde_json::Value {
        serde_json::json!({
            "safe_note": "external acquisition fixture runner is reachable",
            "external_network": self.config.transmission.enabled,
            "profile_registry": self.safe_profile_registry(),
            "active_profile_count": self.config.active_profile_count(),
            "materialization_client": self.materializer.safe_client_kind(),
            "supported_operations": ["enqueue", "cancel", "pause", "resume", "query_status"]
        })
    }

    #[must_use]
    fn safe_profile_registry(&self) -> serde_json::Value {
        let mut profiles = vec![serde_json::json!({
            "runner_profile_id": self.config.default_runner_profile_id,
            "active": self.config.fixture_profile_enabled,
            "mode": "noop",
            "kind": "fixture"
        })];

        profiles.push(serde_json::json!({
            "runner_profile_id": self.config.transmission.profile_id,
            "active": false,
            "enabled": self.config.transmission.enabled,
            "mode": "rpc",
            "kind": "transmission",
            "implementation_status": "configuration_ready",
            "endpoint_configured": !self.config.transmission.rpc_url.trim().is_empty(),
            "auth_configured": self.config.transmission.auth_configured(),
            "timeout_ms": self.config.transmission.timeout_ms,
            "allow_invalid_tls_certificates": self.config.transmission.allow_invalid_tls_certificates
        }));

        serde_json::Value::Array(profiles)
    }

    pub async fn handle_action(
        &self,
        context: FixtureActionContext,
        request: AddonExternalAcquisitionActionRequest,
    ) -> Result<AddonExternalAcquisitionActionResponse, FixtureActionError> {
        if !self.config.fixture_profile_enabled
            || request.runner_profile_id != self.config.default_runner_profile_id
        {
            return Err(FixtureActionError::ProfileUnavailable);
        }

        match request.operation {
            AddonExternalAcquisitionOperation::Enqueue => self.enqueue(context, request).await,
            AddonExternalAcquisitionOperation::Cancel => {
                self.transition_job(
                    &request,
                    AddonExternalAcquisitionRunnerState::Cancelled,
                    AddonExternalAcquisitionActionStatus::Accepted,
                )
                .await
            }
            AddonExternalAcquisitionOperation::Pause => {
                self.transition_job(
                    &request,
                    AddonExternalAcquisitionRunnerState::Paused,
                    AddonExternalAcquisitionActionStatus::Accepted,
                )
                .await
            }
            AddonExternalAcquisitionOperation::Resume => {
                self.transition_job(
                    &request,
                    AddonExternalAcquisitionRunnerState::Running,
                    AddonExternalAcquisitionActionStatus::Accepted,
                )
                .await
            }
            AddonExternalAcquisitionOperation::QueryStatus => self.query_status(&request).await,
        }
    }

    async fn enqueue(
        &self,
        context: FixtureActionContext,
        request: AddonExternalAcquisitionActionRequest,
    ) -> Result<AddonExternalAcquisitionActionResponse, FixtureActionError> {
        {
            let state = self.state.lock().await;
            if let Some(existing_ref) = state.idempotency_index.get(&request.idempotency_key)
                && let Some(job) = state.jobs.get(existing_ref)
            {
                return Ok(job.response(
                    AddonExternalAcquisitionActionStatus::AlreadyExists,
                    request.operation,
                    true,
                ));
            }
        }

        let materialization = self
            .materializer
            .materialize(
                materialization_request(&context, &request)
                    .map_err(FixtureActionError::Materialization)?,
            )
            .await
            .map_err(FixtureActionError::Materialization)?;
        let materialization = FixtureMaterializationSummary {
            client_kind: self.materializer.safe_client_kind(),
            link_type: materialization.material.link_type.as_str().to_owned(),
            password_present: materialization.material.password.is_some(),
            materialization_ref_present: !materialization.materialization_ref.trim().is_empty(),
        };

        let mut state = self.state.lock().await;
        if let Some(existing_ref) = state.idempotency_index.get(&request.idempotency_key)
            && let Some(job) = state.jobs.get(existing_ref)
        {
            return Ok(job.response(
                AddonExternalAcquisitionActionStatus::AlreadyExists,
                request.operation,
                true,
            ));
        }
        state.next_job_index += 1;
        let runner_job_ref = format!("fixture-job-{}", state.next_job_index);
        let job = FixtureJob {
            runner_job_ref: runner_job_ref.clone(),
            runner_profile_id: request.runner_profile_id,
            target_kind: materialized_target_kind(&request.target_ref),
            materialization,
            state: AddonExternalAcquisitionRunnerState::Running,
            progress: AddonExternalAcquisitionProgress {
                percent_milli: Some(0),
                downloaded_bytes: Some(0),
                total_bytes: None,
            },
        };
        state
            .idempotency_index
            .insert(request.idempotency_key, runner_job_ref.clone());
        state.jobs.insert(runner_job_ref, job.clone());

        Ok(job.response(
            AddonExternalAcquisitionActionStatus::Accepted,
            request.operation,
            false,
        ))
    }

    async fn transition_job(
        &self,
        request: &AddonExternalAcquisitionActionRequest,
        next_state: AddonExternalAcquisitionRunnerState,
        status: AddonExternalAcquisitionActionStatus,
    ) -> Result<AddonExternalAcquisitionActionResponse, FixtureActionError> {
        let runner_job_ref = runner_job_ref(&request.target_ref)?;
        let mut state = self.state.lock().await;
        let job = state
            .jobs
            .get_mut(runner_job_ref)
            .ok_or(FixtureActionError::JobNotFound)?;
        job.state = next_state;
        if next_state.is_terminal() {
            job.progress.percent_milli = Some(100_000);
        }

        Ok(job.response(status, request.operation, false))
    }

    async fn query_status(
        &self,
        request: &AddonExternalAcquisitionActionRequest,
    ) -> Result<AddonExternalAcquisitionActionResponse, FixtureActionError> {
        let runner_job_ref = runner_job_ref(&request.target_ref)?;
        let state = self.state.lock().await;
        let job = state
            .jobs
            .get(runner_job_ref)
            .ok_or(FixtureActionError::JobNotFound)?;

        Ok(job.response(
            AddonExternalAcquisitionActionStatus::Accepted,
            request.operation,
            false,
        ))
    }
}

impl FixtureJob {
    fn response(
        &self,
        status: AddonExternalAcquisitionActionStatus,
        operation: AddonExternalAcquisitionOperation,
        idempotent_replay: bool,
    ) -> AddonExternalAcquisitionActionResponse {
        let mut safe_facts = BTreeMap::new();
        safe_facts.insert(
            "runner_profile_id".to_owned(),
            self.runner_profile_id.clone(),
        );
        safe_facts.insert("target_kind".to_owned(), self.target_kind.to_owned());
        safe_facts.insert("operation".to_owned(), operation.as_str().to_owned());
        safe_facts.insert("fixture".to_owned(), "true".to_owned());
        safe_facts.insert(
            "materialization_client".to_owned(),
            self.materialization.client_kind.to_owned(),
        );
        safe_facts.insert(
            "materialized_link_type".to_owned(),
            self.materialization.link_type.clone(),
        );
        safe_facts.insert(
            "materialized_password_present".to_owned(),
            self.materialization.password_present.to_string(),
        );
        safe_facts.insert(
            "materialization_ref_present".to_owned(),
            self.materialization.materialization_ref_present.to_string(),
        );
        safe_facts.insert(
            "idempotent_replay".to_owned(),
            idempotent_replay.to_string(),
        );

        AddonExternalAcquisitionActionResponse {
            schema: crate::manifest::ACTION_RESPONSE_SCHEMA.to_owned(),
            status,
            state: self.state,
            runner_job_ref: Some(self.runner_job_ref.clone()),
            progress: Some(self.progress.clone()),
            retryable: false,
            retry_after_ms: None,
            safe_message: Some("fixture_noop".to_owned()),
            safe_facts,
        }
    }
}

fn runner_job_ref(
    target_ref: &AddonExternalAcquisitionTargetRef,
) -> Result<&str, FixtureActionError> {
    match target_ref {
        AddonExternalAcquisitionTargetRef::RunnerJob { runner_job_ref } => Ok(runner_job_ref),
        AddonExternalAcquisitionTargetRef::SelectedLink { .. }
        | AddonExternalAcquisitionTargetRef::IntakeCandidate { .. } => {
            Err(FixtureActionError::InvalidStatusTarget)
        }
    }
}

#[cfg(test)]
mod tests {
    use nako_addon_protocol::{
        AddonExternalAcquisitionMaterializationRequest,
        AddonExternalAcquisitionMaterializationResponse, AddonExternalAcquisitionMaterializedLink,
        AddonExternalAcquisitionOperation, AddonExternalAcquisitionTargetRef,
        AddonResourceLinkType,
    };
    use std::sync::Mutex as StdMutex;

    use super::*;
    use crate::{
        manifest::ACTION_TASK_ID,
        materialization::{FixtureActionContext, local_action_context},
    };

    #[tokio::test]
    async fn fixture_runner_preserves_idempotent_enqueue() {
        let runner = FixtureRunner::new(Config::default());

        let first = runner
            .handle_action(local_action_context("job-1"), enqueue_request("idem-1"))
            .await
            .unwrap();
        let second = runner
            .handle_action(local_action_context("job-1"), enqueue_request("idem-1"))
            .await
            .unwrap();

        assert_eq!(first.status, AddonExternalAcquisitionActionStatus::Accepted);
        assert_eq!(
            second.status,
            AddonExternalAcquisitionActionStatus::AlreadyExists
        );
        assert_eq!(first.runner_job_ref, second.runner_job_ref);
        assert_eq!(
            second
                .safe_facts
                .get("idempotent_replay")
                .map(String::as_str),
            Some("true")
        );
    }

    #[tokio::test]
    async fn fixture_runner_materialization_enqueue_without_leaking_material() {
        let materializer = RecordingMaterializer::with_response(materialization_response());
        let runner =
            FixtureRunner::with_materializer(Config::default(), Arc::new(materializer.clone()));

        let response = runner
            .handle_action(
                FixtureActionContext::new("job-secret", ACTION_TASK_ID),
                enqueue_request("idem-materialization"),
            )
            .await
            .unwrap();

        assert_eq!(
            response.status,
            AddonExternalAcquisitionActionStatus::Accepted
        );
        assert_eq!(
            response
                .safe_facts
                .get("materialization_client")
                .map(String::as_str),
            Some("recording")
        );
        assert_eq!(
            response
                .safe_facts
                .get("materialized_link_type")
                .map(String::as_str),
            Some("magnet")
        );
        assert_eq!(
            response
                .safe_facts
                .get("materialized_password_present")
                .map(String::as_str),
            Some("true")
        );
        assert_eq!(
            response
                .safe_facts
                .get("materialization_ref_present")
                .map(String::as_str),
            Some("true")
        );

        let response_text = serde_json::to_string(&response).unwrap();
        for forbidden in [
            "magnet:?xt=urn:btih:secret",
            "secret-code",
            "materialization-secret",
            "selected-link-ref",
            "idem-materialization",
        ] {
            assert!(
                !response_text.contains(forbidden),
                "runner response leaked forbidden term: {forbidden}"
            );
        }

        let requests = materializer.requests();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].job_id, "job-secret");
        assert_eq!(requests[0].declaration_id, ACTION_TASK_ID);
        assert_eq!(requests[0].runner_profile_id, "fixture");
        assert_eq!(requests[0].idempotency_key, "idem-materialization");
        assert_eq!(
            requests[0].operation,
            AddonExternalAcquisitionOperation::Enqueue
        );
        assert_eq!(requests[0].audit_ref, "audit-ref");
        assert_eq!(
            requests[0].target_ref,
            AddonExternalAcquisitionTargetRef::SelectedLink {
                selected_link_ref: "selected-link-ref".to_owned()
            }
        );
    }

    #[tokio::test]
    async fn fixture_runner_rejects_materialization_failure_without_echoing_target() {
        let materializer = RecordingMaterializer::with_error(MaterializationError::HostRejected {
            safe_code: "http_client_error".to_owned(),
        });
        let runner = FixtureRunner::with_materializer(Config::default(), Arc::new(materializer));

        let error = runner
            .handle_action(
                FixtureActionContext::new("job-secret", ACTION_TASK_ID),
                enqueue_request("idem-secret"),
            )
            .await
            .unwrap_err();
        let response = error.to_response();

        assert_eq!(
            response.safe_message.as_deref(),
            Some("materialization_http_client_error")
        );
        let response_text = serde_json::to_string(&response).unwrap();
        for forbidden in ["selected-link-ref", "idem-secret", "job-secret"] {
            assert!(
                !response_text.contains(forbidden),
                "materialization failure leaked forbidden term: {forbidden}"
            );
        }
    }

    #[tokio::test]
    async fn fixture_runner_cancels_and_reports_status() {
        let runner = FixtureRunner::new(Config::default());
        let enqueued = runner
            .handle_action(local_action_context("job-2"), enqueue_request("idem-2"))
            .await
            .unwrap();
        let runner_job_ref = enqueued.runner_job_ref.clone().unwrap();

        let cancelled = runner
            .handle_action(
                local_action_context("job-cancel-2"),
                job_request(AddonExternalAcquisitionOperation::Cancel, &runner_job_ref),
            )
            .await
            .unwrap();
        let status = runner
            .handle_action(
                local_action_context("job-status-2"),
                job_request(
                    AddonExternalAcquisitionOperation::QueryStatus,
                    &runner_job_ref,
                ),
            )
            .await
            .unwrap();

        assert_eq!(
            cancelled.state,
            AddonExternalAcquisitionRunnerState::Cancelled
        );
        assert_eq!(status.state, AddonExternalAcquisitionRunnerState::Cancelled);
        assert!(status.state.is_terminal());
    }

    fn enqueue_request(idempotency_key: &str) -> AddonExternalAcquisitionActionRequest {
        AddonExternalAcquisitionActionRequest {
            schema: crate::manifest::ACTION_REQUEST_SCHEMA.to_owned(),
            target_ref: AddonExternalAcquisitionTargetRef::SelectedLink {
                selected_link_ref: "selected-link-ref".to_owned(),
            },
            runner_profile_id: "fixture".to_owned(),
            idempotency_key: idempotency_key.to_owned(),
            operation: AddonExternalAcquisitionOperation::Enqueue,
            audit_ref: Some("audit-ref".to_owned()),
        }
    }

    fn job_request(
        operation: AddonExternalAcquisitionOperation,
        runner_job_ref: &str,
    ) -> AddonExternalAcquisitionActionRequest {
        AddonExternalAcquisitionActionRequest {
            schema: crate::manifest::ACTION_REQUEST_SCHEMA.to_owned(),
            target_ref: AddonExternalAcquisitionTargetRef::RunnerJob {
                runner_job_ref: runner_job_ref.to_owned(),
            },
            runner_profile_id: "fixture".to_owned(),
            idempotency_key: format!("idem-{operation:?}"),
            operation,
            audit_ref: None,
        }
    }

    #[derive(Clone, Debug)]
    struct RecordingMaterializer {
        requests: Arc<StdMutex<Vec<AddonExternalAcquisitionMaterializationRequest>>>,
        response: Result<AddonExternalAcquisitionMaterializationResponse, MaterializationError>,
    }

    impl RecordingMaterializer {
        fn with_response(response: AddonExternalAcquisitionMaterializationResponse) -> Self {
            Self {
                requests: Arc::default(),
                response: Ok(response),
            }
        }

        fn with_error(error: MaterializationError) -> Self {
            Self {
                requests: Arc::default(),
                response: Err(error),
            }
        }

        fn requests(&self) -> Vec<AddonExternalAcquisitionMaterializationRequest> {
            self.requests.lock().unwrap().clone()
        }
    }

    #[async_trait::async_trait]
    impl ExternalAcquisitionMaterializer for RecordingMaterializer {
        fn safe_client_kind(&self) -> &'static str {
            "recording"
        }

        async fn materialize(
            &self,
            request: AddonExternalAcquisitionMaterializationRequest,
        ) -> Result<AddonExternalAcquisitionMaterializationResponse, MaterializationError> {
            self.requests.lock().unwrap().push(request);
            self.response.clone()
        }
    }

    fn materialization_response() -> AddonExternalAcquisitionMaterializationResponse {
        AddonExternalAcquisitionMaterializationResponse {
            schema: nako_addon_protocol::ADDON_EXTERNAL_ACQUISITION_MATERIALIZATION_RESPONSE_SCHEMA
                .to_owned(),
            materialization_ref: "materialization-secret".to_owned(),
            target_ref: AddonExternalAcquisitionTargetRef::SelectedLink {
                selected_link_ref: "selected-link-ref".to_owned(),
            },
            expires_at: "2026-05-29T00:01:00.000Z".to_owned(),
            material: AddonExternalAcquisitionMaterializedLink {
                link_type: AddonResourceLinkType::Magnet,
                uri: "magnet:?xt=urn:btih:secret".to_owned(),
                password: Some("secret-code".to_owned()),
            },
            safe_facts: BTreeMap::new(),
        }
    }
}
