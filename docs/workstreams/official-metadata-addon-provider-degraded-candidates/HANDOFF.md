# Official Metadata Addon Provider Degraded Candidates — Handoff

Status: Complete
Last updated: 2026-05-24

## Current State

The lane is complete. Provider enrichment resilience now preserves usable search-result facts by
returning degraded TMDB and Bangumi candidates when detail enrichment fails after the shared HTTP
runtime policy is exhausted.

## Completed Tasks

- OMPDC-010: degraded candidate policy freeze.
- OMPDC-020: TMDB/Bangumi degraded candidate implementation and provider tests.
- OMPDC-030: closeout with fresh verification evidence.

## Decisions Since Last Update

- Degraded candidates use existing provider-neutral facts and `provider_note`.
- Search request failures stay provider-level failures.
- HTTP runtime retry policy remains unchanged.
- Live network gates remain out of default validation.

## Blockers

- None.

## Follow-Ons

- Live provider network checks are outside the default synthetic gate.
- Richer operator-facing partial-result warnings should be designed separately if needed.
