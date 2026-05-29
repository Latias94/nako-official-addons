use crate::{
    config::JavdbProviderConfig,
    providers::{
        http_runtime::{
            ProviderHttpResult, ProviderHttpRuntime, ProviderHttpTransport,
            ReqwestProviderHttpTransport,
        },
        rendered_page::{RenderedHtmlPage, RenderedPageRuntime},
    },
};

use super::{JAVDB_PROVIDER_ID, JavdbMetadataProvider};

impl JavdbMetadataProvider<ReqwestProviderHttpTransport> {
    pub fn new(config: JavdbProviderConfig) -> ProviderHttpResult<Self> {
        let rendered_pages = RenderedPageRuntime::new(config.rendered_pages.clone())?;
        Ok(Self {
            config,
            rendered_pages,
        })
    }
}

impl<T> JavdbMetadataProvider<T>
where
    T: ProviderHttpTransport,
{
    #[must_use]
    pub fn with_runtime(config: JavdbProviderConfig, runtime: ProviderHttpRuntime<T>) -> Self {
        let rendered_pages =
            RenderedPageRuntime::with_runtime(config.rendered_pages.clone(), runtime);
        Self {
            config,
            rendered_pages,
        }
    }

    pub(super) fn search_url(&self, av_number: &str) -> String {
        super::javdb_search_url(&self.config.base_url, av_number)
    }

    pub(super) fn detail_url(&self, movie_id: &str) -> String {
        format!(
            "{}/v/{}",
            self.config.base_url.trim_end_matches('/'),
            percent_encode_path_segment(movie_id)
        )
    }

    pub(super) fn absolute_url(&self, href: &str) -> String {
        if href.starts_with("http://") || href.starts_with("https://") {
            return href.to_owned();
        }

        format!(
            "{}/{}",
            self.config.base_url.trim_end_matches('/'),
            href.trim_start_matches('/')
        )
    }

    pub(super) async fn render(&self, url: String) -> anyhow::Result<RenderedHtmlPage> {
        let intent = self
            .config
            .rendered_pages
            .intent(&self.config.render_path, url);
        self.rendered_pages
            .render_html(JAVDB_PROVIDER_ID, "render page", intent)
            .await
    }
}

pub(super) fn percent_encode_query(value: &str) -> String {
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

fn percent_encode_path_segment(value: &str) -> String {
    percent_encode_query(value)
}
