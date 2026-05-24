# Official Metadata Addon Provider Search Variant Resilience — Milestones

Status: Complete
Last updated: 2026-05-24

## M0 — Scope And Variant Failure Policy Freeze

Exit criteria:

- Partial search-variant failure policy is explicit.
- All-search-failed behavior remains explicit.
- Non-goals are explicit.

Primary evidence:

- `docs/workstreams/official-metadata-addon-provider-search-variant-resilience/DESIGN.md`
- `docs/workstreams/official-metadata-addon-provider-search-variant-resilience/TODO.md`

## M1 — TMDB Search Variant Resilience

Exit criteria:

- Earlier TMDB search results survive a later variant search failure.
- All TMDB search variants failing still propagates a provider error.
- Existing relevance-budget and degraded-candidate behavior remains covered.

Primary gates:

- `cargo nextest run -p nako-metadata-scraper tmdb ranking title --no-fail-fast`

## M2 — Bangumi Search Variant Resilience

Exit criteria:

- Earlier Bangumi search results survive a later variant search failure.
- All Bangumi search variants failing still propagates a provider error.
- Existing relevance-budget and degraded-candidate behavior remains covered.

Primary gates:

- `cargo nextest run -p nako-metadata-scraper bangumi ranking title --no-fail-fast`

## M3 — Closeout

Exit criteria:

- Fresh gate evidence is recorded.
- Package, workspace, format, and whitespace gates pass.
- Remaining partial-error reporting work is deferred or split.

## Closeout Summary

Closed on 2026-05-24.

- TMDB and Bangumi now preserve usable earlier search-title variant results when a later variant
  search fails after HTTP runtime policy is exhausted.
- Both providers still propagate provider-level errors when every title-variant search fails before
  producing candidates.
- Relevance-budget ranking and degraded candidate fallback remain unchanged.
- Payload-visible partial-search warnings remain follow-on scope.
