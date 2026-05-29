# Official Metadata Addon Provider Relevance Budget

Status: Active
Last updated: 2026-05-24

## Why This Lane Exists

TMDB and Bangumi now merge search results across title variants before detail enrichment. The merge
is still budgeted to three candidates, but the budget is spent in provider response order. That can
let weak raw-title matches consume the entire enrichment budget before stronger normalized-title or
alias-friendly matches are considered.

## Relevant Authority

- `docs/adr/0015-capability-scoped-http-addons-and-automation-providers.md`
- `docs/adr/0020-jellyfin-like-sidecar-addons-with-scoped-api-access.md`
- `docs/workstreams/official-metadata-addon-provider-search-merge`
- `docs/workstreams/official-metadata-addon-provider-degraded-candidates`
- `docs/workstreams/official-metadata-addon-result-quality`

## Problem

- Search merge deduplicates provider IDs, but truncates before comparing result relevance.
- TMDB and Bangumi search result payloads already carry title, date/year, score, and provider ID
  facts that are sufficient for a cheap pre-enrichment relevance pass.
- Final runtime ranking cannot recover candidates that never entered the provider enrichment budget.

## Target State

When this lane closes:

- TMDB collects all configured title-variant search results before applying the enrichment budget.
- Bangumi collects all configured title-variant search results before applying the enrichment budget.
- Both providers choose the enrichment budget using existing provider-neutral candidate facts and
  ranking behavior where practical.
- Duplicate provider IDs are still enriched at most once.
- Search request failures remain provider-level failures.
- Public payload shape, HTTP retry policy, and final runtime ranking remain unchanged.

## In Scope

- `crates/nako-metadata-scraper/src/providers/tmdb.rs`
- `crates/nako-metadata-scraper/src/providers/bangumi.rs`
- Optional shared provider-local helper inside the same modules or engine if it genuinely reduces
  duplication without exposing raw provider payloads.
- Focused fake-transport provider tests.
- Workstream evidence and closeout notes.

## Out Of Scope

- Live provider network gates.
- Changing `MetadataScrapeRuntime` final ranking semantics.
- Payload schema changes.
- Browser-worker, Douban, and host task orchestration.
- Reference repository code, fixtures, selectors, or generated data.

## Architecture Direction

Keep raw result collection provider-local because TMDB and Bangumi search schemas differ. Convert
search results into provider-neutral candidate facts for cheap scoring, then spend the detail
enrichment budget on the strongest deduped results. This preserves the public `MetadataProvider`
contract and avoids hardcoding final ordering hacks into provider-specific code.

## Closeout Condition

This lane can close when:

- TMDB proves a stronger merged search result can displace earlier weak raw results before detail
  enrichment.
- Bangumi proves the same behavior.
- Existing search merge, degraded candidate, and ranking gates remain green.
- Package, formatting, and whitespace gates pass.
