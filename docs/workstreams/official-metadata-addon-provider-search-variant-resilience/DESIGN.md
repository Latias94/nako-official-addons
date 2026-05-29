# Official Metadata Addon Provider Search Variant Resilience

Status: Active
Last updated: 2026-05-24

## Why This Lane Exists

TMDB and Bangumi now search multiple title variants and merge results before detail enrichment. That
improves coverage, but a later title-variant search failure can still fail the entire provider even
when earlier variants already returned usable candidates.

## Relevant Authority

- `docs/adr/0015-capability-scoped-http-addons-and-automation-providers.md`
- `docs/adr/0020-jellyfin-like-sidecar-addons-with-scoped-api-access.md`
- `docs/workstreams/official-metadata-addon-provider-search-merge`
- `docs/workstreams/official-metadata-addon-provider-relevance-budget`
- `docs/workstreams/official-metadata-addon-provider-degraded-candidates`

## Problem

- Search merge increases the number of provider search requests per query.
- A retry-exhausted failure on a later search variant should not discard useful results returned by
  earlier variants.
- A provider should still fail when every attempted search variant fails before any candidate can be
  salvaged.

## Target State

When this lane closes:

- TMDB keeps usable merged search results when a later title-variant search fails.
- Bangumi keeps usable merged search results when a later title-variant search fails.
- Both providers still propagate provider-level errors when all search variants fail before producing
  candidates.
- Candidate-level degraded fallback and relevance-budget ranking continue to work.
- HTTP retry/backoff policy and public payload shape remain unchanged.

## In Scope

- `crates/nako-metadata-scraper/src/providers/tmdb.rs`
- `crates/nako-metadata-scraper/src/providers/bangumi.rs`
- Focused fake-transport provider tests.
- Workstream evidence and closeout notes.

## Out Of Scope

- Live provider network gates.
- HTTP runtime retry/backoff policy changes.
- Payload schema changes for exposing partial search errors.
- Browser-worker, Douban, and host task orchestration.
- Reference repository code, fixtures, selectors, or generated data.

## Architecture Direction

Keep search-variant error policy inside each provider. The HTTP runtime owns retry semantics, while
providers own the distinction between provider-level search failure and salvageable partial search
results. Use redaction-safe warning logs for ignored variant failures and preserve the public
`MetadataProvider::suggest` contract.

## Closeout Condition

This lane can close when:

- TMDB proves earlier search results survive a later variant search failure.
- Bangumi proves earlier search results survive a later variant search failure.
- Both providers still prove all-search-failed behavior propagates.
- Targeted, package, workspace, format, and whitespace gates pass.
