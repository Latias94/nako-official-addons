# Official Metadata Addon Provider Search Payload Resilience

Status: Complete
Last updated: 2026-05-24

## Why This Lane Exists

TMDB and Bangumi search responses are external provider payloads. A single malformed search-result
item should not discard other usable candidates from the same response when those candidates can
still be enriched or degraded normally.

## Relevant Authority

- `docs/adr/0015-capability-scoped-http-addons-and-automation-providers.md`
- `docs/adr/0020-jellyfin-like-sidecar-addons-with-scoped-api-access.md`
- `docs/workstreams/official-metadata-addon-provider-search-variant-resilience`
- `docs/workstreams/official-metadata-addon-provider-relevance-budget`
- `docs/workstreams/official-metadata-addon-provider-degraded-candidates`

## Problem

- TMDB and Bangumi parse the full search response into strongly typed result vectors in one step.
- If one result item has provider drift, such as a missing or wrong-typed ID, the entire search
  response can fail and hide otherwise usable search results.
- Detail responses should remain strict because a malformed detail payload cannot produce a reliable
  enriched candidate.

## Target State

When this lane closes:

- TMDB skips malformed individual search result items while preserving valid items from the same
  response.
- Bangumi skips malformed individual search subject items while preserving valid items from the same
  response.
- Completely malformed top-level search responses still fail as provider search errors.
- Existing title-variant resilience, relevance-budget ranking, and degraded candidate fallback keep
  working.
- Public payload shape, HTTP retry/backoff policy, and live network gates remain unchanged.

## In Scope

- `crates/nako-metadata-scraper/src/providers/tmdb.rs`
- `crates/nako-metadata-scraper/src/providers/bangumi.rs`
- Focused fake-transport provider tests.
- Workstream evidence and closeout notes.

## Out Of Scope

- Tolerant parsing for detail responses.
- Live provider network gates.
- Public payload schema changes.
- Cross-provider ID mapping.
- Reference repository code, fixtures, selectors, or generated data.

## Architecture Direction

Keep tolerant search-item parsing provider-local because each provider owns its search schema. Treat
the top-level response shape as a contract and item-level drift as salvageable. Log skipped items
with redaction-safe warning messages and preserve the existing `MetadataProvider::suggest` contract.

## Closeout Condition

This lane can close when:

- TMDB proves a malformed search result item is skipped while valid items are enriched.
- Bangumi proves a malformed search subject item is skipped while valid items are enriched.
- Targeted, package, workspace, format, and whitespace gates pass.

## Closeout

Closed on 2026-05-24. TMDB and Bangumi both preserve valid search items when sibling items in the
same provider response are malformed. Detail response tolerance, live provider drift checks, and
public payload changes remain out of scope.
