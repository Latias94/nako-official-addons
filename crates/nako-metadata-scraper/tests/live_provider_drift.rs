use anyhow::{Context, Result, bail};
use nako_addon_protocol::{AddonArtworkKind, AddonMetadataPatch};
use nako_metadata_scraper::{
    Config,
    config::{AV_PROVIDER_PRESET_ENV_VAR, BangumiProviderConfig, ProviderId, TmdbProviderConfig},
    engine::{
        AvMetadataFacts, MetadataQuery, ProviderArtworkCandidate, ProviderArtworkCandidateFacts,
        ProviderCandidateFacts, ProviderExternalId, ProviderMetadataCandidate,
    },
    providers::{
        MetadataProvider, ProviderRegistry, bangumi::BangumiMetadataProvider,
        tmdb::TmdbMetadataProvider,
    },
};
use serde_json::{Value, json};

fn live_provider_drift_enabled() -> bool {
    matches!(
        std::env::var("NAKO_METADATA_SCRAPER_LIVE_PROVIDER_DRIFT"),
        Ok(value) if matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        )
    )
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct AvDriftCase {
    provider_id: String,
    av_number: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct DriftFieldHealthReport {
    provider: String,
    candidate_count: usize,
    present_fields: Vec<&'static str>,
    missing_required_fields: Vec<&'static str>,
    missing_optional_fields: Vec<&'static str>,
    counts: DriftFieldHealthCounts,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct DriftFieldHealthCounts {
    external_ids: usize,
    provider_outcomes: usize,
    artwork_candidates: usize,
    av_actors: usize,
    av_all_actors: usize,
    av_directors: usize,
    av_extrafanart_urls: usize,
}

const REQUIRED_AV_FIELDS: &[&str] = &["title", "av_number"];
const OPTIONAL_AV_FIELDS: &[&str] = &[
    "overview",
    "release_date",
    "runtime_minutes",
    "genres",
    "tags",
    "actors",
    "all_actors",
    "directors",
    "series",
    "studio",
    "publisher",
    "maker",
    "label",
    "wanted_count",
    "thumb_url",
    "trailer_url",
    "extrafanart_urls",
    "artwork_candidates",
    "external_ids",
    "provider_outcomes",
];

const AV_PROVIDER_ENABLED_ENV_VARS: &[(&str, &str)] = &[
    ("javdb", "NAKO_METADATA_SCRAPER_PROVIDER_JAVDB_ENABLED"),
    ("dmm", "NAKO_METADATA_SCRAPER_PROVIDER_DMM_ENABLED"),
    ("xcity", "NAKO_METADATA_SCRAPER_PROVIDER_XCITY_ENABLED"),
    ("fc2", "NAKO_METADATA_SCRAPER_PROVIDER_FC2_ENABLED"),
    (
        "fc2ppvdb",
        "NAKO_METADATA_SCRAPER_PROVIDER_FC2PPVDB_ENABLED",
    ),
    (
        "caribbean",
        "NAKO_METADATA_SCRAPER_PROVIDER_CARIBBEAN_ENABLED",
    ),
    ("1pondo", "NAKO_METADATA_SCRAPER_PROVIDER_1PONDO_ENABLED"),
    (
        "10musume",
        "NAKO_METADATA_SCRAPER_PROVIDER_10MUSUME_ENABLED",
    ),
    ("javbus", "NAKO_METADATA_SCRAPER_PROVIDER_JAVBUS_ENABLED"),
    (
        "javlibrary",
        "NAKO_METADATA_SCRAPER_PROVIDER_JAVLIBRARY_ENABLED",
    ),
    ("airav", "NAKO_METADATA_SCRAPER_PROVIDER_AIRAV_ENABLED"),
    ("avsox", "NAKO_METADATA_SCRAPER_PROVIDER_AVSOX_ENABLED"),
    ("mgstage", "NAKO_METADATA_SCRAPER_PROVIDER_MGSTAGE_ENABLED"),
    (
        "prestige",
        "NAKO_METADATA_SCRAPER_PROVIDER_PRESTIGE_ENABLED",
    ),
];

fn parse_av_drift_cases(value: &str) -> Result<Vec<AvDriftCase>> {
    value
        .split([',', ';', '\n'])
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
        .map(|entry| {
            let Some((provider, av_number)) = entry.split_once('=') else {
                bail!("invalid AV drift case `{entry}`; expected provider=AV-NUMBER");
            };
            let provider_id = provider.trim().to_ascii_lowercase();
            let av_number = av_number.trim().to_owned();
            if provider_id.is_empty() || av_number.is_empty() {
                bail!("invalid AV drift case `{entry}`; provider and AV number are required");
            }
            if av_provider_enabled_env_var(&provider_id).is_none() {
                bail!("unsupported AV drift provider `{provider_id}`");
            }
            Ok(AvDriftCase {
                provider_id,
                av_number,
            })
        })
        .collect()
}

fn av_provider_enabled_env_var(provider_id: &str) -> Option<&'static str> {
    AV_PROVIDER_ENABLED_ENV_VARS
        .iter()
        .find_map(|(candidate_provider_id, enabled_env_var)| {
            (*candidate_provider_id == provider_id).then_some(*enabled_env_var)
        })
}

fn av_provider_id_for_enabled_env_var(name: &str) -> Option<&'static str> {
    AV_PROVIDER_ENABLED_ENV_VARS
        .iter()
        .find_map(|(provider_id, enabled_env_var)| {
            (*enabled_env_var == name).then_some(*provider_id)
        })
}

