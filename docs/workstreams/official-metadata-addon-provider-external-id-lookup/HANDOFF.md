# Official Metadata Addon Provider External ID Lookup — Handoff

Status: Complete
Last updated: 2026-05-24

## Current State

The lane is complete. Query external IDs are parsed and ranking rewards exact matches. TMDB now uses
a query-native `tmdb` movie ID for direct detail enrichment before fuzzy title search. Bangumi now
uses a query-native `bangumi` subject ID for direct detail enrichment before fuzzy title search.
Both providers have invalid-ID and failed-direct-lookup fallback coverage. Object-form
`external_ids` now accepts string arrays so real payloads can provide multiple IDs for the same
provider; providers keep trying later same-provider IDs when an earlier valid direct lookup fails.
Repeated provider-native IDs are deduplicated before direct lookup to avoid duplicate external
requests. Array-of-object `external_ids` accepts `value`, `id`, or `external_id` string fields.
Parsed external ID providers and values are trimmed, and empty entries are skipped.

## Completed Tasks

- OMPEIL-010: direct lookup policy freeze.
- OMPEIL-020: TMDB direct movie lookup and fallback tests.
- OMPEIL-030: Bangumi direct subject lookup and fallback tests.
- OMPEIL-040: closeout with fresh gate evidence.
- OMPEIL-050: query payload compatibility addendum for object-value external ID arrays and multi-ID direct lookup continuation.
- OMPEIL-060: direct lookup duplicate request addendum.
- OMPEIL-070: query payload array-object value alias addendum.
- OMPEIL-080: query payload external ID trim addendum.

## Decisions Since Last Update

- Only provider-native `tmdb` and `bangumi` IDs are in scope.
- Invalid provider ID syntax falls back to title search.
- Direct lookup failure tries later same-provider IDs before falling back to title search.
- Object-form `external_ids` supports both string values and arrays of string values.
- Repeated provider-native query IDs are deduplicated after parsing while preserving first-seen order.
- Array-of-object `external_ids` accepts `value`, `id`, or `external_id` string fields.
- Parsed external ID providers and values are trimmed; empty entries are ignored.
- Cross-provider ID mapping remains out of scope.

## Blockers

- None.

## Follow-Ons

- Cross-provider ID mapping, such as IMDB to TMDB or TMDB to Bangumi.
- Live provider payload drift checks.
