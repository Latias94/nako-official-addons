# Official Metadata Addon TMDB IMDb Find Lookup

Status: Complete
Last updated: 2026-05-24

## Why This Lane Exists

The runtime now accepts query `external_ids`, and TMDB already supports native TMDB ID direct lookup.
Many libraries and upstream scrapers carry IMDb IDs instead of TMDB IDs. TMDB's official API exposes
`/3/find/{external_id}` with `external_source=imdb_id`, which can resolve an IMDb ID to a TMDB movie
before detail enrichment. Using that lookup reduces dependence on noisy title/year text search.

## Relevant Authority

- `docs/workstreams/official-metadata-addon-provider-external-id-lookup`
- `docs/workstreams/official-metadata-addon-provider-relevance-budget`
- `docs/workstreams/official-metadata-addon-provider-search-variant-resilience`
- Official TMDB developer documentation for `/3/find/{external_id}` with `external_source=imdb_id`

## Problem

- Query native `tmdb` IDs bypass text search, but query `imdb` IDs still fall through to title search.
- Title search remains useful fallback, but it can be ambiguous for remakes, localized titles, and
  noisy filenames.
- The implementation must not copy reference repository code or fixtures.

## Target State

When this lane closes:

- TMDB uses query `imdb` external IDs to call `/find/{imdb_id}?external_source=imdb_id`.
- The first movie result from that find response is enriched through the existing TMDB movie detail,
  external IDs, and alternative titles path.
- Invalid IMDb IDs, empty find responses, malformed find responses, or failed find requests fall
  back to existing title search.
- Native query `tmdb` IDs continue to take precedence over IMDb find.
- Public payload shape, ranking contracts, HTTP retry/backoff policy, and Bangumi behavior remain
  unchanged.

## In Scope

- `crates/nako-metadata-scraper/src/providers/tmdb.rs`
- Focused fake-transport TMDB provider tests.
- Workstream evidence and handoff docs.

## Out Of Scope

- Bangumi external ID cross-provider lookup.
- TV results, person results, or multi-result disambiguation beyond first movie result.
- Live TMDB network gates.
- Reference repository code, fixtures, selectors, or generated data.

## Architecture Direction

Keep this inside `providers::tmdb`. The provider owns TMDB-specific external ID semantics, while the
shared runtime continues to own retry, timeout, proxy, and JSON boundaries. Treat `/find` as an
optional pre-search accelerator: if it cannot produce a movie ID, fall back to the existing title
search path.

## Closeout Condition

This lane can close when:

- TMDB proves query IMDb IDs resolve through `/find` and then existing detail enrichment.
- TMDB proves failed or empty find responses fall back to title search.
- Native query TMDB ID precedence remains covered.
- Targeted TMDB/ranking/title, package, workspace, format, and whitespace gates pass.

## Closeout

Closed on 2026-05-24. Query IMDb IDs now resolve through TMDB `/find/{imdb_id}` before title search,
then reuse existing movie detail enrichment. Empty, failed, or malformed find responses fall back to
title search, and native TMDB query IDs still take precedence.
