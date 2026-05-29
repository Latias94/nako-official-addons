# Official Metadata Addon Bangumi Year Air Date Filter — Handoff

Status: Complete
Last updated: 2026-05-24

## Current State

The lane is complete. Bangumi search now maps query year into official `filter.air_date`
constraints and omits `air_date` when query year is absent.

## Completed Tasks

- OMBYF-010: scope and filter policy freeze.
- OMBYF-020: Bangumi query-year air-date filter.
- OMBYF-030: closeout with fresh gate evidence.

## Decisions Since Last Update

- Use Bangumi official `filter.air_date` with `>=YYYY-01-01` and `<YYYY+1-01-01`.
- Omit `air_date` when query year is absent.
- Keep subject type and NSFW filters unchanged.
- Keep public payload shape and HTTP retry/backoff policy unchanged.

## Blockers

- None.

## Next Recommended Action

- Open a new lane only when live Bangumi drift checks, richer tag filters, or deeper localized alias
  handling becomes the active priority.
