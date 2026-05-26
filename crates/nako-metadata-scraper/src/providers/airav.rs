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

pub type AiravProviderConfig = RenderedSearchAvProviderConfig;

pub(crate) const AIRAV_SITE: RenderedSearchAvSite = RenderedSearchAvSite {
    provider_id: "airav",
    url_external_id_provider: "airav_url",
    provider_id_enum: ProviderId::Airav,
    default_base_url: "https://www.airav.wiki",
    base_url_env_var: "NAKO_METADATA_SCRAPER_AIRAV_BASE_URL",
    timeout_env_var: "NAKO_METADATA_SCRAPER_AIRAV_TIMEOUT_MS",
    enabled_env_var: "NAKO_METADATA_SCRAPER_PROVIDER_AIRAV_ENABLED",
    capabilities: &[
        "metadata_suggestion",
        "av_number_search",
        "airav_movie_search",
        "airav_direct_url",
        "browser_worker_rendered_html",
    ],
    field_quality: ProviderFieldQualityDescriptor::new(420, 420, 420, 100),
    search_url: RenderedSearchAvSearchUrl::Query {
        path: "/",
        param: "search",
        compact_number: false,
    },
    supported_routes: &[
        AvNumberRoute::Censored,
        AvNumberRoute::Uncensored,
        AvNumberRoute::Fc2,
        AvNumberRoute::Amateur,
        AvNumberRoute::Western,
    ],
    outcome: ProviderOutcome::AiravRenderedHtmlParsed,
    tagline: "AirAV community AV title",
};

const AIRAV_EXTERNAL_ID_CAPABILITIES: &[ProviderExternalIdCapability] = &[
    ProviderExternalIdCapability::new(
        "airav",
        ExternalIdValueKind::Opaque,
        false,
        true,
        &["airav_id"],
        false,
    ),
    ProviderExternalIdCapability::new(
        "airav_url",
        ExternalIdValueKind::Url,
        true,
        true,
        &["airav_url"],
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
        &AIRAV_SITE,
        AIRAV_EXTERNAL_ID_CAPABILITIES,
        load_config,
        build_provider,
    )
}

fn load_config(input: ProviderConfigInput<'_>) -> ProviderConfig {
    crate::providers::rendered_search_av::load_config(input, &AIRAV_SITE, ProviderConfig::airav)
}

fn build_provider(config: &Config) -> ProviderBuildStatus {
    crate::providers::rendered_search_av::build_provider(
        config,
        &AIRAV_SITE,
        ProviderConfig::airav_config,
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
    async fn airav_provider_uses_browser_worker_render_contract_for_search_and_detail() {
        let transport = RenderedAvFixtureTransport::new("airav");
        transport.push_rendered_html(
            "https://airav.example/?search=SSNI-644",
            "AirAV Search",
            r#"
<!doctype html>
<html>
<body>
  <a class="video-item" href="/video/SSNI-644">
    <img alt="SSNI-644 AirAV Title">
  </a>
  <a class="video-item" href="/video/ABP-001">ABP-001 Other Title</a>
</body>
</html>"#,
        );
        transport.push_rendered_html(
            "https://airav.example/video/SSNI-644",
            "SSNI-644 AirAV Title",
            r#"
<!doctype html>
<html>
<head>
  <meta property="og:image" content="/covers/ssni-644.jpg">
  <meta name="description" content="AirAV synthetic outline.">
</head>
<body>
  <main>
    <h1>SSNI-644 AirAV Title</h1>
    <p>Number: SSNI-644</p>
    <p>Release Date: 2024-05-01</p>
    <p>Runtime: 121 minutes</p>
    <p>Studio: Studio Alpha</p>
    <p>Publisher: Label Beta</p>
    <p>Series: Series Gamma</p>
    <a href="/actor/a1">Actor One</a>
    <a href="/genre/drama">Drama</a>
    <a href="/genre/uniform">Uniform</a>
    <span class="score">4.2</span>
    <div class="gallery"><img src="/samples/one.jpg"></div>
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
            &AIRAV_SITE,
            AiravProviderConfig::new(
                "https://airav.example".to_owned(),
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
        assert_eq!(candidate.provider, "airav");
        assert_eq!(candidate.provider_id, "airav:movie:SSNI-644");
        assert_eq!(
            candidate.patch.title.as_deref(),
            Some("SSNI-644 AirAV Title")
        );
        assert_eq!(candidate.patch.release_date.as_deref(), Some("2024-05-01"));
        assert_eq!(candidate.patch.runtime_minutes, Some(121));
        assert_eq!(candidate.facts.community_score_milli, Some(840));
        assert_eq!(
            candidate.facts.av.as_ref().unwrap().actors,
            vec!["Actor One".to_owned()]
        );
        assert_eq!(
            candidate.facts.av.as_ref().unwrap().studio.as_deref(),
            Some("Studio Alpha")
        );
        assert_eq!(
            candidate.facts.av.as_ref().unwrap().publisher.as_deref(),
            Some("Label Beta")
        );
        assert_eq!(candidate.artwork_candidates.len(), 2);

        let requests = transport.requests();
        assert_eq!(requests.len(), 2);
        assert_eq!(
            request_json_body(&requests[0])["url"],
            "https://airav.example/?search=SSNI-644"
        );
        assert_eq!(
            request_json_body(&requests[1])["url"],
            "https://airav.example/video/SSNI-644"
        );
    }
}
