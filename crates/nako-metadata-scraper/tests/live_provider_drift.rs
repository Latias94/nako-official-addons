use anyhow::Result;
use nako_metadata_scraper::{
    config::{BangumiProviderConfig, TmdbProviderConfig},
    engine::MetadataQuery,
    providers::{MetadataProvider, bangumi::BangumiMetadataProvider, tmdb::TmdbMetadataProvider},
};
use serde_json::json;

fn live_provider_drift_enabled() -> bool {
    matches!(
        std::env::var("NAKO_METADATA_SCRAPER_LIVE_PROVIDER_DRIFT"),
        Ok(value) if matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        )
    )
}

#[tokio::test]
#[ignore]
async fn tmdb_live_direct_lookup_smoke() -> Result<()> {
    if !live_provider_drift_enabled() {
        return Ok(());
    }

    let config = TmdbProviderConfig::from_env_lookup(|name| std::env::var(name).ok());
    if config.read_access_token.is_none() {
        return Ok(());
    }

    let default_language = config.language.clone();
    let provider = TmdbMetadataProvider::new(config)?;
    let payload = json!({
        "title": "Fight Club",
        "year": 1999,
        "language": default_language,
        "external_ids": [
            {
                "provider": "tmdb",
                "value": "550"
            }
        ]
    });
    let query = MetadataQuery::from_payload(&payload, &default_language);

    let candidates = provider.suggest(&query).await?;
    assert!(
        !candidates.is_empty(),
        "TMDB live drift check returned no candidates"
    );

    let candidate = &candidates[0];
    assert_eq!(candidate.provider, "tmdb");
    assert_eq!(candidate.provider_id, "550");
    assert!(
        candidate.patch.title.is_some()
            || candidate.patch.original_title.is_some()
            || candidate.patch.sort_title.is_some()
    );

    Ok(())
}

#[tokio::test]
#[ignore]
async fn bangumi_live_direct_lookup_smoke() -> Result<()> {
    if !live_provider_drift_enabled() {
        return Ok(());
    }

    let config = BangumiProviderConfig::from_env_lookup(|name| std::env::var(name).ok());
    let default_language = "zh-CN";
    let provider = BangumiMetadataProvider::new(config)?;
    let payload = json!({
        "title": "新世纪福音战士",
        "year": 1995,
        "language": default_language,
        "external_ids": [
            {
                "provider": "bangumi",
                "value": "265"
            }
        ]
    });
    let query = MetadataQuery::from_payload(&payload, default_language);

    let candidates = provider.suggest(&query).await?;
    assert!(
        !candidates.is_empty(),
        "Bangumi live drift check returned no candidates"
    );

    let candidate = &candidates[0];
    assert_eq!(candidate.provider, "bangumi");
    assert_eq!(candidate.provider_id, "bangumi:subject:265");
    assert!(
        candidate.patch.title.is_some() || candidate.facts.title.is_some(),
        "Bangumi live drift check returned a candidate without any title fields"
    );
    assert!(
        candidate
            .facts
            .external_ids
            .iter()
            .any(|external_id| external_id.provider == "bangumi" && external_id.value == "265")
    );

    Ok(())
}
