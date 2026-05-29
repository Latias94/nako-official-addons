# Official Metadata Addon Provider Search Merge

Status: Complete
Last updated: 2026-05-24

## Why This Lane Exists

The provider breadth lane added alternate-title evidence and raw-search-empty fallback for TMDB and
Bangumi. That improved localized libraries when the first search key returns nothing, but it still
throws away useful results when multiple search keys return non-empty but different candidate sets.

## Relevant Authority

- `docs/adr/0015-capability-scoped-http-addons-and-automation-providers.md`
- `docs/adr/0020-jellyfin-like-sidecar-addons-with-scoped-api-access.md`
- `docs/workstreams/official-metadata-addon-provider-hardening`
- `docs/workstreams/official-metadata-addon-provider-breadth`

## Problem

- TMDB and Bangumi now derive multiple search keys, but provider search stops at the first non-empty
  result set.
- A localized title and a normalized title can legitimately return different useful provider
  candidates.
- Combining those results without a budget would create uncontrolled provider calls.

## Target State

When this lane closes:

- TMDB and Bangumi can merge useful candidates across search-title variants.
- Candidate enrichment has an explicit per-provider budget.
- Duplicate provider IDs are enriched at most once.
- Search merge policy remains provider-local and testable without live network calls.

## In Scope

- `crates/nako-metadata-scraper/src/providers/tmdb.rs`
- `crates/nako-metadata-scraper/src/providers/bangumi.rs`
- focused provider tests using fake transports
- workstream evidence and closeout notes

## Out Of Scope

- Browser-worker and Douban provider behavior
- Host task runtime and bulk scrape orchestration
- Live-network provider gates
- Reference repository code, selectors, fixtures, or generated data
- Transliteration dictionaries beyond existing title normalization

## Architecture Direction

Keep search-variant orchestration inside each provider. The public provider seam remains
`MetadataProvider::suggest`; ranking continues to receive provider-neutral candidate facts. Use a
small helper per provider for deduplicating raw search results before detail enrichment, and keep the
enrichment cap explicit.

## Closeout Condition

This lane can close when:

- TMDB merge behavior is proven through `suggest`.
- Bangumi merge behavior is proven through `suggest`.
- The package gate remains green.
- Formatting and whitespace checks pass.
- Residual localization work is recorded instead of absorbed silently.

## Closeout Summary

Closed on 2026-05-24.

- TMDB and Bangumi now merge search-title variant results before detail enrichment.
- Duplicate provider IDs are enriched once.
- Detail enrichment remains capped at three candidates per provider request.
- Case-only duplicate search keys are skipped by shared title variant generation.
- Provider-specific transliteration, smarter merged-result ranking, and live-provider smoke remain
  follow-on scope.
