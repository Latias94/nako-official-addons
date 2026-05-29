# Official Metadata Addon Provider Enrichment Resilience

Status: Complete
Last updated: 2026-05-24

## Why This Lane Exists

TMDB and Bangumi now search multiple title variants and merge useful results, but candidate detail
enrichment still has an all-or-nothing failure mode. A transient failure on one detail request can
discard every other usable candidate from the same provider.

## Relevant Authority

- `docs/adr/0015-capability-scoped-http-addons-and-automation-providers.md`
- `docs/adr/0020-jellyfin-like-sidecar-addons-with-scoped-api-access.md`
- `docs/workstreams/official-metadata-addon-provider-hardening`
- `docs/workstreams/official-metadata-addon-provider-search-merge`

## Problem

- Provider HTTP runtime retries request-level retryable failures, but providers still treat any
  candidate enrichment error as a provider-level `suggest` failure.
- Search merge increases the number of candidate enrichments, so one bad candidate should not throw
  away other successfully enriched candidates.
- Search request failures should still fail the provider, because there are no candidates to salvage.

## Target State

When this lane closes:

- TMDB isolates a candidate when detail/external-id/alternative-title enrichment fails after runtime
  retry policy is exhausted, so one failed candidate does not fail provider `suggest`.
- Bangumi isolates a candidate when subject detail enrichment fails after runtime retry policy is
  exhausted, so one failed candidate does not fail provider `suggest`.
- Other successfully enriched candidates from the same provider are returned.
- Candidate-level failures are redaction-safe warnings, not payload-visible errors.
- Default gates remain synthetic and do not require live provider network access.

## In Scope

- `crates/nako-metadata-scraper/src/providers/tmdb.rs`
- `crates/nako-metadata-scraper/src/providers/bangumi.rs`
- focused fake-transport tests
- workstream evidence and closeout notes

## Out Of Scope

- Changing provider HTTP retry/backoff semantics
- Live TMDB/Bangumi network smoke
- Browser-worker, Douban, and host task orchestration
- Payload schema changes for exposing partial provider errors
- Reference repository code, fixtures, selectors, or generated data

## Architecture Direction

Keep retry and transport policy inside `ProviderHttpRuntime`. Providers own the distinction between
search-level failure and candidate-level enrichment failure. Use provider-local warning logs for
candidate-level failures and keep the public `MetadataProvider::suggest` contract unchanged.

## Closeout Condition

This lane can close when:

- TMDB candidate-level enrichment failure isolation is proven through `suggest`.
- Bangumi candidate-level enrichment failure isolation is proven through `suggest`.
- Search-level failures remain provider-level failures.
- Package, formatting, and whitespace gates pass.

## Closeout Summary

Closed on 2026-05-24.

- TMDB and Bangumi now isolate individual candidate enrichment failures after HTTP runtime policy is
  exhausted. The later degraded-candidates lane upgrades the final release behavior from skip-only to
  returning degraded candidates built from search-result facts.
- Search request failures still propagate as provider-level failures.
- Successful candidates from the same provider continue to be returned.
- HTTP runtime retry/backoff policy and public payload schema were not changed.
- Payload-visible partial-warning design remains follow-on scope.
