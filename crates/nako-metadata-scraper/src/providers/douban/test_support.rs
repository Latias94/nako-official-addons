use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;

use crate::providers::http_runtime::{
    ProviderHttpError, ProviderHttpRequest, ProviderHttpResponse, ProviderHttpResult,
    ProviderHttpRuntimeConfig, ProviderHttpTransport,
};

use super::DOUBAN_PROVIDER_ID;

#[derive(Clone, Default)]
pub(super) struct FakeTransport {
    responses: Arc<Mutex<VecDeque<ProviderHttpResult<ProviderHttpResponse>>>>,
    requests: Arc<Mutex<Vec<ProviderHttpRequest>>>,
}

impl FakeTransport {
    pub(super) fn push_rendered_html(&self, url: &str, title: &str, html: &str) {
        self.responses
            .lock()
            .unwrap()
            .push_back(Ok(ProviderHttpResponse {
                status: 200,
                body: serde_json::json!({
                    "status": "ok",
                    "url": url,
                    "title": title,
                    "html": html,
                    "text": html,
                    "excerpt": html.chars().take(240).collect::<String>()
                })
                .to_string()
                .into_bytes(),
            }));
    }

    pub(super) fn requests(&self) -> Vec<ProviderHttpRequest> {
        self.requests.lock().unwrap().clone()
    }
}

#[async_trait]
impl ProviderHttpTransport for FakeTransport {
    async fn send(
        &self,
        request: ProviderHttpRequest,
        _config: ProviderHttpRuntimeConfig,
    ) -> ProviderHttpResult<ProviderHttpResponse> {
        self.requests.lock().unwrap().push(request);
        self.responses
            .lock()
            .unwrap()
            .pop_front()
            .unwrap_or_else(|| {
                Err(ProviderHttpError::Transport {
                    provider_id: DOUBAN_PROVIDER_ID,
                    operation: "fake",
                    message: "fake transport response queue was empty".to_owned(),
                    attempts: 0,
                })
            })
    }
}
