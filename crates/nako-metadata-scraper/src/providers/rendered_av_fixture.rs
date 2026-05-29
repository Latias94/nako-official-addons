use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;

use crate::providers::http_runtime::{
    ProviderHttpError, ProviderHttpRequest, ProviderHttpResponse, ProviderHttpResult,
    ProviderHttpRuntimeConfig, ProviderHttpTransport,
};

const FIXTURE_PROVIDER_ID: &str = "rendered_av_fixture";

#[derive(Clone)]
pub(crate) struct RenderedAvFixtureTransport {
    provider_id: &'static str,
    responses: Arc<Mutex<VecDeque<ProviderHttpResult<ProviderHttpResponse>>>>,
    requests: Arc<Mutex<Vec<ProviderHttpRequest>>>,
}

impl Default for RenderedAvFixtureTransport {
    fn default() -> Self {
        Self::new(FIXTURE_PROVIDER_ID)
    }
}

impl RenderedAvFixtureTransport {
    #[must_use]
    pub(crate) fn new(provider_id: &'static str) -> Self {
        Self {
            provider_id,
            responses: Arc::default(),
            requests: Arc::default(),
        }
    }

    pub(crate) fn push_rendered_html(&self, url: &str, title: &str, html: &str) {
        self.responses
            .lock()
            .unwrap()
            .push_back(Ok(ProviderHttpResponse {
                status: 200,
                body: rendered_html_response_body(url, title, html),
            }));
    }

    #[must_use]
    pub(crate) fn requests(&self) -> Vec<ProviderHttpRequest> {
        self.requests.lock().unwrap().clone()
    }

    #[must_use]
    pub(crate) fn rendered_request_urls(&self) -> Vec<String> {
        self.requests()
            .iter()
            .map(request_json_body)
            .filter_map(|body| body["url"].as_str().map(str::to_owned))
            .collect()
    }
}

#[async_trait]
impl ProviderHttpTransport for RenderedAvFixtureTransport {
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
                    provider_id: self.provider_id,
                    operation: "fake",
                    message: "rendered AV fixture response queue was empty".to_owned(),
                    attempts: 0,
                })
            })
    }
}

#[must_use]
pub(crate) fn request_json_body(request: &ProviderHttpRequest) -> serde_json::Value {
    serde_json::from_slice(
        request
            .json_body
            .as_deref()
            .expect("render request should include a JSON body"),
    )
    .expect("render request should include valid JSON")
}

fn rendered_html_response_body(url: &str, title: &str, html: &str) -> Vec<u8> {
    serde_json::json!({
        "status": "ok",
        "url": url,
        "title": title,
        "html": html,
        "text": html,
        "excerpt": html.chars().take(240).collect::<String>()
    })
    .to_string()
    .into_bytes()
}

#[cfg(test)]
mod tests {
    use crate::providers::http_runtime::{
        ProviderHttpMethod, ProviderHttpOperationPolicy, ProviderHttpRequest,
        ProviderHttpRuntimeConfig, ProviderHttpTransport,
    };

    use super::*;

    #[tokio::test]
    async fn rendered_av_provider_fixture_records_render_contract_requests() {
        let transport = RenderedAvFixtureTransport::default();
        transport.push_rendered_html(
            "https://provider.example/detail/SSNI-644",
            "SSNI-644 Fixture Title",
            "<html>fixture</html>",
        );

        let response = transport
            .send(
                ProviderHttpRequest {
                    method: ProviderHttpMethod::Post,
                    provider_id: "provider",
                    operation: "render",
                    url: "http://browser-worker.example/render".to_owned(),
                    query: Vec::new(),
                    headers: Vec::new(),
                    json_body: Some(
                        serde_json::json!({
                            "url": "https://provider.example/detail/SSNI-644"
                        })
                        .to_string()
                        .into_bytes(),
                    ),
                    form_body: Vec::new(),
                    operation_policy: ProviderHttpOperationPolicy::default(),
                },
                ProviderHttpRuntimeConfig::default(),
            )
            .await
            .unwrap();

        let body: serde_json::Value = serde_json::from_slice(&response.body).unwrap();
        assert_eq!(body["html"], "<html>fixture</html>");
        assert_eq!(
            transport.rendered_request_urls(),
            vec!["https://provider.example/detail/SSNI-644".to_owned()]
        );
    }
}
