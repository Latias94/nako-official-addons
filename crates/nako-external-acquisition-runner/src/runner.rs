use std::{collections::BTreeMap, sync::Arc};

use nako_addon_protocol::{
    AddonExternalAcquisitionActionRequest, AddonExternalAcquisitionActionResponse,
    AddonExternalAcquisitionActionStatus, AddonExternalAcquisitionOperation,
    AddonExternalAcquisitionProgress, AddonExternalAcquisitionRunnerState,
    AddonExternalAcquisitionTargetRef,
};
use tokio::sync::Mutex;

use crate::Config;

#[derive(Clone, Debug)]
pub struct FixtureRunner {
    config: Config,
    state: Arc<Mutex<FixtureRunnerState>>,
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
    state: AddonExternalAcquisitionRunnerState,
    progress: AddonExternalAcquisitionProgress,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FixtureActionError {
    ProfileUnavailable,
    InvalidStatusTarget,
    JobNotFound,
}

impl FixtureActionError {
    #[must_use]
    pub const fn safe_error_code(self) -> &'static str {
        match self {
            Self::ProfileUnavailable => "runner_profile_unavailable",
            Self::InvalidStatusTarget => "runner_job_ref_required",
            Self::JobNotFound => "runner_job_not_found",
        }
    }

    #[must_use]
    pub fn to_response(self) -> AddonExternalAcquisitionActionResponse {
        let status = match self {
            Self::JobNotFound => AddonExternalAcquisitionActionStatus::NotFound,
            Self::ProfileUnavailable | Self::InvalidStatusTarget => {
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
            safe_message: Some(self.safe_error_code().to_owned()),
            safe_facts: BTreeMap::new(),
        }
    }
}

impl FixtureRunner {
    #[must_use]
    pub fn new(config: Config) -> Self {
        Self {
            config,
            state: Arc::new(Mutex::new(FixtureRunnerState::default())),
        }
    }

    #[must_use]
    pub fn diagnostics(&self) -> serde_json::Value {
        serde_json::json!({
            "safe_note": "external acquisition fixture runner is reachable",
            "external_network": false,
            "profile_registry": [{
                "runner_profile_id": self.config.default_runner_profile_id,
                "active": self.config.fixture_profile_enabled,
                "mode": "noop"
            }],
            "active_profile_count": self.config.active_profile_count(),
            "supported_operations": ["enqueue", "cancel", "pause", "resume", "query_status"]
        })
    }

    pub async fn handle_action(
        &self,
        request: AddonExternalAcquisitionActionRequest,
    ) -> Result<AddonExternalAcquisitionActionResponse, FixtureActionError> {
        if !self.config.fixture_profile_enabled
            || request.runner_profile_id != self.config.default_runner_profile_id
        {
            return Err(FixtureActionError::ProfileUnavailable);
        }

        match request.operation {
            AddonExternalAcquisitionOperation::Enqueue => self.enqueue(request).await,
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
        request: AddonExternalAcquisitionActionRequest,
    ) -> Result<AddonExternalAcquisitionActionResponse, FixtureActionError> {
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
            target_kind: target_kind(&request.target_ref),
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

fn target_kind(target_ref: &AddonExternalAcquisitionTargetRef) -> &'static str {
    match target_ref {
        AddonExternalAcquisitionTargetRef::SelectedLink { .. } => "selected_link",
        AddonExternalAcquisitionTargetRef::IntakeCandidate { .. } => "intake_candidate",
        AddonExternalAcquisitionTargetRef::RunnerJob { .. } => "runner_job",
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
        AddonExternalAcquisitionOperation, AddonExternalAcquisitionTargetRef,
    };

    use super::*;

    #[tokio::test]
    async fn fixture_runner_preserves_idempotent_enqueue() {
        let runner = FixtureRunner::new(Config::default());

        let first = runner
            .handle_action(enqueue_request("idem-1"))
            .await
            .unwrap();
        let second = runner
            .handle_action(enqueue_request("idem-1"))
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
    async fn fixture_runner_cancels_and_reports_status() {
        let runner = FixtureRunner::new(Config::default());
        let enqueued = runner
            .handle_action(enqueue_request("idem-2"))
            .await
            .unwrap();
        let runner_job_ref = enqueued.runner_job_ref.clone().unwrap();

        let cancelled = runner
            .handle_action(job_request(
                AddonExternalAcquisitionOperation::Cancel,
                &runner_job_ref,
            ))
            .await
            .unwrap();
        let status = runner
            .handle_action(job_request(
                AddonExternalAcquisitionOperation::QueryStatus,
                &runner_job_ref,
            ))
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
}
