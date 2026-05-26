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

pub type XcityProviderConfig = RenderedSearchAvProviderConfig;

pub(crate) const XCITY_SITE: RenderedSearchAvSite = RenderedSearchAvSite {
    provider_id: "xcity",
    url_external_id_provider: "xcity_url",
    provider_id_enum: ProviderId::Xcity,
    default_base_url: "https://xcity.jp",
    base_url_env_var: "NAKO_METADATA_SCRAPER_XCITY_BASE_URL",
    timeout_env_var: "NAKO_METADATA_SCRAPER_XCITY_TIMEOUT_MS",
    enabled_env_var: "NAKO_METADATA_SCRAPER_PROVIDER_XCITY_ENABLED",
    capabilities: &[
        "metadata_suggestion",
        "av_number_search",
        "xcity_movie_search",
        "xcity_direct_url",
        "browser_worker_rendered_html",
    ],
    field_quality: ProviderFieldQualityDescriptor::new(560, 300, 560, 200),
    search_url: RenderedSearchAvSearchUrl::Query {
        path: "/result_published/",
        param: "q",
        compact_number: true,
    },
    supported_routes: &[AvNumberRoute::Censored, AvNumberRoute::Amateur],
    outcome: ProviderOutcome::XcityRenderedHtmlParsed,
    tagline: "XCity AV title",
};

const XCITY_EXTERNAL_ID_CAPABILITIES: &[ProviderExternalIdCapability] = &[
    ProviderExternalIdCapability::new(
        "xcity",
        ExternalIdValueKind::Opaque,
        false,
        true,
        &["xcity_id"],
        false,
    ),
    ProviderExternalIdCapability::new(
        "xcity_url",
        ExternalIdValueKind::Url,
        true,
        true,
        &["xcity_url"],
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
        &XCITY_SITE,
        XCITY_EXTERNAL_ID_CAPABILITIES,
        load_config,
        build_provider,
    )
}

fn load_config(input: ProviderConfigInput<'_>) -> ProviderConfig {
    crate::providers::rendered_search_av::load_config(input, &XCITY_SITE, ProviderConfig::xcity)
}

fn build_provider(config: &Config) -> ProviderBuildStatus {
    crate::providers::rendered_search_av::build_provider(
        config,
        &XCITY_SITE,
        ProviderConfig::xcity_config,
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
    async fn xcity_provider_compacts_av_number_for_search_and_parses_detail() {
        let transport = RenderedAvFixtureTransport::new("xcity");
        transport.push_rendered_html(
            "https://xcity.example/result_published/?q=SSNI644",
            "XCity Search",
            r#"
<!doctype html>
<html>
<body>
  <table class="resultList">
    <tr><td><a href="/avod/detail/?id=147036">SSNI-644 XCity Title</a></td></tr>
  </table>
</body>
</html>"#,
        );
        transport.push_rendered_html(
            "https://xcity.example/avod/detail/?id=147036",
            "SSNI-644 XCity Title",
            r#"
<!doctype html>
<html>
<head><meta property="og:image" content="https://img.example/xcity-cover.jpg"></head>
<body>
  <main>
    <h1>SSNI-644 XCity Title</h1>
    <p>品番: SSNI-644</p>
    <p>発売日: 2024-05-03</p>
    <p>収録時間: 123分</p>
    <p>メーカー: XCity Studio</p>
    <p>レーベル: XCity Label</p>
    <a href="/idol/detail/?id=1">Actor One</a>
    <a href="/list/genre/?id=2">Drama</a>
    <video><source src="https://xcity.example/sample/ssni644.mp4" type="video/mp4"></video>
  </main>
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
            &XCITY_SITE,
            XcityProviderConfig::new(
                "https://xcity.example".to_owned(),
                "http://browser-worker.example".to_owned(),
                "/render".to_owned(),
                10_000,
            ),
            runtime,
        );

        let candidates = provider
            .suggest(&MetadataQuery::from_payload(
                &serde_json::json!({"file_name": "SSNI-00644.mp4"}),
                "zh-CN",
            ))
            .await
            .unwrap();

        assert_eq!(candidates.len(), 1);
        let candidate = &candidates[0];
        assert_eq!(candidate.provider, "xcity");
        assert_eq!(candidate.provider_id, "xcity:movie:147036");
        assert_eq!(
            candidate.patch.title.as_deref(),
            Some("SSNI-644 XCity Title")
        );
        assert_eq!(
            candidate.facts.av.as_ref().unwrap().trailer_url.as_deref(),
            Some("https://xcity.example/sample/ssni644.mp4")
        );
        assert_eq!(
            request_json_body(&transport.requests()[0])["url"],
            "https://xcity.example/result_published/?q=SSNI644"
        );
    }
}