fn av_field_health_report(
    provider_id: &str,
    candidates: &[ProviderMetadataCandidate],
) -> DriftFieldHealthReport {
    let mut present_fields = Vec::new();
    let mut missing_required_fields = Vec::new();
    let mut missing_optional_fields = Vec::new();
    let mut counts = DriftFieldHealthCounts::default();

    if let Some(candidate) = candidates.first() {
        for field in REQUIRED_AV_FIELDS {
            if av_field_present(candidate, field) {
                present_fields.push(*field);
            } else {
                missing_required_fields.push(*field);
            }
        }
        for field in OPTIONAL_AV_FIELDS {
            if av_field_present(candidate, field) {
                present_fields.push(*field);
            } else {
                missing_optional_fields.push(*field);
            }
        }
        counts = av_field_counts(candidate);
    } else {
        missing_required_fields.extend(REQUIRED_AV_FIELDS);
        missing_optional_fields.extend(OPTIONAL_AV_FIELDS);
    }

    DriftFieldHealthReport {
        provider: provider_id.to_owned(),
        candidate_count: candidates.len(),
        present_fields,
        missing_required_fields,
        missing_optional_fields,
        counts,
    }
}

fn av_field_present(candidate: &ProviderMetadataCandidate, field: &str) -> bool {
    match field {
        "title" => {
            candidate.patch.title.is_some()
                || candidate.patch.original_title.is_some()
                || candidate.patch.sort_title.is_some()
                || candidate.facts.title.is_some()
        }
        "overview" => candidate.patch.overview.is_some(),
        "release_date" => {
            candidate.patch.release_date.is_some() || candidate.facts.release_year.is_some()
        }
        "runtime_minutes" => candidate.patch.runtime_minutes.is_some(),
        "genres" => candidate
            .patch
            .genres
            .as_ref()
            .is_some_and(|values| !values.is_empty()),
        "tags" => candidate
            .patch
            .tags
            .as_ref()
            .is_some_and(|values| !values.is_empty()),
        "actors" => candidate
            .facts
            .av
            .as_ref()
            .is_some_and(|av| !av.actors.is_empty()),
        "all_actors" => candidate
            .facts
            .av
            .as_ref()
            .is_some_and(|av| !av.all_actors.is_empty()),
        "directors" => candidate
            .facts
            .av
            .as_ref()
            .is_some_and(|av| !av.directors.is_empty()),
        "series" => candidate
            .facts
            .av
            .as_ref()
            .and_then(|av| av.series.as_ref())
            .is_some(),
        "studio" => candidate
            .facts
            .av
            .as_ref()
            .and_then(|av| av.studio.as_ref())
            .is_some(),
        "publisher" => candidate
            .facts
            .av
            .as_ref()
            .and_then(|av| av.publisher.as_ref())
            .is_some(),
        "maker" => candidate
            .facts
            .av
            .as_ref()
            .and_then(|av| av.maker.as_ref())
            .is_some(),
        "label" => candidate
            .facts
            .av
            .as_ref()
            .and_then(|av| av.label.as_ref())
            .is_some(),
        "wanted_count" => candidate
            .facts
            .av
            .as_ref()
            .and_then(|av| av.wanted_count)
            .is_some(),
        "thumb_url" => candidate
            .facts
            .av
            .as_ref()
            .and_then(|av| av.thumb_url.as_ref())
            .is_some(),
        "trailer_url" => candidate
            .facts
            .av
            .as_ref()
            .and_then(|av| av.trailer_url.as_ref())
            .is_some(),
        "extrafanart_urls" => candidate
            .facts
            .av
            .as_ref()
            .is_some_and(|av| !av.extrafanart_urls.is_empty()),
        "artwork_candidates" => !candidate.artwork_candidates.is_empty(),
        "external_ids" => !candidate.facts.external_ids.is_empty(),
        "av_number" => candidate
            .facts
            .external_ids
            .iter()
            .any(|external_id| external_id.provider == "av_number"),
        "provider_outcomes" => !candidate.facts.provider_outcomes.is_empty(),
        _ => false,
    }
}

