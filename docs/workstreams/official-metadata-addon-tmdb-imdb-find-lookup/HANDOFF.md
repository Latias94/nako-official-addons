# Official Metadata Addon TMDB IMDb Find Lookup — Handoff

Status: Complete
Last updated: 2026-05-24

## Current State

The lane is complete. TMDB now uses query IMDb IDs to resolve a movie through the official
`/find/{imdb_id}` API before falling back to title search. When a payload carries multiple valid
IMDb IDs, TMDB now keeps trying later IDs if an earlier find result is empty or fails. Query IMDb
IDs tolerate uppercase or mixed-case `tt` prefixes and normalize find requests to lowercase `tt`.
Repeated normalized IMDb IDs are deduplicated before TMDB find requests.

## Completed Tasks

- OMITF-010: scope and lookup policy freeze.
- OMITF-020: TMDB IMDb find lookup.
- OMITF-030: closeout with fresh gate evidence.
- OMITF-040: query IMDb multi-ID continuation addendum.
- OMITF-050: query IMDb case normalization addendum.
- OMITF-060: query IMDb duplicate request addendum.

## Decisions Since Last Update

- Use official TMDB `/find/{external_id}` with `external_source=imdb_id`.
- Keep native query `tmdb` ID direct lookup ahead of IMDb find.
- Treat failed, empty, or malformed find as a fallback to the next valid IMDb ID, then title search
  after all valid IMDb IDs are exhausted.
- Normalize query IMDb IDs to lowercase `tt` while keeping digit suffix validation strict.
- Deduplicate repeated normalized IMDb IDs while preserving first-seen order.
- Reuse the existing movie detail, external IDs, and alternative titles enrichment path after find.

## Blockers

- None.

## Next Recommended Action

- Open a new lane only when richer TMDB find disambiguation, live TMDB drift checks, or another
  provider-specific external ID lookup becomes the active priority.
