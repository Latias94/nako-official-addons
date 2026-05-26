use std::sync::Arc;

use nako_addon_protocol::{AddonResourceRequest, AddonResourceResponse};

use crate::{
    nako_runtime::{NakoRuntimeClient, NakoRuntimeTransport},
    providers::MetadataProvider,
};

use super::{
    MetadataQuery, MetadataScrapeOutcome, ProviderExternalIdCapability, ProviderFieldPolicy,
    ProviderRunPolicy, QueryExternalIdAlias, artwork, orchestration, response, scrape, writeback,
};

#[derive(Clone)]
pub struct MetadataScrapeRuntime<T = crate::nako_runtime::ReqwestNakoRuntimeTransport>
where
    T: NakoRuntimeTransport,
{
    default_language: String,
    external_id_aliases: Arc<Vec<QueryExternalIdAlias>>,
    external_id_capabilities: Arc<Vec<ProviderExternalIdCapability>>,
    default_provider_field_policy: Arc<ProviderFieldPolicy>,
    providers: Arc<Vec<Box<dyn MetadataProvider>>>,
    nako_runtime: Option<NakoRuntimeClient<T>>,
}

impl<T> MetadataScrapeRuntime<T>
where
    T: NakoRuntimeTransport,
{
    #[must_use]
    pub fn new(
        default_language: impl Into<String>,
        providers: Vec<Box<dyn MetadataProvider>>,
        nako_runtime: Option<NakoRuntimeClient<T>>,
    ) -> Self {
        Self::with_external_id_aliases(default_language, Vec::new(), providers, nako_runtime)
    }

    #[must_use]
    pub fn with_external_id_aliases(
        default_language: impl Into<String>,
        external_id_aliases: Vec<QueryExternalIdAlias>,
        providers: Vec<Box<dyn MetadataProvider>>,
        nako_runtime: Option<NakoRuntimeClient<T>>,
    ) -> Self {
        Self::with_external_id_aliases_and_provider_field_policy(
            default_language,
            external_id_aliases,
            ProviderFieldPolicy::default(),
            providers,
            nako_runtime,
        )
    }

    #[must_use]
    pub fn with_external_id_aliases_and_provider_field_policy(
        default_language: impl Into<String>,
        external_id_aliases: Vec<QueryExternalIdAlias>,
        default_provider_field_policy: ProviderFieldPolicy,
        providers: Vec<Box<dyn MetadataProvider>>,
        nako_runtime: Option<NakoRuntimeClient<T>>,
    ) -> Self {
        Self {
            default_language: default_language.into(),
            external_id_aliases: Arc::new(external_id_aliases),
            external_id_capabilities: Arc::new(Vec::new()),
            default_provider_field_policy: Arc::new(default_provider_field_policy),
            providers: Arc::new(providers),
            nako_runtime,
        }
    }

    #[must_use]
    pub fn with_external_id_capabilities(
        default_language: impl Into<String>,
        external_id_capabilities: Vec<ProviderExternalIdCapability>,
        providers: Vec<Box<dyn MetadataProvider>>,
        nako_runtime: Option<NakoRuntimeClient<T>>,
    ) -> Self {
        Self::with_external_id_capabilities_and_provider_field_policy(
            default_language,
            external_id_capabilities,
            ProviderFieldPolicy::default(),
            providers,
            nako_runtime,
        )
    }

    #[must_use]
    pub fn with_external_id_capabilities_and_provider_field_policy(
        default_language: impl Into<String>,
        external_id_capabilities: Vec<ProviderExternalIdCapability>,
        default_provider_field_policy: ProviderFieldPolicy,
        providers: Vec<Box<dyn MetadataProvider>>,
        nako_runtime: Option<NakoRuntimeClient<T>>,
    ) -> Self {
        Self {
            default_language: default_language.into(),
            external_id_aliases: Arc::new(Vec::new()),
            external_id_capabilities: Arc::new(external_id_capabilities),
            default_provider_field_policy: Arc::new(default_provider_field_policy),
            providers: Arc::new(providers),
            nako_runtime,
        }
    }

    pub async fn scrape(&self, request: AddonResourceRequest) -> AddonResourceResponse {
        let outcome = self
            .scrape_outcome(&request.request_id, &request.payload)
            .await;
        response::metadata_response(request, outcome)
    }

    pub(crate) async fn scrape_outcome(
        &self,
        request_id: &str,
        payload: &serde_json::Value,
    ) -> MetadataScrapeOutcome {
        let query = if self.external_id_capabilities.is_empty() {
            MetadataQuery::from_payload_with_external_id_aliases(
                payload,
                &self.default_language,
                self.external_id_aliases.as_ref().as_slice(),
            )
        } else {
            MetadataQuery::from_payload_with_external_id_capabilities(
                payload,
                &self.default_language,
                self.external_id_capabilities.as_ref().as_slice(),
            )
        };
        let writeback_request = writeback::metadata_writeback_input_from_payload(payload);
        let artwork_writeback_request = artwork::artwork_writeback_input_from_payload(payload);
        let provider_field_policy = ProviderFieldPolicy::from_payload_or_default(
            payload,
            self.default_provider_field_policy.as_ref(),
        );
        let provider_run_policy = ProviderRunPolicy::from_payload(payload);
        let suggestions = orchestration::suggest_candidates(
            self.providers.as_ref().as_slice(),
            &query,
            self.external_id_capabilities.as_ref().as_slice(),
            &provider_field_policy,
            &provider_run_policy,
        )
        .await;
        let selected_candidate = suggestions.candidates.first().cloned();
        let writeback_result = writeback::maybe_submit_metadata_writeback(
            self.nako_runtime.as_ref(),
            request_id,
            &query,
            selected_candidate.as_ref(),
            writeback_request,
        )
        .await;
        let artwork_writeback_result = writeback::maybe_submit_artwork_writeback(
            self.nako_runtime.as_ref(),
            request_id,
            &query,
            &suggestions.candidates,
            artwork_writeback_request,
        )
        .await;

        MetadataScrapeOutcome {
            av: scrape::av_facts_for_outcome(payload, &query),
            query,
            candidates: suggestions.candidates,
            provider_execution: suggestions.execution,
            writeback_result,
            artwork_writeback_result,
        }
    }
}