fn av_field_counts(candidate: &ProviderMetadataCandidate) -> DriftFieldHealthCounts {
    let av = candidate.facts.av.as_ref();

    DriftFieldHealthCounts {
        external_ids: candidate.facts.external_ids.len(),
        provider_outcomes: candidate.facts.provider_outcomes.len(),
        artwork_candidates: candidate.artwork_candidates.len(),
        av_actors: av.map_or(0, |av| av.actors.len()),
        av_all_actors: av.map_or(0, |av| av.all_actors.len()),
        av_directors: av.map_or(0, |av| av.directors.len()),
        av_extrafanart_urls: av.map_or(0, |av| av.extrafanart_urls.len()),
    }
}

impl DriftFieldHealthReport {
    fn to_redaction_safe_json(&self) -> Value {
        json!({
            "schema": "nako.metadata-scraper.live-provider-drift.field-health.v1",
            "provider": self.provider,
            "candidate_count": self.candidate_count,
            "present_fields": self.present_fields,
            "missing_required_fields": self.missing_required_fields,
            "missing_optional_fields": self.missing_optional_fields,
            "counts": {
                "external_ids": self.counts.external_ids,
                "provider_outcomes": self.counts.provider_outcomes,
                "artwork_candidates": self.counts.artwork_candidates,
                "av_actors": self.counts.av_actors,
                "av_all_actors": self.counts.av_all_actors,
                "av_directors": self.counts.av_directors,
                "av_extrafanart_urls": self.counts.av_extrafanart_urls,
            }
        })
    }
}

fn av_live_drift_cases_from_env() -> Result<Vec<AvDriftCase>> {
    let Ok(value) = std::env::var("NAKO_METADATA_SCRAPER_LIVE_AV_PROVIDER_DRIFT_CASES") else {
        return Ok(Vec::new());
    };

    parse_av_drift_cases(&value)
}

fn live_av_config(cases: &[AvDriftCase]) -> Config {
    Config::from_env_lookup(|name| {
        if name == AV_PROVIDER_PRESET_ENV_VAR {
            return Some("manual".to_owned());
        }
        if name == "NAKO_METADATA_SCRAPER_PROVIDER_FIXTURE_ENABLED" {
            return Some("false".to_owned());
        }
        if let Some(provider_id) = av_provider_id_for_enabled_env_var(name) {
            let enabled = cases.iter().any(|case| case.provider_id == provider_id);
            return Some(enabled.to_string());
        }

        std::env::var(name).ok()
    })
}

#[tokio::test]
#[ignore]
async fn av_live_provider_field_health_smoke() -> Result<()> {
    if !live_provider_drift_enabled() {
        return Ok(());
    }

    let cases = av_live_drift_cases_from_env()?;
    if cases.is_empty() {
        return Ok(());
    }

    let providers = ProviderRegistry::from_config(live_av_config(&cases)).providers();

    for case in cases {
        let Some(provider) = providers
            .iter()
            .find(|provider| provider.id().as_str() == case.provider_id)
        else {
            bail!(
                "AV live drift provider `{}` was not ready; check provider config, browser-worker, and proxy policy",
                case.provider_id
            );
        };

        let payload = json!({
            "title": case.av_number,
            "language": "ja-JP",
            "av_number": case.av_number,
            "external_ids": [
                {
                    "provider": "av_number",
                    "value": case.av_number,
                }
            ]
        });
        let query = MetadataQuery::from_payload(&payload, "ja-JP");
        let candidates = provider
            .suggest(&query)
            .await
            .with_context(|| format!("AV live drift provider `{}` failed", case.provider_id))?;
        let report = av_field_health_report(&case.provider_id, &candidates);
        eprintln!("{}", report.to_redaction_safe_json());

        assert!(
            report.candidate_count > 0,
            "AV live drift provider `{}` returned no candidates",
            case.provider_id
        );
        assert!(
            report.missing_required_fields.is_empty(),
            "AV live drift provider `{}` missing required fields {:?}",
            case.provider_id,
            report.missing_required_fields
        );
    }

    Ok(())
}

