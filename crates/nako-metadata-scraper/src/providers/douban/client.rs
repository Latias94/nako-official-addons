use crate::{
    config::DoubanProviderConfig,
    providers::http_runtime::{
        ProviderHttpResult, ProviderHttpRuntime, ProviderHttpTransport,
        ReqwestProviderHttpTransport,
    },
    providers::rendered_page::{RenderedHtmlPage, RenderedPageRuntime},
};

use super::{DOUBAN_PROVIDER_ID, DoubanMetadataProvider};

impl DoubanMetadataProvider<ReqwestProviderHttpTransport> {
    pub fn new(config: DoubanProviderConfig) -> ProviderHttpResult<Self> {
        let rendered_pages = RenderedPageRuntime::new(config.rendered_pages.clone())?;
        Ok(Self {
            config,
            rendered_pages,
        })
    }
}

impl<T> DoubanMetadataProvider<T>
where
    T: ProviderHttpTransport,
{
    #[must_use]
    pub fn with_runtime(config: DoubanProviderConfig, runtime: ProviderHttpRuntime<T>) -> Self {
        let rendered_pages =
            RenderedPageRuntime::with_runtime(config.rendered_pages.clone(), runtime);
        Self {
            config,
            rendered_pages,
        }
    }

    pub(super) fn search_url(&self, title: &str) -> String {
        format!(
            "{}?search_text={}",
            self.config.search_base_url.trim_end_matches('?'),
            percent_encode_query(title)
        )
    }

    pub(super) async fn render(&self, url: String) -> anyhow::Result<RenderedHtmlPage> {
        let intent = self
            .config
            .rendered_pages
            .intent(&self.config.render_path, url);
        self.rendered_pages
            .render_html(DOUBAN_PROVIDER_ID, "render page", intent)
            .await
    }
}

fn percent_encode_query(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.as_bytes() {
        match *byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(char::from(*byte));
            }
            b' ' => encoded.push_str("%20"),
            byte => encoded.push_str(&format!("%{byte:02X}")),
        }
    }
    encoded
}
