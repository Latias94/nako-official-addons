# Official Metadata Addon Provider Degraded Candidates

Status: Complete
Last updated: 2026-05-24

## Why This Lane Exists

Provider enrichment resilience currently skips candidates whose detail enrichment fails. That keeps
other candidates usable, but it still discards valuable search-result facts that TMDB and Bangumi
already returned. In weak network conditions, a degraded candidate is often better than no candidate
when the search result has enough title, year, score, and provider ID facts.

## Relevant Authority

- `docs/adr/0015-capability-scoped-http-addons-and-automation-providers.md`
- `docs/adr/0020-jellyfin-like-sidecar-addons-with-scoped-api-access.md`
- `docs/workstreams/official-metadata-addon-provider-enrichment-resilience`
- `docs/workstreams/official-metadata-addon-provider-search-merge`

## Problem

- TMDB search results already include title, original title, overview, release date, genre IDs, score,
  vote count, and poster/backdrop paths, but failed detail enrichment currently discards them.
- Bangumi search results already include subject title, localized title, date, rating, images, tags,
  and infobox, but failed detail enrichment currently discards them.
- The UI/ranking path can already carry provider-neutral `provider_note`, so degraded candidates can
  be marked without changing payload shape.

## Target State

When this lane closes:

- TMDB returns a degraded metadata candidate built from search-result facts when detail enrichment
  fails.
- Bangumi returns a degraded metadata candidate built from search-result facts when subject detail
  enrichment fails.
- Degraded candidates include a redaction-safe `provider_note`.
- Search-level failures remain provider-level failures.
- Fully enriched candidates still use detail responses when available.

## In Scope

- `crates/nako-metadata-scraper/src/providers/tmdb.rs`
- `crates/nako-metadata-scraper/src/providers/bangumi.rs`
- focused fake-transport tests
- workstream evidence and closeout notes

## Out Of Scope

- Live provider network gates
- HTTP runtime retry/backoff policy changes
- Payload schema changes
- Browser-worker, Douban, and host task orchestration
- Reference repository code, fixtures, selectors, or generated data

## Architecture Direction

Keep degraded candidate construction inside provider modules, because search-result schemas are
provider-specific. Reuse existing candidate fact fields and `provider_note` rather than introducing
a new public metadata schema. Ranking should treat degraded candidates through existing facts and
score reasons.

## Closeout Condition

This lane can close when:

- TMDB degraded candidate behavior is proven through `suggest`.
- Bangumi degraded candidate behavior is proven through `suggest`.
- Search-level failures still propagate.
- Package, formatting, and whitespace gates pass.

## Closeout Summary

Closed on 2026-05-24. TMDB and Bangumi now preserve usable search-result facts by returning
degraded candidates when per-candidate detail enrichment fails after the shared HTTP runtime policy
is exhausted. Search request failures still propagate as provider-level failures, and fully enriched
candidates continue to prefer detail responses.