#[test]
fn av_drift_case_parser_accepts_comma_semicolon_and_newline_separators() {
    let cases = parse_av_drift_cases("javdb=SSNI-644; fc2=FC2-1723984\n1pondo=010116_001").unwrap();

    assert_eq!(
        cases,
        vec![
            AvDriftCase {
                provider_id: "javdb".to_owned(),
                av_number: "SSNI-644".to_owned(),
            },
            AvDriftCase {
                provider_id: "fc2".to_owned(),
                av_number: "FC2-1723984".to_owned(),
            },
            AvDriftCase {
                provider_id: "1pondo".to_owned(),
                av_number: "010116_001".to_owned(),
            },
        ]
    );
}

#[test]
fn av_drift_live_config_enables_only_case_providers() {
    let cases = vec![AvDriftCase {
        provider_id: "javdb".to_owned(),
        av_number: "SSNI-644".to_owned(),
    }];
    let config = live_av_config(&cases);

    assert_eq!(config.av_provider_preset.as_str(), "manual");
    assert!(!config.provider_enabled(ProviderId::Fixture));
    assert!(config.provider_enabled(ProviderId::Javdb));
    assert!(!config.provider_enabled(ProviderId::Dmm));
    assert!(!config.provider_enabled(ProviderId::Fc2));
    assert!(!config.provider_enabled(ProviderId::Prestige));
}

#[test]
fn av_field_health_report_does_not_emit_raw_candidate_values() {
    let candidate = ProviderMetadataCandidate {
        provider: "javdb".to_owned(),
        provider_id: "raw-provider-id-should-not-appear".to_owned(),
        patch: AddonMetadataPatch {
            title: Some("Raw Sensitive Title".to_owned()),
            overview: Some("Raw sensitive overview".to_owned()),
            release_date: Some("2026-01-01".to_owned()),
            runtime_minutes: Some(120),
            tags: Some(vec!["Raw Tag".to_owned()]),
            ..AddonMetadataPatch::default()
        },
        facts: ProviderCandidateFacts {
            title: Some("Raw Facts Title".to_owned()),
            av: Some(AvMetadataFacts {
                actors: vec!["Raw Actor".to_owned()],
                all_actors: vec!["Raw Actor".to_owned(), "Raw Extra Actor".to_owned()],
                directors: vec!["Raw Director".to_owned()],
                series: Some("Raw Series".to_owned()),
                studio: Some("Raw Studio".to_owned()),
                thumb_url: Some("https://sensitive.example/thumb.jpg".to_owned()),
                trailer_url: Some("https://sensitive.example/trailer.mp4".to_owned()),
                extrafanart_urls: vec!["https://sensitive.example/fanart.jpg".to_owned()],
                ..AvMetadataFacts::default()
            }),
            external_ids: vec![ProviderExternalId {
                provider: "av_number".to_owned(),
                value: "SSNI-644".to_owned(),
            }],
            ..ProviderCandidateFacts::default()
        },
        artwork_candidates: vec![ProviderArtworkCandidate {
            provider: "javdb".to_owned(),
            provider_id: "raw-artwork-id-should-not-appear".to_owned(),
            facts: ProviderArtworkCandidateFacts {
                kind: AddonArtworkKind::Poster,
                source_url: "https://sensitive.example/poster.jpg".to_owned(),
                language: None,
                width: Some(800),
                height: Some(1200),
            },
        }],
    };

    let report = av_field_health_report("javdb", &[candidate]).to_redaction_safe_json();
    let rendered = report.to_string();

    assert!(rendered.contains("present_fields"));
    assert!(rendered.contains("candidate_count"));
    assert!(!rendered.contains("Raw Sensitive Title"));
    assert!(!rendered.contains("Raw Actor"));
    assert!(!rendered.contains("sensitive.example"));
    assert!(!rendered.contains("SSNI-644"));
    assert!(!rendered.contains("raw-provider-id-should-not-appear"));
    assert!(!rendered.contains("raw-artwork-id-should-not-appear"));
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
