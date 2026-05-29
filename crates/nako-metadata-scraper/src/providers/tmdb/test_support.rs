use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;

use crate::providers::http_runtime::{
    ProviderHttpError, ProviderHttpRequest, ProviderHttpResponse, ProviderHttpResult,
    ProviderHttpRuntimeConfig, ProviderHttpTransport,
};

#[derive(Clone, Default)]
pub(super) struct FakeTransport {
    responses: Arc<Mutex<VecDeque<ProviderHttpResult<ProviderHttpResponse>>>>,
    requests: Arc<Mutex<Vec<ProviderHttpRequest>>>,
    configs: Arc<Mutex<Vec<ProviderHttpRuntimeConfig>>>,
}

impl FakeTransport {
    pub(super) fn push(&self, response: ProviderHttpResult<ProviderHttpResponse>) {
        self.responses.lock().unwrap().push_back(response);
    }

    pub(super) fn requests(&self) -> Vec<ProviderHttpRequest> {
        self.requests.lock().unwrap().clone()
    }

    pub(super) fn configs(&self) -> Vec<ProviderHttpRuntimeConfig> {
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
                    operation: "fake",
                    message: "fake transport response queue was empty".to_owned(),
                    attempts: 0,
                })
            })
    }
}
