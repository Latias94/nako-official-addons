# Official Metadata Addon Provider External ID Lookup

Status: Complete
Last updated: 2026-05-24

## Why This Lane Exists

The metadata query can already carry provider external IDs, and ranking rewards exact external ID
matches. TMDB and Bangumi still ignore their own IDs during provider lookup and start with fuzzy
title search, which adds unnecessary network calls and can introduce weaker candidates before the
known provider object is fetched.

## Relevant Authority

- `docs/adr/0015-capability-scoped-http-addons-and-automation-providers.md`
- `docs/adr/0020-jellyfin-like-sidecar-addons-with-scoped-api-access.md`
- `docs/workstreams/official-metadata-addon-provider-relevance-budget`
- `docs/workstreams/official-metadata-addon-provider-search-variant-resilience`
- `docs/workstreams/official-metadata-addon-result-quality`

## Problem

- `MetadataQuery.external_ids` supports provider IDs, but TMDB/Bangumi do not use their own IDs as
  direct lookup inputs.
- Direct provider IDs are stronger than fuzzy title search and should avoid unnecessary search calls.
- Direct lookup should still reuse existing enrichment, degraded candidate, and ranking behavior.

## Target State

When this lane closes:

- TMDB uses a query `tmdb` external ID to fetch and enrich that movie directly before title search.
- Bangumi uses a query `bangumi` external ID to fetch and enrich that subject directly before title
  search.
- Invalid or unsupported external IDs fall back to existing title-search behavior.
- Direct lookup failures fall back to existing title-search behavior rather than failing the whole
  provider when title search can still produce candidates.
- HTTP retry/backoff policy and public payload shape remain unchanged.

## In Scope

- `crates/nako-metadata-scraper/src/providers/tmdb.rs`
- `crates/nako-metadata-scraper/src/providers/bangumi.rs`
- Small helper functions if they reduce duplication without exposing provider internals.
- Focused fake-transport provider tests.
- Workstream evidence and closeout notes.

## Out Of Scope

- Mapping IMDB/Wikidata/Bangumi/TMDB IDs across providers.
- Live provider network gates.
- Payload schema changes.
- Browser-worker, Douban, and host task orchestration.
- Reference repository code, fixtures, selectors, or generated data.

## Architecture Direction

Keep direct lookup provider-local because each provider owns its ID syntax and detail API. Use the
existing `MetadataProvider::suggest` contract and existing candidate mapping paths. Treat invalid ID
syntax as absent and preserve title-search fallback.

## Closeout Condition

This lane can close when:

- TMDB direct lookup is proven through `suggest` without a search request.
- Bangumi direct lookup is proven through `suggest` without a search request.
- Invalid IDs and direct-lookup failures fall back to title search.
- Targeted, package, workspace, format, and whitespace gates pass.

## Closeout

Closed on 2026-05-24. TMDB and Bangumi both use provider-native query external IDs for direct
detail lookup before fuzzy title search, with invalid-ID and failed-direct-lookup fallback coverage.
Cross-provider ID mapping remains deferred follow-on scope.
