# Official Metadata Addon Provider Partial Search Diagnostics

Status: Complete
Last updated: 2026-05-24

## Why This Lane Exists

TMDB and Bangumi now preserve usable search results when later title-variant searches fail. That
improves provider resilience, but callers cannot see that a candidate came from a partial provider
search unless they inspect logs. The Addon already exposes redaction-safe `provider_note` evidence,
so partial-search diagnostics can become payload-visible without changing the public protocol shape.

## Relevant Authority

- `docs/workstreams/official-metadata-addon-provider-search-variant-resilience`
- `docs/workstreams/official-metadata-addon-provider-search-payload-resilience`
- `docs/workstreams/official-metadata-addon-provider-enrichment-resilience`
- `docs/workstreams/official-metadata-addon-provider-relevance-budget`

## Problem

- Search-variant failures are intentionally provider-local and no longer fail the whole provider
  when earlier variants returned candidates.
- The surviving candidates still look fully searched in the response evidence.
- Operator-facing diagnostics should remain redaction-safe and should not expose raw provider error
  bodies, request URLs, tokens, or search terms.

## Target State

When this lane closes:

- TMDB candidates include a redaction-safe provider note when one or more title-variant searches
  failed after at least one search result was preserved.
- Bangumi candidates include the same kind of provider note under the same boundary.
- Existing degraded-candidate and partial-enrichment notes are preserved and composed with the new
  partial-search diagnostic.
- All-search-failed behavior remains a provider error.
- Public payload shape, HTTP retry/backoff policy, provider request fan-out, and live network gates
  remain unchanged.

## In Scope

- `crates/nako-metadata-scraper/src/providers/tmdb.rs`
- `crates/nako-metadata-scraper/src/providers/bangumi.rs`
- Focused fake-transport provider tests.
- Workstream evidence and handoff docs.

## Out Of Scope

- New response fields or protocol schema changes.
- Raw provider error details in payloads.
- Live TMDB/Bangumi network drift checks.
- Reference repository code, fixtures, selectors, or generated data.

## Architecture Direction

Keep partial-search diagnostics provider-local. Providers already know whether a title-variant
search failed and already shape provider facts. Use `ProviderCandidateFacts::provider_note` as the
existing evidence channel, and compose safe note fragments so degraded and partial-enrichment
diagnostics are not overwritten.

## Closeout Condition

This lane can close when:

- TMDB proves preserved candidates surface a partial-search provider note.
- Bangumi proves preserved candidates surface a partial-search provider note.
- Existing all-search-failed behavior remains covered.
- Targeted provider/ranking/title, package, workspace, format, and whitespace gates pass.

## Closeout

Closed on 2026-05-24. TMDB and Bangumi now expose redaction-safe partial title-variant search
diagnostics through `provider_note` when candidates survive a later search failure. The public
payload shape and HTTP runtime policy were unchanged.
