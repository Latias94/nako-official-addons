use crate::{
    Config,
    config::{ProviderConfig, ProviderId},
    engine::{
        ExternalIdValueKind, ProviderExternalIdCapability, ProviderFieldQualityDescriptor,
        ProviderOutcome,
        av::{AV_NUMBER_EXTERNAL_ID_PROVIDER, AvNumberRoute},
    },
    providers::{
        ProviderBuildStatus, ProviderConfigInput,
        registry::ProviderCatalogEntry,
        rendered_search_av::{
            RenderedSearchAvProviderConfig, RenderedSearchAvSearchUrl, RenderedSearchAvSite,
        },
    },
};

pub type AvsoxProviderConfig = RenderedSearchAvProviderConfig;

pub(crate) const AVSOX_SITE: RenderedSearchAvSite = RenderedSearchAvSite {
    provider_id: "avsox",
    url_external_id_provider: "avsox_url",
    provider_id_enum: ProviderId::Avsox,
    default_base_url: "https://avsox.click",
    base_url_env_var: "NAKO_METADATA_SCRAPER_AVSOX_BASE_URL",
    timeout_env_var: "NAKO_METADATA_SCRAPER_AVSOX_TIMEOUT_MS",
    enabled_env_var: "NAKO_METADATA_SCRAPER_PROVIDER_AVSOX_ENABLED",
    capabilities: &[
        "metadata_suggestion",
        "av_number_search",
        "avsox_movie_search",
        "avsox_direct_url",
        "browser_worker_rendered_html",
    ],
    field_quality: ProviderFieldQualityDescriptor::new(380, 350, 380, 0),
    search_url: RenderedSearchAvSearchUrl::Path {
        prefix: "/cn/search/",
        compact_number: false,
    },
    supported_routes: &[
        AvNumberRoute::Censored,
        AvNumberRoute::Uncensored,
        AvNumberRoute::Fc2,
        AvNumberRoute::Amateur,
        AvNumberRoute::Western,
    ],
    outcome: ProviderOutcome::AvsoxRenderedHtmlParsed,
    tagline: "AVSox community AV title",
};

const AVSOX_EXTERNAL_ID_CAPABILITIES: &[ProviderExternalIdCapability] = &[
    ProviderExternalIdCapability::new(
        "avsox",
        ExternalIdValueKind::Opaque,
        false,
        true,
        &["avsox_id"],
        false,
    ),
    ProviderExternalIdCapability::new(
        "avsox_url",
        ExternalIdValueKind::Url,
        true,
        true,
        &["avsox_url"],
        false,
    ),
    ProviderExternalIdCapability::new(
        AV_NUMBER_EXTERNAL_ID_PROVIDER,
        ExternalIdValueKind::Opaque,
        true,
        true,
        &[],
        false,
    ),
];

#[must_use]
pub(crate) fn catalog_entry() -> ProviderCatalogEntry {
    crate::providers::rendered_search_av::catalog_entry(
        &AVSOX_SITE,
        AVSOX_EXTERNAL_ID_CAPABILITIES,
        load_config,
        build_provider,
    )
}

fn load_config(input: ProviderConfigInput<'_>) -> ProviderConfig {
    crate::providers::rendered_search_av::load_config(input, &AVSOX_SITE, ProviderConfig::avsox)
}

fn build_provider(config: &Config) -> ProviderBuildStatus {
    crate::providers::rendered_search_av::build_provider(
        config,
        &AVSOX_SITE,
        ProviderConfig::avsox_config,
    )
}

#[cfg(test)]
mod tests {
    use crate::{
        engine::MetadataQuery,
        providers::{
            MetadataProvider,
            http_runtime::{ProviderHttpRuntime, ProviderHttpRuntimeConfig},
            rendered_av_fixture::{RenderedAvFixtureTransport, request_json_body},
            rendered_search_av::RenderedSearchAvMetadataProvider,
        },
    };

    use super::*;

    #[tokio::test]
    async fn avsox_provider_uses_path_search_and_parses_detail() {
        let transport = RenderedAvFixtureTransport::new("avsox");
        transport.push_rendered_html(
            "https://avsox.example/cn/search/SSNI-644",
            "AVSox Search",
            r#"
<!doctype html>
<html>
<body>
  <a class="movie-box" href="/cn/movie/abc123">
    <span>SSNI-644 AVSox Title</span>
  </a>
</body>
</html>"#,
        );
        transport.push_rendered_html(
            "https://avsox.example/cn/movie/abc123",
            "SSNI-644 AVSox Title",
            r#"
<!doctype html>
<html>
<head><meta property="og:image" content="//img.example/avsox-cover.jpg"></head>
<body>
  <article>
    <h3>SSNI-644 AVSox Title</h3>
    <p>识别码: SSNI-644</p>
    <p>发行日期: 2024-05-02</p>
    <p>长度: 122分钟</p>
    <p>制作商: Studio Alpha</p>
    <a href="/cn/star/a1">Actor One</a>
    <a href="/cn/genre/drama">Drama</a>
    <div class="sample"><img src="//img.example/sample.jpg"></div>
  </article>
</body>
</html>"#,
        );
        let runtime = ProviderHttpRuntime::with_transport(
            ProviderHttpRuntimeConfig {
                retry_backoff_ms: 0,
                ..ProviderHttpRuntimeConfig::default()
            },
            transport.clone(),
        );
        let provider = RenderedSearchAvMetadataProvider::with_runtime(
            &AVSOX_SITE,
            AvsoxProviderConfig::new(
                "https://avsox.example".to_owned(),
                "http://browser-worker.example".to_owned(),
                "/render".to_owned(),
                10_000,
            ),
            runtime,
        );

        let candidates = provider
            .suggest(&MetadataQuery::from_payload(
                &serde_json::json!({"file_name": "SSNI-00644.mkv"}),
                "zh-CN",
            ))
            .await
            .unwrap();

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].provider, "avsox");
        assert_eq!(candidates[0].provider_id, "avsox:movie:abc123");
        assert_eq!(
            candidates[0].patch.title.as_deref(),
            Some("SSNI-644 AVSox Title")
        );
        assert_eq!(
            candidates[0].facts.av.as_ref().unwrap().studio.as_deref(),
            Some("Studio Alpha")
        );
        assert_eq!(
            request_json_body(&transport.requests()[0])["url"],
            "https://avsox.example/cn/search/SSNI-644"
        );
    }
}
