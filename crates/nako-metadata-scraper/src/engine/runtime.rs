use std::sync::Arc;

use nako_addon_protocol::{AddonResourceRequest, AddonResourceResponse};

use crate::{
    nako_runtime::{NakoRuntimeClient, NakoRuntimeTransport},
    providers::MetadataProvider,
};

use super::{MetadataQuery, QueryExternalIdAlias, artwork, orchestration, response, writeback};

#[derive(Clone)]
pub struct MetadataScrapeRuntime<T = crate::nako_runtime::ReqwestNakoRuntimeTransport>
where
    T: NakoRuntimeTransport,
{
    default_language: String,
    external_id_aliases: Arc<Vec<QueryExternalIdAlias>>,
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
        Self {
            default_language: default_language.into(),
            external_id_aliases: Arc::new(external_id_aliases),
            providers: Arc::new(providers),
            nako_runtime,
        }
    }

    pub async fn scrape(&self, request: AddonResourceRequest) -> AddonResourceResponse {
        let query = MetadataQuery::from_payload_with_external_id_aliases(
            &request.payload,
            &self.default_language,
            self.external_id_aliases.as_ref().as_slice(),
        );
        let writeback_request = writeback::MetadataWritebackInput::from_payload(&request.payload);
        let artwork_writeback_request =
            artwork::ArtworkWritebackInput::from_payload(&request.payload);
        let candidates =
            orchestration::suggest_candidates(self.providers.as_ref().as_slice(), &query).await;
        let selected_candidate = candidates.first().cloned();
        let writeback_result = writeback::maybe_submit_metadata_writeback(
            self.nako_runtime.as_ref(),
            &request.request_id,
            &query,
            selected_candidate.as_ref(),
            writeback_request,
        )
        .await;
        let artwork_writeback_result = writeback::maybe_submit_artwork_writeback(
            self.nako_runtime.as_ref(),
            &request.request_id,
            &query,
            &candidates,
            artwork_writeback_request,
        )
        .await;

        response::metadata_response(
            request,
            &query,
            candidates,
            writeback_result,
            artwork_writeback_result,
        )
    }
}
