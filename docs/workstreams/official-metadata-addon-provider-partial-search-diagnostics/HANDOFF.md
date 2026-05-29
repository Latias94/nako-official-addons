# Official Metadata Addon Provider Partial Search Diagnostics — Handoff

Status: Complete
Last updated: 2026-05-24

## Current State

The lane is complete. TMDB and Bangumi now surface redaction-safe partial title-variant search
diagnostics through existing provider notes when earlier search results are preserved after a later
variant search fails.

## Completed Tasks

- OMPSD-010: scope and diagnostic policy freeze.
- OMPSD-020: TMDB partial-search provider note.
- OMPSD-030: Bangumi partial-search provider note.
- OMPSD-040: closeout with fresh gate evidence.

## Decisions Since Last Update

- Use `ProviderCandidateFacts::provider_note` as the existing payload-visible diagnostic channel.
- Keep raw provider error details out of payloads.
- Preserve degraded and partial-enrichment notes by composing safe note fragments.
- Do not change public payload shape, provider request fan-out, or HTTP retry/backoff policy.

## Blockers

- None.

## Next Recommended Action

- Open a new lane only when live provider drift checks or deeper localized alias handling becomes the
  active priority.
