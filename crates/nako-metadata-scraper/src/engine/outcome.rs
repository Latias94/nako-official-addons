#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProviderOutcome {
    FixtureStaticCandidate,
    BrowserWorkerRenderedText,
    TmdbMovieEnriched,
    TmdbMoviePartiallyEnriched,
    TmdbMovieDegraded,
    TmdbPartialTitleVariantSearchFailure,
    BangumiSubjectEnriched,
    BangumiSubjectDegraded,
    BangumiPartialTitleVariantSearchFailure,
    DoubanRenderedHtmlParsed,
    JavdbRenderedHtmlParsed,
    DmmRenderedHtmlParsed,
    XcityRenderedHtmlParsed,
    Fc2RenderedHtmlParsed,
    Fc2ppvdbRenderedHtmlParsed,
    CaribbeanRenderedHtmlParsed,
    OnePondoRenderedHtmlParsed,
    TenMusumeRenderedHtmlParsed,
    JavbusRenderedHtmlParsed,
    JavlibraryRenderedHtmlParsed,
    AiravRenderedHtmlParsed,
    AvsoxRenderedHtmlParsed,
    MgstageRenderedHtmlParsed,
    PrestigeOfficialApiParsed,
}

#[must_use]
pub fn render_provider_note(
    outcomes: &[ProviderOutcome],
    legacy_note: Option<&str>,
) -> Option<String> {
    let mut fragments = outcomes
        .iter()
        .map(|outcome| outcome.safe_note())
        .collect::<Vec<_>>();

    if let Some(legacy_note) = legacy_note.map(str::trim).filter(|note| !note.is_empty()) {
        fragments.push(legacy_note);
    }

    (!fragments.is_empty()).then(|| fragments.join(" "))
}

impl ProviderOutcome {
    const fn safe_note(self) -> &'static str {
        match self {
            Self::FixtureStaticCandidate => {
                "Fixture provider echoes normalized title for smoke testing."
            }
            Self::BrowserWorkerRenderedText => {
                "Browser worker rendered a page and returned normalized text."
            }
            Self::TmdbMovieEnriched => {
                "TMDB movie candidate enriched with search, detail, and external ID responses."
            }
            Self::TmdbMoviePartiallyEnriched => {
                "TMDB movie candidate partially enriched with search and detail responses after secondary enrichment failure."
            }
            Self::TmdbMovieDegraded => {
                "TMDB movie candidate degraded from search response after enrichment failure."
            }
            Self::TmdbPartialTitleVariantSearchFailure => {
                "TMDB provider preserved candidates after partial title-variant search failure."
            }
            Self::BangumiSubjectEnriched => {
                "Bangumi subject candidate enriched with search and detail responses."
            }
            Self::BangumiSubjectDegraded => {
                "Bangumi subject candidate degraded from search response after enrichment failure."
            }
            Self::BangumiPartialTitleVariantSearchFailure => {
                "Bangumi provider preserved candidates after partial title-variant search failure."
            }
            Self::DoubanRenderedHtmlParsed => {
                "Douban candidate parsed from browser-worker rendered HTML."
            }
            Self::JavdbRenderedHtmlParsed => {
                "JavDB AV candidate parsed from browser-worker rendered HTML."
            }
            Self::DmmRenderedHtmlParsed => {
                "DMM AV candidate parsed from browser-worker rendered HTML."
            }
            Self::XcityRenderedHtmlParsed => {
                "XCity AV candidate parsed from browser-worker rendered HTML."
            }
            Self::Fc2RenderedHtmlParsed => {
                "FC2 AV candidate parsed from browser-worker rendered HTML."
            }
            Self::Fc2ppvdbRenderedHtmlParsed => {
                "FC2PPVDB AV candidate parsed from browser-worker rendered HTML."
            }
            Self::CaribbeanRenderedHtmlParsed => {
                "Caribbeancom AV candidate parsed from browser-worker rendered HTML."
            }
            Self::OnePondoRenderedHtmlParsed => {
                "1Pondo AV candidate parsed from browser-worker rendered HTML."
            }
            Self::TenMusumeRenderedHtmlParsed => {
                "10Musume AV candidate parsed from browser-worker rendered HTML."
            }
            Self::JavbusRenderedHtmlParsed => {
                "JavBus AV candidate parsed from browser-worker rendered HTML."
            }
            Self::JavlibraryRenderedHtmlParsed => {
                "JavLibrary AV candidate parsed from browser-worker rendered HTML."
            }
            Self::AiravRenderedHtmlParsed => {
                "AirAV AV candidate parsed from browser-worker rendered HTML."
            }
            Self::AvsoxRenderedHtmlParsed => {
                "AVSox AV candidate parsed from browser-worker rendered HTML."
            }
            Self::MgstageRenderedHtmlParsed => {
                "MGStage AV candidate parsed from browser-worker rendered HTML."
            }
            Self::PrestigeOfficialApiParsed => {
                "Prestige AV candidate parsed from the official JSON API."
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_outcomes_render_redaction_safe_notes() {
        let note = render_provider_note(
            &[
                ProviderOutcome::TmdbMoviePartiallyEnriched,
                ProviderOutcome::TmdbPartialTitleVariantSearchFailure,
            ],
            None,
        )
        .unwrap();

        assert!(note.contains("partially enriched"));
        assert!(note.contains("partial title-variant search failure"));
        assert!(!note.contains("Bearer"));
        assert!(!note.contains("https://"));
    }

    #[test]
    fn provider_outcome_renderer_preserves_legacy_note_fallback() {
        assert_eq!(
            render_provider_note(&[], Some("safe legacy note")).as_deref(),
            Some("safe legacy note")
        );
    }
}
