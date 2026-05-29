# Official Metadata Addon Bangumi Year Air Date Filter

Status: Complete
Last updated: 2026-05-24

## Why This Lane Exists

Bangumi search currently uses the query title and configured subject types, but it ignores the query
year. TMDB already narrows text search by release year. Bangumi's official OpenAPI schema for
`POST /v0/search/subjects` exposes a `filter.air_date` array that can constrain release dates. Using
the query year there reduces ambiguous same-title matches without adding network calls.

## Relevant Authority

- `docs/workstreams/official-metadata-bangumi-provider-baseline`
- `docs/workstreams/official-metadata-addon-provider-search-variant-resilience`
- `docs/workstreams/official-metadata-addon-provider-relevance-budget`
- Official Bangumi OpenAPI `POST /v0/search/subjects` request schema, `filter.air_date`

## Problem

- Bangumi same-title or localized-title searches can return subjects across different years.
- Ranking can penalize mismatched years after results are returned, but irrelevant years still take
  search response budget and detail enrichment budget.
- Query year is already available and can safely narrow Bangumi search at the provider boundary.

## Target State

When this lane closes:

- Bangumi search requests include `filter.air_date` with `>=YYYY-01-01` and `<YYYY+1-01-01` when
  query year is present.
- Bangumi search requests omit `air_date` when query year is absent.
- Existing subject type and NSFW filters remain unchanged.
- Title-variant resilience, relevance-budget selection, degraded candidates, and payload resilience
  continue to work.
- Public payload shape, HTTP retry/backoff policy, and live network gates remain unchanged.

## In Scope

- `crates/nako-metadata-scraper/src/providers/bangumi.rs`
- Focused fake-transport provider tests.
- Workstream evidence and handoff docs.

## Out Of Scope

- Other Bangumi search filters such as tags, rating, rank, or rating_count.
- Runtime configuration changes.
- Live Bangumi network tests.
- Reference repository code, fixtures, selectors, or generated data.

## Architecture Direction

Keep the year-to-air-date mapping inside `providers::bangumi` because it is a provider-specific
search contract. Pass `MetadataQuery` into the Bangumi search request builder so all title variants
inherit the same year filter. Preserve the shared HTTP runtime and ranking contracts.

## Closeout Condition

This lane can close when:

- Bangumi proves year-bearing queries include the expected `air_date` range in every search request.
- Bangumi proves yearless queries omit `air_date`.
- Targeted Bangumi/ranking/title, package, workspace, format, and whitespace gates pass.

## Closeout

Closed on 2026-05-24. Bangumi search requests now include official `filter.air_date` constraints
when query year is present and omit the field when year is absent. Existing subject type and NSFW
filters, title-variant behavior, public payload shape, and HTTP runtime policy were preserved.
