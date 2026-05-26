use crate::{
    config::Fc2ProviderConfig,
    providers::{
        http_runtime::{
            ProviderHttpResult, ProviderHttpRuntime, ProviderHttpTransport,
            ReqwestProviderHttpTransport,
        },
        rendered_page::{RenderedHtmlPage, RenderedPageRuntime},
    },
};

use super::{FC2_PROVIDER_ID, Fc2MetadataProvider};

impl Fc2MetadataProvider<ReqwestProviderHttpTransport> {
    pub fn new(config: Fc2ProviderConfig) -> ProviderHttpResult<Self> {
        let rendered_pages = RenderedPageRuntime::new(config.rendered_pages.clone())?;
        Ok(Self {
            config,
            rendered_pages,
        })
    }
}

impl<T> Fc2MetadataProvider<T>
where
    T: ProviderHttpTransport,
{
    #[must_use]
    pub fn with_runtime(config: Fc2ProviderConfig, runtime: ProviderHttpRuntime<T>) -> Self {
        let rendered_pages =
            RenderedPageRuntime::with_runtime(config.rendered_pages.clone(), runtime);
        Self {
            config,
            rendered_pages,
        }
    }

    pub(super) fn detail_url(&self, article_id: &str) -> String {
        format!(
            "{}/article/{}/",
            self.config.base_url.trim_end_matches('/'),
            article_id
        )
    }

    pub(super) async fn render(&self, url: String) -> anyhow::Result<RenderedHtmlPage> {
        let intent = self
            .config
            .rendered_pages
            .intent(&self.config.render_path, url);
        self.rendered_pages
            .render_html(FC2_PROVIDER_ID, "render page", intent)
            .await
    }
}
