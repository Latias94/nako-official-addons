use std::{collections::BTreeMap, fmt, sync::Arc};

use async_trait::async_trait;
use nako_addon_client::{
    AddonClientError, AddonTransport, NakoRuntimeClient, NakoRuntimeClientConfig,
    ReqwestAddonTransport,
};
use nako_addon_protocol::{
    ADDON_EXTERNAL_ACQUISITION_MATERIALIZATION_REQUEST_SCHEMA,
    ADDON_EXTERNAL_ACQUISITION_MATERIALIZATION_RESPONSE_SCHEMA,
    AddonExternalAcquisitionMaterializationPurpose, AddonExternalAcquisitionMaterializationRequest,
    AddonExternalAcquisitionMaterializationResponse, AddonExternalAcquisitionMaterializedLink,
    AddonExternalAcquisitionTargetRef, AddonResourceLinkType,
};

use crate::{Config, manifest::ACTION_TASK_ID};

pub type SharedMaterializer = Arc<dyn ExternalAcquisitionMaterializer>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FixtureActionContext {
    pub job_id: String,
    pub declaration_id: String,
}

impl FixtureActionContext {
    #[must_use]
    pub fn new(job_id: impl Into<String>, declaration_id: impl Into<String>) -> Self {
        Self {
            job_id: job_id.into(),
            declaration_id: declaration_id.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MaterializationError {
    AuditRefRequired,
    Unavailable,
    HostRejected { safe_code: String },
}

impl MaterializationError {
    #[must_use]
    pub fn safe_error_code(&self) -> String {
        match self {
            Self::AuditRefRequired => "audit_ref_required".to_owned(),
            Self::Unavailable => "materialization_unavailable".to_owned(),
            Self::HostRejected { safe_code } => format!("materialization_{safe_code}"),
        }
    }
}

#[async_trait]
pub trait ExternalAcquisitionMaterializer: fmt::Debug + Send + Sync {
    fn safe_client_kind(&self) -> &'static str;

    async fn materialize(
        &self,
        request: AddonExternalAcquisitionMaterializationRequest,
    ) -> Result<AddonExternalAcquisitionMaterializationResponse, MaterializationError>;
}

#[derive(Clone, Debug, Default)]
pub struct FixtureLocalMaterializer;

#[async_trait]
impl ExternalAcquisitionMaterializer for FixtureLocalMaterializer {
    fn safe_client_kind(&self) -> &'static str {
        "fixture_local"
    }

    async fn materialize(
        &self,
        request: AddonExternalAcquisitionMaterializationRequest,
    ) -> Result<AddonExternalAcquisitionMaterializationResponse, MaterializationError> {
        let mut safe_facts = BTreeMap::new();
        safe_facts.insert(
            "materialization_client".to_owned(),
            "fixture_local".to_owned(),
        );
        safe_facts.insert("link_type".to_owned(), "web".to_owned());

        Ok(AddonExternalAcquisitionMaterializationResponse {
            schema: ADDON_EXTERNAL_ACQUISITION_MATERIALIZATION_RESPONSE_SCHEMA.to_owned(),
            materialization_ref: "fixture-local-materialization".to_owned(),
            target_ref: request.target_ref,
            expires_at: "2026-05-29T00:01:00.000Z".to_owned(),
            material: AddonExternalAcquisitionMaterializedLink {
                link_type: AddonResourceLinkType::Web,
                uri: "https://fixture.invalid/external-acquisition/noop".to_owned(),
                password: None,
            },
            safe_facts,
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct UnavailableMaterializer;

#[async_trait]
impl ExternalAcquisitionMaterializer for UnavailableMaterializer {
    fn safe_client_kind(&self) -> &'static str {
        "unavailable"
    }

    async fn materialize(
        &self,
        _request: AddonExternalAcquisitionMaterializationRequest,
    ) -> Result<AddonExternalAcquisitionMaterializationResponse, MaterializationError> {
        Err(MaterializationError::Unavailable)
    }
}

#[derive(Clone)]
pub struct NakoRuntimeMaterializer<T = ReqwestAddonTransport>
where
    T: AddonTransport,
{
    client: NakoRuntimeClient<T>,
}

impl NakoRuntimeMaterializer<ReqwestAddonTransport> {
    #[must_use]
    pub fn new(config: NakoRuntimeClientConfig) -> Self {
        Self {
            client: NakoRuntimeClient::new(config),
        }
    }
}

impl<T> NakoRuntimeMaterializer<T>
where
    T: AddonTransport,
{
    #[must_use]
    pub fn with_transport(config: NakoRuntimeClientConfig, transport: T) -> Self {
        Self {
            client: NakoRuntimeClient::with_transport(config, transport),
        }
    }
}

impl<T> fmt::Debug for NakoRuntimeMaterializer<T>
where
    T: AddonTransport,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("NakoRuntimeMaterializer")
            .field("client", &"<redacted>")
            .finish()
    }
}

#[async_trait]
impl<T> ExternalAcquisitionMaterializer for NakoRuntimeMaterializer<T>
where
    T: AddonTransport + Send + Sync,
{
    fn safe_client_kind(&self) -> &'static str {
        "nako_runtime"
    }

    async fn materialize(
        &self,
        request: AddonExternalAcquisitionMaterializationRequest,
    ) -> Result<AddonExternalAcquisitionMaterializationResponse, MaterializationError> {
        self.client
            .materialize_external_acquisition(request)
            .await
            .map_err(MaterializationError::from)
    }
}

impl From<AddonClientError> for MaterializationError {
    fn from(value: AddonClientError) -> Self {
        Self::HostRejected {
            safe_code: value.safe_code().to_owned(),
        }
    }
}

#[must_use]
pub fn materializer_from_config(config: &Config) -> SharedMaterializer {
    if let Some(client_config) = config.nako_materialization.runtime_client_config() {
        return Arc::new(NakoRuntimeMaterializer::new(client_config));
    }
    if config.nako_materialization.enabled {
        return Arc::new(UnavailableMaterializer);
    }

    Arc::new(FixtureLocalMaterializer)
}

pub fn materialization_request(
    context: &FixtureActionContext,
    action: &nako_addon_protocol::AddonExternalAcquisitionActionRequest,
) -> Result<AddonExternalAcquisitionMaterializationRequest, MaterializationError> {
    let audit_ref = action
        .audit_ref
        .clone()
        .ok_or(MaterializationError::AuditRefRequired)?;

    Ok(AddonExternalAcquisitionMaterializationRequest {
        schema: ADDON_EXTERNAL_ACQUISITION_MATERIALIZATION_REQUEST_SCHEMA.to_owned(),
        job_id: context.job_id.clone(),
        declaration_id: context.declaration_id.clone(),
        target_ref: action.target_ref.clone(),
        runner_profile_id: action.runner_profile_id.clone(),
        idempotency_key: action.idempotency_key.clone(),
        operation: action.operation,
        audit_ref,
        purpose: AddonExternalAcquisitionMaterializationPurpose::ExternalAcquisitionEnqueue,
    })
}

#[must_use]
pub fn local_action_context(job_id: impl Into<String>) -> FixtureActionContext {
    FixtureActionContext::new(job_id, ACTION_TASK_ID)
}

#[must_use]
pub fn materialized_target_kind(target_ref: &AddonExternalAcquisitionTargetRef) -> &'static str {
    match target_ref {
        AddonExternalAcquisitionTargetRef::SelectedLink { .. } => "selected_link",
        AddonExternalAcquisitionTargetRef::IntakeCandidate { .. } => "intake_candidate",
        AddonExternalAcquisitionTargetRef::RunnerJob { .. } => "runner_job",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn materialization_runtime_materializer_debug_redacts_token() {
        let materializer = NakoRuntimeMaterializer::new(NakoRuntimeClientConfig {
            base_url: "https://nako.example".to_owned(),
            addon_token: "addon-token-secret".to_owned(),
            timeout_ms: 1500,
        });

        let debug = format!("{materializer:?}");

        assert!(debug.contains("<redacted>"));
        assert!(!debug.contains("addon-token-secret"));
        assert!(!debug.contains("https://nako.example"));
    }

    #[test]
    fn materialization_enabled_without_credentials_uses_unavailable_client() {
        let mut config = Config::default();
        config.nako_materialization.enabled = true;

        let materializer = materializer_from_config(&config);

        assert_eq!(materializer.safe_client_kind(), "unavailable");
    }
}
